use glam::{Vec3, Vec4};
use crate::celestial::{CelestialCadence, TunedCadence};

/// Command-line argument-driven celestial cadence decorator.
pub struct ArgumentCelestial {
    arguments: Vec<String>,
    fallback: Box<dyn CelestialCadence>,
}

impl ArgumentCelestial {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(arguments: Vec<String>, fallback: Box<dyn CelestialCadence>) -> Self {
        Self { arguments, fallback }
    }

    /// Secondary constructor capturing arguments from the active process environment.
    pub fn process(fallback: Box<dyn CelestialCadence>) -> Self {
        Self::new(std::env::args().collect(), fallback)
    }

    fn value_of(&self, flag: &str) -> Option<f32> {
        self.arguments
            .windows(2)
            .find(|pair| pair[0] == flag)
            .and_then(|pair| pair[1].parse::<f32>().ok())
            .or_else(|| {
                self.arguments
                    .iter()
                    .find(|arg| arg.starts_with(flag) && arg[flag.len()..].starts_with('='))
                    .and_then(|arg| arg.split_once('='))
                    .and_then(|(_, val)| val.parse::<f32>().ok())
            })
    }

    fn read_duration(&self) -> Option<f32> {
        self.value_of("--day-cycle")
            .or_else(|| self.value_of("--day-cycle-seconds"))
    }
}

impl CelestialCadence for ArgumentCelestial {
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
        Box::new(Self::new(self.arguments.clone(), self.fallback.clone_cadence()))
    }
}
