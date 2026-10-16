use glam::{Vec3, Vec4};
use crate::celestial::{CelestialCadence, TunedCadence};
use crate::cycle_duration::CycleDuration;

/// Process environment variable-driven celestial cadence decorator.
pub struct EnvironmentCelestial {
    cadence: Box<dyn CelestialCadence>,
}

impl EnvironmentCelestial {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(cadence: Box<dyn CelestialCadence>) -> Self {
        Self { cadence }
    }

    /// Secondary constructor reading process environment variables.
    pub fn process(fallback: Box<dyn CelestialCadence>) -> Self {
        if let Ok(val) = std::env::var("LOW_CUBE_CYCLE") {
            if let Some(dur) = CycleDuration::parse(&val) {
                return Self::new(Box::new(TunedCadence::new(dur.seconds(), fallback)));
            }
        }
        Self::new(fallback)
    }
}

impl CelestialCadence for EnvironmentCelestial {
    fn duration_seconds(&self) -> f32 {
        self.cadence.duration_seconds()
    }

    fn day_progress(&self, elapsed_seconds: f64) -> f32 {
        self.cadence.day_progress(elapsed_seconds)
    }

    fn sun_direction(&self, progress: f32) -> Vec3 {
        self.cadence.sun_direction(progress)
    }

    fn sky_color(&self, progress: f32) -> Vec4 {
        self.cadence.sky_color(progress)
    }

    fn clone_cadence(&self) -> Box<dyn CelestialCadence> {
        Box::new(Self::new(self.cadence.clone_cadence()))
    }
}
