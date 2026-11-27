mod bind;

use std::collections::VecDeque;
use std::net::{SocketAddr, UdpSocket};
use std::sync::{Arc, Mutex};
use std::time::Instant;
use glam::Vec3;
use world::{
    Callsign, CompanionPresence, CraftIdentity, CraftPose, DeckFrame, DrivenCraft, ExposedStratum,
    MeshTarget, PilotCraft, PlanetaryWorld, ProximityUtterance, SolarProportions,
    WorldFact,
};
use crate::aura::HostIdentity;
use crate::chronicle_decorators::CompactedChronicle;
use crate::chronicle_log::{ChronicleLog, ChronicleStore};
use crate::chronicled_planet::ChronicledPlanet;
use crate::host_state::{AuthoredChronicle, HostState};
use crate::host_wire::HostWire;
use crate::mailbox::{PortalBeat, PortalMailbox, SocketLoop};
use crate::seal::HandshakeIdentity;
use crate::snapshot_chronicle::SnapshottedChronicle;
use crate::survivor_ledger::{MemorySurvivors, SurvivorStore};
use crate::ticket::PortalTicket;
use crate::wormhole::PortalCode;
use self::bind::HostSocket;

/// Authoritative P2P World-Cube sanctuary owning `PlanetaryWorld` lock-free alongside `HostWire`.
pub struct SanctuaryHost {
    world: Box<dyn PlanetaryWorld>,
    wire: Arc<Mutex<HostWire>>,
    mailbox: Arc<PortalMailbox>,
    standby: SocketLoop,
}

impl SanctuaryHost {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn with_wire(
        world: Box<dyn PlanetaryWorld>,
        wire: Arc<Mutex<HostWire>>,
        mailbox: Arc<PortalMailbox>,
        standby: SocketLoop,
    ) -> Self {
        Self { world, wire, mailbox, standby }
    }

    /// Secondary constructor assembling an authoritative host without spawning a background standby thread.
    pub fn new(socket: UdpSocket, world: Box<dyn PlanetaryWorld>, identity: HostIdentity, state: HostState) -> Self {
        let wire = Arc::new(Mutex::new(HostWire::new(socket, identity, state, VecDeque::new())));
        let mailbox = Arc::new(PortalMailbox::active());
        let standby = SocketLoop::detached(Arc::clone(&mailbox));
        Self::with_wire(world, wire, mailbox, standby)
    }

    pub fn bind(bind_addr: &str, world: Box<dyn PlanetaryWorld>, blocks_per_edge: u32) -> std::io::Result<Self> {
        Self::bind_cosmos(bind_addr, world, SolarProportions::true_scale(blocks_per_edge))
    }

    pub fn bind_cosmos(bind_addr: &str, world: Box<dyn PlanetaryWorld>, proportions: SolarProportions) -> std::io::Result<Self> {
        let store = Box::new(SnapshottedChronicle::wrapping(CompactedChronicle::wrapping(ChronicleLog::empty())));
        Self::bind_cosmos_with_store(bind_addr, world, proportions, store)
    }

    pub fn bind_cosmos_with_store(
        bind_addr: &str, world: Box<dyn PlanetaryWorld>, proportions: SolarProportions, chronicle: Box<dyn ChronicleStore>,
    ) -> std::io::Result<Self> {
        Self::bind_cosmos_with_stores(bind_addr, world, proportions, chronicle, Box::new(MemorySurvivors::empty()))
    }

    pub fn bind_named_cosmos(
        bind_addr: &str, callsign: Callsign, world: Box<dyn PlanetaryWorld>, proportions: SolarProportions,
    ) -> std::io::Result<Self> {
        let store = Box::new(SnapshottedChronicle::wrapping(CompactedChronicle::wrapping(ChronicleLog::empty())));
        Self::bind_cosmos_with_identity(bind_addr, HandshakeIdentity::generate(), callsign, world, proportions, store, Box::new(MemorySurvivors::empty()))
    }

    pub fn bind_cosmos_with_stores(
        bind_addr: &str, world: Box<dyn PlanetaryWorld>, proportions: SolarProportions, chronicle: Box<dyn ChronicleStore>, survivors: Box<dyn SurvivorStore>,
    ) -> std::io::Result<Self> {
        Self::bind_cosmos_with_identity(bind_addr, HandshakeIdentity::generate(), Callsign::host(), world, proportions, chronicle, survivors)
    }

    pub fn bind_cosmos_with_identity(
        bind_addr: &str, host_keys: HandshakeIdentity, callsign: Callsign, world: Box<dyn PlanetaryWorld>,
        proportions: SolarProportions, chronicle: Box<dyn ChronicleStore>, survivors: Box<dyn SurvivorStore>,
    ) -> std::io::Result<Self> {
        HostSocket::new(bind_addr).bind_cosmos_with_identity(host_keys, callsign, world, proportions, chronicle, survivors)
    }

