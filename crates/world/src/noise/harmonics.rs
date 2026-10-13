use super::frequency::SpatialFrequency;
use glam::Vec3;

/// Multi-octave fractal harmonic decorator compounding spatial frequencies.
pub struct FractalHarmonics {
    base: Box<dyn SpatialFrequency>,
    octaves: u32,
    persistence: f32,
    lacunarity: f32,
}

impl FractalHarmonics {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(
        base: Box<dyn SpatialFrequency>,
        octaves: u32,
        persistence: f32,
        lacunarity: f32,
    ) -> Self {
        Self {
            base,
            octaves,
            persistence,
            lacunarity,
        }
    }
}

impl SpatialFrequency for FractalHarmonics {
    fn sample(&self, point: Vec3) -> f32 {
        let mut total = 0.0;
        let mut frequency = 1.0;
        let mut amplitude = 1.0;
        let mut max_value = 0.0;

        for _ in 0..self.octaves {
            total += self.base.sample(point * frequency) * amplitude;
            max_value += amplitude;
            amplitude *= self.persistence;
            frequency *= self.lacunarity;
        }

        if max_value > 0.0 {
            total / max_value
        } else {
            0.0
        }
    }
}
