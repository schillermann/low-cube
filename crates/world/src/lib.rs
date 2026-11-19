use glam::Vec3;

mod beveled_world;
mod biome;
mod block;
mod chronicle;
mod frame;
mod mesh_target;
mod noise;
mod null_face;
mod planet;
mod presence;
mod quad;
mod relief;
mod topology;
mod tree;
mod wire_vector;

#[cfg(test)]
mod beveled_world_tests;
#[cfg(test)]
mod gravity_tests;
#[cfg(test)]
mod mesh_target_tests;

pub use beveled_world::BeveledGravity;
pub use biome::BiomedCubeWorld;
pub use block::{
    BlockMedium, BlockMold, CabinSeatMesh, CardinalTurn, CornerHeading, CuboidShape, DiagonalSlope,
    ElevatedShape, FieldCameraTripod, FootwellPlinth, GentleRidge, GentleValley, GentleWedge, GridCell,
    MediumShape, MineralSeam, MoldArchetype, PalmMolds, PedalClusterMesh, PyramidPeak, RampWedge,
    RidgeCorner, StairCorner, StairFlight, ValleyCorner, VoxelShape, WalnutFloorPan, WedgeHeading,
    WheelProfile,
};
pub use chronicle::WorldFact;
pub use frame::SurfaceFrame;
pub use mesh_target::{
    CountedMeshTarget, CrateredMeshTarget, MeadowCrown, NaturalMeadowTarget, TurfMeshTarget,
};
pub use noise::{CoherentNoise, DomainWarp, FractalHarmonics, SpatialFrequency};
pub use null_face::NullFace;
pub use planet::{CubePlanet, PatchCoord};
pub use presence::{CompanionPresence, DrivenCraft, PilotCraft, PresenceAttire, PresenceKinematics};
pub use relief::{
    BlueprintCatalog, CraftBlueprint, CraftIdentity, CraftPose, CrustVault, ExposedStratum,
    FacetedReliefFace, HelmLocks, HollowVeins, LivingVeins, MeadowRing, MeadowStamp,
    ReliefElevation, ReliefFacet, ReliefPalette, SeatOccupancy, SubterraneanVeins,
    SurfaceSightRay, TerracedRelief, VehicleAssembly, VehicleAxes, VehiclePart, WorldCraft,
    MEADOW_RING_SLOTS,
};
pub use topology::{
    BevelTransition, BeveledBoundary, ClimateZone, CoordinateFold, CubicBoundary, FacePerimeter,
    FoldedPoint, OpenBoundary, PerimeterCrossing, PerimeterEdge, PerimeterFold, PlanetaryTopology,
    SeamBevel, SeamBoundary, SeamFold, Topology,
};
pub use tree::{
    Arboreal, ArborealFacet, ArcticPine, ClearedPlot, CleaveOutcome, ConicalSkirt,
    DesertSucculent, FloorLitter, ForestedFace, FullCubeGround, HewnTimberTarget,
    JungleEmergent, MeridianGround, PlantingGround, PolyhedralCrown, PrismTrunk,
    SolidArboreal, SpawnSanctuaryGround, StandingTree, SteppeAcacia, SurfaceFind,
    SurfaceGround, TemperateOak, TimberDepository, TimberYield, TreeSpecies,
    VolcanicSpire, WoodlandDepository, WoodlandGround, ZonedForest,
};
pub use quad::Quad;

/// Interface for objects accepting generated mesh primitives.
pub trait MeshTarget {
    fn push_quad(&mut self, quad: Quad);
}

/// Interface representing a planetary gravitational field.
pub trait GravityField {
    fn pull_direction_at(&self, position: Vec3) -> Vec3;
    fn surface_up_at(&self, position: Vec3) -> Vec3;
}

