use crate::flight::GravitationalFlight;
use crate::physics::{PlanetaryPhysics, TunedPhysics};
use crate::toml_document::TomlDocument;

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

    fn read_gravity(&self) -> Option<f32> {
        TomlDocument::from_file(&self.path)?
            .property("gravity")?
            .parse::<f32>()
            .ok()
    }
}

impl PlanetaryPhysics for FilePhysics {
    fn surface_flight(&self, eye_height: f32) -> Box<dyn GravitationalFlight> {
        if let Some(grav) = self.read_gravity() {
            TunedPhysics::new(grav).surface_flight(eye_height)
        } else {
            self.fallback.surface_flight(eye_height)
        }
    }
}
