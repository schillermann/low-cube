use glam::Vec3;

mod beveled_world;
mod biome;
mod block;
mod frame;
mod mesh_target;
mod noise;
mod null_face;
mod planet;
mod quad;
mod relief;
mod topology;
mod tree;

#[cfg(test)]
mod beveled_world_tests;
#[cfg(test)]
mod gravity_tests;
#[cfg(test)]
mod mesh_target_tests;

pub use beveled_world::BeveledWorld;
pub use biome::BiomedCubeWorld;
pub use block::{
    BlockMedium, BlockMold, CabinSeatMesh, CardinalTurn, CornerHeading, CuboidShape, DiagonalSlope,
    ElevatedShape, FieldCameraTripod, GentleRidge, GentleValley, GentleWedge, GridCell, MediumShape,
    MoldArchetype, PyramidPeak, RampWedge, RidgeCorner, StairCorner, StairFlight,
    ValleyCorner, VoxelShape, WedgeHeading, WheelProfile,
};
pub use frame::SurfaceFrame;
pub use mesh_target::{
    CountedMeshTarget, CrateredMeshTarget, MeadowCrown, NaturalMeadowTarget, TurfMeshTarget,
};
pub use noise::{CoherentNoise, DomainWarp, FractalHarmonics, SpatialFrequency};
pub use null_face::NullFace;
pub use planet::CubePlanet;
pub use relief::{
    CrustVault, FacetedReliefFace, ReliefElevation, ReliefFacet, ReliefPalette, SurfaceSightRay,
    TerracedRelief, VehicleAssembly, VehicleAxes, VehiclePart,
};
pub use topology::{
    BevelTransition, ClimateZone, CoordinateFold, FacePerimeter, FoldedFace,
    FoldedPoint, PerimeterCrossing, PerimeterEdge, PlanetaryTopology, SeamBevel,
    SeamFold, Topology,
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

/// Interface for collecting surface quads ("Tell, Don't Ask").
pub trait MeshTarget {
    fn push_quad(&mut self, quad: Quad);
}

/// Interface for planetary gravity fields.
pub trait GravityField {
    fn pull_direction_at(&self, position: Vec3) -> Vec3;
    fn surface_up_at(&self, position: Vec3) -> Vec3;
}

/// Core interface for a planetary voxel body.
pub trait PlanetaryWorld: GravityField {
    fn emit_surface(&self, target: &mut dyn MeshTarget);
    fn constrain_to_surface(&self, position: Vec3, player_height: f32) -> Vec3;
    fn canopy_drag(&self, _position: Vec3) -> f32 {
        0.0
    }
    fn excavate_surface(&mut self, eye: Vec3, direction: Vec3) -> bool {
        self.harvest_surface(eye, direction).is_some()
    }
    fn harvest_surface(&mut self, _eye: Vec3, _direction: Vec3) -> Option<BlockMedium> {
        None
    }
    fn loosen_surface(&mut self, _eye: Vec3, _direction: Vec3, _stage: u8) -> Option<BlockMedium> {
        None
    }
    fn strike_timber(&mut self, _eye: Vec3, _direction: Vec3) -> CleaveOutcome {
        CleaveOutcome::Missed
    }
    fn forage_surface(&mut self, _eye: Vec3, _direction: Vec3) -> SurfaceFind {
        SurfaceFind::EmptyHanded
    }
    fn place_molded_surface_block(&mut self, _eye: Vec3, _direction: Vec3, _avatar_pos: Vec3, _mold: BlockMold) -> bool {
        false
    }
    fn adjust_rig(&mut self, _eye: Vec3, _direction: Vec3, _delta_steps: i32, _rotate_quarter: bool) -> bool {
        false
    }
    fn lower_rig(&mut self, _eye: Vec3, _direction: Vec3) -> bool {
        false
    }
    fn board_vehicle(&mut self, _eye: Vec3, _direction: Vec3, _holding_block: bool) -> Option<VehicleAssembly> {
        None
    }
    fn park_vehicle(&mut self, _assembly: VehicleAssembly) -> bool {
        false
    }
    fn placed_camera_tripod(&self) -> Option<(Vec3, Vec3, Vec3)> {
        None
    }
    fn step_surface(&mut self, _dt: f32) -> bool {
        false
    }
    fn weightless_at(&self, _position: Vec3) -> bool {
        false
    }
    fn lethal_at(&self, _position: Vec3) -> bool {
        false
    }
    fn confine_beneath_crust(&self, position: Vec3) -> Vec3 { position }
    fn face_count(&self) -> usize {
        1
    }
    fn emit_face(&self, _index: usize, target: &mut dyn MeshTarget) {
        self.emit_surface(target);
    }
    fn emit_modified_face(&self, target: &mut dyn MeshTarget) -> usize {
        self.emit_surface(target);
        0
    }
    fn emit_meadow_around(&self, _center: Vec3, _view_dir: Vec3, _target: &mut dyn MeshTarget) {}
    fn visible_slice_mask(&self, _position: Vec3) -> u32 { u32::MAX }
}

/// Interface representing an individual face of a planetary body.
pub trait Face: Send + Sync {
    fn matches(&self, point: Vec3) -> bool;
    fn surface_up(&self) -> Vec3;
    fn pull_direction(&self) -> Vec3;
    fn constrain(&self, point: Vec3, distance: f32) -> Vec3;
    fn emit_surface(&self, target: &mut dyn MeshTarget);
    fn emit_terrain(&self, target: &mut dyn MeshTarget) {
        self.emit_surface(target);
    }
    fn emit_flora(&self, _target: &mut dyn MeshTarget) {}
    fn emit_meadow_around(&self, _center: Vec3, _view_dir: Vec3, _target: &mut dyn MeshTarget) {}
    fn canopy_drag(&self, _point: Vec3) -> f32 {
        0.0
    }
    fn excavate(&mut self, eye: Vec3, direction: Vec3) -> bool {
        self.harvest(eye, direction).is_some()
    }
    fn harvest(&mut self, _eye: Vec3, _direction: Vec3) -> Option<BlockMedium> {
        None
    }
    fn loosen(&mut self, _eye: Vec3, _direction: Vec3, _stage: u8) -> Option<BlockMedium> {
        None
    }
    fn strike_timber(&mut self, _eye: Vec3, _direction: Vec3) -> CleaveOutcome {
        CleaveOutcome::Missed
    }
    fn forage_timber(&mut self, _eye: Vec3, _direction: Vec3) -> SurfaceFind {
        SurfaceFind::EmptyHanded
    }
    fn place_molded_block(&mut self, _eye: Vec3, _direction: Vec3, _avatar_pos: Vec3, _mold: BlockMold) -> bool {
        false
    }
    fn adjust_rig(&mut self, _eye: Vec3, _direction: Vec3, _delta_steps: i32, _rotate_quarter: bool) -> bool {
        false
    }
    fn lower_rig(&mut self, _eye: Vec3, _direction: Vec3) -> bool {
        false
    }
    fn board_vehicle(&mut self, _eye: Vec3, _direction: Vec3, _holding_block: bool) -> Option<VehicleAssembly> {
        None
    }
    fn park_vehicle(&mut self, _assembly: VehicleAssembly) -> bool {
        false
    }
    fn placed_camera_tripod(&self) -> Option<(Vec3, Vec3, Vec3)> {
        None
    }
    fn step_surface(&mut self, _dt: f32) -> bool {
        false
    }
    fn step_flora(&mut self, _dt: f32) -> bool {
        false
    }
    fn weightless_at(&self, _point: Vec3) -> bool {
        false
    }
    fn lethal_at(&self, _point: Vec3) -> bool {
        false
    }
    fn confine_beneath_crust(&self, point: Vec3) -> Vec3 { point }
}

