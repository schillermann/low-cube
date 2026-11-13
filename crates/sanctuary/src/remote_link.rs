use std::collections::VecDeque;
use std::net::UdpSocket;
use std::time::{Duration, Instant};
use glam::{Quat, Vec3};
use world::{CompanionPresence, PlanetaryWorld, PresenceKinematics, WorldFact};
use crate::chronicle_log::DirtySlices;
use crate::envelope::{CrackNotice, SanctuaryEnvelope};
use crate::link_state::{LinkPhase, LinkSync};
use crate::seal::HandshakeIdentity;
use crate::ticket::{BeaconRoute, JoinTarget, PortalTicket};
use crate::ward::ReturnPath;

/// Encrypted UDP client link managing NAT hole-punching, relay fallback, and chronicle sync.
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
        let kin = PresenceKinematics::new(Vec3::new(0.0, 32.0, 0.0), Quat::IDENTITY, Quat::IDENTITY, (0.0, 0.0));
        let initial_pres = CompanionPresence::new(0, kin, (0.0, 0.0, 0.0, 0.0), ([0; 9], [0; 4]));
        let phase = LinkPhase::Knocking { keys: HandshakeIdentity::generate(), retry_clock: 1.0 };
        let sync = LinkSync::new(0, 1.0, Vec::new(), initial_pres);
        Ok(Self::new(socket, target, phase, sync))
    }

    pub fn connected(&self) -> bool {
        matches!(self.phase, LinkPhase::Sealed { .. })
    }

    pub fn sync_local_presence(&mut self, presence: CompanionPresence) {
        let cid = match self.phase {
            LinkPhase::Knocking { .. } => 0,
            LinkPhase::Sealed { companion_id, .. } => companion_id,
        };
        self.sync.local_presence = presence.with_slot(cid);
    }

    pub fn companions(&self, out: &mut Vec<CompanionPresence>) {
        out.clear();
        out.extend_from_slice(&self.sync.companions);
    }

    pub fn ticket_string(&self) -> Option<String> {
        match self.target {
            JoinTarget::Ticket(t) => Some(t.encode()),
            JoinTarget::Direct(addr) => Some(addr.to_string()),
        }
    }

    fn emit_knock(&self, keys: &HandshakeIdentity) {
        let mut knock = Vec::with_capacity(36);
        knock.extend_from_slice(b"LKNK");
        knock.extend_from_slice(&keys.public_bytes());
        match self.target {
            JoinTarget::Direct(addr) => {
                let _ = self.socket.send_to(&knock, addr);
            }
            JoinTarget::Ticket(ticket) => {
                let _ = self.socket.send_to(&knock, ticket.direct());
                if let BeaconRoute::Assisted(beacon) = ticket.beacon() {
                    let mut rv = Vec::with_capacity(12);
                    rv.extend_from_slice(b"LBRV");
                    rv.extend_from_slice(&ticket.portal_id());
                    let _ = self.socket.send_to(&rv, beacon);
                    ReturnPath::Relayed { beacon, portal_id: ticket.portal_id(), slot: 0 }
                        .transmit(&self.socket, &knock);
                }
            }
        }
    }

    /// Performs initial handshake wait (up to `timeout`) returning `(blocks_per_edge, tron_mode)` on welcome.
    pub fn await_welcome(&mut self, timeout: Duration, fallback_blocks: u32, fallback_tron: bool) -> (u32, bool) {
        let deadline = Instant::now() + timeout;
        let mut buf = [0u8; 1500];
        while Instant::now() < deadline {
            if let LinkPhase::Knocking { ref keys, .. } = self.phase {
                self.emit_knock(keys);
            }
            std::thread::sleep(Duration::from_millis(8));
            while let Ok((len, src)) = self.socket.recv_from(&mut buf) {
                if let Some(cfg) = self.try_accept_welcome(src, &buf[..len]) {
                    return cfg;
                }
            }
        }
        (fallback_blocks, fallback_tron)
    }

    fn try_accept_welcome(&mut self, src: std::net::SocketAddr, pkt: &[u8]) -> Option<(u32, bool)> {
        if pkt.len() >= 10 && &pkt[0..4] == b"LBPU" {
            if let Some(peer) = PortalTicket::read_v4(&pkt[4..10]) {
                if let LinkPhase::Knocking { ref keys, .. } = self.phase {
                    let mut knock = Vec::with_capacity(36);
                    knock.extend_from_slice(b"LKNK");
                    knock.extend_from_slice(&keys.public_bytes());
                    let _ = self.socket.send_to(&knock, peer);
                }
            }
            return None;
        }
        let (path, inner) = if pkt.len() > 14 && &pkt[0..4] == b"LBFW" {
            let mut portal_id = [0u8; 8];
            portal_id.copy_from_slice(&pkt[4..12]);
            (ReturnPath::Relayed { beacon: src, portal_id, slot: 0 }, &pkt[14..])
        } else {
            (ReturnPath::Direct(src), pkt)
        };
        if inner.len() < 45 || &inner[0..4] != b"LWEL" {
            return None;
        }
        let mut host_pub = [0u8; 32];
        host_pub.copy_from_slice(&inner[4..36]);
        if let JoinTarget::Ticket(ticket) = self.target {
            if ticket.host_key() != host_pub {
                return None;
            }
        }
        let cid = u32::from_le_bytes(inner[36..40].try_into().ok()?);
        let blocks = u32::from_le_bytes(inner[40..44].try_into().ok()?);
        let tron = inner[44] != 0;
        if let LinkPhase::Knocking { ref keys, .. } = self.phase {
            let seal = keys.establish(host_pub, true);
            self.phase = LinkPhase::Sealed { companion_id: cid, seal, path };
            self.sync.local_presence = self.sync.local_presence.with_slot(cid);
            return Some((blocks, tron));
        }
        None
    }

    pub fn send_petition(&mut self, fact: WorldFact) {
        if let LinkPhase::Sealed { companion_id, ref mut seal, path } = self.phase {
            let (mut payload, mut frame) = (Vec::new(), Vec::new());
            SanctuaryEnvelope::GuestPetition { fact }.inscribe(&mut payload);
            seal.seal(companion_id, &payload, &mut frame);
            path.transmit(&self.socket, &frame);
        }
    }

    fn handle_sealed_envelope(
        &mut self,
        env: SanctuaryEnvelope,
        world: &mut dyn PlanetaryWorld,
        predicted: &mut VecDeque<WorldFact>,
        slices: &DirtySlices,
    ) -> bool {
        let faces = world.face_count();
        match env {
            SanctuaryEnvelope::HostChronicle { base_seq, facts } => {
                if base_seq > self.sync.ack_seq {
                    return false;
                }
                let skip = (self.sync.ack_seq - base_seq) as usize;
                let mut changed = false;
                for fact in facts.into_iter().skip(skip) {
                    self.sync.ack_seq += 1;
                    if let Some(pos) = predicted.iter().position(|p| *p == fact) {
                        let _ = predicted.remove(pos);
                    } else if fact.enact_on(world) {
                        slices.mark_fact(&fact, faces);
                        changed = true;
                    }
                }
                changed
            }
            SanctuaryEnvelope::HostPulse { crack, companions, .. } => {
                self.sync.companions = companions;
                if let CrackNotice::Active { eye, direction, stage } = crack {
                    if world.loosen_surface(eye, direction, stage).is_some() {
                        slices.mark_fact(&WorldFact::Loosened { eye, direction, stage }, faces);
                        return true;
                    }
                }
                false
            }
            _ => false,
        }
    }

    pub fn pump(
        &mut self,
        dt: f32,
        world: &mut dyn PlanetaryWorld,
        predicted: &mut VecDeque<WorldFact>,
        slices: &DirtySlices,
    ) -> bool {
        let mut changed = false;
        let mut buf = [0u8; 1500];
        while let Ok((len, src)) = self.socket.recv_from(&mut buf) {
            let pkt = &buf[..len];
            if matches!(self.phase, LinkPhase::Knocking { .. }) {
                let _ = self.try_accept_welcome(src, pkt);
                continue;
            }
            if pkt.len() >= 10 && &pkt[0..4] == b"LBPU" {
                if let Some(peer) = PortalTicket::read_v4(&pkt[4..10]) {
                    let _ = self.socket.send_to(b"LPCH", peer);
                }
                continue;
            }
            let (rx_path, inner) = if pkt.len() > 14 && &pkt[0..4] == b"LBFW" {
                let mut portal_id = [0u8; 8];
                portal_id.copy_from_slice(&pkt[4..12]);
                (ReturnPath::Relayed { beacon: src, portal_id, slot: 0 }, &pkt[14..])
            } else {
                (ReturnPath::Direct(src), pkt)
            };
            let decoded = if let LinkPhase::Sealed { ref mut seal, ref mut path, .. } = self.phase {
                seal.open_frame(inner).and_then(|(_, plain)| {
                    if matches!(rx_path, ReturnPath::Direct(_)) {
                        *path = rx_path;
                    }
                    SanctuaryEnvelope::decode(&plain)
                })
            } else {
                None
            };
            if let Some(env) = decoded {
                if self.handle_sealed_envelope(env, world, predicted, slices) {
                    changed = true;
                }
            }
        }

        match self.phase {
            LinkPhase::Knocking { ref keys, ref mut retry_clock } => {
                *retry_clock += dt;
                if *retry_clock >= 0.20 {
                    *retry_clock = 0.0;
                    self.emit_knock(keys);
                }
            }
            LinkPhase::Sealed { companion_id, ref mut seal, path } => {
                self.sync.pulse_clock += dt;
                if self.sync.pulse_clock >= 0.05 {
                    self.sync.pulse_clock = 0.0;
                    let (mut payload, mut frame) = (Vec::new(), Vec::new());
                    let pulse = SanctuaryEnvelope::GuestPulse {
                        ack_seq: self.sync.ack_seq,
                        presence: self.sync.local_presence,
                    };
                    pulse.inscribe(&mut payload);
                    seal.seal(companion_id, &payload, &mut frame);
                    path.transmit(&self.socket, &frame);
                }
            }
        }
        changed
    }
}
