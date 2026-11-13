use std::net::{SocketAddr, UdpSocket};
use world::{CompanionPresence, ExposedStratum, PlanetaryWorld, WorldFact};
use crate::chronicle_log::{ChronicleLog, DirtySlices};
use crate::cohort::{GuestCohort, HostIdentity, HostPresence, HostState};
use crate::envelope::{CrackNotice, SanctuaryEnvelope};
use crate::seal::HandshakeIdentity;
use crate::ticket::{BeaconRoute, PortalTicket};
use crate::ward::ReturnPath;

/// Authoritative multiplayer World-Cube sanctuary enforcing anti-cheat and zero-knowledge ore disclosure.
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

    /// Secondary constructor binding a non-blocking UDP socket around an authoritative `PlanetaryWorld`.
    pub fn bind(
        bind_addr: &str,
        world: Box<dyn PlanetaryWorld>,
        blocks_per_edge: u32,
        tron_mode: bool,
        beacon: BeaconRoute,
    ) -> std::io::Result<Self> {
        let socket = UdpSocket::bind(bind_addr)?;
        socket.set_nonblocking(true)?;
        let identity = HostIdentity::new(HandshakeIdentity::generate(), blocks_per_edge, tron_mode, beacon);
        let state = HostState::new(ChronicleLog::empty(), GuestCohort::empty(), HostPresence::Headless, CrackNotice::Quiet);
        Ok(Self::new(socket, world, identity, state))
    }

    pub fn world(&self) -> &dyn PlanetaryWorld { self.world.as_ref() }
    pub fn world_mut(&mut self) -> &mut dyn PlanetaryWorld { self.world.as_mut() }
    pub fn slices(&self) -> &DirtySlices { self.state.chronicle.slices() }
    pub fn local_addr(&self) -> std::io::Result<SocketAddr> { self.socket.local_addr() }

    pub fn ticket(&self) -> PortalTicket {
        let bound = self.socket.local_addr().expect("host socket bound");
        let shareable = PortalTicket::shareable_local_endpoint(bound);
        PortalTicket::new(self.identity.keys().public_bytes(), shareable, self.identity.beacon())
    }

    pub fn sync_host_presence(&mut self, presence: CompanionPresence) {
        self.state.presence = HostPresence::Occupied(presence.with_slot(1));
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
            self.state.crack = CrackNotice::Active { eye, direction, stage };
        }
        self.state.chronicle.record(fact, faces);
        self.flush_disclosures();
    }

    fn answer_knock(&mut self, path: ReturnPath, guest_pub: [u8; 32]) {
        let cid = self.state.cohort.admit_guest(&self.identity, self.world.as_ref(), path, guest_pub);
        let mut reply = Vec::with_capacity(45);
        reply.extend_from_slice(b"LWEL");
        reply.extend_from_slice(&self.identity.keys().public_bytes());
        reply.extend_from_slice(&cid.to_le_bytes());
        reply.extend_from_slice(&self.identity.blocks_per_edge().to_le_bytes());
        reply.push(u8::from(self.identity.tron_mode()));
        path.transmit(&self.socket, &reply);
    }

    fn apply_guest_petition(&mut self, cid: u32, fact: WorldFact) -> bool {
        let Some(ward) = self.state.cohort.ward_mut(cid) else { return false };
        match fact {
            WorldFact::Loosened { eye, direction, stage } => {
                if !ward.eye_within_reach(eye, direction) { return false; }
                if self.world.loosen_surface(eye, direction, stage.min(4)).is_some() {
                    self.state.crack = CrackNotice::Active { eye, direction, stage: stage.min(4) };
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

    fn process_inner_frame(&mut self, path: ReturnPath, pkt: &[u8]) -> bool {
        if pkt.len() >= 36 && &pkt[0..4] == b"LKNK" {
            let mut pub_key = [0u8; 32];
            pub_key.copy_from_slice(&pkt[4..36]);
            self.answer_knock(path, pub_key);
            return false;
        }
        if pkt.len() < 30 || &pkt[0..2] != b"LS" {
            return false;
        }
        let Ok(cid_bytes) = pkt[2..6].try_into() else { return false };
        let cid = u32::from_le_bytes(cid_bytes);
        let Some(body) = self.state.cohort.ward_mut(cid).and_then(|w| w.open_datagram(path, pkt)) else {
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

    /// Advances network sockets, guest wards, beacon registration, and broadcasts pulses (`true` if geometry changed).
    pub fn pump_network(&mut self, dt: f32) -> bool {
        let pulse_ready = self.state.cohort.advance_wards(dt);
        let mut mutated = false;
        let mut buf = [0u8; 1500];
        while let Ok((len, src)) = self.socket.recv_from(&mut buf) {
            let pkt = &buf[..len];
            if pkt.len() >= 10 && &pkt[0..4] == b"LBPU" {
                if let Some(peer) = PortalTicket::read_v4(&pkt[4..10]) {
                    let _ = self.socket.send_to(b"LPCH", peer);
                }
            } else if pkt.len() > 14 && &pkt[0..4] == b"LBFW" {
                let mut portal_id = [0u8; 8];
                portal_id.copy_from_slice(&pkt[4..12]);
                let slot = u16::from_le_bytes([pkt[12], pkt[13]]);
                let path = ReturnPath::Relayed { beacon: src, portal_id, slot };
                if self.process_inner_frame(path, &pkt[14..]) {
                    mutated = true;
                }
            } else if self.process_inner_frame(ReturnPath::Direct(src), pkt) {
                mutated = true;
            }
        }
        if pulse_ready {
            if let Some(beacon_addr) = self.identity.beacon().address() {
                let bound = self.socket.local_addr().expect("host bound");
                let lan = PortalTicket::shareable_local_endpoint(bound);
                let mut reg = Vec::with_capacity(18);
                reg.extend_from_slice(b"LBHO");
                reg.extend_from_slice(&self.identity.keys().portal_id());
                PortalTicket::push_v4(lan, &mut reg);
                let _ = self.socket.send_to(&reg, beacon_addr);
            }
            let crack = self.state.crack;
            self.state.crack = CrackNotice::Quiet;
            self.state.cohort.broadcast(&self.socket, &self.state.chronicle, self.state.presence, crack);
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
