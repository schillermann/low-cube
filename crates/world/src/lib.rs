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
    Block, BlockMold, CardinalTurn, CornerHeading, CuboidShape, DiagonalSlope, DirtBlock,
    ElevatedShape, FormBlock, GentleRidge, GentleValley, GentleWedge, GridCell, MaterialBlock,
    MoldArchetype, PyramidPeak, RampWedge, RidgeCorner, SandBlock, SnowBlock, StairCorner,
    StairFlight, StoneBlock, ValleyCorner, VoxelShape, WedgeHeading,
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
    Arboreal, ArborealFacet, ArcticPine, ClearedPlot, ConicalSkirt, DesertSucculent,
    ForestedFace, FullCubeGround, JungleEmergent, MeridianGround, PlantingGround,
    PolyhedralCrown, PrismTrunk, SolidArboreal, SpawnSanctuaryGround, StandingTree,
    SteppeAcacia, SurfaceGround, TemperateOak, TimberDepository, TimberYield,
    TreeSpecies, VolcanicSpire, WoodlandGround, ZonedForest,
};

/// Encapsulated geometric quad entity with vertices, normal, and surface color.
#[derive(Clone, Copy, Debug)]
pub struct Quad {
    v0: Vec3, v1: Vec3, v2: Vec3, v3: Vec3, normal: Vec3, color: Vec3,
}

impl Quad {
    /// Primary constructor: assigns fields directly without computation.
    pub fn new(v0: Vec3, v1: Vec3, v2: Vec3, v3: Vec3, normal: Vec3, color: Vec3) -> Self {
        Self { v0, v1, v2, v3, normal, color }
    }

    /// Feeds its encapsulated data to a target without exposing raw getters.
    pub fn transfer<T>(&self, consumer: impl FnOnce(Vec3, Vec3, Vec3, Vec3, Vec3, Vec3) -> T) -> T {
        consumer(self.v0, self.v1, self.v2, self.v3, self.normal, self.color)
    }

    /// Emits this quad directly when stage == 0, or subdivides it into a recessed 3D digging crater for stages 1..=3.
    pub fn emit_crater(&self, stage: u8, target: &mut dyn MeshTarget) {
        if stage == 0 { target.push_quad(*self); return; }
        let bilerp = |fu: f32, fv: f32| {
            self.v0 * ((1.0 - fu) * (1.0 - fv)) + self.v1 * (fu * (1.0 - fv)) + self.v2 * (fu * fv) + self.v3 * ((1.0 - fu) * fv)
        };
        let s = stage.min(3) as f32;
        let (r0, depth) = (0.40 - s * 0.10, s * 0.09);
        let (r1, f0, f1, drop) = (1.0 - r0, r0 + 0.08, 0.92 - r0, self.normal * depth);
        let o = [self.v0, self.v1, self.v2, self.v3];
        let m = [bilerp(r0, r0), bilerp(r1, r0), bilerp(r1, r1), bilerp(r0, r1)];
        let b = [bilerp(f0, f0) - drop, bilerp(f1, f0) - drop, bilerp(f1, f1) - drop, bilerp(f0, f1) - drop];
        let (wall_col, floor_col) = ((self.color * 0.72).clamp(Vec3::ZERO, Vec3::ONE), (self.color * 0.52).clamp(Vec3::ZERO, Vec3::ONE));
        for i in 0..4 {
            let j = (i + 1) & 3;
            target.push_quad(Self::new(o[i], o[j], m[j], m[i], self.normal, self.color));
            let wn = (b[j] - b[i]).cross(m[i] - b[i]).normalize_or_zero();
            target.push_quad(Self::new(b[i], b[j], m[j], m[i], if wn.length_squared() > 0.5 { wn } else { self.normal }, wall_col));
        }
        target.push_quad(Self::new(b[0], b[1], b[2], b[3], self.normal, floor_col));
    }
}

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
    fn loosen_surface(&mut self, _eye: Vec3, _direction: Vec3, _stage: u8) -> bool {
        false
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
    fn loosen(&mut self, _eye: Vec3, _direction: Vec3, _stage: u8) -> bool {
        false
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

