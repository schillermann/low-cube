use crate::flight::{GravitationalFlight, SurfaceFlight};

/// Interface representing planetary flight and jump kinematics.
pub trait PlanetaryPhysics: Send + Sync {
    fn surface_flight(&self, eye_height: f32) -> Box<dyn GravitationalFlight>;
}

/// Planetary physics with explicit jump height and gravity.
pub struct TunedPhysics {
    jump_height: f32,
    gravity: f32,
}

impl TunedPhysics {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(jump_height: f32, gravity: f32) -> Self {
        Self {
            jump_height,
            gravity,
        }
    }
}

impl PlanetaryPhysics for TunedPhysics {
    fn surface_flight(&self, eye_height: f32) -> Box<dyn GravitationalFlight> {
        Box::new(SurfaceFlight::calibrated(self.jump_height, self.gravity, eye_height))
    }
}

/// Terrestrial planetary physics modeling Earth gravity (9.81 m/s²) and human vertical leap (0.522 m).
pub struct EarthPhysics {
    tuned: TunedPhysics,
}

impl EarthPhysics {
    /// Primary constructor initializing authentic Earth parameters.
    pub fn new() -> Self {
        Self {
            tuned: TunedPhysics::new(0.522, 9.81),
        }
    }
}

impl PlanetaryPhysics for EarthPhysics {
    fn surface_flight(&self, eye_height: f32) -> Box<dyn GravitationalFlight> {
        self.tuned.surface_flight(eye_height)
    }
}
