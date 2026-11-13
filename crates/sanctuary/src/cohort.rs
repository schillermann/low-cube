use std::collections::HashMap;
use std::net::UdpSocket;
use glam::{Quat, Vec3};
use world::{CompanionPresence, PlanetaryWorld, PresenceKinematics};
use crate::chronicle_log::ChronicleLog;
use crate::envelope::{CrackNotice, SanctuaryEnvelope};
use crate::seal::HandshakeIdentity;
use crate::ticket::BeaconRoute;
use crate::ward::{GuestWard, ReturnPath, WardVitals};

/// Cryptographic identity, grid resolution, and NAT beacon configuration of a `SanctuaryHost`.
pub struct HostIdentity {
    keys: HandshakeIdentity,
    blocks_per_edge: u32,
    tron_mode: bool,
    beacon: BeaconRoute,
}

impl HostIdentity {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(keys: HandshakeIdentity, blocks_per_edge: u32, tron_mode: bool, beacon: BeaconRoute) -> Self {
        Self { keys, blocks_per_edge, tron_mode, beacon }
    }

    pub const fn keys(&self) -> &HandshakeIdentity { &self.keys }
    pub const fn blocks_per_edge(&self) -> u32 { self.blocks_per_edge }
    pub const fn tron_mode(&self) -> bool { self.tron_mode }
    pub const fn beacon(&self) -> BeaconRoute { self.beacon }
}

/// Polymorphic local-host avatar presence (`Headless` for dedicated servers, `Occupied` for `--host`).
#[derive(Clone, Copy, Debug)]
pub enum HostPresence {
    Headless,
    Occupied(CompanionPresence),
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

    pub fn admit_guest(&mut self, identity: &HostIdentity, world: &dyn PlanetaryWorld, path: ReturnPath, guest_pub: [u8; 32]) -> u32 {
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
        self.wards.entry(cid).or_insert_with(|| GuestWard::new(seal, path, pres, WardVitals::fresh()));
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

    pub fn broadcast(&mut self, socket: &UdpSocket, chronicle: &ChronicleLog, host_presence: HostPresence, crack: CrackNotice) {
        let (mut payload, mut scratch, mut roster) = (Vec::new(), Vec::new(), Vec::new());
        if let HostPresence::Occupied(p) = host_presence {
            roster.push(p);
        }
        for ward in self.wards.values() {
            roster.push(ward.presence());
        }
        let total_facts = chronicle.total();
        for (&cid, ward) in &mut self.wards {
            if let Some((base_seq, facts)) = chronicle.batch_from(ward.ack_seq()) {
                SanctuaryEnvelope::HostChronicle { base_seq, facts }.inscribe(&mut payload);
                ward.send_sealed(socket, cid, &payload, &mut scratch);
            }
            let others: Vec<CompanionPresence> = roster.iter().copied().filter(|p| !p.same_slot(cid)).collect();
            SanctuaryEnvelope::HostPulse { total_facts, crack, companions: others }.inscribe(&mut payload);
            ward.send_sealed(socket, cid, &payload, &mut scratch);
        }
    }
}

/// Mutable chronicle, guest cohort, local presence, and transient crack state of `SanctuaryHost`.
pub struct HostState {
    pub(crate) chronicle: ChronicleLog,
    pub(crate) cohort: GuestCohort,
    pub(crate) presence: HostPresence,
    pub(crate) crack: CrackNotice,
}

impl HostState {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(chronicle: ChronicleLog, cohort: GuestCohort, presence: HostPresence, crack: CrackNotice) -> Self {
        Self { chronicle, cohort, presence, crack }
    }
}
