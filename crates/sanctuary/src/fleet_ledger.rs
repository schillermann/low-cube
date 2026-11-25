use glam::Vec3;
use world::{
    BlueprintCatalog, CraftBlueprint, CraftIdentity, CraftPose, DeckFrame, DrivenCraft, HelmLocks,
    MeshTarget, VehicleAssembly, VesselSink, WorldEarthworks,
};

/// Encapsulated blueprint catalog, helm ownership locks, and optimistic race-revocation list.
#[derive(Clone, Debug)]
pub struct FleetLedger {
    catalog: BlueprintCatalog,
    locks: HelmLocks,
    revoked: Vec<CraftIdentity>,
}

impl FleetLedger {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(
        catalog: BlueprintCatalog,
        locks: HelmLocks,
        revoked: Vec<CraftIdentity>,
    ) -> Self {
        Self { catalog, locks, revoked }
    }

    pub const fn empty() -> Self {
        Self::new(BlueprintCatalog::empty(), HelmLocks::empty(), Vec::new())
    }

    pub fn register_blueprint(&mut self, blueprint: CraftBlueprint) -> bool {
        self.catalog.register(blueprint)
    }

    pub fn contains_blueprint(&self, id: CraftIdentity) -> bool {
        self.catalog.contains(id)
    }

    pub fn fusion_equipped(&self, id: CraftIdentity) -> bool {
        self.catalog.find(id).is_some_and(CraftBlueprint::fusion_equipped)
    }

    pub fn claim_helm(&mut self, craft: CraftIdentity, pilot_slot: u32) -> bool {
        self.locks.claim(craft, pilot_slot)
    }

    pub fn release_helm(&mut self, craft: CraftIdentity, pilot_slot: u32) -> bool {
        self.locks.release(craft, pilot_slot)
    }

    pub fn force_release(&mut self, craft: CraftIdentity) {
        for slot in 0..64 {
            let _ = self.locks.release(craft, slot);
        }
    }

    pub fn release_slot(&mut self, pilot_slot: u32) {
        self.locks.release_all_for_slot(pilot_slot);
    }

    pub fn helmed_by(&self, craft: CraftIdentity, pilot_slot: u32) -> bool {
        self.locks.helmed_by(craft, pilot_slot)
    }

    pub fn revoke_local_helm(&mut self, craft: CraftIdentity, winner_slot: u32, local_slot: u32) {
        let _ = self.locks.release(craft, local_slot);
        let _ = self.locks.claim(craft, winner_slot);
        if !self.revoked.contains(&craft) {
            self.revoked.push(craft);
        }
    }

    pub fn take_revoked(&mut self, craft: CraftIdentity) -> bool {
        if let Some(pos) = self.revoked.iter().position(|c| *c == craft) {
            self.revoked.swap_remove(pos);
            true
        } else {
            false
        }
    }

    pub fn reconstruct_assembly(&self, craft: CraftIdentity, pose: CraftPose) -> Option<VehicleAssembly> {
        let bp = self.catalog.find(craft)?.clone();
        Some(VehicleAssembly::from_blueprint(bp, pose))
    }

    pub fn park_by_id(
        &mut self,
        craft: CraftIdentity,
        pose: CraftPose,
        world: &mut dyn WorldEarthworks,
    ) -> bool {
        let Some(assembly) = self.reconstruct_assembly(craft, pose) else {
            return false;
        };
        self.force_release(craft);
        world.park_vehicle(assembly)
    }

    pub fn emit_crafts(&self, crafts: &[DrivenCraft], target: &mut dyn MeshTarget) {
        self.catalog.emit_crafts(crafts, target);
    }

    pub fn present_vessels(&self, crafts: &[DrivenCraft], sink: &mut dyn VesselSink) {
        self.catalog.present_vessels(crafts, sink);
    }

    pub fn constrain_crafts(
        &self,
        crafts: &[DrivenCraft],
        position: Vec3,
        player_height: f32,
    ) -> Option<Vec3> {
        self.catalog.constrain_crafts(crafts, position, player_height)
    }

    pub fn find_passenger_seat(
        &self,
        crafts: &[DrivenCraft],
        eye: Vec3,
        direction: Vec3,
    ) -> Option<(CraftIdentity, Vec3, (Vec3, Vec3, Vec3))> {
        self.catalog.find_passenger_seat(crafts, eye, direction)
    }

    pub fn sample_deck_at(&self, crafts: &[DrivenCraft], position: Vec3, player_height: f32) -> Option<DeckFrame> {
        self.catalog.sample_deck_at(crafts, position, player_height)
    }

    pub fn resolve_deck_pose(&self, crafts: &[DrivenCraft], id: CraftIdentity) -> Option<(CraftPose, f32)> {
        self.catalog.resolve_deck_pose(crafts, id)
    }

    pub fn constrain_craft_by_id(
        &self,
        crafts: &[DrivenCraft],
        id: CraftIdentity,
        position: Vec3,
        player_height: f32,
    ) -> Option<Vec3> {
        self.catalog.constrain_craft_by_id(crafts, id, position, player_height)
    }
}
