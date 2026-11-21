use std::net::{SocketAddr, UdpSocket};
use glam::Vec3;
use world::{
    CompanionPresence, CraftIdentity, CraftPose, DeckFrame, DrivenCraft, ExposedStratum,
    MeshTarget, PilotCraft, PlanetaryWorld, SightCone, WorldFact,
};
use fauna::BoarSounder;
use crate::aura::{HostAura, HostIdentity};
use crate::chronicle_log::ChronicleLog;
use crate::chronicled_planet::ChronicledPlanet;
use crate::cohort::GuestCohort;
use crate::host_state::HostState;
use crate::nat::NatAperture;
use crate::seal::HandshakeIdentity;
use crate::ticket::PortalTicket;
use crate::wormhole::{PortalCode, WormholeGate};

/// Authoritative P2P World-Cube sanctuary enforcing anti-cheat and zero-knowledge ore disclosure.
pub struct SanctuaryHost {
    socket: UdpSocket,
    world: Box<dyn PlanetaryWorld>,
    identity: HostIdentity,
    state: HostState,
}

impl SanctuaryHost {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(socket: UdpSocket, world: Box<dyn PlanetaryWorld>, identity: HostIdentity, state: HostState) -> Self {
        Self { socket, world, identity, state }
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
        let secret = PortalCode::draw_secret();
        let socket = Self::bind_slot_socket(bind_addr, secret)?;
        socket.set_nonblocking(true)?;
        let endpoint = NatAperture::interactive().open_host(&socket);
        Ok(Self::assemble_bound(socket, endpoint, secret, world, blocks_per_edge))
    }

    /// Binds a UDP socket advertising a known external NAT endpoint (for port-forwarded or simulated NAT).
    pub fn bind_with_endpoint(
        bind_addr: &str,
        advertised: SocketAddr,
        world: Box<dyn PlanetaryWorld>,
        blocks_per_edge: u32,
    ) -> std::io::Result<Self> {
        let secret = PortalCode::draw_secret();
        let socket = Self::bind_slot_socket(bind_addr, secret)?;
        socket.set_nonblocking(true)?;
        Ok(Self::assemble_bound(socket, advertised, secret, world, blocks_per_edge))
    }

    fn assemble_bound(
        socket: UdpSocket,
        endpoint: SocketAddr,
        secret: u8,
        world: Box<dyn PlanetaryWorld>,
        blocks_per_edge: u32,
    ) -> Self {
        let code = PortalCode::from_endpoint(endpoint, secret);
        let identity = HostIdentity::new(HandshakeIdentity::generate(), blocks_per_edge, endpoint);
        let state = HostState::new(
            ChronicleLog::empty(),
            GuestCohort::empty(),
            HostAura::headless(),
            WormholeGate::with_code(code),
        );
        Self::new(socket, world, identity, state)
    }

    pub fn world(&self) -> &dyn PlanetaryWorld { self.world.as_ref() }
    pub fn world_mut(&mut self) -> &mut dyn PlanetaryWorld { self.world.as_mut() }
    pub fn local_addr(&self) -> std::io::Result<SocketAddr> { self.socket.local_addr() }
    pub const fn code(&self) -> PortalCode { self.state.code() }
    pub fn ticket(&self) -> PortalTicket { PortalTicket::new(self.identity.keys().public_bytes(), self.identity.endpoint()) }
    pub fn punch_nat_peer(&mut self, peer: SocketAddr) { self.state.punch_peer(&self.socket, peer); }
    pub fn sync_host_presence(&mut self, presence: CompanionPresence) { self.state.occupy_host(presence); }
    pub fn sync_host_craft(&mut self, craft: PilotCraft) { self.state.pilot_host(craft); }
    pub fn guest_companions(&self, out: &mut Vec<CompanionPresence>) { self.state.guest_companions(out); }
    pub fn guest_crafts(&self, out: &mut Vec<DrivenCraft>) { self.state.guest_crafts(out); }
    pub fn emit_guest_crafts(&self, target: &mut dyn MeshTarget) { self.state.emit_guest_crafts(target); }
    pub fn blend_companions(&mut self, subtick_secs: f32) { self.state.blend_subtick(subtick_secs); }

    pub fn step_host_wildlife(&mut self, listener: Vec3, view_dir: Vec3) -> Option<Vec3> {
        self.state.wildlife_mut().step_host_listener(listener, view_dir, self.world.as_ref())
    }

    pub fn emit_wildlife(&self, cone: SightCone, target: &mut dyn MeshTarget) {
        self.state.wildlife().emit_visible(cone, target);
    }

    pub fn strike_host_boar(&mut self, eye: Vec3, direction: Vec3, power: u8, max_reach: f32) -> (bool, bool) {
        let (hit, drops) = self
            .state
            .wildlife_mut()
            .evaluate_strike(eye, direction, power, max_reach, true, self.world.as_ref());
        let dropped = drops.is_some_and(|d| self.with_chronicled(|w| BoarSounder::cast_meat_drops(d, w)));
        (hit, dropped)
    }

    pub fn present_guest_vessels(&self, sink: &mut dyn world::VesselSink) {
        let mut crafts = Vec::new();
        self.state.guest_crafts(&mut crafts);
        self.state.fleet().present_vessels(&crafts, sink);
    }

