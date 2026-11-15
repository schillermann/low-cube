use std::collections::HashMap;
use std::net::{SocketAddr, UdpSocket};
use glam::{Quat, Vec3};
use world::{CompanionPresence, ExposedStratum, PlanetaryWorld, PresenceKinematics, WorldFact};
use crate::chronicle_log::ChronicleLog;
use crate::chronicled_planet::FactJournal;
use crate::envelope::{CrackNotice, SanctuaryEnvelope};
use crate::seal::HandshakeIdentity;
use crate::ticket::PortalTicket;
use crate::ward::{GuestWard, WardVitals};
use crate::warded_planet::WardedPlanet;
use crate::wormhole::{PortalCode, WormholeGate};

/// Cryptographic identity, grid resolution, and advertised P2P endpoint of a `SanctuaryHost`.
pub struct HostIdentity {
    keys: HandshakeIdentity,
    blocks_per_edge: u32,
    tron_mode: bool,
    endpoint: SocketAddr,
}

impl HostIdentity {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(keys: HandshakeIdentity, blocks_per_edge: u32, tron_mode: bool, endpoint: SocketAddr) -> Self {
        Self { keys, blocks_per_edge, tron_mode, endpoint }
    }

    pub const fn keys(&self) -> &HandshakeIdentity { &self.keys }
    pub const fn blocks_per_edge(&self) -> u32 { self.blocks_per_edge }
    pub const fn tron_mode(&self) -> bool { self.tron_mode }
    pub const fn endpoint(&self) -> SocketAddr { self.endpoint }
}

/// Polymorphic local-host avatar presence (`Headless` for dedicated servers, `Occupied` for `--host`).
#[derive(Clone, Copy, Debug)]
pub enum HostPresence {
    Headless,
    Occupied(CompanionPresence),
}

/// Author-tagged transient dig crack preventing echo feedback to its originating companion.
#[derive(Clone, Copy, Debug)]
pub struct PeerCrack {
    author_slot: u32,
    notice: CrackNotice,
}

impl PeerCrack {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(author_slot: u32, notice: CrackNotice) -> Self {
        Self { author_slot, notice }
    }

    pub const fn authored_by(self, slot: u32) -> bool {
        self.author_slot == slot
    }

    pub const fn notice(self) -> CrackNotice {
        self.notice
    }
}

/// Strictly encapsulated local host presence and per-companion dig-crack queue for cohort broadcast.
pub struct HostAura {
    presence: HostPresence,
    cracks: Vec<PeerCrack>,
}

impl HostAura {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(presence: HostPresence, cracks: Vec<PeerCrack>) -> Self {
        Self { presence, cracks }
    }

    pub const fn headless() -> Self {
        Self::new(HostPresence::Headless, Vec::new())
    }

    pub fn occupy(&mut self, presence: CompanionPresence) {
        self.presence = HostPresence::Occupied(presence.with_slot(1));
    }

    pub const fn presence(&self) -> HostPresence {
        self.presence
    }

    pub fn record_crack(&mut self, author_slot: u32, notice: CrackNotice) {
        if self.cracks.len() >= 16 {
            self.cracks.remove(0);
        }
        self.cracks.push(PeerCrack::new(author_slot, notice));
    }

    pub fn drain_cracks(&mut self) -> Vec<PeerCrack> {
        std::mem::take(&mut self.cracks)
    }
}

/// Connected companion wards indexed by `companion_id` and X25519 public key.
pub struct GuestCohort {
    wards: HashMap<u32, GuestWard>,
    key_index: HashMap<[u8; 32], u32>,
    next_id: u32,
    pulse_clock: f32,
}

impl GuestCohort {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(wards: HashMap<u32, GuestWard>, key_index: HashMap<[u8; 32], u32>, next_id: u32, pulse_clock: f32) -> Self {
        Self { wards, key_index, next_id, pulse_clock }
    }

    pub fn empty() -> Self {
        Self::new(HashMap::new(), HashMap::new(), 2, 0.0)
    }

    pub fn ward_mut(&mut self, cid: u32) -> Option<&mut GuestWard> {
        self.wards.get_mut(&cid)
    }

    pub fn guest_companions(&self, out: &mut Vec<CompanionPresence>) {
        out.clear();
        for ward in self.wards.values() {
            out.push(ward.presence());
        }
    }

    pub fn admit_guest(&mut self, identity: &HostIdentity, world: &dyn PlanetaryWorld, peer: SocketAddr, guest_pub: [u8; 32]) -> u32 {
        let cid = if let Some(&existing) = self.key_index.get(&guest_pub) {
            existing
        } else {
            let id = self.next_id;
            self.next_id = self.next_id.wrapping_add(1).max(2);
            self.key_index.insert(guest_pub, id);
            id
        };
        let seal = identity.keys().establish(guest_pub, false);
        let spawn = world.constrain_to_surface(Vec3::new(0.0, 64.0, 0.0), 0.0);
        let kin = PresenceKinematics::new(spawn, Quat::IDENTITY, Quat::IDENTITY, (0.0, 0.0));
        let pres = CompanionPresence::new(cid, kin, (0.0, 0.0, 0.0, 0.0), ([0; 9], [0; 4]));
        self.wards.entry(cid).or_insert_with(|| GuestWard::new(seal, peer, pres, WardVitals::fresh()));
        cid
    }

