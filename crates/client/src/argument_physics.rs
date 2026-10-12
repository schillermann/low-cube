use crate::flight::GravitationalFlight;
use crate::physics::{PlanetaryPhysics, TunedPhysics};

/// Command-line argument-driven planetary physics decorator.
pub struct ArgumentPhysics {
    arguments: Vec<String>,
    fallback: Box<dyn PlanetaryPhysics>,
}

impl ArgumentPhysics {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(arguments: Vec<String>, fallback: Box<dyn PlanetaryPhysics>) -> Self {
        Self { arguments, fallback }
    }

    /// Secondary constructor capturing arguments from the active process environment.
    pub fn process(fallback: Box<dyn PlanetaryPhysics>) -> Self {
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
}

impl PlanetaryPhysics for ArgumentPhysics {
    fn surface_flight(&self, eye_height: f32) -> Box<dyn GravitationalFlight> {
        let jump = self.value_of("--jump-height");
        let gravity = self.value_of("--gravity");

        if jump.is_some() || gravity.is_some() {
            let height = jump.unwrap_or(0.522);
            let grav = gravity.unwrap_or(9.81);
            TunedPhysics::new(height, grav).surface_flight(eye_height)
        } else {
            self.fallback.surface_flight(eye_height)
        }
    }
}
