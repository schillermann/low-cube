use glam::Vec3;
use crate::threshold::{StepThreshold, SurfaceStep};
use super::{AirborneFlight, FlightContext, FlightPhase, FlightProgress, GroundedFlight};

const CLIFF_BONK_RISE: f32 = 0.35;
const TOP_REACHED_PENETRATION: f32 = 0.04;
const SLOPE_ORIGIN_TOLERANCE: f32 = -0.01;

/// Polymorphic airborne trajectory contract decorated by cliff rebound and weightless core/shaft flight.
pub trait AerialTrajectory {
    fn resolve(&self, ctx: &FlightContext, step: AerialKinematics) -> FlightProgress;
}

/// Pre-evaluated vertical and horizontal step kinematics for an airborne frame.
#[derive(Clone, Copy, Debug)]
pub struct AerialKinematics {
    surface_up: Vec3,
    next_speed: f32,
    airborne_stride: Vec3,
    vertical_step: Vec3,
}

impl AerialKinematics {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(surface_up: Vec3, next_speed: f32, airborne_stride: Vec3, vertical_step: Vec3) -> Self {
        Self { surface_up, next_speed, airborne_stride, vertical_step }
    }

    pub fn from_flight(flight: &AirborneFlight, ctx: &FlightContext) -> Self {
        let (v_speed, momentum, _) = flight.inspect_state();
        let (surface_up, vertical_speed, effective_stride) = ctx.reorient_edge_flight(v_speed);
        let gravity_delta = ctx.gravitational_acceleration(vertical_speed);
        let next_speed = ctx.brake_canopy(vertical_speed - gravity_delta);
        let airborne_stride = if momentum.length_squared() > 1e-4 {
            effective_stride + momentum * ctx.dt()
        } else {
            effective_stride
        };
        let vertical_step = surface_up * (next_speed * ctx.dt());
        Self::new(surface_up, next_speed, airborne_stride, vertical_step)
    }

    pub fn rebound_point(&self, ctx: &FlightContext, threshold: StepThreshold) -> (Vec3, bool, f32) {
        let raw_rebound = ctx.resolve_aerial_stride(self.airborne_stride, self.surface_up, self.vertical_step, threshold);
        let rebound = if self.next_speed > 0.0 { ctx.confine_crust(raw_rebound) } else { raw_rebound };
        let ceiling_bonk = (rebound - raw_rebound).dot(self.surface_up) < -1e-4;
        let speed = if ceiling_bonk { self.next_speed.min(0.0) } else { self.next_speed };
        (rebound, ceiling_bonk, speed)
    }
}

/// Base parabolic trajectory resolving ceiling contact, slope walking, and touchdown landing.
pub struct ParabolicTrajectory {
    flight: AirborneFlight,
}

impl ParabolicTrajectory {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(flight: AirborneFlight) -> Self {
        Self { flight }
    }
}

impl AerialTrajectory for ParabolicTrajectory {
    fn resolve(&self, ctx: &FlightContext, step: AerialKinematics) -> FlightProgress {
        let (_, momentum, threshold) = self.flight.inspect_state();
        let (rebound, ceiling_bonk, next_speed) = step.rebound_point(ctx, threshold);
        let supported = ctx.constrain_surface(rebound);
        let clearance = ctx.clearance(rebound, supported);
        let orig_pt = ctx.candidate_point(Vec3::ZERO);
        let orig_surf = ctx.constrain_surface(orig_pt);
        let penetration = (supported - rebound).dot(step.surface_up);
        let on_walkable_slope = (orig_pt - orig_surf).dot(step.surface_up) >= SLOPE_ORIGIN_TOLERANCE
            && SurfaceStep::new(orig_surf, supported, step.surface_up).walkable_with(threshold);
        let top_reached = penetration > 0.0 && penetration <= TOP_REACHED_PENETRATION;
        let descending = next_speed <= 0.0 && !ceiling_bonk;
        let safe_rebound = if penetration > 0.0 && (descending || on_walkable_slope || top_reached) {
            supported
        } else {
            rebound
        };
        if (descending || (penetration > 0.0 && (on_walkable_slope || top_reached))) && clearance.floor_contact(threshold) {
            let surf_momentum = ctx.evaluate_touchdown(-next_speed.min(0.0), supported);
            FlightProgress::new(
                FlightPhase::Grounded(GroundedFlight::with_momentum(surf_momentum, threshold)),
                supported,
            )
        } else {
            let next_momentum = ctx.damp_aerial_momentum(momentum);
            FlightProgress::new(
                FlightPhase::Airborne(AirborneFlight::with_momentum(next_speed, next_momentum, threshold)),
                safe_rebound,
            )
        }
    }
}