    pub fn constrain_with_crafts(&self, position: Vec3, player_height: f32) -> Vec3 {
        let ground = self.world.constrain_to_surface(position, player_height);
        let up = self.world.surface_up_at(position);
        let Some(craft_top) = self.state.constrain_guest_crafts(position, player_height, up) else {
            return ground;
        };
        if craft_top.dot(up) > ground.dot(up) { craft_top } else { ground }
    }

    pub fn board_passenger_seat(&self, eye: Vec3, direction: Vec3) -> Option<(CraftIdentity, Vec3, (Vec3, Vec3, Vec3))> {
        let mut crafts = Vec::new();
        self.state.guest_crafts(&mut crafts);
        self.state.fleet().find_passenger_seat(&crafts, eye, direction)
    }

    pub fn passenger_seat_frame(&self, craft: CraftIdentity, local_seat: Vec3) -> Option<(Vec3, Vec3, Vec3)> {
        let mut crafts = Vec::new();
        self.state.guest_crafts(&mut crafts);
        crafts.into_iter().find(|c| c.identity() == craft).map(|c| c.pose().world_seat_at(local_seat))
    }

    pub fn sample_deck_frame(&self, position: Vec3, player_height: f32) -> Option<DeckFrame> {
        self.state
            .sample_guest_deck(position, player_height)
            .or_else(|| self.world.sample_deck_frame(position, player_height))
    }

    pub fn resolve_deck_pose(&self, craft: CraftIdentity) -> Option<(CraftPose, f32)> {
        let mut crafts = Vec::new();
        self.state.guest_crafts(&mut crafts);
        self.state
            .fleet()
            .resolve_deck_pose(&crafts, craft)
            .or_else(|| self.world.resolve_deck_pose(craft))
    }

    pub fn constrain_on_craft(&self, craft: CraftIdentity, position: Vec3, player_height: f32) -> Option<Vec3> {
        let mut crafts = Vec::new();
        self.state.guest_crafts(&mut crafts);
        self.state
            .fleet()
            .constrain_craft_by_id(&crafts, craft, position, player_height)
            .or_else(|| self.world.constrain_on_craft(craft, position, player_height))
    }

    pub fn flush_disclosures(&mut self) {
        let mut revealed: Vec<(u8, ExposedStratum)> = Vec::new();
        self.world.drain_disclosures(&mut revealed);
        for (face, stratum) in revealed {
            self.state.record_disclosure(face, stratum);
        }
    }

    /// Executes an authoritative local host action through the `ChronicledPlanet` decorator and flushes newly exposed ores.
    pub fn with_chronicled<R>(&mut self, op: impl FnOnce(&mut ChronicledPlanet<'_>) -> R) -> R {
        let out = {
            let mut decorated = ChronicledPlanet::new(self.world.as_mut(), &mut self.state);
            op(&mut decorated)
        };
        self.flush_disclosures();
        out
    }

    pub fn record_local_fact(&mut self, fact: WorldFact) {
        self.state.record_authored_fact(1, fact);
        self.flush_disclosures();
    }

    fn answer_knock(&mut self, peer: SocketAddr, guest_pub: [u8; 32]) {
        let cid = self.state.admit_guest(&self.identity, self.world.as_ref(), peer, guest_pub);
        let mut reply = Vec::with_capacity(44);
        reply.extend_from_slice(b"LWEL");
        reply.extend_from_slice(&self.identity.keys().public_bytes());
        reply.extend_from_slice(&cid.to_le_bytes());
        reply.extend_from_slice(&self.identity.blocks_per_edge().to_le_bytes());
        let _ = self.socket.send_to(&reply, peer);
    }

    fn process_datagram(&mut self, src: SocketAddr, pkt: &[u8]) -> bool {
        if pkt.len() == 37 && &pkt[0..4] == b"LPK1" {
            let ticket = self.ticket();
            self.state.answer_wormhole(&self.socket, src, pkt, ticket);
            return false;
        }
        if pkt.len() >= 36 && &pkt[0..4] == b"LKNK" {
            let mut pub_key = [0u8; 32];
            pub_key.copy_from_slice(&pkt[4..36]);
            self.answer_knock(src, pub_key);
            return false;
        }
        if pkt.len() < 30 || &pkt[0..2] != b"LS" {
            return false;
        }
        let Ok(cid_bytes) = pkt[2..6].try_into() else { return false };
        let cid = u32::from_le_bytes(cid_bytes);
        let mutated = self.state.process_companion_datagram(cid, src, pkt, self.world.as_mut());
        if mutated {
            self.flush_disclosures();
        }
        mutated
    }

    /// Advances P2P UDP sockets, NAT hole-punch pulses, authoritative wildlife, and guest wards (`true` if geometry changed).
    pub fn pump_network(&mut self, dt: f32) -> bool {
        let (pulse_ready, mut mutated) = self.state.step_clocks(&self.socket, dt, self.world.as_mut());
        let mut buf = [0u8; 1500];
        while let Ok((len, src)) = self.socket.recv_from(&mut buf) {
            if self.process_datagram(src, &buf[..len]) {
                mutated = true;
            }
        }
        if pulse_ready {
            self.state.broadcast(&self.socket);
        }
        mutated
    }

    /// Full authoritative simulation tick for headless dedicated `sanctuary` instances.
    pub fn tick(&mut self, dt: f32) -> bool {
        let regrown = self.world.step_surface(dt);
        let net_changed = self.pump_network(dt);
        regrown || net_changed
    }
}
