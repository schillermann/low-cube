mod blueprint;
mod deck;
mod identity;
mod pose;
mod spatial;

use glam::Vec3;
use crate::MeshTarget;
use crate::presence::DrivenCraft;
pub use self::blueprint::{
    CellRole, CraftBlueprint, ExhaustJet, HullOptic, HullWireframe, JetTone, OpticMount,
    WireframeCell, MAX_EXHAUST_JETS, MAX_WIRE_CELLS,
};
pub use self::deck::DeckFrame;
pub use self::identity::CraftIdentity;
pub use self::pose::{CraftPose, SeatOccupancy};
pub(in crate::relief) use self::spatial::PosedHull;

/// Focused sink receiving active, parked, and orbital vessels for GPU-posed local-hull rendering.
pub trait VesselSink {
    fn receive_vessel(&mut self, blueprint: &CraftBlueprint, pose: CraftPose);
}

/// Living planetary vehicle entity uniting identity, immutable blueprint, pose, and seat occupancy.
#[derive(Clone, Debug, PartialEq)]
pub struct WorldCraft {
    identity: CraftIdentity,
    blueprint: CraftBlueprint,
    pose: CraftPose,
    occupancy: SeatOccupancy,
}

impl WorldCraft {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(
        identity: CraftIdentity,
        blueprint: CraftBlueprint,
        pose: CraftPose,
        occupancy: SeatOccupancy,
    ) -> Self {
        Self { identity, blueprint, pose, occupancy }
    }

    pub const fn identity(&self) -> CraftIdentity {
        self.identity
    }

    pub const fn occupancy(&self) -> SeatOccupancy {
        self.occupancy
    }

    pub fn emit(&self, target: &mut dyn MeshTarget) {
        self.blueprint.emit_at(self.pose, target);
    }
}

/// Planetary blueprint catalog resolving 80-byte `DrivenCraft` poses into full 3D vehicle geometry and collision.
#[derive(Clone, Debug)]
pub struct BlueprintCatalog {
    entries: Vec<CraftBlueprint>,
}

impl BlueprintCatalog {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(entries: Vec<CraftBlueprint>) -> Self {
        Self { entries }
    }

    pub const fn empty() -> Self {
        Self::new(Vec::new())
    }

    pub fn contains(&self, id: CraftIdentity) -> bool {
        self.entries.iter().any(|b| b.identity() == id)
    }

    pub fn register(&mut self, blueprint: CraftBlueprint) -> bool {
        if !blueprint.identity().assigned() || self.contains(blueprint.identity()) {
            return false;
        }
        self.entries.push(blueprint);
        true
    }

    pub fn find(&self, id: CraftIdentity) -> Option<&CraftBlueprint> {
        self.entries.iter().find(|b| b.identity() == id)
    }

    pub fn emit_crafts(&self, crafts: &[DrivenCraft], target: &mut dyn MeshTarget) {
        for craft in crafts {
            if let Some(bp) = self.find(craft.identity()) {
                bp.emit_at(craft.pose(), target);
            }
        }
    }

    pub fn present_vessels(&self, crafts: &[DrivenCraft], sink: &mut dyn VesselSink) {
        for craft in crafts {
            if let Some(bp) = self.find(craft.identity()) {
                sink.receive_vessel(bp, craft.pose());
            }
        }
    }

    pub fn constrain_crafts(&self, crafts: &[DrivenCraft], position: Vec3, player_height: f32) -> Option<Vec3> {
        let mut best: Option<Vec3> = None;
        for craft in crafts {
            let Some(bp) = self.find(craft.identity()) else { continue };
            if let Some(supported) = bp.constrain_at(craft.pose(), position, player_height) {
                let up = craft.pose().normal();
                if best.is_none_or(|cur| supported.dot(up) > cur.dot(up)) {
                    best = Some(supported);
                }
            }
        }
        best
    }

    pub fn find_passenger_seat(
        &self,
        crafts: &[DrivenCraft],
        eye: Vec3,
        direction: Vec3,
    ) -> Option<(CraftIdentity, Vec3, (Vec3, Vec3, Vec3))> {
        for craft in crafts {
            let Some(bp) = self.find(craft.identity()) else { continue };
            if let Some(local) = bp.passenger_seat_along_ray(craft.pose(), eye, direction) {
                return Some((craft.identity(), local, craft.pose().world_seat_at(local)));
            }
        }
        None
    }

    pub fn sample_deck_at(&self, crafts: &[DrivenCraft], position: Vec3, player_height: f32) -> Option<DeckFrame> {
        for craft in crafts {
            let Some(bp) = self.find(craft.identity()) else { continue };
            if let Some(frame) = DeckFrame::probe(bp, craft.pose(), position, player_height) {
                return Some(frame);
            }
        }
        None
    }

    pub fn resolve_deck_pose(&self, crafts: &[DrivenCraft], id: CraftIdentity) -> Option<(CraftPose, f32)> {
        let craft = crafts.iter().find(|c| c.identity() == id)?;
        Some((craft.pose(), DeckFrame::thrust_from_pose(craft.pose())))
    }

    pub fn constrain_craft_by_id(
        &self,
        crafts: &[DrivenCraft],
        id: CraftIdentity,
        position: Vec3,
        player_height: f32,
    ) -> Option<Vec3> {
        let craft = crafts.iter().find(|c| c.identity() == id)?;
        let bp = self.find(id)?;
        bp.constrain_at(craft.pose(), position, player_height)
    }
}

/// Authoritative per-vehicle helm ownership table preventing vehicle duplication across simultaneous P2P clicks.
#[derive(Clone, Debug)]
pub struct HelmLocks {
    claims: Vec<(CraftIdentity, SeatOccupancy)>,
}

impl HelmLocks {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(claims: Vec<(CraftIdentity, SeatOccupancy)>) -> Self {
        Self { claims }
    }

    pub const fn empty() -> Self {
        Self::new(Vec::new())
    }

    pub fn claim(&mut self, craft: CraftIdentity, pilot_slot: u32) -> bool {
        if !craft.assigned() {
            return false;
        }
        if let Some((_, occ)) = self.claims.iter_mut().find(|(id, _)| *id == craft) {
            let Some(next) = occ.try_claim_helm(pilot_slot) else { return false };
            *occ = next;
            return true;
        }
        self.claims.push((craft, SeatOccupancy::Helmed { pilot_slot }));
        true
    }

    pub fn release(&mut self, craft: CraftIdentity, pilot_slot: u32) -> bool {
        if let Some(idx) = self.claims.iter().position(|(id, occ)| *id == craft && occ.helmed_by(pilot_slot)) {
            self.claims.swap_remove(idx);
            true
        } else {
            false
        }
    }

    pub fn release_all_for_slot(&mut self, pilot_slot: u32) {
        self.claims.retain(|(_, occ)| !occ.helmed_by(pilot_slot));
    }

    pub fn helmed_by(&self, craft: CraftIdentity, pilot_slot: u32) -> bool {
        self.claims.iter().any(|(id, occ)| *id == craft && occ.helmed_by(pilot_slot))
    }
}
