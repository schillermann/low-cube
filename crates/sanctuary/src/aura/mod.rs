mod cracks;

use std::net::SocketAddr;
use glam::Vec3;
use world::{
    Callsign, CompanionPresence, CosmicFrame, CraftIdentity, CraftPose, DeckFrame, DrivenCraft,
    FrameHandoff, PilotCraft, PlanetaryWorld, ProximityUtterance, SolarProportions, VesselBerth,
};
use crate::admin_ward::SpawnBeacons;
use crate::envelope::CrackNotice;
use crate::fleet_ledger::FleetLedger;
use crate::host_sounder::HostSounder;
use crate::seal::HandshakeIdentity;
use crate::spatial_chronicle::SpatialFactIndex;
pub use self::cracks::{PeerCrack, SectorCracks};

/// Cryptographic identity, solar-system proportions, endpoint, and host `Callsign` of a `SanctuaryHost` (`<= 4` fields).
pub struct HostIdentity {
    keys: HandshakeIdentity,
    proportions: SolarProportions,
    endpoint: SocketAddr,
    callsign: Callsign,
}

impl HostIdentity {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn with_callsign(
        keys: HandshakeIdentity,
        proportions: SolarProportions,
        endpoint: SocketAddr,
        callsign: Callsign,
    ) -> Self {
        Self { keys, proportions, endpoint, callsign }
    }

    pub fn with_proportions(keys: HandshakeIdentity, proportions: SolarProportions, endpoint: SocketAddr) -> Self {
        Self::with_callsign(keys, proportions, endpoint, Callsign::host())
    }

    pub fn new(keys: HandshakeIdentity, blocks_per_edge: u32, endpoint: SocketAddr) -> Self {
        Self::with_proportions(keys, SolarProportions::true_scale(blocks_per_edge), endpoint)
    }

    pub const fn keys(&self) -> &HandshakeIdentity { &self.keys }
    pub const fn blocks_per_edge(&self) -> u32 { self.proportions.earth_blocks() }
    pub const fn proportions(&self) -> SolarProportions { self.proportions }
    pub const fn endpoint(&self) -> SocketAddr { self.endpoint }
    pub const fn callsign(&self) -> Callsign { self.callsign }
}

/// Polymorphic local-host avatar presence (`Headless` for dedicated servers, `Occupied` for `--host`).
#[derive(Clone, Copy, Debug)]
pub enum HostPresence {
    Headless,
    Occupied(CompanionPresence),
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

/// Strictly encapsulated local host seat + berth + orbital clock, `AuraEvents`, `FleetLedger`, and `HostSounder`.
pub struct HostAura {
    seat: ((HostPresence, PilotCraft, VesselBerth), (CosmicFrame, f64)),
    events: AuraEvents,
    fleet: FleetLedger,
    wildlife: HostSounder,
}

impl HostAura {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn with_cosmos_seat(
        seat: ((HostPresence, PilotCraft, VesselBerth), (CosmicFrame, f64)),
        events: AuraEvents,
        fleet: FleetLedger,
        wildlife: HostSounder,
    ) -> Self {
        Self { seat, events, fleet, wildlife }
    }

    pub const fn with_events(
        seat: (HostPresence, PilotCraft),
        events: AuraEvents,
        fleet: FleetLedger,
        wildlife: HostSounder,
    ) -> Self {
        Self::with_cosmos_seat(((seat.0, seat.1, VesselBerth::planetary()), (CosmicFrame::earth(), 0.0)), events, fleet, wildlife)
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

    pub const fn host_frame(&self) -> CosmicFrame { (self.seat.1).0 }
    pub const fn host_berth(&self) -> VesselBerth { (self.seat.0).2 }
    pub const fn host_craft(&self) -> PilotCraft { (self.seat.0).1 }
    pub const fn epoch_secs(&self) -> f64 { (self.seat.1).1 }
    pub fn sync_epoch(&mut self, epoch_secs: f64) { (self.seat.1).1 = epoch_secs; }
    pub fn advance_epoch(&mut self, dt: f32) { (self.seat.1).1 += f64::from(dt.max(0.0)); }

    pub fn reanchor_host(&mut self, handoff: FrameHandoff) {
        (self.seat.1).0 = handoff.next_frame();
        if !(self.seat.0).2.aboard_any() {
            if let HostPresence::Occupied(p) = (self.seat.0).0 {
                (self.seat.0).0 = HostPresence::Occupied(p.shifted(handoff.pos_shift()));
            }
        }
        (self.seat.0).1 = (self.seat.0).1.shifted(handoff.pos_shift());
    }

    pub fn occupy(&mut self, presence: CompanionPresence) {
        self.occupy_berth(presence, VesselBerth::planetary());
    }

    pub fn occupy_berth(&mut self, presence: CompanionPresence, berth: VesselBerth) {
        (self.seat.0).0 = HostPresence::Occupied(presence.with_slot(1));
        (self.seat.0).2 = berth;
    }

    pub fn pilot(&mut self, craft: PilotCraft) {
        (self.seat.0).1 = craft.with_slot(1);
    }

    pub const fn presence(&self) -> HostPresence { (self.seat.0).0 }

    pub fn host_hull_pose(&self, craft_id: CraftIdentity) -> Option<CraftPose> {
        match (self.seat.0).1 {
            PilotCraft::Driving(dc) if dc.identity() == craft_id => Some(dc.pose()),
            _ => None,
        }
    }

    pub fn occupied_position(&self) -> Option<Vec3> {
        let HostPresence::Occupied(p) = (self.seat.0).0 else { return None };
        let raw = p.transfer(|_, kin, _, _| kin.transfer(|pos, _, _, _, _| pos));
        if let Some(id) = (self.seat.0).2.craft_id() {
            if let Some(pose) = self.host_hull_pose(id) {
                return Some(pose.world_point_at(raw));
            }
        }
        Some(raw)
    }

    pub fn pedestrian_observer(&self, world: &dyn PlanetaryWorld) -> Option<(Vec3, bool)> {
        if !self.host_frame().earth_centered()
            || (self.seat.0).2.aboard_any()
            || matches!((self.seat.0).1, PilotCraft::Driving(_))
        {
            return None;
        }
        let HostPresence::Occupied(p) = (self.seat.0).0 else { return None };
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

    pub fn push_host_craft_with_speed(&self, out: &mut Vec<(CosmicFrame, DrivenCraft, f32)>) {
        if let PilotCraft::Driving(dc) = (self.seat.0).1 {
            out.push((self.host_frame(), dc, DeckFrame::thrust_from_pose(dc.pose()) * 180.0));
        }
    }

    pub fn record_crack(&mut self, author_slot: u32, frame: CosmicFrame, notice: CrackNotice) {
        self.events.cracks.record_in_frame(author_slot, frame, notice);
    }

    pub fn drain_cracks(&mut self) -> SectorCracks { self.events.cracks.drain_into_buckets() }

    pub fn receive_utterance(&mut self, speaker_frame: CosmicFrame, utterance: ProximityUtterance, earshot_meters: f32) {
        if speaker_frame != self.host_frame() {
            return;
        }
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
