use glam::Vec3;

/// Interface for querying neighbor voxel occlusion and material properties ("Tell, Don't Ask").
pub trait VoxelNeighborhood: Send + Sync {
    /// Determines whether the voxel at relative position (x, y, z) is opaque and occludes.
    fn occluding_at(&self, x: i32, y: i32, z: i32) -> bool;

    /// Dispatches the color of the voxel at (x, y, z).
    fn color_at(&self, x: i32, y: i32, z: i32) -> Vec3;
}

/// A neighborhood where every surrounding position is empty space (void).
pub struct VoidNeighborhood;

impl VoxelNeighborhood for VoidNeighborhood {
    fn occluding_at(&self, _x: i32, _y: i32, _z: i32) -> bool {
        false
    }

    fn color_at(&self, _x: i32, _y: i32, _z: i32) -> Vec3 {
        Vec3::ZERO
    }
}
