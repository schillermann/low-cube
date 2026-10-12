use std::fs;
use crate::flight::GravitationalFlight;
use crate::physics::{PlanetaryPhysics, TunedPhysics};

/// Configuration file-driven planetary physics decorator.
pub struct FilePhysics {
    path: String,
    fallback: Box<dyn PlanetaryPhysics>,
}

impl FilePhysics {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(path: String, fallback: Box<dyn PlanetaryPhysics>) -> Self {
        Self { path, fallback }
    }

    /// Secondary constructor targeting the server.toml configuration file.
    pub fn server(fallback: Box<dyn PlanetaryPhysics>) -> Self {
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

    fn read_values(&self) -> Option<(f32, f32)> {
        let content = fs::read_to_string(&self.path).ok()?;
        let jump = Self::property_in_text(&content, "jump_height");
        let gravity = Self::property_in_text(&content, "gravity");

        if jump.is_some() || gravity.is_some() {
            Some((jump.unwrap_or(0.522), gravity.unwrap_or(9.81)))
        } else {
            None
        }
    }
}

impl PlanetaryPhysics for FilePhysics {
    fn surface_flight(&self, eye_height: f32) -> Box<dyn GravitationalFlight> {
        if let Some((height, grav)) = self.read_values() {
            TunedPhysics::new(height, grav).surface_flight(eye_height)
        } else {
            self.fallback.surface_flight(eye_height)
        }
    }
}
