use super::frequency::SpatialFrequency;
use glam::Vec3;

/// Coordinate space warp decorator producing folded and fluid strata.
pub struct DomainWarp {
    target: Box<dyn SpatialFrequency>,
    warp_x: Box<dyn SpatialFrequency>,
    warp_z: Box<dyn SpatialFrequency>,
    amplitude: f32,
}

impl DomainWarp {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(
        target: Box<dyn SpatialFrequency>,
        warp_x: Box<dyn SpatialFrequency>,
        warp_z: Box<dyn SpatialFrequency>,
        amplitude: f32,
    ) -> Self {
        Self {
            target,
            warp_x,
            warp_z,
            amplitude,
        }
    }
}

impl SpatialFrequency for DomainWarp {
    fn sample(&self, point: Vec3) -> f32 {
        let offset = Vec3::new(
            self.warp_x.sample(point) * self.amplitude,
            0.0,
            self.warp_z.sample(point) * self.amplitude,
        );
        self.target.sample(point + offset)
    }
}