/// Focused interface for surface excavation, timber cleaving, block molding, and craft commissioning.
pub trait WorldEarthworks {
    fn excavate_surface(&mut self, eye: Vec3, direction: Vec3) -> bool {
        self.harvest_surface(eye, direction).is_some()
    }
    fn harvest_surface(&mut self, _eye: Vec3, _direction: Vec3) -> Option<BlockMedium> { None }
    fn loosen_surface(&mut self, _eye: Vec3, _direction: Vec3, _stage: u8) -> Option<BlockMedium> { None }
    fn strike_timber(&mut self, _eye: Vec3, _direction: Vec3) -> CleaveOutcome { CleaveOutcome::Missed }
    fn forage_surface(&mut self, _eye: Vec3, _direction: Vec3) -> SurfaceFind { SurfaceFind::EmptyHanded }
    fn cast_surface_find(&mut self, _eye: Vec3, _direction: Vec3, _find: SurfaceFind) -> bool { false }
    fn place_molded_surface_block(&mut self, _eye: Vec3, _direction: Vec3, _avatar_pos: Vec3, _mold: BlockMold) -> bool { false }
    fn adjust_rig(&mut self, _eye: Vec3, _direction: Vec3, _delta_steps: i32, _rotate_quarter: bool) -> bool { false }
    fn lower_rig(&mut self, _eye: Vec3, _direction: Vec3) -> bool { false }
    fn board_vehicle(&mut self, _eye: Vec3, _direction: Vec3, _holding_block: bool) -> Option<VehicleAssembly> { None }
    fn park_vehicle(&mut self, _assembly: VehicleAssembly) -> bool { false }
    fn register_craft_blueprint(&mut self, _blueprint: CraftBlueprint) -> bool { false }
    fn claim_vehicle_helm(&mut self, _craft: CraftIdentity, _pilot_slot: u32, eye: Vec3, direction: Vec3) -> bool {
        self.board_vehicle(eye, direction, true).is_some()
    }
    fn park_craft_by_id(&mut self, _craft: CraftIdentity, _pose: CraftPose) -> bool { false }
    fn disclose_stratum(&mut self, _face_index: usize, _stratum: ExposedStratum) {}
}

/// Core interface for a planetary voxel body.
pub trait PlanetaryWorld: GravityField + WorldEarthworks + Send {
    fn emit_surface(&self, target: &mut dyn MeshTarget);
    fn constrain_to_surface(&self, position: Vec3, player_height: f32) -> Vec3;
    fn canopy_drag(&self, _position: Vec3) -> f32 { 0.0 }
    fn placed_camera_tripod(&self) -> Option<(Vec3, Vec3, Vec3)> { None }
    fn step_surface(&mut self, _dt: f32) -> bool { false }
    fn step_local_flora(&mut self, dt: f32) -> bool { self.step_surface(dt) }
    fn lethal_at(&self, _position: Vec3) -> bool { false }
    fn confine_beneath_crust(&self, position: Vec3) -> Vec3 { position }
    fn face_count(&self) -> usize { 1 }
    fn emit_face(&self, _index: usize, target: &mut dyn MeshTarget) { self.emit_surface(target); }
    fn emit_modified_face(&self, target: &mut dyn MeshTarget) -> usize {
        self.emit_surface(target);
        0
    }
    fn pending_slices(&self) -> bool { false }
    fn drain_modified_slice(&mut self, _target: &mut dyn MeshTarget) -> Option<usize> { None }
    fn emit_meadow_around(&self, _center: Vec3, _view_dir: Vec3, _target: &mut dyn MeshTarget) {}
    fn arboreal_perch_near(&self, _center: Vec3, _selector: usize) -> Option<Vec3> { None }
    fn visible_slice_mask(&self, _position: Vec3) -> u32 { u32::MAX }
    fn drain_disclosures(&mut self, _out: &mut Vec<(u8, ExposedStratum)>) {}
    fn sync_presence(&mut self, _presence: CompanionPresence) {}
    fn sync_pilot_craft(&mut self, _craft: PilotCraft) {}
    fn active_companions(&self, _out: &mut Vec<CompanionPresence>) {}
    fn active_crafts(&self, _out: &mut Vec<DrivenCraft>) {}
    fn blend_companions(&mut self, _subtick_secs: f32) {}
    fn portal_ticket(&self) -> Option<String> { None }
    fn portal_phrase(&self) -> Option<String> { None }
    fn punch_nat_peer(&mut self, _peer: std::net::SocketAddr) -> bool { false }
    fn pump_portal(&mut self, _dt: f32) -> bool { false }
    fn board_passenger_seat(&self, _eye: Vec3, _direction: Vec3) -> Option<(CraftIdentity, Vec3, (Vec3, Vec3, Vec3))> { None }
    fn passenger_seat_frame(&self, _craft: CraftIdentity, _local_seat: Vec3) -> Option<(Vec3, Vec3, Vec3)> { None }
    fn helm_revoked(&mut self, _craft: CraftIdentity) -> bool { false }
    fn emit_active_crafts(&self, _target: &mut dyn MeshTarget) {}
    fn face_patch_count(&self, _slice: usize) -> usize { 0 }
    fn emit_face_patch(&self, _slice: usize, _patch_slot: usize, _target: &mut dyn MeshTarget) {}
    fn drain_modified_patch(&mut self, _target: &mut dyn MeshTarget) -> Option<(usize, usize)> { None }
    fn drain_stale_meadow_patch(&self, _center: Vec3, _ring: &mut MeadowRing, _target: &mut dyn MeshTarget) -> Option<usize> { None }
}

