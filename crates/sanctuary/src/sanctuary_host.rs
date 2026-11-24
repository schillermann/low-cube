use std::collections::VecDeque;
use std::net::{SocketAddr, UdpSocket};
use std::sync::{Arc, Mutex};
use std::time::Instant;
use glam::Vec3;
use world::{
    CompanionPresence, CraftIdentity, CraftPose, DeckFrame, DrivenCraft, ExposedStratum,
    MeshTarget, PilotCraft, PlanetaryWorld, ProximityUtterance, SightCone, WorldFact,
};
use fauna::BoarSounder;
use crate::aura::{HostAura, HostIdentity};
use crate::chronicle_decorators::CompactedChronicle;
use crate::chronicle_log::{ChronicleLog, ChronicleStore};
use crate::chronicled_planet::ChronicledPlanet;
use crate::cohort::GuestCohort;
use crate::host_state::HostState;
use crate::host_wire::HostWire;
use crate::mailbox::{PortalBeat, PortalMailbox, SocketLoop};
use crate::nat::NatAperture;
use crate::seal::HandshakeIdentity;
use crate::ticket::PortalTicket;
use crate::wormhole::{PortalCode, WormholeGate};

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

    pub fn spawn_standby(mut self) -> Self {
        let beat: Arc<Mutex<dyn PortalBeat>> = Arc::clone(&self.wire) as _;
        self.standby = SocketLoop::spawn(beat, Arc::clone(&self.mailbox));
        self
    }

    pub fn standby_active(&self) -> bool {
        self.standby.active()
    }

    fn bind_slot_socket(bind_addr: &str, secret: u8) -> std::io::Result<UdpSocket> {
        if let Some(host_part) = bind_addr.strip_suffix(":0") {
            let preferred = PortalCode::slot_port_for_secret(secret);
            let candidate = format!("{host_part}:{preferred}");
            if let Ok(sock) = UdpSocket::bind(&candidate) {
                return Ok(sock);
            }
        }
        UdpSocket::bind(bind_addr)
    }

    /// Binds a non-blocking UDP socket and discovers the external P2P endpoint via STUN & UPnP/NAT-PMP.
    pub fn bind(bind_addr: &str, world: Box<dyn PlanetaryWorld>, blocks_per_edge: u32) -> std::io::Result<Self> {
        Self::bind_cosmos(bind_addr, world, world::SolarProportions::true_scale(blocks_per_edge))
    }

    /// Binds a non-blocking UDP socket advertising full `SolarProportions` during guest handshakes.
    pub fn bind_cosmos(
        bind_addr: &str,
        world: Box<dyn PlanetaryWorld>,
        proportions: world::SolarProportions,
    ) -> std::io::Result<Self> {
        let store = Box::new(CompactedChronicle::wrapping(ChronicleLog::empty()));
        Self::bind_cosmos_with_store(bind_addr, world, proportions, store)
    }

    /// Binds a non-blocking UDP socket backed by an explicit `ChronicleStore` decorator chain.
    pub fn bind_cosmos_with_store(
        bind_addr: &str,
        world: Box<dyn PlanetaryWorld>,
        proportions: world::SolarProportions,
        chronicle: Box<dyn ChronicleStore>,
    ) -> std::io::Result<Self> {
        let secret = PortalCode::draw_secret();
        let socket = Self::bind_slot_socket(bind_addr, secret)?;
        socket.set_nonblocking(true)?;
        let endpoint = NatAperture::interactive().open_host(&socket);
        Ok(Self::assemble_with_store(socket, endpoint, secret, world, proportions, chronicle))
    }

    /// Binds a UDP socket advertising a known external NAT endpoint.
    pub fn bind_with_endpoint(
        bind_addr: &str,
        advertised: SocketAddr,
        world: Box<dyn PlanetaryWorld>,
        blocks_per_edge: u32,
    ) -> std::io::Result<Self> {
        let secret = PortalCode::draw_secret();
        let socket = Self::bind_slot_socket(bind_addr, secret)?;
        socket.set_nonblocking(true)?;
        let store = Box::new(CompactedChronicle::wrapping(ChronicleLog::empty()));
        Ok(Self::assemble_with_store(
            socket,
            advertised,
            secret,
            world,
            world::SolarProportions::true_scale(blocks_per_edge),
            store,
        ))
    }

    fn assemble_with_store(
        socket: UdpSocket,
        endpoint: SocketAddr,
        secret: u8,
        world: Box<dyn PlanetaryWorld>,
        proportions: world::SolarProportions,
        chronicle: Box<dyn ChronicleStore>,
    ) -> Self {
        let code = PortalCode::from_endpoint(endpoint, secret);
        let identity = HostIdentity::with_proportions(HandshakeIdentity::generate(), proportions, endpoint);
        let mut aura = HostAura::headless();
        let initial_epoch = world.celestial_atlas().ephemeris().map_or(0.0, |e| e.epoch_secs());
        aura.sync_epoch(initial_epoch);
        aura.beacons_mut().refresh_from(world.as_ref());
        let state = HostState::with_store(
            chronicle,
            GuestCohort::empty(),
            aura,
            WormholeGate::with_code(code),
        );
        Self::new(socket, world, identity, state)
    }

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
    pub fn reconcile_cosmic_frame(&mut self, local_pos: Vec3, local_vel: Vec3) -> Option<world::FrameHandoff> {
        let handoff = self.world.reconcile_cosmic_frame(local_pos, local_vel)?;
        self.write_wire(|w| w.state_mut().reanchor_host(handoff));
        Some(handoff)
    }
    pub fn blend_companions(&mut self, subtick_secs: f32) {
        let blend = if self.mailbox.standby_needed(Instant::now()) { 0.0 } else { subtick_secs };
        self.write_wire(|w| w.state_mut().blend_subtick(blend));
    }

    pub fn step_host_wildlife(&mut self, listener: Vec3, view_dir: Vec3) -> Option<Vec3> {
        let world_ref = self.world.as_ref();
        self.write_wire(|w| w.state_mut().wildlife_mut().step_host_listener(listener, view_dir, world_ref))
    }

    pub fn emit_wildlife(&self, cone: SightCone, target: &mut dyn MeshTarget) {
        self.read_wire(|w| w.state().wildlife().emit_visible(cone, target));
    }

    pub fn strike_host_boar(&mut self, eye: Vec3, direction: Vec3, power: u8, max_reach: f32) -> (bool, bool) {
        let world_ref = self.world.as_ref();
        let (hit, drops) = self.write_wire(|w| {
            w.state_mut().wildlife_mut().evaluate_strike(eye, direction, power, max_reach, true, world_ref)
        });
        let dropped = drops.is_some_and(|d| self.with_chronicled(|w| BoarSounder::cast_meat_drops(d, w)));
        (hit, dropped)
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
        if !revealed.is_empty() {
            self.write_wire(|w| {
                for (face, stratum) in revealed {
                    w.state_mut().record_disclosure(face, stratum);
                }
            });
        }
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
        if mutated {
            self.flush_disclosures();
        }
        mutated
    }

    pub fn tick(&mut self, dt: f32) -> bool {
        let regrown = self.world.step_surface(dt);
        let net_changed = self.pump_network(dt);
        regrown || net_changed
    }
}
