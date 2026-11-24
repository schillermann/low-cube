mod bind;

use std::collections::VecDeque;
use std::net::{SocketAddr, UdpSocket};
use std::time::{Duration, Instant};
use glam::Vec3;
use world::{
    Callsign, CompanionPresence, CosmicFrame, DrivenCraft, PilotCraft, PlanetaryWorld,
    SurvivorWelcome, WorldFact,
};
use crate::envelope::{InventoryNotice, SanctuaryEnvelope};
use crate::fleet_ledger::FleetLedger;
use crate::glide::SmoothRoster;
use crate::link_state::{LinkPhase, LinkSync, WelcomeFrame};
use crate::passport::{FixedPassport, KnockPassport, PassportVault};
use crate::seal::HandshakeIdentity;
use crate::ticket::JoinTarget;
use crate::wormhole::PunchCode;
use self::bind::LinkSocket;

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
        Self::bind_with_vault(target, Box::new(FixedPassport::unassigned()))
    }

    pub fn bind_with_identity(target: JoinTarget, keys: HandshakeIdentity) -> std::io::Result<Self> {
        Self::bind_with_vault(target, Box::new(FixedPassport::pinned(&keys)))
    }

    pub fn bind_with_vault(target: JoinTarget, vault: Box<dyn PassportVault>) -> std::io::Result<Self> {
        LinkSocket::new(target).open_with_vault(vault)
    }

    pub fn bind_on_socket(socket: UdpSocket, guest_endpoint: SocketAddr, target: JoinTarget) -> Self {
        Self::bind_socket_with_vault(socket, guest_endpoint, target, Box::new(FixedPassport::unassigned()))
    }

    pub fn bind_socket_with_identity(
        socket: UdpSocket, guest_endpoint: SocketAddr, target: JoinTarget, keys: HandshakeIdentity,
    ) -> Self {
        Self::bind_socket_with_vault(socket, guest_endpoint, target, Box::new(FixedPassport::pinned(&keys)))
    }

    pub fn bind_socket_with_vault(
        socket: UdpSocket, guest_endpoint: SocketAddr, target: JoinTarget, vault: Box<dyn PassportVault>,
    ) -> Self {
        LinkSocket::new(target).assemble(socket, guest_endpoint, vault)
    }

    pub fn connected(&self) -> bool { matches!(self.phase, LinkPhase::Sealed { .. }) }
    pub const fn companion_slot(&self) -> u32 {
        match self.phase {
            LinkPhase::SeekingWormhole { .. } | LinkPhase::Knocking { .. } => 0,
            LinkPhase::Sealed { companion_id, .. } => companion_id,
        }
    }
    pub const fn assigned_callsign(&self) -> Callsign { self.phase.assigned_callsign() }
    pub fn unresolved_wormhole(&self) -> bool { matches!(self.phase, LinkPhase::SeekingWormhole { .. }) }
    pub fn punch_code(&self) -> Option<PunchCode> { self.phase.punch_code() }
    pub(crate) const fn roster(&self) -> &SmoothRoster { self.sync.roster() }
    pub fn sync_local_presence(&mut self, presence: CompanionPresence) { self.sync.update_presence(presence, self.companion_slot()); }
    pub fn sync_local_berth_presence(&mut self, presence: CompanionPresence, berth: world::VesselBerth) {
        self.sync.update_berth_presence(presence, berth, self.companion_slot());
    }
    pub fn sync_local_craft(&mut self, craft: PilotCraft) { self.sync.update_craft(craft, self.companion_slot()); }
    pub fn companions(&self, out: &mut Vec<CompanionPresence>) { self.sync.copy_companions(out); }
    pub fn copy_crafts(&self, out: &mut Vec<DrivenCraft>) { self.sync.copy_crafts(out); }
    pub fn crafts(&self, out: &mut Vec<DrivenCraft>) { self.sync.copy_crafts(out); }
    pub const fn assigned_spawn(&self) -> Option<Vec3> { self.sync.assigned_spawn() }
    pub const fn assigned_survivor(&self) -> Option<SurvivorWelcome> { self.sync.assigned_survivor() }
    pub fn take_pending_epoch(&mut self) -> Option<f64> { self.sync.take_pending_epoch() }
    pub fn take_pending_anchor(&mut self) -> Option<(CosmicFrame, Vec3)> { self.sync.take_pending_anchor() }
    pub fn anchor_local_frame(&mut self, frame: CosmicFrame) { self.sync.anchor_local_frame(frame); }
    pub fn apply_frame_handoff(&mut self, handoff: world::FrameHandoff) { self.sync.apply_frame_handoff(handoff); }
    pub fn drain_utterances(&mut self, out: &mut Vec<world::ProximityUtterance>) { self.sync.drain_utterances(out); }
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
            let placeholder = KnockPassport::from_vault(self.target, Box::new(FixedPassport::unassigned()));
            let old = std::mem::replace(&mut self.phase, LinkPhase::Knocking { passport: placeholder, retry_clock: 0.0 });
            if let LinkPhase::SeekingWormhole { mut passport, .. } = old {
                passport.retarget(self.target);
                passport.emit_knock(self.target, &self.socket);
                self.phase = LinkPhase::Knocking { passport, retry_clock: 0.0 };
            }
        }
    }

    pub fn step_handshake(&mut self, dt: f32) -> Option<u32> {
        self.step_cosmos_handshake(dt).map(world::SolarProportions::earth_blocks)
    }

    pub fn step_cosmos_handshake(&mut self, dt: f32) -> Option<world::SolarProportions> {
        let mut buf = [0u8; 1500];
        while let Ok((len, src)) = self.socket.recv_from(&mut buf) {
            if matches!(self.phase, LinkPhase::SeekingWormhole { .. }) {
                self.try_advance_wormhole(src, &buf[..len]);
            } else if let Some(prop) = self.try_accept_welcome(src, &buf[..len]) {
                return Some(prop);
            }
        }
        self.phase.step_retry(dt, self.target, &self.socket);
        None
    }

    pub fn try_welcome_within(&mut self, timeout: Duration) -> Option<u32> {
        self.try_welcome_cosmos_within(timeout).map(world::SolarProportions::earth_blocks)
    }

    pub fn try_welcome_cosmos_within(&mut self, timeout: Duration) -> Option<world::SolarProportions> {
        let deadline = Instant::now() + timeout;
        while Instant::now() < deadline {
            if let Some(prop) = self.step_cosmos_handshake(0.01) {
                return Some(prop);
            }
            std::thread::sleep(Duration::from_millis(8));
        }
        None
    }

    pub fn await_welcome(&mut self, timeout: Duration, fallback_blocks: u32) -> u32 {
        self.try_welcome_within(timeout).unwrap_or(fallback_blocks)
    }

    fn try_accept_welcome(&mut self, src: SocketAddr, pkt: &[u8]) -> Option<world::SolarProportions> {
        WelcomeFrame::new(src, self.target, pkt).accept(&mut self.phase, &mut self.sync)
    }

    pub fn send_petition(&mut self, fact: WorldFact) {
        if let LinkPhase::Sealed { companion_id, ref mut seal, peer, .. } = self.phase {
            if self.sync.urgent_pulse()
                || matches!(fact, WorldFact::CraftCommissioned { .. } | WorldFact::HelmClaimed { .. } | WorldFact::HelmReleased { .. })
            {
                self.sync.flush_pulse(companion_id, seal, peer, &self.socket);
            }
            let (mut payload, mut frame) = (Vec::new(), Vec::new());
            SanctuaryEnvelope::GuestPetition { fact }.inscribe(&mut payload);
            seal.seal(companion_id, &payload, &mut frame);
            let _ = self.socket.send_to(&frame, peer);
        }
    }

    pub fn send_inventory_notice(&mut self, notice: InventoryNotice) {
        if let LinkPhase::Sealed { companion_id, ref mut seal, peer, .. } = self.phase {
            let (mut payload, mut frame) = (Vec::new(), Vec::new());
            SanctuaryEnvelope::GuestInventory { notice }.inscribe(&mut payload);
            seal.seal(companion_id, &payload, &mut frame);
            let _ = self.socket.send_to(&frame, peer);
        }
    }

    pub fn send_utterance(&mut self, raw_text: &str) {
        let trimmed = raw_text.trim();
        if trimmed.is_empty() {
            return;
        }
        if let LinkPhase::Sealed { companion_id, ref mut seal, peer, .. } = self.phase {
            let (mut payload, mut frame) = (Vec::new(), Vec::new());
            SanctuaryEnvelope::GuestUtterance { text: trimmed.to_string() }.inscribe(&mut payload);
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
        if let LinkPhase::Sealed { companion_id, ref mut seal, peer, .. } = self.phase {
            self.sync.step_pulse(dt, companion_id, seal, peer, &self.socket);
        }
        self.sync.roster().active_craft_count()
    }

    fn sync_welcome_world(&mut self, world: &mut dyn PlanetaryWorld) -> bool {
        let mut changed = false;
        if let Some(ep) = self.sync.take_pending_epoch() {
            world.align_epoch(ep);
            changed = true;
        }
        if let Some((frame, pos)) = self.sync.take_pending_anchor() {
            world.anchor_to_frame(frame, pos);
            changed = true;
        }
        changed
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
            let _ = self.sync_welcome_world(world);
            return (false, 0);
        }
        let mut changed = self.sync_welcome_world(world);
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
        if let LinkPhase::Sealed { companion_id, ref mut seal, peer, .. } = self.phase {
            self.sync.step_pulse(dt, companion_id, seal, peer, &self.socket);
        }
        (changed, self.sync.roster().active_craft_count())
    }
}