/// Interface representing an individual face of a planetary body.
pub trait Face: Send + Sync {
    fn matches(&self, point: Vec3) -> bool;
    fn surface_up(&self) -> Vec3;
    fn pull_direction(&self) -> Vec3 { -self.surface_up() }
    fn constrain(&self, point: Vec3, distance: f32) -> Vec3;
    fn emit_surface(&self, target: &mut dyn MeshTarget);
    fn emit_terrain(&self, target: &mut dyn MeshTarget) { self.emit_surface(target); }
    fn emit_flora(&self, _target: &mut dyn MeshTarget) {}
    fn emit_meadow_around(&self, _center: Vec3, _view_dir: Vec3, _target: &mut dyn MeshTarget) {}
    fn terrain_patch_count(&self) -> usize { 0 }
    fn emit_terrain_patch(&self, _patch_slot: usize, _target: &mut dyn MeshTarget) {}
    fn pending_terrain_patches(&self) -> bool { false }
    fn drain_terrain_patch(&mut self, _target: &mut dyn MeshTarget) -> Option<usize> { None }
    fn clear_terrain_patches(&mut self) {
        struct NullTarget;
        impl MeshTarget for NullTarget {
            fn push_quad(&mut self, _quad: Quad) {}
        }
        let mut sink = NullTarget;
        while self.drain_terrain_patch(&mut sink).is_some() {}
    }
    fn meadow_center_sector(&self, _center: Vec3) -> (i32, i32) { (0, 0) }
    fn meadow_sector_revision(&self, _gsu: i32, _gsv: i32) -> u32 { 0 }
    fn emit_meadow_sector(&self, _gsu: i32, _gsv: i32, _lod: u8, _target: &mut dyn MeshTarget) {}
    fn arboreal_perch_near(&self, _center: Vec3, _selector: usize) -> Option<Vec3> { None }
    fn canopy_drag(&self, _point: Vec3) -> f32 { 0.0 }
    fn excavate(&mut self, eye: Vec3, direction: Vec3) -> bool { self.harvest(eye, direction).is_some() }
    fn harvest(&mut self, _eye: Vec3, _direction: Vec3) -> Option<BlockMedium> { None }
    fn loosen(&mut self, _eye: Vec3, _direction: Vec3, _stage: u8) -> Option<BlockMedium> { None }
    fn strike_timber(&mut self, _eye: Vec3, _direction: Vec3) -> CleaveOutcome { CleaveOutcome::Missed }
    fn forage_timber(&mut self, _eye: Vec3, _direction: Vec3) -> SurfaceFind { SurfaceFind::EmptyHanded }
    fn cast_litter(&mut self, _eye: Vec3, _direction: Vec3, _find: SurfaceFind) -> bool { false }
    fn place_molded_block(&mut self, _eye: Vec3, _direction: Vec3, _avatar_pos: Vec3, _mold: BlockMold) -> bool { false }
    fn adjust_rig(&mut self, _eye: Vec3, _direction: Vec3, _delta_steps: i32, _rotate_quarter: bool) -> bool { false }
    fn lower_rig(&mut self, _eye: Vec3, _direction: Vec3) -> bool { false }
    fn board_vehicle(&mut self, _eye: Vec3, _direction: Vec3, _holding_block: bool) -> Option<VehicleAssembly> { None }
    fn park_vehicle(&mut self, _assembly: VehicleAssembly) -> bool { false }
    fn placed_camera_tripod(&self) -> Option<(Vec3, Vec3, Vec3)> { None }
    fn step_surface(&mut self, _dt: f32) -> bool { false }
    fn step_local_flora(&mut self, dt: f32) -> bool { self.step_surface(dt) }
    fn step_flora(&mut self, _dt: f32) -> bool { false }
    fn lethal_at(&self, _point: Vec3) -> bool { false }
    fn confine_beneath_crust(&self, point: Vec3) -> Vec3 { point }
    fn accept_disclosure(&mut self, _stratum: ExposedStratum) {}
    fn drain_disclosures(&mut self, _out: &mut Vec<ExposedStratum>) {}
}
