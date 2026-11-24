use std::collections::HashMap;
use std::net::SocketAddr;
use glam::Vec3;
use world::{
    CompanionPresence, DeckFrame, DrivenCraft, PilotCraft, PlanetaryWorld, ProximityUtterance,
    SolarProportions,
};
use crate::admin_ward::SpawnBeacons;
use crate::audience::NEARBY_SIGHT_METERS;
use crate::envelope::CrackNotice;
use crate::fleet_ledger::FleetLedger;
use crate::host_sounder::HostSounder;
use crate::seal::HandshakeIdentity;
use crate::spatial_chronicle::SpatialFactIndex;

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

/// Spatially bucketed transient dig cracks (`16` per `160 m` sector) preventing cross-zone eviction (`<= 4` fields).
pub struct SectorCracks {
    buckets: HashMap<(i32, i32, i32), Vec<PeerCrack>>,
}

impl SectorCracks {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(buckets: HashMap<(i32, i32, i32), Vec<PeerCrack>>) -> Self {
        Self { buckets }
    }

    pub fn empty() -> Self {
        Self::new(HashMap::new())
    }

    fn cell_key(pos: Vec3) -> (i32, i32, i32) {
        (
            (pos.x / NEARBY_SIGHT_METERS).floor() as i32,
            (pos.y / NEARBY_SIGHT_METERS).floor() as i32,
            (pos.z / NEARBY_SIGHT_METERS).floor() as i32,
        )
    }

    pub fn record(&mut self, author_slot: u32, notice: CrackNotice) {
        let eye = notice.eye_point();
        if !eye.is_finite() {
            return;
        }
        if self.buckets.len() >= 256 {
            self.buckets.clear();
        }
        let bucket = self.buckets.entry(Self::cell_key(eye)).or_default();
        if bucket.len() >= 16 {
            bucket.remove(0);
        }
        bucket.push(PeerCrack::new(author_slot, notice));
    }

    pub fn drain_into_buckets(&mut self) -> Self {
        Self::new(std::mem::take(&mut self.buckets))
    }

    pub fn collect_near(&self, anchor: Vec3, exclude_slot: u32, radius: f32) -> Vec<CrackNotice> {
        if !anchor.is_finite() || self.buckets.is_empty() {
            return Vec::new();
        }
        let (cx, cy, cz) = Self::cell_key(anchor);
        let mut out = Vec::new();
        for dz in -1..=1 {
            for dy in -1..=1 {
                for dx in -1..=1 {
                    if let Some(list) = self.buckets.get(&(cx + dx, cy + dy, cz + dz)) {
                        for &c in list {
                            if !c.authored_by(exclude_slot) && c.notice().near(anchor, radius) && out.len() < 16 {
                                out.push(c.notice());
                            }
                        }
                    }
                }
            }
        }
        out
    }
}

/// Sector-isolated crack buckets, host chat inbox, admin `SpawnBeacons`, and spatial chronicle index (`<= 4` fields).
pub struct AuraEvents {
    cracks: SectorCracks,
    utterances: Vec<ProximityUtterance>,
    beacons: SpawnBeacons,
    fact_index: SpatialFactIndex,
}

impl AuraEvents {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(
        cracks: SectorCracks,
        utterances: Vec<ProximityUtterance>,
        beacons: SpawnBeacons,
        fact_index: SpatialFactIndex,
    ) -> Self {
        Self { cracks, utterances, beacons, fact_index }
    }

    pub fn empty() -> Self {
        Self::new(SectorCracks::empty(), Vec::new(), SpawnBeacons::empty(), SpatialFactIndex::empty())
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

    pub fn new(
        seat: (HostPresence, PilotCraft),
        cracks: Vec<PeerCrack>,
        fleet: FleetLedger,
        wildlife: HostSounder,
    ) -> Self {
        let mut sector_cracks = SectorCracks::empty();
        for c in cracks {
            sector_cracks.record(if c.authored_by(1) { 1 } else { 0 }, c.notice());
        }
        Self::with_events(
            seat,
            AuraEvents::new(sector_cracks, Vec::new(), SpawnBeacons::empty(), SpatialFactIndex::empty()),
            fleet,
            wildlife,
        )
    }

    pub fn headless() -> Self {
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
    pub fn fact_index_mut(&mut self) -> &mut SpatialFactIndex { &mut self.events.fact_index }

    pub fn push_host_craft(&self, out: &mut Vec<DrivenCraft>) {
        self.seat.1.push_active(out);
    }

    pub fn push_host_craft_with_speed(&self, out: &mut Vec<(DrivenCraft, f32)>) {
        if let PilotCraft::Driving(dc) = self.seat.1 {
            out.push((dc, DeckFrame::thrust_from_pose(dc.pose()) * 180.0));
        }
    }

    pub fn record_crack(&mut self, author_slot: u32, notice: CrackNotice) {
        self.events.cracks.record(author_slot, notice);
    }

    pub fn drain_cracks(&mut self) -> SectorCracks {
        self.events.cracks.drain_into_buckets()
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
