use std::net::{SocketAddr, UdpSocket};
use world::{CompanionPresence, ExposedStratum, PlanetaryWorld, WorldFact};
use crate::chronicle_log::{ChronicleLog, DirtySlices};
use crate::cohort::{GuestCohort, HostAura, HostIdentity, HostPresence, HostState};
use crate::envelope::{CrackNotice, SanctuaryEnvelope};
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
    pub fn bind(
        bind_addr: &str,
        world: Box<dyn PlanetaryWorld>,
        blocks_per_edge: u32,
        tron_mode: bool,
    ) -> std::io::Result<Self> {
        let secret = PortalCode::draw_secret();
        let socket = Self::bind_slot_socket(bind_addr, secret)?;
        socket.set_nonblocking(true)?;
        let endpoint = NatAperture::interactive().open_host(&socket);
        Ok(Self::assemble_bound(socket, endpoint, secret, world, blocks_per_edge, tron_mode))
    }

    /// Binds a UDP socket advertising a known external NAT endpoint (for port-forwarded or simulated NAT).
    pub fn bind_with_endpoint(
        bind_addr: &str,
        advertised: SocketAddr,
        world: Box<dyn PlanetaryWorld>,
        blocks_per_edge: u32,
        tron_mode: bool,
    ) -> std::io::Result<Self> {
        let secret = PortalCode::draw_secret();
        let socket = Self::bind_slot_socket(bind_addr, secret)?;
        socket.set_nonblocking(true)?;
        Ok(Self::assemble_bound(socket, advertised, secret, world, blocks_per_edge, tron_mode))
    }

    fn assemble_bound(
        socket: UdpSocket,
        endpoint: SocketAddr,
        secret: u8,
        world: Box<dyn PlanetaryWorld>,
        blocks_per_edge: u32,
        tron_mode: bool,
    ) -> Self {
        let code = PortalCode::from_endpoint(endpoint, secret);
        let identity = HostIdentity::new(HandshakeIdentity::generate(), blocks_per_edge, tron_mode, endpoint);
        let aura = HostAura::new(HostPresence::Headless, CrackNotice::Quiet);
        let state = HostState::new(ChronicleLog::empty(), GuestCohort::empty(), aura, WormholeGate::with_code(code));
        Self::new(socket, world, identity, state)
    }

    pub fn world(&self) -> &dyn PlanetaryWorld { self.world.as_ref() }
    pub fn world_mut(&mut self) -> &mut dyn PlanetaryWorld { self.world.as_mut() }
    pub fn slices(&self) -> &DirtySlices { self.state.chronicle.slices() }
    pub fn local_addr(&self) -> std::io::Result<SocketAddr> { self.socket.local_addr() }
    pub const fn code(&self) -> PortalCode { self.state.wormhole.code() }

    pub fn ticket(&self) -> PortalTicket {
        PortalTicket::new(self.identity.keys().public_bytes(), self.identity.endpoint())
    }

    /// Punches an outbound UDP hole through the host's FRITZ!Box / NAT router toward `peer`.
    pub fn punch_nat_peer(&mut self, peer: SocketAddr) {
        self.state.wormhole.punch_peer(&self.socket, peer);
    }

    pub fn sync_host_presence(&mut self, presence: CompanionPresence) {
        self.state.aura.presence = HostPresence::Occupied(presence.with_slot(1));
    }

    pub fn guest_companions(&self, out: &mut Vec<CompanionPresence>) {
        self.state.cohort.guest_companions(out);
    }

    pub fn flush_disclosures(&mut self) {
        let mut revealed: Vec<(u8, ExposedStratum)> = Vec::new();
        self.world.drain_disclosures(&mut revealed);
        let faces = self.world.face_count();
        for (face, stratum) in revealed {
            self.state.chronicle.record(WorldFact::StratumDisclosed { face, stratum }, faces);
        }
    }

    pub fn record_local_fact(&mut self, fact: WorldFact) {
        let faces = self.world.face_count();
        if let WorldFact::Loosened { eye, direction, stage } = fact {
            self.state.aura.crack = CrackNotice::Active { eye, direction, stage };
        }
        self.state.chronicle.record(fact, faces);
        self.flush_disclosures();
    }

    fn answer_knock(&mut self, peer: SocketAddr, guest_pub: [u8; 32]) {
        let cid = self.state.cohort.admit_guest(&self.identity, self.world.as_ref(), peer, guest_pub);
        let mut reply = Vec::with_capacity(45);
        reply.extend_from_slice(b"LWEL");
        reply.extend_from_slice(&self.identity.keys().public_bytes());
        reply.extend_from_slice(&cid.to_le_bytes());
        reply.extend_from_slice(&self.identity.blocks_per_edge().to_le_bytes());
        reply.push(u8::from(self.identity.tron_mode()));
        let _ = self.socket.send_to(&reply, peer);
    }

    fn apply_guest_petition(&mut self, cid: u32, fact: WorldFact) -> bool {
        let Some(ward) = self.state.cohort.ward_mut(cid) else { return false };
        match fact {
            WorldFact::Loosened { eye, direction, stage } => {
                if !ward.eye_within_reach(eye, direction) { return false; }
                if self.world.loosen_surface(eye, direction, stage.min(4)).is_some() {
                    self.state.aura.crack = CrackNotice::Active { eye, direction, stage: stage.min(4) };
                    self.state.chronicle.record(fact, self.world.face_count());
                    return true;
                }
                false
            }
            WorldFact::Harvested { eye, direction } => {
                if !ward.eye_within_reach(eye, direction) || !ward.consume_harvest_token() { return false; }
                if self.world.harvest_surface(eye, direction).is_some() {
                    ward.credit_blocks(1);
                    self.record_local_fact(fact);
                    return true;
                }
                false
            }
            WorldFact::MoldPlaced { eye, direction, avatar_pos, mold } => {
                if !ward.eye_within_reach(eye, direction) || !ward.spend_block() { return false; }
                if self.world.place_molded_surface_block(eye, direction, avatar_pos, mold) {
                    self.record_local_fact(fact);
                    return true;
                }
                ward.credit_blocks(1);
                false
            }
            WorldFact::TimberStruck { eye, direction } => {
                if !ward.eye_within_reach(eye, direction) { return false; }
                let outcome = self.world.strike_timber(eye, direction);
                if outcome.struck() {
                    if outcome.felled() { ward.credit_blocks(8); }
                    self.record_local_fact(fact);
                    return true;
                }
                false
            }
            WorldFact::Foraged { eye, direction } => {
                if !ward.eye_within_reach(eye, direction) { return false; }
                if self.world.forage_surface(eye, direction).gathered() {
                    ward.credit_blocks(4);
                    self.record_local_fact(fact);
                    return true;
                }
                false
            }
            WorldFact::RigAdjusted { eye, direction, delta_steps, rotate_quarter } => {
                if !ward.eye_within_reach(eye, direction) { return false; }
                if self.world.adjust_rig(eye, direction, delta_steps.clamp(-1, 1), rotate_quarter) {
                    self.record_local_fact(fact);
                    return true;
                }
                false
            }
            WorldFact::RigLowered { eye, direction } => {
                if !ward.eye_within_reach(eye, direction) { return false; }
                if self.world.lower_rig(eye, direction) {
                    self.record_local_fact(fact);
                    return true;
                }
                false
            }
            WorldFact::StratumDisclosed { .. } => false,
        }
    }

    fn process_datagram(&mut self, src: SocketAddr, pkt: &[u8]) -> bool {
        if pkt.len() == 37 && &pkt[0..4] == b"LPK1" {
            let ticket = self.ticket();
            self.state.wormhole.answer_hello(&self.socket, src, pkt, ticket);
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
        let Some(body) = self.state.cohort.ward_mut(cid).and_then(|w| w.open_datagram(src, pkt)) else {
            return false;
        };
        match SanctuaryEnvelope::decode(&body) {
            Some(SanctuaryEnvelope::GuestPulse { ack_seq, presence }) => {
                if let Some(ward) = self.state.cohort.ward_mut(cid) {
                    ward.absorb_pulse(cid, ack_seq, presence, self.world.as_ref());
                }
                false
            }
            Some(SanctuaryEnvelope::GuestPetition { fact }) => self.apply_guest_petition(cid, fact),
            _ => false,
        }
    }

    /// Advances P2P UDP sockets, NAT hole-punch pulses, and guest wards (`true` if geometry changed).
    pub fn pump_network(&mut self, dt: f32) -> bool {
        self.state.wormhole.step(&self.socket, dt);
        let pulse_ready = self.state.cohort.advance_wards(dt);
        let mut mutated = false;
        let mut buf = [0u8; 1500];
        while let Ok((len, src)) = self.socket.recv_from(&mut buf) {
            if self.process_datagram(src, &buf[..len]) {
                mutated = true;
            }
        }
        if pulse_ready {
            let crack = self.state.aura.crack;
            self.state.aura.crack = CrackNotice::Quiet;
            self.state.cohort.broadcast(&self.socket, &self.state.chronicle, self.state.aura.presence, crack);
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
