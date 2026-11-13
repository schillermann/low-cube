use std::collections::VecDeque;
use std::net::{SocketAddr, UdpSocket};
use std::time::{Duration, Instant};
use glam::{Quat, Vec3};
use world::{CompanionPresence, PlanetaryWorld, PresenceKinematics, WorldFact};
use crate::chronicle_log::DirtySlices;
use crate::envelope::SanctuaryEnvelope;
use crate::link_state::{LinkPhase, LinkSync};
use crate::nat::NatAperture;
use crate::seal::HandshakeIdentity;
use crate::ticket::JoinTarget;
use crate::wormhole::{PunchCode, WormholePledge};

/// Encrypted P2P UDP client link managing PAKE wormhole discovery, NAT hole-punching, and chronicle sync.
pub struct RemoteLink {
    socket: UdpSocket,
    target: JoinTarget,
    phase: LinkPhase,
    sync: LinkSync,
}

impl RemoteLink {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(socket: UdpSocket, target: JoinTarget, phase: LinkPhase, sync: LinkSync) -> Self {
        Self { socket, target, phase, sync }
    }

    pub fn bind(target: JoinTarget) -> std::io::Result<Self> {
        let socket = UdpSocket::bind("0.0.0.0:0")?;
        socket.set_nonblocking(true)?;
        let _ = socket.set_broadcast(true);
        let guest_endpoint = match target {
            JoinTarget::Wormhole(code) => NatAperture::interactive().probe_guest(&socket, code.endpoint()),
            JoinTarget::Ticket(_) | JoinTarget::Direct(_) => socket.local_addr()?,
        };
        Ok(Self::bind_on_socket(socket, guest_endpoint, target))
    }

    /// Assembles a non-blocking `RemoteLink` over `socket` advertising `guest_endpoint` for NAT counter-punching.
    pub fn bind_on_socket(socket: UdpSocket, guest_endpoint: SocketAddr, target: JoinTarget) -> Self {
        let _ = socket.set_nonblocking(true);
        let _ = socket.set_broadcast(true);
        let kin = PresenceKinematics::new(Vec3::new(0.0, 32.0, 0.0), Quat::IDENTITY, Quat::IDENTITY, (0.0, 0.0));
        let initial_pres = CompanionPresence::new(0, kin, (0.0, 0.0, 0.0, 0.0), ([0; 9], [0; 4]));
        let keys = HandshakeIdentity::generate();
        let phase = match target {
            JoinTarget::Wormhole(code) => {
                let pledge = WormholePledge::begin(code, guest_endpoint);
                pledge.transmit_hello(&socket);
                LinkPhase::SeekingWormhole { pledge, keys, retry_clock: 0.0 }
            }
            JoinTarget::Ticket(_) | JoinTarget::Direct(_) => {
                LinkPhase::emit_knock(target, &socket, &keys);
                LinkPhase::Knocking { keys, retry_clock: 0.0 }
            }
        };
        let sync = LinkSync::new(0, 1.0, Vec::new(), initial_pres);
        Self::new(socket, target, phase, sync)
    }

    pub fn connected(&self) -> bool {
        matches!(self.phase, LinkPhase::Sealed { .. })
    }

    pub fn unresolved_wormhole(&self) -> bool {
        matches!(self.phase, LinkPhase::SeekingWormhole { .. })
    }

    pub fn punch_code(&self) -> Option<PunchCode> {
        self.phase.punch_code()
    }

    pub fn sync_local_presence(&mut self, presence: CompanionPresence) {
        let cid = match self.phase {
            LinkPhase::SeekingWormhole { .. } | LinkPhase::Knocking { .. } => 0,
            LinkPhase::Sealed { companion_id, .. } => companion_id,
        };
        self.sync.update_presence(presence, cid);
    }

    pub fn companions(&self, out: &mut Vec<CompanionPresence>) {
        self.sync.copy_companions(out);
    }

    pub fn ticket_string(&self) -> Option<String> {
        match self.target {
            JoinTarget::Ticket(t) => Some(t.encode()),
            JoinTarget::Direct(addr) => Some(addr.to_string()),
            JoinTarget::Wormhole(code) => Some(code.encode()),
        }
    }

    pub fn phrase_string(&self) -> Option<String> {
        match self.target {
            JoinTarget::Wormhole(code) => Some(code.encode()),
            _ => None,
        }
    }

    fn try_advance_wormhole(&mut self, src: SocketAddr, pkt: &[u8]) {
        if let Some(ticket) = self.phase.try_resolve_wormhole(src, pkt) {
            self.target = JoinTarget::Ticket(ticket);
            let old = std::mem::replace(
                &mut self.phase,
                LinkPhase::Knocking { keys: HandshakeIdentity::generate(), retry_clock: 0.0 },
            );
            if let LinkPhase::SeekingWormhole { keys, .. } = old {
                LinkPhase::emit_knock(self.target, &self.socket, &keys);
                self.phase = LinkPhase::Knocking { keys, retry_clock: 0.0 };
            }
        }
    }