/// Decorator intercepting cliff-wall impacts before parabolic touchdown resolution.
pub struct CliffReboundTrajectory<T: AerialTrajectory> {
    inner: T,
    flight: AirborneFlight,
}

impl<T: AerialTrajectory> CliffReboundTrajectory<T> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(inner: T, flight: AirborneFlight) -> Self {
        Self { inner, flight }
    }
}

impl<T: AerialTrajectory> AerialTrajectory for CliffReboundTrajectory<T> {
    fn resolve(&self, ctx: &FlightContext, step: AerialKinematics) -> FlightProgress {
        let (_, momentum, threshold) = self.flight.inspect_state();
        let (rebound, _, next_speed) = step.rebound_point(ctx, threshold);
        let supported = ctx.constrain_surface(rebound);
        let clearance = ctx.clearance(rebound, supported);
        let orig_pt = ctx.candidate_point(Vec3::ZERO);
        let orig_surf = ctx.constrain_surface(orig_pt);
        if clearance.cliff_impact(threshold)
            && (supported - orig_surf).dot(step.surface_up) > CLIFF_BONK_RISE
            && (supported - orig_pt).dot(step.surface_up) > CLIFF_BONK_RISE
        {
            let unlifted = rebound - step.surface_up * step.vertical_step.dot(step.surface_up).max(0.0);
            let bonked = if ctx.clearance(unlifted, ctx.constrain_surface(unlifted)).cliff_impact(threshold) {
                orig_pt + step.surface_up * step.vertical_step.dot(step.surface_up).min(0.0)
            } else {
                unlifted
            };
            let next_momentum = ctx.damp_aerial_momentum(momentum);
            FlightProgress::new(
                FlightPhase::Airborne(AirborneFlight::with_momentum(next_speed.min(0.0), next_momentum, threshold)),
                bonked,
            )
        } else {
            self.inner.resolve(ctx, step)
        }
    }
}

/// Decorator intercepting weightless core and upward-shaft flight before cliff/parabolic resolution.
pub struct WeightlessTrajectory<T: AerialTrajectory> {
    inner: T,
    flight: AirborneFlight,
}

impl<T: AerialTrajectory> WeightlessTrajectory<T> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(inner: T, flight: AirborneFlight) -> Self {
        Self { inner, flight }
    }
}

impl<T: AerialTrajectory> AerialTrajectory for WeightlessTrajectory<T> {
    fn resolve(&self, ctx: &FlightContext, step: AerialKinematics) -> FlightProgress {
        if ctx.weightless_ascent(step.next_speed) {
            let (_, momentum, threshold) = self.flight.inspect_state();
            let raw = ctx.candidate_point(step.airborne_stride) + step.vertical_step;
            let confined = ctx.confine_crust(raw);
            let bonked_speed = if (confined - raw).dot(step.surface_up) < -1e-4 {
                step.next_speed.min(0.0)
            } else {
                step.next_speed
            };
            let next_momentum = ctx.damp_aerial_momentum(momentum);
            FlightProgress::new(
                FlightPhase::Airborne(AirborneFlight::with_momentum(bonked_speed, next_momentum, threshold)),
                confined,
            )
        } else {
            self.inner.resolve(ctx, step)
        }
    }
}
