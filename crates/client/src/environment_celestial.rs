use glam::{Vec3, Vec4};
use crate::celestial::{CelestialCadence, TunedCadence};

/// Process environment variable-driven celestial cadence decorator.
pub struct EnvironmentCelestial {
    fallback: Box<dyn CelestialCadence>,
}

impl EnvironmentCelestial {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(fallback: Box<dyn CelestialCadence>) -> Self {
        Self { fallback }
    }

    fn read_var(key: &str) -> Option<f32> {
        std::env::var(key).ok().and_then(|v| v.parse::<f32>().ok())
    }

    fn read_duration(&self) -> Option<f32> {
        Self::read_var("LOW_CUBE_DAY_CYCLE")
    }
}

impl CelestialCadence for EnvironmentCelestial {
    fn duration_seconds(&self) -> f32 {
        if let Some(dur) = self.read_duration() {
            dur
        } else {
            self.fallback.duration_seconds()
        }
    }

    fn day_progress(&self, elapsed_seconds: f64) -> f32 {
        if let Some(dur) = self.read_duration() {
            TunedCadence::new(dur).day_progress(elapsed_seconds)
        } else {
            self.fallback.day_progress(elapsed_seconds)
        }
    }

    fn sun_direction(&self, progress: f32) -> Vec3 {
        self.fallback.sun_direction(progress)
    }

    fn sky_color(&self, progress: f32) -> Vec4 {
        self.fallback.sky_color(progress)
    }

    fn clone_cadence(&self) -> Box<dyn CelestialCadence> {
        Box::new(Self::new(self.fallback.clone_cadence()))
    }
}