    /// Steps non-blocking PAKE + X25519 handshake by `dt`, returning `Some((blocks, tron))` upon `LWEL`.
    pub fn step_handshake(&mut self, dt: f32) -> Option<(u32, bool)> {
        let mut buf = [0u8; 1500];
        while let Ok((len, src)) = self.socket.recv_from(&mut buf) {
            if matches!(self.phase, LinkPhase::SeekingWormhole { .. }) {
                self.try_advance_wormhole(src, &buf[..len]);
            } else if let Some(cfg) = self.try_accept_welcome(src, &buf[..len]) {
                return Some(cfg);
            }
        }
        match self.phase {
            LinkPhase::SeekingWormhole { ref pledge, ref mut retry_clock, .. } => {
                *retry_clock += dt;
                if *retry_clock >= 0.16 {
                    *retry_clock = 0.0;
                    pledge.transmit_hello(&self.socket);
                }
            }
            LinkPhase::Knocking { ref keys, ref mut retry_clock } => {
                *retry_clock += dt;
                if *retry_clock >= 0.16 {
                    *retry_clock = 0.0;
                    LinkPhase::emit_knock(self.target, &self.socket, keys);
                }
            }
            LinkPhase::Sealed { .. } => {}
        }
        None
    }

    pub fn try_welcome_within(&mut self, timeout: Duration) -> Option<(u32, bool)> {
        let deadline = Instant::now() + timeout;
        while Instant::now() < deadline {
            if let Some(cfg) = self.step_handshake(0.01) {
                return Some(cfg);
            }
            std::thread::sleep(Duration::from_millis(8));
        }
        None
    }

    pub fn await_welcome(&mut self, timeout: Duration, fallback_blocks: u32, fallback_tron: bool) -> (u32, bool) {
        self.try_welcome_within(timeout).unwrap_or((fallback_blocks, fallback_tron))
    }

    fn try_accept_welcome(&mut self, src: SocketAddr, pkt: &[u8]) -> Option<(u32, bool)> {
        if pkt.len() < 45 || &pkt[0..4] != b"LWEL" {
            return None;
        }
        let mut host_pub = [0u8; 32];
        host_pub.copy_from_slice(&pkt[4..36]);
        if let JoinTarget::Ticket(ticket) = self.target {
            if ticket.host_key() != host_pub {
                return None;
            }
        }
        let cid = u32::from_le_bytes(pkt[36..40].try_into().ok()?);
        let blocks = u32::from_le_bytes(pkt[40..44].try_into().ok()?);
        let tron = pkt[44] != 0;
        if let LinkPhase::Knocking { ref keys, .. } = self.phase {
            let seal = keys.establish(host_pub, true);
            self.phase = LinkPhase::Sealed { companion_id: cid, seal, peer: src };
            self.sync.assign_slot(cid);
            return Some((blocks, tron));
        }
        None
    }

    pub fn send_petition(&mut self, fact: WorldFact) {
        if let LinkPhase::Sealed { companion_id, ref mut seal, peer } = self.phase {
            let (mut payload, mut frame) = (Vec::new(), Vec::new());
            SanctuaryEnvelope::GuestPetition { fact }.inscribe(&mut payload);
            seal.seal(companion_id, &payload, &mut frame);
            let _ = self.socket.send_to(&frame, peer);
        }
    }

    pub fn pump(
        &mut self,
        dt: f32,
        world: &mut dyn PlanetaryWorld,
        predicted: &mut VecDeque<WorldFact>,
        slices: &DirtySlices,
    ) -> bool {
        if !self.connected() {
            let _ = self.step_handshake(dt);
            return false;
        }
        let mut changed = false;
        let mut buf = [0u8; 1500];
        while let Ok((len, src)) = self.socket.recv_from(&mut buf) {
            let pkt = &buf[..len];
            let decoded = if let LinkPhase::Sealed { ref mut seal, ref mut peer, .. } = self.phase {
                seal.open_frame(pkt).and_then(|(_, plain)| {
                    *peer = src;
                    SanctuaryEnvelope::decode(&plain)
                })
            } else {
                None
            };
            if let Some(env) = decoded {
                if self.sync.absorb_envelope(env, world, predicted, slices) {
                    changed = true;
                }
            }
        }
        if let LinkPhase::Sealed { companion_id, ref mut seal, peer } = self.phase {
            self.sync.step_pulse(dt, companion_id, seal, peer, &self.socket);
        }
        changed
    }
}
