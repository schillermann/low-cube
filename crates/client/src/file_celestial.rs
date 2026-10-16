use std::fs;
use glam::{Vec3, Vec4};
use crate::celestial::{CelestialCadence, TunedCadence};
use crate::cycle_duration::CycleDuration;

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
        let content = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => return Self::new(fallback),
        };
        if let Some(seconds) = Self::property_in_text(&content) {
            Self::new(Box::new(TunedCadence::new(seconds, fallback)))
        } else {
            Self::new(fallback)
        }
    }

    fn property_in_text(content: &str) -> Option<f32> {
        for line in content.lines().map(str::trim) {
            if line.is_empty() || line.starts_with('#') || (line.starts_with('[') && line.ends_with(']')) {
                continue;
            }
            if let Some((key, val)) = line.split_once('=') {
                if key.trim().eq_ignore_ascii_case(Self::KEY) {
                    let raw = val.split('#').next().unwrap_or("").trim();
                    let unquoted = raw.trim_matches('"').trim_matches('\'').trim();
                    if let Some(cycle) = CycleDuration::parse(unquoted) {
                        return Some(cycle.seconds());
                    }
                }
            }
        }
        None
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
