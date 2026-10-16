use std::fs;
use glam::{Vec3, Vec4};
use crate::celestial::{CelestialCadence, TunedCadence};

/// Configuration file-driven celestial cadence decorator.
pub struct FileCelestial {
    path: String,
    fallback: Box<dyn CelestialCadence>,
}

impl FileCelestial {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(path: String, fallback: Box<dyn CelestialCadence>) -> Self {
        Self { path, fallback }
    }

    /// Secondary constructor targeting the server.toml configuration file.
    pub fn server(fallback: Box<dyn CelestialCadence>) -> Self {
        Self::new("server.toml".to_string(), fallback)
    }

    fn property_in_text(content: &str, key_name: &str) -> Option<f32> {
        content
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty() && !line.starts_with('#') && !line.starts_with('['))
            .find_map(|line| {
                let (key, val) = line.split_once('=')?;
                if key.trim() == key_name {
                    val.split('#').next()?.trim().parse::<f32>().ok()
                } else {
                    None
                }
            })
    }

    fn read_duration(&self) -> Option<f32> {
        let content = fs::read_to_string(&self.path).ok()?;
        Self::property_in_text(&content, "day_cycle_seconds")
    }
}

impl CelestialCadence for FileCelestial {
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
        Box::new(Self::new(self.path.clone(), self.fallback.clone_cadence()))
    }
}
