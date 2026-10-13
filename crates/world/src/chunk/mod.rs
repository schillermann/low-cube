use glam::Vec3;
use crate::MeshTarget;

mod barrier;
mod bedrock_shell;
mod coord;
mod crust;
mod faceted_chunk;
mod greedy;
mod greedy_mesh;
mod grid;
mod neighborhood;
mod planetary_neighborhood;
mod stratified;
mod uniform;
mod void;
mod world;

#[cfg(test)]
mod bedrock_tests;
#[cfg(test)]
mod faceted_tests;
#[cfg(test)]
mod greedy_tests;
#[cfg(test)]
mod seam_tests;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod world_tests;

pub use barrier::VoxelBarrier;
pub use bedrock_shell::BedrockShell;
pub use coord::{VolumeCoord, VoxelCoord, VOLUME_EDGE_LENGTH, VOXEL_COUNT_PER_VOLUME};
pub use crust::PlanetaryCrust;
pub use faceted_chunk::FacetedVolume;
pub use greedy::FacetSlice;
pub use greedy_mesh::GreedyMesh;
pub use grid::SparsePlanetaryGrid;
pub use neighborhood::{VoidNeighborhood, VoxelNeighborhood};
pub use planetary_neighborhood::PlanetaryNeighborhood;
pub use stratified::StratifiedVolume;
pub use uniform::MonolithVolume;
pub use void::VoidVolume;
pub use world::VolumePlanet;

/// Behavioral interface for a 3D subvolume in a cubic planetary world ("Tell, Don't Ask").
pub trait VoxelVolume: Send + Sync {
    /// Checks if the local voxel at the given discrete coordinate is solid and occluding.
    fn occluding_at(&self, coord: VoxelCoord) -> bool;

    /// Delivers the occupancy and surface color of a local voxel to a mutable consumer closure.
    fn inspect_voxel(&self, coord: VoxelCoord, consumer: &mut dyn FnMut(bool, Vec3));

    /// Instructs the volume to emit its visible exterior quads directly to a mesh target.
    fn emit_mesh(
        &self,
        origin: Vec3,
        neighborhood: &dyn VoxelNeighborhood,
        target: &mut dyn MeshTarget,
    );
}
