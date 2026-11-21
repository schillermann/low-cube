use std::collections::VecDeque;
use std::net::{SocketAddr, UdpSocket};
use std::time::{Duration, Instant};
use glam::{Quat, Vec3};
use world::{
    CompanionPresence, DrivenCraft, PilotCraft, PlanetaryWorld, PresenceAttire, PresenceKinematics,
    WorldFact,
};
use crate::envelope::SanctuaryEnvelope;
use crate::fleet_ledger::FleetLedger;
use crate::glide::SmoothRoster;
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

    pub fn bind_on_socket(socket: UdpSocket, guest_endpoint: SocketAddr, target: JoinTarget) -> Self {
        let _ = socket.set_nonblocking(true);
        let _ = socket.set_broadcast(true);
        let kin = PresenceKinematics::new(Vec3::new(0.0, 32.0, 0.0), Quat::IDENTITY, Quat::IDENTITY, (0.0, 0.0));
        let initial_pres = CompanionPresence::new(0, kin, (0.0, 0.0, 0.0, 0.0), PresenceAttire::squire());
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
        let sync = LinkSync::new(0, 1.0, SmoothRoster::empty(), (initial_pres, PilotCraft::Pedestrian));
        Self::new(socket, target, phase, sync)
    }

    pub fn connected(&self) -> bool { matches!(self.phase, LinkPhase::Sealed { .. }) }
    pub const fn companion_slot(&self) -> u32 {
        match self.phase {
            LinkPhase::SeekingWormhole { .. } | LinkPhase::Knocking { .. } => 0,
            LinkPhase::Sealed { companion_id, .. } => companion_id,
        }
    }
    pub fn unresolved_wormhole(&self) -> bool { matches!(self.phase, LinkPhase::SeekingWormhole { .. }) }
    pub fn punch_code(&self) -> Option<PunchCode> { self.phase.punch_code() }
    pub(crate) const fn roster(&self) -> &SmoothRoster { self.sync.roster() }
    pub fn sync_local_presence(&mut self, presence: CompanionPresence) {
        self.sync.update_presence(presence, self.companion_slot());
    }
    pub fn sync_local_craft(&mut self, craft: PilotCraft) {
        self.sync.update_craft(craft, self.companion_slot());
    }
    pub fn companions(&self, out: &mut Vec<CompanionPresence>) { self.sync.copy_companions(out); }
    pub fn crafts(&self, out: &mut Vec<DrivenCraft>) { self.sync.copy_crafts(out); }
    pub fn blend_subtick(&mut self, subtick_secs: f32) { self.sync.blend_subtick(subtick_secs); }

    pub fn step_wildlife(&mut self, dt: f32, listener: Vec3, view_dir: Vec3, world: &dyn PlanetaryWorld) -> Option<Vec3> {
        let at = self.sync.roster().render_secs();
        self.sync.wildlife_mut().step_listener(at, dt, listener, view_dir, world)
    }

    pub fn intercept_boar_strike(&mut self, eye: Vec3, view_dir: Vec3, max_reach: f32, world: &dyn PlanetaryWorld) -> bool {
        let at = self.sync.roster().render_secs();
        self.sync.wildlife_mut().intercept_strike(at, eye, view_dir, max_reach, world)
    }

    pub fn emit_wildlife(&self, cone: world::SightCone, world: &dyn PlanetaryWorld, target: &mut dyn world::MeshTarget) {
        let at = self.sync.roster().render_secs();
        self.sync.wildlife().emit_visible(at, cone, world, target);
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

    pub fn step_handshake(&mut self, dt: f32) -> Option<u32> {
        let mut buf = [0u8; 1500];
        while let Ok((len, src)) = self.socket.recv_from(&mut buf) {
            if matches!(self.phase, LinkPhase::SeekingWormhole { .. }) {
                self.try_advance_wormhole(src, &buf[..len]);
            } else if let Some(blocks) = self.try_accept_welcome(src, &buf[..len]) {
                return Some(blocks);
            }
        }
        self.phase.step_retry(dt, self.target, &self.socket);
        None
    }

    pub fn try_welcome_within(&mut self, timeout: Duration) -> Option<u32> {
        let deadline = Instant::now() + timeout;
        while Instant::now() < deadline {
            if let Some(blocks) = self.step_handshake(0.01) {
                return Some(blocks);
            }
            std::thread::sleep(Duration::from_millis(8));
        }
        None
    }

    pub fn await_welcome(&mut self, timeout: Duration, fallback_blocks: u32) -> u32 {
        self.try_welcome_within(timeout).unwrap_or(fallback_blocks)
    }

    fn try_accept_welcome(&mut self, src: SocketAddr, pkt: &[u8]) -> Option<u32> {
        if pkt.len() < 44 || &pkt[0..4] != b"LWEL" {
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
        if let LinkPhase::Knocking { ref keys, .. } = self.phase {
            let seal = keys.establish(host_pub, true);
            self.phase = LinkPhase::Sealed { companion_id: cid, seal, peer: src };
            self.sync.assign_slot(cid);
            return Some(blocks);
        }
        None
    }

    pub fn send_petition(&mut self, fact: WorldFact) {
        if let LinkPhase::Sealed { companion_id, ref mut seal, peer } = self.phase {
            if self.sync.urgent_pulse()
                || matches!(
                    fact,
                    WorldFact::CraftCommissioned { .. }
                        | WorldFact::HelmClaimed { .. }
                        | WorldFact::HelmReleased { .. }
                )
            {
                self.sync.flush_pulse(companion_id, seal, peer, &self.socket);
            }
            let (mut payload, mut frame) = (Vec::new(), Vec::new());
            SanctuaryEnvelope::GuestPetition { fact }.inscribe(&mut payload);
            seal.seal(companion_id, &payload, &mut frame);
            let _ = self.socket.send_to(&frame, peer);
        }
    }

    pub fn pump_standby(&mut self, dt: f32, deferred: &mut VecDeque<SanctuaryEnvelope>) -> usize {
        if !self.connected() {
            let _ = self.step_handshake(dt);
            return 0;
        }
        let mut buf = [0u8; 1500];
        while let Ok((len, src)) = self.socket.recv_from(&mut buf) {
            if let Some(env) = self.phase.unseal_envelope(src, &buf[..len]) {
                self.sync.absorb_standby_envelope(env, deferred);
            }
        }
        if let LinkPhase::Sealed { companion_id, ref mut seal, peer } = self.phase {
            self.sync.step_pulse(dt, companion_id, seal, peer, &self.socket);
        }
        self.sync.roster().active_craft_count()
    }

    pub fn pump(
        &mut self,
        dt: f32,
        deferred: &mut VecDeque<SanctuaryEnvelope>,
        world: &mut dyn PlanetaryWorld,
        fleet: &mut FleetLedger,
        predicted: &mut VecDeque<WorldFact>,
    ) -> (bool, usize) {
        if !self.connected() {
            let _ = self.step_handshake(dt);
            return (false, 0);
        }
        let mut changed = false;
        while let Some(env) = deferred.pop_front() {
            if self.sync.absorb_envelope(env, world, fleet, predicted) {
                changed = true;
            }
        }
        let mut buf = [0u8; 1500];
        while let Ok((len, src)) = self.socket.recv_from(&mut buf) {
            if let Some(env) = self.phase.unseal_envelope(src, &buf[..len]) {
                if self.sync.absorb_envelope(env, world, fleet, predicted) {
                    changed = true;
                }
            }
        }
        if let LinkPhase::Sealed { companion_id, ref mut seal, peer } = self.phase {
            self.sync.step_pulse(dt, companion_id, seal, peer, &self.socket);
        }
        (changed, self.sync.roster().active_craft_count())
    }
}
