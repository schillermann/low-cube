use glam::{Vec3, Vec4};
use crate::celestial::{CelestialCadence, TunedCadence};
use crate::command_flags::CommandFlags;
use crate::cycle_duration::CycleDuration;

/// Command-line argument-driven celestial cadence decorator.
pub struct ArgumentCelestial {
    cadence: Box<dyn CelestialCadence>,
}

impl ArgumentCelestial {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(cadence: Box<dyn CelestialCadence>) -> Self {
        Self { cadence }
    }

    /// Secondary constructor capturing arguments from the active process environment.
    pub fn process(fallback: Box<dyn CelestialCadence>) -> Self {
        let args: Vec<String> = std::env::args().collect();
        Self::from_arguments(&args, fallback)
    }

    /// Secondary constructor parsing an explicit arguments slice.
    pub fn from_arguments(args: &[String], fallback: Box<dyn CelestialCadence>) -> Self {
        if let Some(raw) = CommandFlags::from_slice(args).value_of("--cycle") {
            if let Some(dur) = CycleDuration::parse(&raw) {
                return Self::new(Box::new(TunedCadence::new(dur.seconds(), fallback)));
            }
        }
        Self::new(fallback)
    }
}

impl CelestialCadence for ArgumentCelestial {
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
