use glam::Vec3;
use crate::threshold::StepThreshold;

mod canopy;
mod context;
mod gravity;
mod incline;
mod progress;
mod slope;
#[cfg(test)]
mod tests;

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

    pub fn launch(&self, _strength: f32, _momentum: Vec3) -> FlightPhase {
        FlightPhase::Airborne(*self)
    }

    pub fn advance(&self, ctx: &FlightContext) -> FlightProgress {
        let (surface_up, vertical_speed, effective_stride) = ctx.reorient_edge_flight(self.vertical_speed);
        let gravity_delta = ctx.gravitational_acceleration(vertical_speed);
        let unbraked_speed = vertical_speed - gravity_delta;
        let next_speed = ctx.brake_canopy(unbraked_speed);

        let airborne_stride = if self.momentum.length_squared() > 1e-4 {
            effective_stride + self.momentum * ctx.dt()
        } else {
            effective_stride
        };

        let vertical_step = surface_up * (next_speed * ctx.dt());
        if ctx.weightless_ascent(next_speed) {
            let raw = ctx.candidate_point(airborne_stride) + vertical_step;
            let confined = ctx.confine_crust(raw);
            let bonked_speed = if (confined - raw).dot(surface_up) < -1e-4 { next_speed.min(0.0) } else { next_speed };
            let next_momentum = ctx.damp_aerial_momentum(self.momentum);
            return FlightProgress::new(
                FlightPhase::Airborne(AirborneFlight::with_momentum(bonked_speed, next_momentum, self.threshold)),
                confined,
            );
        }
        let rebound = ctx.resolve_aerial_stride(airborne_stride, surface_up, vertical_step, self.threshold);
        let supported = ctx.constrain_surface(rebound);
        let clearance = ctx.clearance(rebound, supported);
        let orig_pt = ctx.candidate_point(Vec3::ZERO);
        let orig_surf = ctx.constrain_surface(orig_pt);

        if clearance.cliff_impact(self.threshold)
            && (supported - orig_surf).dot(surface_up) > 0.35
            && (supported - orig_pt).dot(surface_up) > 0.35
        {
            let unlifted = rebound - surface_up * vertical_step.dot(surface_up).max(0.0);
            let bonked = if ctx.clearance(unlifted, ctx.constrain_surface(unlifted)).cliff_impact(self.threshold) {
                ctx.candidate_point(Vec3::ZERO) + surface_up * vertical_step.dot(surface_up).min(0.0)
            } else {
                unlifted
            };
            let next_momentum = ctx.damp_aerial_momentum(self.momentum);
            return FlightProgress::new(
                FlightPhase::Airborne(AirborneFlight::with_momentum(next_speed.min(0.0), next_momentum, self.threshold)),
                bonked,
            );
        }

        let penetration = (supported - rebound).dot(surface_up);
        let safe_rebound = if penetration > 0.0 {
            supported
        } else {
            rebound
        };

        if (next_speed <= 0.0 || penetration > 0.0) && clearance.floor_contact(self.threshold) {
            let surf_momentum = ctx.evaluate_touchdown(-next_speed.min(0.0), supported);
            FlightProgress::new(
                FlightPhase::Grounded(GroundedFlight::with_momentum(surf_momentum, self.threshold)),
                supported,
            )
        } else {
            let next_momentum = ctx.damp_aerial_momentum(self.momentum);
            FlightProgress::new(
                FlightPhase::Airborne(AirborneFlight::with_momentum(next_speed, next_momentum, self.threshold)),
                safe_rebound,
            )
        }
    }
}