    pub fn bind_journaled_cosmos(
        bind_addr: &str,
        journal_path: impl AsRef<std::path::Path>,
        callsign: Callsign,
        proportions: SolarProportions,
        starter_kit: world::SatchelManifest,
    ) -> std::io::Result<Self> {
        HostSocket::new(bind_addr).bind_journaled_cosmos(journal_path.as_ref(), callsign, proportions, starter_kit)
    }

    pub fn bind_with_endpoint(
        bind_addr: &str, advertised: SocketAddr, world: Box<dyn PlanetaryWorld>, blocks_per_edge: u32,
    ) -> std::io::Result<Self> {
        HostSocket::new(bind_addr).bind_with_endpoint(advertised, world, blocks_per_edge)
    }

    pub fn spawn_standby(mut self) -> Self {
        let beat: Arc<Mutex<dyn PortalBeat>> = Arc::clone(&self.wire) as _;
        self.standby = SocketLoop::spawn(beat, Arc::clone(&self.mailbox));
        self
    }

    pub fn standby_active(&self) -> bool { self.standby.active() }
    fn read_wire<R>(&self, op: impl FnOnce(&HostWire) -> R) -> R {
        let guard = self.wire.lock().expect("HostWire mutex poisoned");
        op(&guard)
    }

    fn write_wire<R>(&self, op: impl FnOnce(&mut HostWire) -> R) -> R {
        let mut guard = self.wire.lock().expect("HostWire mutex poisoned");
        op(&mut guard)
    }

    pub fn world(&self) -> &dyn PlanetaryWorld { self.world.as_ref() }
    pub fn world_mut(&mut self) -> &mut dyn PlanetaryWorld { self.world.as_mut() }
    pub fn local_addr(&self) -> std::io::Result<SocketAddr> { self.read_wire(|w| w.socket().local_addr()) }
    pub fn code(&self) -> PortalCode { self.read_wire(|w| w.state().code()) }
    pub fn ticket(&self) -> PortalTicket { self.read_wire(HostWire::ticket) }
    pub fn punch_nat_peer(&mut self, peer: SocketAddr) { self.write_wire(|w| w.punch_peer(peer)); }
    pub fn sync_host_presence(&mut self, presence: CompanionPresence) { self.write_wire(|w| w.state_mut().occupy_host(presence)); }
    pub fn commit_host_survivor(&mut self, frame: world::CosmicFrame, position: Vec3, manifest: world::SatchelManifest) {
        self.write_wire(|w| {
            let pk = w.identity().keys().public_bytes();
            let cs = w.identity().callsign();
            w.state_mut().commit_host_owner(pk, world::SurvivorRecord::with_callsign(frame, position, manifest, cs), false);
        });
    }
    pub fn host_welcome(&self) -> Option<world::SurvivorWelcome> {
        self.read_wire(|w| w.state().lookup_host_owner(w.identity().keys().public_bytes()))
    }
    pub fn sync_host_berth_presence(&mut self, presence: CompanionPresence, berth: world::VesselBerth) {
        self.write_wire(|w| w.state_mut().occupy_host_berth(presence, berth));
    }
    pub fn sync_host_craft(&mut self, craft: PilotCraft) { self.write_wire(|w| w.state_mut().pilot_host(craft)); }
    pub fn guest_companions(&self, out: &mut Vec<CompanionPresence>) { self.read_wire(|w| w.guest_companions(out)); }
    pub fn guest_crafts(&self, out: &mut Vec<DrivenCraft>) { self.read_wire(|w| w.guest_crafts(out)); }
    pub fn utter_host_nearby(&mut self, text: &str) { self.write_wire(|w| w.utter_from_host(text)); }
    pub fn drain_host_utterances(&mut self, out: &mut Vec<ProximityUtterance>) { self.write_wire(|w| w.state_mut().drain_host_utterances(out)); }
    pub fn emit_guest_crafts(&self, target: &mut dyn MeshTarget) { self.read_wire(|w| w.emit_guest_crafts(target)); }
    pub fn align_epoch(&mut self, epoch_secs: f64) {
        self.world.align_epoch(epoch_secs);
        self.write_wire(|w| w.state_mut().sync_epoch(epoch_secs));
    }
    pub fn anchor_to_frame(&mut self, frame: world::CosmicFrame, local_pos: Vec3) {
        self.world.anchor_to_frame(frame, local_pos);
        let handoff = world::FrameHandoff::new(frame, local_pos, Vec3::ZERO, Vec3::ZERO);
        self.write_wire(|w| w.state_mut().reanchor_host(handoff));
    }
    pub fn reconcile_cosmic_frame(&mut self, local_pos: Vec3, local_vel: Vec3) -> Option<world::FrameHandoff> {
        let handoff = self.world.reconcile_cosmic_frame(local_pos, local_vel)?;
        self.write_wire(|w| w.state_mut().reanchor_host(handoff));
        Some(handoff)
    }
    pub fn blend_companions(&mut self, subtick_secs: f32) {
        let blend = if self.mailbox.standby_needed(Instant::now()) { 0.0 } else { subtick_secs };
        self.write_wire(|w| w.state_mut().blend_subtick(blend));
    }

