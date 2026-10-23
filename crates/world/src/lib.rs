use glam::Vec3;

mod beveled_world;
mod biome;
mod block;
mod centripetal_world;
mod chunk;
mod cluster;
mod elevated_face;
mod face;
mod frame;
mod gallery;
mod landscape;
mod mesh_target;
mod mountain;
mod noise;
mod null_face;
mod planet;
mod quad;
mod relief;
mod tinted_face;
mod topology;
mod tree;

#[cfg(test)]
mod beveled_world_tests;
#[cfg(test)]
mod centripetal_tests;
#[cfg(test)]
mod cluster_tests;
#[cfg(test)]
mod elevation_tint_tests;
#[cfg(test)]
mod gallery_tests;
#[cfg(test)]
mod gravity_tests;
#[cfg(test)]
mod mesh_target_tests;
#[cfg(test)]
mod mountain_tests;

pub use beveled_world::BeveledWorld;
pub use biome::BiomedCubeWorld;
pub use centripetal_world::CentripetalWorld;
pub use block::{
    Block, BlockMedium, BlockMold, CardinalTurn, CornerHeading, CuboidShape, DiagonalSlope,
    DirtBlock, ElevatedShape, FormBlock, GentleRidge, GentleValley, GentleWedge, GridCell,
    MaterialBlock, MediumShape, MoldArchetype, PyramidPeak, RampWedge, RidgeCorner, SandBlock,
    SnowBlock, StairCorner, StairFlight, StoneBlock, ValleyCorner, VoxelShape, WedgeHeading,
};

pub use chunk::{
    BedrockShell, FacetedVolume, MonolithVolume, PlanetaryCrust,
    PlanetaryNeighborhood, SparsePlanetaryGrid, StratifiedVolume, VoidNeighborhood,
    VoidVolume, VolumeCoord, VolumePlanet, VoxelBarrier, VoxelCoord,
    VoxelNeighborhood, VoxelVolume, VOLUME_EDGE_LENGTH, VOXEL_COUNT_PER_VOLUME,
};
pub use cluster::VoxelCluster;
pub use elevated_face::ElevatedFace;
pub use face::PlanarFace;
pub use frame::SurfaceFrame;
pub use gallery::BlockGallery;
pub use landscape::{DesertDunes, GlacialMassif, JungleKarst, SteppeTerrace, VolcanicCaldera};
pub use mesh_target::{
    ColoredMeshTarget, CountedMeshTarget, CrateredMeshTarget, OffsetMeshTarget, TintedMeshTarget,
    TurfMeshTarget,
};
pub use mountain::VoxelMountain;
pub use noise::{CoherentNoise, DomainWarp, FractalHarmonics, SpatialFrequency};
pub use null_face::NullFace;
pub use planet::CubePlanet;
pub use relief::{
    CavitySpan, FacetedReliefFace, NullCavity, ReliefElevation, ReliefFacet, ReliefPalette,
    SubterraneanCavity, SurfaceSightRay, TerracedRelief, ZonedCavity,
};
pub use tinted_face::TintedFace;
pub use topology::{
    BevelTransition, ClimateZone, CoordinateFold, CubeFace, FacePerimeter, FoldedFace,
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
    fn excavate_surface(&mut self, _eye: Vec3, _direction: Vec3) -> bool {
        false
    }
    fn surface_medium(&self, _eye: Vec3, _direction: Vec3) -> BlockMedium {
        BlockMedium::TurfEarth
    }
    fn loosen_surface(&mut self, _eye: Vec3, _direction: Vec3, _stage: u8) -> bool {
        false
    }
    fn strike_timber(&mut self, _eye: Vec3, _direction: Vec3) -> CleaveOutcome {
        CleaveOutcome::Missed
    }
    fn forage_surface(&mut self, _eye: Vec3, _direction: Vec3) -> SurfaceFind {
        SurfaceFind::EmptyHanded
    }
    fn place_surface_block(&mut self, eye: Vec3, direction: Vec3, avatar_pos: Vec3) -> bool {
        self.place_molded_surface_block(eye, direction, avatar_pos, BlockMold::cube())
    }
    fn place_molded_surface_block(&mut self, _eye: Vec3, _direction: Vec3, _avatar_pos: Vec3, _mold: BlockMold) -> bool {
        false
    }
    fn step_surface(&mut self, _dt: f32) -> bool {
        false
    }
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
}

/// Interface representing an individual face of a planetary body.
pub trait Face: Send + Sync {
    fn matches(&self, point: Vec3) -> bool;
    fn surface_up(&self) -> Vec3;
    fn pull_direction(&self) -> Vec3;
    fn constrain(&self, point: Vec3, distance: f32) -> Vec3;
    fn emit_surface(&self, target: &mut dyn MeshTarget);
    fn canopy_drag(&self, _point: Vec3) -> f32 {
        0.0
    }
    fn excavate(&mut self, _eye: Vec3, _direction: Vec3) -> bool {
        false
    }
    fn surface_medium(&self, _eye: Vec3, _direction: Vec3) -> BlockMedium {
        BlockMedium::TurfEarth
    }
    fn loosen(&mut self, _eye: Vec3, _direction: Vec3, _stage: u8) -> bool {
        false
    }
    fn strike_timber(&mut self, _eye: Vec3, _direction: Vec3) -> CleaveOutcome {
        CleaveOutcome::Missed
    }
    fn forage_timber(&mut self, _eye: Vec3, _direction: Vec3) -> SurfaceFind {
        SurfaceFind::EmptyHanded
    }
    fn place_block(&mut self, eye: Vec3, direction: Vec3, avatar_pos: Vec3) -> bool {
        self.place_molded_block(eye, direction, avatar_pos, BlockMold::cube())
    }
    fn place_molded_block(&mut self, _eye: Vec3, _direction: Vec3, _avatar_pos: Vec3, _mold: BlockMold) -> bool {
        false
    }
    fn step_surface(&mut self, _dt: f32) -> bool {
        false
    }
}

