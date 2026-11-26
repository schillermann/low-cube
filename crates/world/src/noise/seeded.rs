use glam::Vec3;
use crate::WorldSeed;
use super::SpatialFrequency;

/// `SpatialFrequency` decorator translating evaluation coordinates by a deterministic `WorldSeed` vector (`2` fields).
pub struct SeededFrequency<F> {
    origin: F,
    shift: Vec3,
}

impl<F> SeededFrequency<F> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(origin: F, shift: Vec3) -> Self {
        Self { origin, shift }
    }

    /// Secondary constructor deriving the 3D domain shift from `seed` (`Vec3::ZERO` for `WorldSeed::origin()`).
    pub fn seeded(origin: F, seed: WorldSeed) -> Self {
        Self::new(origin, seed.spatial_shift())
    }
}

impl<F: SpatialFrequency> SpatialFrequency for SeededFrequency<F> {
    fn sample(&self, point: Vec3) -> f32 {
        self.origin.sample(point + self.shift)
    }
}
