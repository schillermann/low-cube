use glam::Vec3;
use crate::threshold::StepThreshold;

mod aerial;
mod canopy;
mod context;
mod gravity;
mod incline;
mod progress;
mod slope;
#[cfg(test)]
mod tests;

use aerial::{AerialKinematics, AerialTrajectory, CliffReboundTrajectory, ParabolicTrajectory};
pub use canopy::CanopyBraking;
pub use context::FlightContext;
pub use gravity::AsymmetricGravity;
pub use incline::InclinePace;
pub use progress::FlightProgress;
pub use slope::{SkiLaunch, SlopeSurfing};

/// Polymorphic vertical flight phase under planetary gravity.
#[derive(Clone, Copy, Debug)]
pub enum FlightPhase {
    Grounded(GroundedFlight),
    Airborne(AirborneFlight),
}

impl FlightPhase {
    pub fn launch(&self, strength: f32, momentum: Vec3) -> Self {
        match self {
            Self::Grounded(g) => g.launch(strength, momentum),
            Self::Airborne(a) => a.launch(strength, momentum),
        }
    }

    pub fn advance(&self, ctx: &FlightContext) -> FlightProgress {
        match self {
            Self::Grounded(g) => g.advance(ctx),
            Self::Airborne(a) => a.advance(ctx),
        }
    }
}

/// Grounded state resting on the planetary surface floor.
#[derive(Clone, Copy, Debug)]
pub struct GroundedFlight {
    momentum: Vec3,
    threshold: StepThreshold,
}

impl GroundedFlight {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(threshold: StepThreshold) -> Self {
        Self::with_momentum(Vec3::ZERO, threshold)
    }

    /// Secondary constructor initializing with existing tangential slide momentum.
    pub fn with_momentum(momentum: Vec3, threshold: StepThreshold) -> Self {
        Self { momentum, threshold }
    }

    /// Secondary constructor with standard human stride calibration.
    pub fn human() -> Self {
        Self::new(StepThreshold::human_stride())
    }

    pub fn launch(&self, strength: f32, boost_momentum: Vec3) -> FlightPhase {
        FlightPhase::Airborne(AirborneFlight::with_momentum(
            strength,
            self.momentum + boost_momentum,
            self.threshold,
        ))
    }

    pub fn advance(&self, ctx: &FlightContext) -> FlightProgress {
        let stride = ctx.paced_ground_stride();
        let horizontal = ctx.candidate_point(stride);
        let surface_pos = ctx.constrain_surface(horizontal);
        let clearance = ctx.clearance(horizontal, surface_pos);
        let next_momentum = ctx.damp_surf_momentum(self.momentum);

        if !ctx.origin_supported() {
            ctx.resolve_precipice(self.threshold, ctx.candidate_point(Vec3::ZERO))
        } else if !clearance.passable_with(ctx.permits_seam_fold()) {
            ctx.resolve_precipice(self.threshold, horizontal)
        } else {
            let next_pos = ctx.resolve_paced_stride(stride, self.momentum, self.threshold);
            FlightProgress::new(
                FlightPhase::Grounded(GroundedFlight::with_momentum(next_momentum, self.threshold)),
                next_pos,
            )
        }
    }
}

/// Airborne state executing continuous parabolic trajectory under planetary gravity.
#[derive(Clone, Copy, Debug)]
pub struct AirborneFlight {
    vertical_speed: f32,
    momentum: Vec3,
    threshold: StepThreshold,
}

impl AirborneFlight {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(vertical_speed: f32, threshold: StepThreshold) -> Self {
        Self::with_momentum(vertical_speed, Vec3::ZERO, threshold)
    }

    /// Secondary constructor initializing with carried horizontal momentum.
    pub fn with_momentum(vertical_speed: f32, momentum: Vec3, threshold: StepThreshold) -> Self {
        Self {
            vertical_speed,
            momentum,
            threshold,
        }
    }

    pub(super) const fn inspect_state(&self) -> (f32, Vec3, StepThreshold) {
        (self.vertical_speed, self.momentum, self.threshold)
    }

    pub fn launch(&self, _strength: f32, _momentum: Vec3) -> FlightPhase {
        FlightPhase::Airborne(*self)
    }

    pub fn advance(&self, ctx: &FlightContext) -> FlightProgress {
        let kinematics = AerialKinematics::from_flight(self, ctx);
        let trajectory = CliffReboundTrajectory::new(ParabolicTrajectory::new(*self), *self);
        trajectory.resolve(ctx, kinematics)
    }
}