    pub fn present_guest_vessels(&self, sink: &mut dyn world::VesselSink) {
        if self.mailbox.active_crafts() > 0 {
            self.read_wire(|w| w.present_vessels(sink));
        }
    }

    pub fn constrain_with_crafts(&self, position: Vec3, player_height: f32) -> Vec3 {
        let ground = self.world.constrain_to_surface(position, player_height);
        if self.mailbox.active_crafts() == 0 {
            return ground;
        }
        let up = self.world.surface_up_at(position);
        let Some(craft_top) = self.read_wire(|w| w.state().constrain_guest_crafts(position, player_height, up)) else {
            return ground;
        };
        if craft_top.dot(up) > ground.dot(up) { craft_top } else { ground }
    }

    pub fn board_passenger_seat(&self, eye: Vec3, direction: Vec3) -> Option<(CraftIdentity, Vec3, (Vec3, Vec3, Vec3))> {
        self.read_wire(|w| w.board_passenger_seat(eye, direction))
    }

    pub fn passenger_seat_frame(&self, craft: CraftIdentity, local_seat: Vec3) -> Option<(Vec3, Vec3, Vec3)> {
        self.read_wire(|w| w.passenger_seat_frame(craft, local_seat))
    }

    pub fn sample_deck_frame(&self, position: Vec3, player_height: f32) -> Option<DeckFrame> {
        if self.mailbox.active_crafts() > 0 {
            if let Some(deck) = self.read_wire(|w| w.state().sample_guest_deck(position, player_height)) {
                return Some(deck);
            }
        }
        self.world.sample_deck_frame(position, player_height)
    }

    pub fn resolve_deck_pose(&self, craft: CraftIdentity) -> Option<(CraftPose, f32)> {
        self.read_wire(|w| w.resolve_deck_pose(craft)).or_else(|| self.world.resolve_deck_pose(craft))
    }

    pub fn constrain_on_craft(&self, craft: CraftIdentity, position: Vec3, player_height: f32) -> Option<Vec3> {
        self.read_wire(|w| w.constrain_on_craft(craft, position, player_height))
            .or_else(|| self.world.constrain_on_craft(craft, position, player_height))
    }

    pub fn flush_disclosures(&mut self) {
        let mut revealed: Vec<(u8, ExposedStratum)> = Vec::new();
        self.world.drain_disclosures(&mut revealed);
        let mut guard = self.wire.lock().expect("HostWire mutex poisoned");
        for (face, stratum) in revealed {
            guard.state_mut().record_disclosure(face, stratum);
        }
        let eh = guard.identity().proportions().earth_half();
        guard.state_mut().compact_sectors(eh, self.world.as_mut());
    }

    pub fn with_chronicled<R>(&mut self, op: impl FnOnce(&mut ChronicledPlanet<'_>) -> R) -> R {
        let out = {
            let mut guard = self.wire.lock().expect("HostWire mutex poisoned");
            let mut decorated = ChronicledPlanet::new(self.world.as_mut(), guard.state_mut());
            let r = op(&mut decorated);
            guard.state_mut().refresh_beacons(self.world.as_ref());
            r
        };
        self.flush_disclosures();
        out
    }

    pub fn record_local_fact(&mut self, fact: WorldFact) {
        self.write_wire(|w| w.state_mut().record_authored_fact(1, fact));
        self.flush_disclosures();
    }

    pub fn pump_network(&mut self, dt: f32) -> bool {
        self.mailbox.record_main_beat(Instant::now());
        let (mutated, crafts) = {
            let mut guard = self.wire.lock().expect("HostWire mutex poisoned");
            guard.pump_authoritative(dt, self.world.as_mut())
        };
        self.mailbox.note_active_crafts(crafts);
        mutated
    }

    pub fn step_active_surface(&mut self, dt: f32) -> bool {
        let (mask, bodies) = self.read_wire(|w| w.state().occupied_zones_and_bodies());
        self.world.step_active_zones(dt, mask, &bodies)
    }

    pub fn tick(&mut self, dt: f32) -> bool {
        let regrown = self.step_active_surface(dt);
        let net_changed = self.pump_network(dt);
        regrown || net_changed
    }
}
