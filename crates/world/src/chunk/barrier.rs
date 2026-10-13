use glam::Vec3;

/// Interface for physical containment boundaries and structural protection barriers ("Tell, Don't Ask").
pub trait VoxelBarrier: Send + Sync {
    /// Declarative predicate testing whether a spatial position lies safely outside the forbidden inner void.
    fn contained(&self, position: Vec3) -> bool;

    /// Arrests penetration into the forbidden inner void, projecting the vector back to the barrier surface.
    fn arrest_penetration(&self, origin: Vec3, destination: Vec3) -> Vec3;

    /// Declarative predicate testing whether the voxel at a world position is an indestructible barrier block.
    fn indestructible_at(&self, position: Vec3) -> bool;
}
