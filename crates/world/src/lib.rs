use glam::Vec3;

mod beveled_world;
mod biome;
mod block;
mod cluster;
mod elevated_face;
mod face;
mod frame;
mod gallery;
mod mesh_target;
mod mountain;
mod null_face;
mod planet;
mod tinted_face;
mod topology;

#[cfg(test)]
mod beveled_world_tests;
#[cfg(test)]
mod cluster_tests;
#[cfg(test)]
mod face_decorators_tests;
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
pub use block::{
    Block, CornerHeading, CuboidShape, DirtBlock, ElevatedShape, FormBlock, GridCell,
    PyramidPeak, RampWedge, RidgeCorner, SandBlock, SnowBlock, StairFlight,
    StoneBlock, MaterialBlock, ValleyCorner, VoxelShape, WedgeHeading,
};
pub use cluster::VoxelCluster;
pub use elevated_face::ElevatedFace;
pub use face::PlanarFace;
pub use frame::SurfaceFrame;
pub use gallery::BlockGallery;
pub use mesh_target::{ColoredMeshTarget, CountedMeshTarget, OffsetMeshTarget, TintedMeshTarget};
pub use mountain::VoxelMountain;
pub use null_face::NullFace;
pub use planet::CubePlanet;
pub use tinted_face::TintedFace;
pub use topology::{
    BevelTransition, CoordinateFold, CubeFace, FacePerimeter, FoldedFace,
    FoldedPoint, PerimeterCrossing, PerimeterEdge, PlanetaryTopology, SeamBevel,
    SeamFold, Topology,
};

/// Encapsulated geometric quad entity with vertices, normal, and surface color.
#[derive(Clone, Copy, Debug)]
pub struct Quad {
    v0: Vec3,
    v1: Vec3,
    v2: Vec3,
    v3: Vec3,
    normal: Vec3,
    color: Vec3,
}

impl Quad {
    /// Primary constructor: assigns fields directly without computation.
    pub fn new(v0: Vec3, v1: Vec3, v2: Vec3, v3: Vec3, normal: Vec3, color: Vec3) -> Self {
        Self {
            v0,
            v1,
            v2,
            v3,
            normal,
            color,
        }
    }

    /// Feeds its encapsulated data to a target without exposing raw getters.
    pub fn transfer<T>(&self, consumer: impl FnOnce(Vec3, Vec3, Vec3, Vec3, Vec3, Vec3) -> T) -> T {
        consumer(self.v0, self.v1, self.v2, self.v3, self.normal, self.color)
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
}

/// Interface representing an individual face of a planetary body.
pub trait Face: Send + Sync {
    fn matches(&self, point: Vec3) -> bool;
    fn surface_up(&self) -> Vec3;
    fn pull_direction(&self) -> Vec3;
    fn constrain(&self, point: Vec3, distance: f32) -> Vec3;
    fn emit_surface(&self, target: &mut dyn MeshTarget);
}

