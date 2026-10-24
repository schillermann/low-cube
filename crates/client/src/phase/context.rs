use glam::Vec3;
use world::PlanetaryWorld;
use crate::envelope::BodyEnvelope;
use crate::gait::LocomotionGait;
use crate::stride::SurfaceStride;
use crate::threshold::{StepThreshold, SurfaceClearance};
use super::canopy::CanopyBraking;
use super::gravity::FlightGravity;
use super::incline::InclinePace;
use super::progress::FlightProgress;
use super::slope::SlopeSurfing;

/// Unified physical simulation context for a flight phase transition.
pub struct FlightContext<'a> {
    dt: f32, position: Vec3, stride: Vec3, surface_up: Vec3, eye_height: f32, envelope: BodyEnvelope,
    gravity: &'a dyn FlightGravity, canopy: CanopyBraking, surfing: SlopeSurfing, incline: InclinePace,
    gait: &'a dyn LocomotionGait, world: &'a dyn PlanetaryWorld,
}

impl<'a> FlightContext<'a> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        dt: f32, position: Vec3, stride: Vec3, surface_up: Vec3, eye_height: f32, envelope: BodyEnvelope,
        gravity: &'a dyn FlightGravity, canopy: CanopyBraking, surfing: SlopeSurfing, incline: InclinePace,
        gait: &'a dyn LocomotionGait, world: &'a dyn PlanetaryWorld,
    ) -> Self {
        Self { dt, position, stride, surface_up, eye_height, envelope, gravity, canopy, surfing, incline, gait, world }
    }

    pub fn dt(&self) -> f32 { self.dt }
    pub fn commanded_velocity(&self) -> Vec3 { if self.dt > 1e-6 { self.stride / self.dt } else { Vec3::ZERO } }
    pub fn constrain_surface(&self, point: Vec3) -> Vec3 { self.envelope.constrain_with_height(point, self.eye_height, self.world) }
    pub fn current_surface_up(&self) -> Vec3 { self.world.surface_up_at(self.position) }
    pub fn clearance(&self, point: Vec3, surface_point: Vec3) -> SurfaceClearance { SurfaceClearance::between(self.position, point, surface_point, self.surface_up, self.world) }
    pub fn permits_seam_fold(&self) -> bool { self.gait.permits_seam_fold() }

    pub fn weightless_ascent(&self, vertical_speed: f32) -> bool {
        let shaft_ascent = (self.position - self.constrain_surface(self.position)).dot(self.surface_up) > 2.5 && vertical_speed > 0.0;
        self.world.weightless_at(self.position) || shaft_ascent
    }

    pub fn confine_crust(&self, point: Vec3) -> Vec3 {
        self.world.confine_beneath_crust(point)
    }

    pub fn origin_supported(&self) -> bool {
        self.clearance(self.position, self.constrain_surface(self.position)).passable_with(false)
    }

    pub fn resolve_precipice(&self, threshold: StepThreshold, horizontal: Vec3) -> FlightProgress {
        self.gait.resolve_precipice(self, threshold, horizontal)
    }

    pub fn gravitational_acceleration(&self, vertical_speed: f32) -> f32 {
        if self.weightless_ascent(vertical_speed) {
            let damp = if vertical_speed < 0.0 { 12.0 } else { 8.0 };
            if !self.gait.permits_seam_fold() { (vertical_speed - (-1.8)).max(0.0) * (6.0 * self.dt).min(1.0) } else { (vertical_speed - if self.world.weightless_at(self.position) { 0.0 } else { 1e-4 }).max(vertical_speed.min(0.0)) * (damp * self.dt).min(1.0) }
        } else {
            self.gravity.delta_velocity(vertical_speed, self.dt)
        }
    }

    pub fn brake_canopy(&self, vertical_speed: f32) -> f32 {
        let drag = self.world.canopy_drag(self.position);
        self.canopy.brake_fall(vertical_speed, drag, self.dt)
    }

    pub fn evaluate_touchdown(&self, impact_speed: f32, landing_pos: Vec3) -> Vec3 {
        self.surfing.evaluate_touchdown(
            impact_speed,
            self.stride,
            self.surface_up,
            landing_pos,
            self.eye_height,
            self.world,
        )
    }

    pub fn damp_surf_momentum(&self, momentum: Vec3) -> Vec3 {
        self.surfing.damp_momentum(momentum, self.dt)
    }

    pub fn damp_aerial_momentum(&self, momentum: Vec3) -> Vec3 {
        if momentum.length_squared() > 1e-4 {
            momentum * (1.0 - (0.5 * self.dt).min(1.0))
        } else {
            Vec3::ZERO
        }
    }

    pub fn paced_ground_stride(&self) -> Vec3 {
        self.incline.pace_stride(
            self.stride,
            self.position,
            self.surface_up,
            self.eye_height,
            self.world,
        )
    }

    pub fn candidate_point(&self, stride: Vec3) -> Vec3 {
        if stride.length_squared() > 1e-6 {
            self.position + stride
        } else {
            self.position
        }
    }

    pub fn resolve_ground_stride(&self, threshold: StepThreshold) -> Vec3 {
        self.resolve_paced_stride(self.stride, Vec3::ZERO, threshold)
    }

    pub fn resolve_paced_stride(
        &self,
        stride: Vec3,
        momentum: Vec3,
        threshold: StepThreshold,
    ) -> Vec3 {
        let effective_stride = if momentum.length_squared() > 1e-4 {
            stride + momentum * self.dt
        } else {
            stride
        };
        SurfaceStride::new(effective_stride, self.surface_up).resolve_ground(
            self.position,
            self.eye_height,
            self.envelope,
            threshold,
            self.permits_seam_fold(),
            self.world,
        )
    }

    pub fn reorient_edge_flight(&self, vertical_speed: f32) -> (Vec3, f32, Vec3) {
        let current_up = self.current_surface_up();
        if self.surface_up.dot(current_up) < 0.99 {
            let v_3d = self.surface_up * vertical_speed + self.commanded_velocity();
            let new_vertical_speed = v_3d.dot(current_up);
            let new_tangent_v = v_3d - current_up * new_vertical_speed;
            let effective_stride = new_tangent_v * self.dt;
            (current_up, new_vertical_speed, effective_stride)
        } else {
            (self.surface_up, vertical_speed, self.stride)
        }
    }

    pub fn resolve_aerial_stride(
        &self,
        stride: Vec3,
        surface_up: Vec3,
        vertical_step: Vec3,
        threshold: StepThreshold,
    ) -> Vec3 {
        SurfaceStride::new(stride, surface_up)
            .resolve_aerial(self.position, vertical_step, self.eye_height, self.envelope, threshold, self.world)
    }
}