    pub fn advance_wards(&mut self, dt: f32) -> bool {
        for ward in self.wards.values_mut() {
            ward.step(dt);
        }
        self.wards.retain(|_, w| w.alive());
        self.pulse_clock += dt;
        if self.pulse_clock >= 0.05 {
            self.pulse_clock = 0.0;
            true
        } else {
            false
        }
    }

    pub fn broadcast(&mut self, socket: &UdpSocket, chronicle: &ChronicleLog, aura: &mut HostAura) {
        let (mut payload, mut scratch, mut roster) = (Vec::new(), Vec::new(), Vec::new());
        if let HostPresence::Occupied(p) = aura.presence() {
            roster.push(p);
        }
        for ward in self.wards.values() {
            roster.push(ward.presence());
        }
        let peer_cracks = aura.drain_cracks();
        let total_facts = chronicle.total();
        for (&cid, ward) in &mut self.wards {
            if let Some((base_seq, facts)) = chronicle.batch_from(ward.ack_seq()) {
                SanctuaryEnvelope::HostChronicle { base_seq, facts }.inscribe(&mut payload);
                ward.send_sealed(socket, cid, &payload, &mut scratch);
            }
            let others: Vec<CompanionPresence> = roster.iter().copied().filter(|p| !p.same_slot(cid)).collect();
            let cracks: Vec<CrackNotice> = peer_cracks
                .iter()
                .copied()
                .filter(|c| !c.authored_by(cid))
                .map(PeerCrack::notice)
                .collect();
            SanctuaryEnvelope::HostPulse { total_facts, cracks, companions: others }.inscribe(&mut payload);
            ward.send_sealed(socket, cid, &payload, &mut scratch);
        }
    }
}

/// Strictly encapsulated chronicle, guest cohort, host aura, and SPAKE2 wormhole gate of `SanctuaryHost`.
pub struct HostState {
    chronicle: ChronicleLog,
    cohort: GuestCohort,
    aura: HostAura,
    wormhole: WormholeGate,
}

impl HostState {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(
        chronicle: ChronicleLog,
        cohort: GuestCohort,
        aura: HostAura,
        wormhole: WormholeGate,
    ) -> Self {
        Self { chronicle, cohort, aura, wormhole }
    }

    pub const fn code(&self) -> PortalCode { self.wormhole.code() }
    pub fn punch_peer(&mut self, socket: &UdpSocket, peer: SocketAddr) { self.wormhole.punch_peer(socket, peer); }
    pub fn occupy_host(&mut self, presence: CompanionPresence) { self.aura.occupy(presence); }
    pub fn guest_companions(&self, out: &mut Vec<CompanionPresence>) { self.cohort.guest_companions(out); }
    pub fn answer_wormhole(&mut self, socket: &UdpSocket, src: SocketAddr, pkt: &[u8], ticket: PortalTicket) {
        self.wormhole.answer_hello(socket, src, pkt, ticket);
    }

    pub fn admit_guest(&mut self, identity: &HostIdentity, world: &dyn PlanetaryWorld, peer: SocketAddr, guest_pub: [u8; 32]) -> u32 {
        self.cohort.admit_guest(identity, world, peer, guest_pub)
    }

    pub fn record_authored_fact(&mut self, author_slot: u32, fact: WorldFact) {
        if let WorldFact::Loosened { eye, direction, stage } = fact {
            self.aura.record_crack(author_slot, CrackNotice::new(eye, direction, stage.min(4)));
        }
        self.chronicle.record(fact);
    }

    pub fn record_disclosure(&mut self, face: u8, stratum: ExposedStratum) {
        self.chronicle.record(WorldFact::StratumDisclosed { face, stratum });
    }

    pub fn process_companion_datagram(&mut self, cid: u32, src: SocketAddr, pkt: &[u8], world: &mut dyn PlanetaryWorld) -> bool {
        let Some(body) = self.cohort.ward_mut(cid).and_then(|w| w.open_datagram(src, pkt)) else {
            return false;
        };
        match SanctuaryEnvelope::decode(&body) {
            Some(SanctuaryEnvelope::GuestPulse { ack_seq, presence }) => {
                if let Some(ward) = self.cohort.ward_mut(cid) {
                    ward.absorb_pulse(cid, ack_seq, presence, world);
                }
                false
            }
            Some(SanctuaryEnvelope::GuestPetition { fact }) if !matches!(fact, WorldFact::StratumDisclosed { .. }) => {
                let enacted = self
                    .cohort
                    .ward_mut(cid)
                    .is_some_and(|ward| fact.enact_on(&mut WardedPlanet::new(world, ward)));
                if enacted {
                    self.record_authored_fact(cid, fact);
                }
                enacted
            }
            _ => false,
        }
    }

    pub fn step_clocks(&mut self, socket: &UdpSocket, dt: f32) -> bool {
        self.wormhole.step(socket, dt);
        self.cohort.advance_wards(dt)
    }

    pub fn broadcast(&mut self, socket: &UdpSocket) {
        self.cohort.broadcast(socket, &self.chronicle, &mut self.aura);
    }
}

impl FactJournal for HostState {
    fn inscribe_fact(&mut self, fact: WorldFact) {
        self.record_authored_fact(1, fact);
    }
}
