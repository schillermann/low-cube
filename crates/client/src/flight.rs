use glam::Vec3;
use world::PlanetaryWorld;
use crate::gait::LocomotionGait;
use crate::phase::{FlightContext, FlightPhase, GroundedFlight};

/// Interface representing vertical flight and gravitational physics on a planetary surface.
pub trait GravitationalFlight: Send + Sync {
    fn jump(&mut self);
    fn advance(
        &mut self,
        dt: f32,
        position: Vec3,
        stride: Vec3,
        gait: &dyn LocomotionGait,
        surface_up: Vec3,
        world: &dyn PlanetaryWorld,
    ) -> Vec3;
}

/// Polymorphic planetary flight entity coordinating GroundedFlight and AirborneFlight phases.
pub struct SurfaceFlight {
    phase: FlightPhase,
    jump_strength: f32,
    gravity: f32,
    eye_height: f32,
}

impl SurfaceFlight {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(
        phase: FlightPhase,
        jump_strength: f32,
        gravity: f32,
        eye_height: f32,
    ) -> Self {
        Self {
            phase,
            jump_strength,
            gravity,
            eye_height,
        }
    }

    /// Secondary constructor calculating jump velocity from configured jump height and gravity.
    pub fn calibrated(jump_height: f32, gravity: f32, eye_height: f32) -> Self {
        let jump_strength = (2.0 * gravity * jump_height).max(0.0).sqrt();
        Self::new(
            FlightPhase::Grounded(GroundedFlight::human()),
            jump_strength,
            gravity,
            eye_height,
        )
    }

    /// Secondary constructor initializing in grounded state with authentic Earth gravity and realistic human leap.
    #[cfg(test)]
    pub fn terrestrial(eye_height: f32) -> Self {
        Self::calibrated(0.522, 9.81, eye_height)
    }
}

impl GravitationalFlight for SurfaceFlight {
    fn jump(&mut self) {
        self.phase = self.phase.jump(self.jump_strength);
    }

    fn advance(
        &mut self,
        dt: f32,
        position: Vec3,
        stride: Vec3,
        gait: &dyn LocomotionGait,
        surface_up: Vec3,
        world: &dyn PlanetaryWorld,
    ) -> Vec3 {
        let context = FlightContext::new(
            dt,
            position,
            stride,
            surface_up,
            self.eye_height,
            self.gravity,
            gait,
            world,
        );
        self.phase.advance(&context).apply(&mut self.phase)
    }
}
