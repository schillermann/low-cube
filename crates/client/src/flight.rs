use glam::Vec3;
use world::PlanetaryWorld;
use crate::gait::LocomotionGait;
use crate::phase::{
    AirborneFlight, AsymmetricGravity, CanopyBraking, FlightContext, FlightPhase, GroundedFlight,
    InclinePace, SkiLaunch, SlopeSurfing,
};
use crate::threshold::StepThreshold;

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
        let in_shaft = (position - world.constrain_to_surface(position, self.eye_height)).dot(surface_up) > 2.5;
        if self.jump_requested {
            self.jump_requested = false;
            if world.weightless_at(position) || in_shaft {
                self.phase = FlightPhase::Airborne(AirborneFlight::new(self.jump_strength, StepThreshold::human_stride()));
            } else {
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
        let advanced = self.phase.advance(&context).apply(&mut self.phase);
        let next_pos = if world.weightless_at(position) || world.weightless_at(advanced) {
            world.confine_beneath_crust(advanced)
        } else {
            advanced
        };
        if (next_pos - position).dot(surface_up) > 0.0
            && !world.weightless_at(next_pos)
            && (next_pos - world.constrain_to_surface(next_pos, self.eye_height)).dot(surface_up) > 2.5
        {
            let ortho = if surface_up.x.abs() < 0.5 { Vec3::X } else { Vec3::Z };
            let rim = world.constrain_to_surface(next_pos + ortho * 0.5 + surface_up * 32.0, self.eye_height);
            if (next_pos - rim).dot(surface_up) >= -0.05 && !world.weightless_at(rim) {
                self.phase = FlightPhase::Grounded(GroundedFlight::human());
                return rim;
            }
            let next_in_shaft = (next_pos - world.constrain_to_surface(next_pos, self.eye_height)).dot(surface_up) > 2.5;
            if next_in_shaft {
                self.phase = FlightPhase::Airborne(AirborneFlight::new(self.jump_strength, StepThreshold::human_stride()));
            }
        }
        if world.lethal_at(next_pos) || world.lethal_at(position) {
            self.phase = FlightPhase::Airborne(AirborneFlight::new(0.0, StepThreshold::human_stride()));
            return if world.lethal_at(position) { position } else { next_pos };
        }
        next_pos
    }
}
