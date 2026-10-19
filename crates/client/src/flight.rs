use glam::Vec3;
use world::PlanetaryWorld;
use crate::gait::LocomotionGait;
use crate::phase::{
    AsymmetricGravity, CanopyBraking, FlightContext, FlightPhase, GroundedFlight,
    InclinePace, SkiLaunch, SlopeSurfing,
};

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
    gravity: AsymmetricGravity,
    canopy: CanopyBraking,
    surfing: SlopeSurfing,
    ski_launch: SkiLaunch,
    incline: InclinePace,
    eye_height: f32,
    jump_requested: bool,
}

impl SurfaceFlight {
    /// Primary constructor doing nothing except assigning arguments to fields.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        phase: FlightPhase,
        jump_strength: f32,
        gravity: AsymmetricGravity,
        canopy: CanopyBraking,
        surfing: SlopeSurfing,
        ski_launch: SkiLaunch,
        incline: InclinePace,
        eye_height: f32,
    ) -> Self {
        Self {
            phase,
            jump_strength,
            gravity,
            canopy,
            surfing,
            ski_launch,
            incline,
            eye_height,
            jump_requested: false,
        }
    }

    /// Secondary constructor calculating jump velocity from configured jump height and gravity.
    pub fn calibrated(jump_height: f32, gravity: f32, eye_height: f32) -> Self {
        let jump_strength = (2.0 * gravity * jump_height).max(0.0).sqrt();
        Self::new(
            FlightPhase::Grounded(GroundedFlight::human()),
            jump_strength,
            AsymmetricGravity::terrestrial(gravity),
            CanopyBraking::calibrated(),
            SlopeSurfing::calibrated(),
            SkiLaunch::calibrated(),
            InclinePace::calibrated(),
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
        self.jump_requested = true;
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
        if self.jump_requested {
            self.jump_requested = false;
            let (launch_speed, launch_momentum) = self.ski_launch.evaluate_launch(
                self.jump_strength,
                stride,
                surface_up,
                position,
                dt,
                self.eye_height,
                world,
            );
            self.phase = self.phase.launch(launch_speed, launch_momentum);
        }

        let context = FlightContext::new(
            dt,
            position,
            stride,
            surface_up,
            self.eye_height,
            &self.gravity,
            self.canopy,
            self.surfing,
            self.incline,
            gait,
            world,
        );
        self.phase.advance(&context).apply(&mut self.phase)
    }
}
