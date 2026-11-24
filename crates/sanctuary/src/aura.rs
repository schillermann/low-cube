use std::net::SocketAddr;
use glam::Vec3;
use world::{
    CompanionPresence, DrivenCraft, PilotCraft, PlanetaryWorld, ProximityUtterance,
    SolarProportions,
};
use crate::admin_ward::SpawnBeacons;
use crate::envelope::CrackNotice;
use crate::fleet_ledger::FleetLedger;
use crate::host_sounder::HostSounder;
use crate::seal::HandshakeIdentity;

/// Cryptographic identity, solar-system proportions, and advertised P2P endpoint of a `SanctuaryHost`.
pub struct HostIdentity {
    keys: HandshakeIdentity,
    proportions: SolarProportions,
    endpoint: SocketAddr,
}

impl HostIdentity {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn with_proportions(keys: HandshakeIdentity, proportions: SolarProportions, endpoint: SocketAddr) -> Self {
        Self { keys, proportions, endpoint }
    }

    pub const fn new(keys: HandshakeIdentity, blocks_per_edge: u32, endpoint: SocketAddr) -> Self {
        Self::with_proportions(keys, SolarProportions::true_scale(blocks_per_edge), endpoint)
    }

    pub const fn keys(&self) -> &HandshakeIdentity { &self.keys }
    pub const fn blocks_per_edge(&self) -> u32 { self.proportions.earth_blocks() }
    pub const fn proportions(&self) -> SolarProportions { self.proportions }
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

/// Transient crack queue, host chat inbox, and admin `SpawnBeacons` registry (`<= 4` fields).
pub struct AuraEvents {
    cracks: Vec<PeerCrack>,
    utterances: Vec<ProximityUtterance>,
    beacons: SpawnBeacons,
}

impl AuraEvents {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(
        cracks: Vec<PeerCrack>,
        utterances: Vec<ProximityUtterance>,
        beacons: SpawnBeacons,
    ) -> Self {
        Self { cracks, utterances, beacons }
    }

    pub const fn empty() -> Self {
        Self::new(Vec::new(), Vec::new(), SpawnBeacons::empty())
    }
}

/// Strictly encapsulated local host seat, transient `AuraEvents`, `FleetLedger`, and authoritative `HostSounder`.
pub struct HostAura {
    seat: (HostPresence, PilotCraft),
    events: AuraEvents,
    fleet: FleetLedger,
    wildlife: HostSounder,
}

impl HostAura {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn with_events(
        seat: (HostPresence, PilotCraft),
        events: AuraEvents,
        fleet: FleetLedger,
        wildlife: HostSounder,
    ) -> Self {
        Self { seat, events, fleet, wildlife }
    }

    pub const fn new(
        seat: (HostPresence, PilotCraft),
        cracks: Vec<PeerCrack>,
        fleet: FleetLedger,
        wildlife: HostSounder,
    ) -> Self {
        Self::with_events(seat, AuraEvents::new(cracks, Vec::new(), SpawnBeacons::empty()), fleet, wildlife)
    }

    pub const fn headless() -> Self {
        Self::with_events(
            (HostPresence::Headless, PilotCraft::Pedestrian),
            AuraEvents::empty(),
            FleetLedger::empty(),
            HostSounder::seeded(),
        )
    }

    pub fn occupy(&mut self, presence: CompanionPresence) {
        self.seat.0 = HostPresence::Occupied(presence.with_slot(1));
    }

    pub fn pilot(&mut self, craft: PilotCraft) {
        self.seat.1 = craft.with_slot(1);
    }

    pub const fn presence(&self) -> HostPresence {
        self.seat.0
    }

    pub fn occupied_position(&self) -> Option<Vec3> {
        let HostPresence::Occupied(p) = self.seat.0 else {
            return None;
        };
        Some(p.transfer(|_, kin, _, _| kin.transfer(|pos, _, _, _, _| pos)))
    }

    pub fn pedestrian_observer(&self, world: &dyn PlanetaryWorld) -> Option<(Vec3, bool)> {
        if matches!(self.seat.1, PilotCraft::Driving(_)) {
            return None;
        }
        let HostPresence::Occupied(p) = self.seat.0 else {
            return None;
        };
        p.transfer(|_, kin, _, _| {
            kin.transfer(|pos, _, _, eye_h, _| {
                world.woodland_habitat(pos).then_some((pos, eye_h > 0.05 && eye_h < 0.62))
            })
        })
    }

    pub const fn fleet(&self) -> &FleetLedger { &self.fleet }
    pub fn fleet_mut(&mut self) -> &mut FleetLedger { &mut self.fleet }
    pub const fn wildlife(&self) -> &HostSounder { &self.wildlife }
    pub fn wildlife_mut(&mut self) -> &mut HostSounder { &mut self.wildlife }
    pub fn beacons_mut(&mut self) -> &mut SpawnBeacons { &mut self.events.beacons }

    pub fn push_host_craft(&self, out: &mut Vec<DrivenCraft>) {
        self.seat.1.push_active(out);
    }

    pub fn record_crack(&mut self, author_slot: u32, notice: CrackNotice) {
        if self.events.cracks.len() >= 16 {
            self.events.cracks.remove(0);
        }
        self.events.cracks.push(PeerCrack::new(author_slot, notice));
    }

    pub fn drain_cracks(&mut self) -> Vec<PeerCrack> {
        std::mem::take(&mut self.events.cracks)
    }

    pub fn receive_utterance(&mut self, utterance: ProximityUtterance, earshot_meters: f32) {
        if let Some(pos) = self.occupied_position() {
            if utterance.audible_at(pos, earshot_meters) {
                if self.events.utterances.len() >= 32 {
                    self.events.utterances.remove(0);
                }
                self.events.utterances.push(utterance);
            }
        }
    }

    pub fn drain_utterances(&mut self, out: &mut Vec<ProximityUtterance>) {
        out.append(&mut self.events.utterances);
    }
}
