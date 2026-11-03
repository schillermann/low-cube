use crate::command_flags::CommandFlags;
use crate::flight::GravitationalFlight;
use crate::physics::{PlanetaryPhysics, TunedPhysics};

/// Command-line argument-driven planetary physics decorator.
pub struct ArgumentPhysics {
    flags: CommandFlags,
    fallback: Box<dyn PlanetaryPhysics>,
}

impl ArgumentPhysics {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn with_flags(flags: CommandFlags, fallback: Box<dyn PlanetaryPhysics>) -> Self {
        Self { flags, fallback }
    }

    /// Secondary constructor wrapping an explicit argument vector.
    pub fn new(arguments: Vec<String>, fallback: Box<dyn PlanetaryPhysics>) -> Self {
        Self::with_flags(CommandFlags::new(arguments), fallback)
    }

    /// Secondary constructor capturing arguments from the active process environment.
    pub fn process(fallback: Box<dyn PlanetaryPhysics>) -> Self {
        Self::new(std::env::args().collect(), fallback)
    }
}

impl PlanetaryPhysics for ArgumentPhysics {
    fn surface_flight(&self, eye_height: f32) -> Box<dyn GravitationalFlight> {
        if let Some(gravity) = self.flags.float_of("--gravity") {
            TunedPhysics::new(gravity).surface_flight(eye_height)
        } else {
            self.fallback.surface_flight(eye_height)
        }
    }
}
