use crate::flight::{GravitationalFlight, SurfaceFlight};

/// Interface representing planetary flight and jump kinematics.
pub trait PlanetaryPhysics: Send + Sync {
    fn surface_flight(&self, eye_height: f32) -> Box<dyn GravitationalFlight>;
}

/// Planetary physics with configured gravity, deriving jump height naturally.
pub struct TunedPhysics {
    gravity: f32,
}

impl TunedPhysics {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(gravity: f32) -> Self {
        Self { gravity }
    }

    /// Derived vertical jump height inversely proportional to gravity: h = 5.22 / g.
    fn jump_height(&self) -> f32 {
        if self.gravity > 0.0 {
            5.22 / self.gravity
        } else {
            0.522
        }
    }
}

impl PlanetaryPhysics for TunedPhysics {
    fn surface_flight(&self, eye_height: f32) -> Box<dyn GravitationalFlight> {
        Box::new(SurfaceFlight::calibrated(self.jump_height(), self.gravity, eye_height))
    }
}

/// Terrestrial planetary physics modeling Earth gravity (10 m/s²).
pub struct EarthPhysics {
    tuned: TunedPhysics,
}

impl EarthPhysics {
    /// Primary constructor initializing authentic Earth parameters.
    pub fn new() -> Self {
        Self {
            tuned: TunedPhysics::new(10.0),
        }
    }
}

impl PlanetaryPhysics for EarthPhysics {
    fn surface_flight(&self, eye_height: f32) -> Box<dyn GravitationalFlight> {
        self.tuned.surface_flight(eye_height)
    }
}
