use glam::Vec3;

/// Interface for spatial frequency evaluation in continuous 3D space ("Tell, Don't Ask").
pub trait SpatialFrequency: Send + Sync {
    /// Evaluates continuous frequency amplitude at a 3D coordinate in [-1.0, 1.0].
    fn sample(&self, point: Vec3) -> f32;
}
