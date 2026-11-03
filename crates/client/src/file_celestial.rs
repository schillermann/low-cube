use glam::{Vec3, Vec4};
use crate::celestial::{CelestialCadence, TunedCadence};
use crate::cycle_duration::CycleDuration;
use crate::toml_document::TomlDocument;

/// Configuration file-driven celestial cadence decorator.
pub struct FileCelestial {
    cadence: Box<dyn CelestialCadence>,
}

impl FileCelestial {
    const KEY: &'static str = "cycle";

    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(cadence: Box<dyn CelestialCadence>) -> Self {
        Self { cadence }
    }

    /// Secondary constructor targeting the server.toml configuration file.
    pub fn server(fallback: Box<dyn CelestialCadence>) -> Self {
        Self::from_path("server.toml", fallback)
    }

    /// Secondary constructor loading celestial cadence configuration from an explicit file path.
    pub fn from_path(path: &str, fallback: Box<dyn CelestialCadence>) -> Self {
        let Some(doc) = TomlDocument::from_file(path) else {
            return Self::new(fallback);
        };
        if let Some(cycle) = doc.property(Self::KEY).and_then(|raw| CycleDuration::parse(&raw)) {
            Self::new(Box::new(TunedCadence::new(cycle.seconds(), fallback)))
        } else {
            Self::new(fallback)
        }
    }
}

impl CelestialCadence for FileCelestial {
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
