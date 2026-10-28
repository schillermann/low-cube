use glam::Vec3;
use world::PlanetaryWorld;
use crate::envelope::BodyEnvelope;
use crate::gait::LocomotionGait;
use crate::phase::{
    AirborneFlight, AsymmetricGravity, CanopyBraking, FlightContext, FlightPhase, GroundedFlight,
    InclinePace, SkiLaunch, SlopeSurfing,
};
use crate::threshold::StepThreshold;

/// Interface representing vertical flight and gravitational physics on a planetary surface.
pub trait GravitationalFlight: Send + Sync {
    fn jump(&mut self);
    fn descend(&mut self) {}
    fn ground(&mut self) {}
    fn apply_envelope(&mut self, _envelope: BodyEnvelope) {}
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

struct FlightState {
    phase: FlightPhase,
    jump_strength: f32,
    eye_height: f32,
    vertical_thrust: i8,
}

struct FlightAerodynamics {
    gravity: AsymmetricGravity,
    canopy: CanopyBraking,
    envelope: BodyEnvelope,
}

struct SlopeKinematics {
    surfing: SlopeSurfing,
    ski_launch: SkiLaunch,
    incline: InclinePace,
}

/// Polymorphic planetary flight entity coordinating GroundedFlight and AirborneFlight phases.
pub struct SurfaceFlight {
    state: FlightState,
    aero: FlightAerodynamics,
    slope: SlopeKinematics,
}

impl SurfaceFlight {
    /// Primary constructor doing nothing except assigning arguments to fields.
    #[allow(clippy::too_many_arguments)]
    pub fn with_envelope(
        phase: FlightPhase,
        jump_strength: f32,
        gravity: AsymmetricGravity,
        canopy: CanopyBraking,
        surfing: SlopeSurfing,
        ski_launch: SkiLaunch,
        incline: InclinePace,
        eye_height: f32,
        envelope: BodyEnvelope,
    ) -> Self {
        Self {
            state: FlightState {
                phase,
                jump_strength,
                eye_height,
                vertical_thrust: 0,
            },
            aero: FlightAerodynamics {
                gravity,
                canopy,
                envelope,
            },
            slope: SlopeKinematics {
                surfing,
                ski_launch,
                incline,
            },
        }
    }

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
        Self::with_envelope(
            phase,
            jump_strength,
            gravity,
            canopy,
            surfing,
            ski_launch,
            incline,
            eye_height,
            BodyEnvelope::youth(),
        )
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
        self.state.vertical_thrust = 1;
    }

    fn descend(&mut self) {
        self.state.vertical_thrust = -1;
    }

    fn ground(&mut self) {
        self.state.phase = FlightPhase::Grounded(GroundedFlight::human());
        self.state.vertical_thrust = 0;
    }

    fn apply_envelope(&mut self, envelope: BodyEnvelope) {
        self.aero.envelope = envelope;
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
        let constrained = self.aero.envelope.constrain_with_height(position, self.state.eye_height, world);
        let in_shaft = (position - constrained).dot(surface_up) > 2.5;
        let thrust = self.state.vertical_thrust;
        self.state.vertical_thrust = 0;
        if thrust > 0 {
            if world.weightless_at(position) || in_shaft {
                self.state.phase = FlightPhase::Airborne(AirborneFlight::new(
                    self.state.jump_strength,
                    StepThreshold::human_stride(),
                ));
            } else {
                let (launch_speed, launch_momentum) = self.slope.ski_launch.evaluate_launch(
                    self.state.jump_strength,
                    stride,
                    surface_up,
                    position,
                    dt,
                    self.state.eye_height,
                    world,
                );
                self.state.phase = self.state.phase.launch(launch_speed, launch_momentum);
            }
        } else if thrust < 0 && world.weightless_at(position) {
            self.state.phase = FlightPhase::Airborne(AirborneFlight::new(
                -self.state.jump_strength,
                StepThreshold::human_stride(),
            ));
        }

        let context = FlightContext::new(
            dt,
            position,
            stride,
            surface_up,
            self.state.eye_height,
            self.aero.envelope,
            &self.aero.gravity,
            self.aero.canopy,
            self.slope.surfing,
            self.slope.incline,
            gait,
            world,
        );
        let advanced = self.state.phase.advance(&context).apply(&mut self.state.phase);
        let next_pos = if world.weightless_at(position) || world.weightless_at(advanced) || in_shaft {
            context.confine_crust(advanced)
        } else {
            advanced
        };
        if (next_pos - position).dot(surface_up) > 0.0
            && !world.weightless_at(next_pos)
            && (next_pos - world.constrain_to_surface(next_pos, self.state.eye_height)).dot(surface_up) > 2.5
        {
            let ortho = if surface_up.x.abs() < 0.5 { Vec3::X } else { Vec3::Z };
            let rim = world.constrain_to_surface(
                next_pos + ortho * 0.5 + surface_up * 32.0,
                self.state.eye_height,
            );
            if (next_pos - rim).dot(surface_up) >= -0.05 && !world.weightless_at(rim) {
                self.state.phase = FlightPhase::Grounded(GroundedFlight::human());
                return rim;
            }
        }
        if world.lethal_at(next_pos) || world.lethal_at(position) {
            self.state.phase = FlightPhase::Airborne(AirborneFlight::new(0.0, StepThreshold::human_stride()));
            return if world.lethal_at(position) { position } else { next_pos };
        }
        next_pos
    }
}
