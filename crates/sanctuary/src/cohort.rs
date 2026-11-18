use std::collections::HashMap;
use std::net::{SocketAddr, UdpSocket};
use glam::{Quat, Vec3};
use world::{
    CompanionPresence, CraftBlueprint, CraftIdentity, DrivenCraft, ExposedStratum, MeshTarget,
    PilotCraft, PlanetaryWorld, PresenceKinematics, WorldFact,
};
use crate::aura::{HostAura, HostIdentity, HostPresence, PeerCrack};
use crate::chronicle_log::ChronicleLog;
use crate::chronicled_planet::FactJournal;
use crate::envelope::{CrackNotice, SanctuaryEnvelope};
use crate::fleet_ledger::FleetLedger;
use crate::ticket::PortalTicket;
use crate::ward::{GuestWard, WardVitals};
use crate::warded_planet::WardedPlanet;
use crate::wormhole::{PortalCode, WormholeGate};

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
            out.push(ward.smooth_presence());
        }
    }

    pub fn guest_crafts(&self, out: &mut Vec<DrivenCraft>) {
        out.clear();
        for ward in self.wards.values() {
            ward.push_smooth_craft(out);
        }
    }

    pub fn blend_subtick(&mut self, subtick_secs: f32) {
        for ward in self.wards.values_mut() {
            ward.blend_subtick(subtick_secs);
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

    pub fn advance_wards(&mut self, dt: f32, abandoned_out: &mut Vec<(u32, PilotCraft)>) -> bool {
        for ward in self.wards.values_mut() {
            ward.step(dt);
        }
        let expired: Vec<u32> = self
            .wards
            .iter()
            .filter_map(|(&cid, w)| (!w.alive()).then_some(cid))
            .collect();
        for cid in expired {
            if let Some(dead) = self.wards.remove(&cid) {
                abandoned_out.push((cid, dead.into_pilot_craft()));
            }
        }
        self.pulse_clock += dt;
        if self.pulse_clock >= 0.025 {
            self.pulse_clock = 0.0;
            true
        } else {
            false
        }
    }

    pub fn broadcast(&mut self, socket: &UdpSocket, chronicle: &ChronicleLog, aura: &mut HostAura) {
        let (mut payload, mut scratch, mut roster, mut crafts) = (Vec::new(), Vec::new(), Vec::new(), Vec::new());
        if let HostPresence::Occupied(p) = aura.presence() {
            roster.push(p);
            aura.push_host_craft(&mut crafts);
        }
        for ward in self.wards.values() {
            roster.push(ward.presence());
            ward.push_authoritative_craft(&mut crafts);
        }
        let peer_cracks = aura.drain_cracks();
        let total_facts = chronicle.total();
        for (&cid, ward) in &mut self.wards {
            if let Some((base_seq, facts)) = chronicle.batch_from(ward.ack_seq()) {
                SanctuaryEnvelope::HostChronicle { base_seq, facts }.inscribe(&mut payload);
                ward.send_sealed(socket, cid, &payload, &mut scratch);
            }
            let others: Vec<CompanionPresence> = roster.iter().copied().filter(|p| !p.same_slot(cid)).collect();
            let other_crafts: Vec<DrivenCraft> = crafts.iter().copied().filter(|c| !c.same_slot(cid)).collect();
            let cracks: Vec<CrackNotice> = peer_cracks
                .iter()
                .copied()
                .filter(|c| !c.authored_by(cid))
                .map(PeerCrack::notice)
                .collect();
            SanctuaryEnvelope::HostPulse {
                total_facts,
                cracks,
                companions: others,
                crafts: other_crafts,
            }
            .inscribe(&mut payload);
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
    pub fn pilot_host(&mut self, craft: PilotCraft) { self.aura.pilot(craft); }
    pub fn guest_companions(&self, out: &mut Vec<CompanionPresence>) { self.cohort.guest_companions(out); }
    pub fn guest_crafts(&self, out: &mut Vec<DrivenCraft>) { self.cohort.guest_crafts(out); }
    pub const fn fleet(&self) -> &FleetLedger { self.aura.fleet() }
    pub fn blend_subtick(&mut self, subtick_secs: f32) { self.cohort.blend_subtick(subtick_secs); }
    pub fn answer_wormhole(&mut self, socket: &UdpSocket, src: SocketAddr, pkt: &[u8], ticket: PortalTicket) {
        self.wormhole.answer_hello(socket, src, pkt, ticket);
    }

    pub fn emit_guest_crafts(&self, target: &mut dyn MeshTarget) {
        let mut crafts = Vec::new();
        self.cohort.guest_crafts(&mut crafts);
        self.aura.fleet().emit_crafts(&crafts, target);
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
            Some(SanctuaryEnvelope::GuestPulse { ack_seq, presence, craft }) => {
                let vetted_craft = match craft {
                    PilotCraft::Driving(dc) if !self.aura.fleet().helmed_by(dc.identity(), cid) => PilotCraft::Pedestrian,
                    other => other,
                };
                if let Some(ward) = self.cohort.ward_mut(cid) {
                    ward.absorb_pulse(cid, ack_seq, presence, vetted_craft, world);
                }
                false
            }
            Some(SanctuaryEnvelope::GuestPetition { fact }) if !matches!(fact, WorldFact::StratumDisclosed { .. }) => {
                let stamped = match fact {
                    WorldFact::HelmClaimed { craft, eye, direction, .. } => {
                        WorldFact::HelmClaimed { craft, pilot_slot: cid, eye, direction }
                    }
                    other => other,
                };
                let fleet = self.aura.fleet_mut();
                let enacted = self
                    .cohort
                    .ward_mut(cid)
                    .is_some_and(|ward| stamped.enact_on(&mut WardedPlanet::new(world, ward, fleet, cid)));
                if enacted {
                    self.record_authored_fact(cid, stamped);
                }
                enacted
            }
            _ => false,
        }
    }

    pub fn step_clocks(&mut self, socket: &UdpSocket, dt: f32, world: &mut dyn PlanetaryWorld) -> (bool, bool) {
        self.wormhole.step(socket, dt);
        let mut abandoned = Vec::new();
        let pulse_ready = self.cohort.advance_wards(dt, &mut abandoned);
        let mut parked_any = false;
        for (cid, pilot_craft) in abandoned {
            if let PilotCraft::Driving(dc) = pilot_craft {
                if self.aura.fleet_mut().park_by_id(dc.identity(), dc.pose(), world) {
                    self.chronicle.record(WorldFact::HelmReleased {
                        craft: dc.identity(),
                        pose: dc.pose(),
                    });
                    parked_any = true;
                }
            }
            self.aura.fleet_mut().release_slot(cid);
        }
        (pulse_ready, parked_any)
    }

    pub fn broadcast(&mut self, socket: &UdpSocket) {
        self.cohort.broadcast(socket, &self.chronicle, &mut self.aura);
    }
}

impl FactJournal for HostState {
    fn inscribe_fact(&mut self, fact: WorldFact) {
        self.record_authored_fact(1, fact);
    }

    fn local_slot(&self) -> u32 {
        1
    }

    fn register_local_blueprint(&mut self, blueprint: CraftBlueprint) -> bool {
        self.aura.fleet_mut().register_blueprint(blueprint)
    }

    fn claim_local_helm(&mut self, craft: CraftIdentity, pilot_slot: u32) -> bool {
        self.aura.fleet_mut().claim_helm(craft, pilot_slot)
    }

    fn release_local_helm(&mut self, craft: CraftIdentity) {
        let _ = self.aura.fleet_mut().release_helm(craft, 1);
    }
}
