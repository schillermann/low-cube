use crate::flight::GravitationalFlight;
use crate::physics::{PlanetaryPhysics, TunedPhysics};

/// Process environment variable-driven planetary physics decorator.
pub struct EnvironmentPhysics {
    fallback: Box<dyn PlanetaryPhysics>,
}

impl EnvironmentPhysics {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(fallback: Box<dyn PlanetaryPhysics>) -> Self {
        Self { fallback }
    }

    fn read_var(key: &str) -> Option<f32> {
        std::env::var(key).ok().and_then(|v| v.parse::<f32>().ok())
    }
}

impl PlanetaryPhysics for EnvironmentPhysics {
    fn surface_flight(&self, eye_height: f32) -> Box<dyn GravitationalFlight> {
        if let Some(gravity) = Self::read_var("LOW_CUBE_GRAVITY") {
            TunedPhysics::new(gravity).surface_flight(eye_height)
        } else {
            self.fallback.surface_flight(eye_height)
        }
    }
}
