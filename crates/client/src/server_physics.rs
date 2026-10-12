use crate::argument_physics::ArgumentPhysics;
use crate::environment_physics::EnvironmentPhysics;
use crate::file_physics::FilePhysics;
use crate::flight::GravitationalFlight;
use crate::physics::{EarthPhysics, PlanetaryPhysics};

/// Server launch physics decorator coordinating CLI arguments, environment variables, config file, and Earth baseline.
pub struct ServerPhysics {
    physics: Box<dyn PlanetaryPhysics>,
}

impl ServerPhysics {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(physics: Box<dyn PlanetaryPhysics>) -> Self {
        Self { physics }
    }

    /// Secondary constructor assembling the full decorator chain in priority order.
    pub fn assembled() -> Self {
        Self::new(Box::new(
            ArgumentPhysics::process(Box::new(
                EnvironmentPhysics::new(Box::new(
                    FilePhysics::server(Box::new(
                        EarthPhysics::new(),
                    )),
                )),
            )),
        ))
    }
}

impl PlanetaryPhysics for ServerPhysics {
    fn surface_flight(&self, eye_height: f32) -> Box<dyn GravitationalFlight> {
        self.physics.surface_flight(eye_height)
    }
}
