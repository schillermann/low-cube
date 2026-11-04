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

struct FlightStep {
    dt: f32,
    position: Vec3,
    stride: Vec3,
    surface_up: Vec3,
}

struct FlightBody {
    eye_height: f32,
    envelope: BodyEnvelope,
}

struct FlightDynamics<'a> {
    gravity: &'a dyn FlightGravity,
    canopy: CanopyBraking,
    surfing: SlopeSurfing,
    incline: InclinePace,
}

struct FlightTerrain<'a> {
    gait: &'a dyn LocomotionGait,
    world: &'a dyn PlanetaryWorld,
}

/// Unified physical simulation context for a flight phase transition.
pub struct FlightContext<'a> {
    step: FlightStep,
    body: FlightBody,
    dynamics: FlightDynamics<'a>,
    terrain: FlightTerrain<'a>,
}

impl<'a> FlightContext<'a> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        dt: f32,
        position: Vec3,
        stride: Vec3,
        surface_up: Vec3,
        eye_height: f32,
        envelope: BodyEnvelope,
        gravity: &'a dyn FlightGravity,
        canopy: CanopyBraking,
        surfing: SlopeSurfing,
        incline: InclinePace,
        gait: &'a dyn LocomotionGait,
        world: &'a dyn PlanetaryWorld,
    ) -> Self {
        Self {
            step: FlightStep { dt, position, stride, surface_up },
            body: FlightBody { eye_height, envelope },
            dynamics: FlightDynamics { gravity, canopy, surfing, incline },
            terrain: FlightTerrain { gait, world },
        }
    }

    pub fn dt(&self) -> f32 { self.step.dt }
    pub fn commanded_velocity(&self) -> Vec3 { if self.step.dt > 1e-6 { self.step.stride / self.step.dt } else { Vec3::ZERO } }
    pub fn constrain_surface(&self, point: Vec3) -> Vec3 { self.body.envelope.constrain_with_height(point, self.body.eye_height, self.terrain.world) }
    pub fn current_surface_up(&self) -> Vec3 { self.terrain.world.surface_up_at(self.step.position) }
    pub fn clearance(&self, point: Vec3, surface_point: Vec3) -> SurfaceClearance { SurfaceClearance::between(self.step.position, point, surface_point, self.step.surface_up, self.terrain.world) }
    pub fn permits_seam_fold(&self) -> bool { self.terrain.gait.permits_seam_fold() }

    pub fn weightless_ascent(&self, _vertical_speed: f32) -> bool {
        self.terrain.world.weightless_at(self.step.position)
            || self.body.envelope.in_upward_shaft(self.step.position, self.step.surface_up, self.body.eye_height, self.terrain.world)
    }

    pub fn confine_crust(&self, point: Vec3) -> Vec3 {
        if self.body.envelope.in_upward_shaft(point, self.step.surface_up, self.body.eye_height, self.terrain.world) {
            self.terrain.world.confine_beneath_crust(point)
        } else {
            self.body.envelope.confine_beneath_crust(point, self.step.position, self.step.surface_up, self.terrain.world)
        }
    }

    pub fn origin_supported(&self) -> bool {
        self.clearance(self.step.position, self.constrain_surface(self.step.position)).passable_with(false)
    }

    pub fn resolve_precipice(&self, threshold: StepThreshold, horizontal: Vec3) -> FlightProgress {
        self.terrain.gait.resolve_precipice(self, threshold, horizontal)
    }

    pub fn gravitational_acceleration(&self, vertical_speed: f32) -> f32 {
        let probe = if vertical_speed < 0.0 { self.step.position + self.step.surface_up * 0.25 } else { self.step.position };
        if self.terrain.world.weightless_at(probe) {
            if vertical_speed < 0.0 {
                vertical_speed - vertical_speed.max(-3.6) * (1.0 - (16.0 * self.step.dt).min(1.0))
            } else {
                vertical_speed * (8.0 * self.step.dt).min(1.0)
            }
        } else {
            self.dynamics.gravity.delta_velocity(vertical_speed, self.step.dt)
        }
    }

    pub fn brake_canopy(&self, vertical_speed: f32) -> f32 {
        let drag = self.terrain.world.canopy_drag(self.step.position);
        self.dynamics.canopy.brake_fall(vertical_speed, drag, self.step.dt)
    }

    pub fn evaluate_touchdown(&self, impact_speed: f32, landing_pos: Vec3) -> Vec3 {
        self.dynamics.surfing.evaluate_touchdown(
            impact_speed,
            self.step.stride,
            self.step.surface_up,
            landing_pos,
            self.body.eye_height,
            self.terrain.world,
        )
    }

    pub fn damp_surf_momentum(&self, momentum: Vec3) -> Vec3 {
        self.dynamics.surfing.damp_momentum(momentum, self.step.dt)
    }

    pub fn damp_aerial_momentum(&self, momentum: Vec3) -> Vec3 {
        if momentum.length_squared() > 1e-4 {
            momentum * (1.0 - (0.5 * self.step.dt).min(1.0))
        } else {
            Vec3::ZERO
        }
    }

    pub fn paced_ground_stride(&self) -> Vec3 {
        self.dynamics.incline.pace_stride(
            self.step.stride,
            self.step.position,
            self.step.surface_up,
            self.body.eye_height,
            self.terrain.world,
        )
    }

    pub fn candidate_point(&self, stride: Vec3) -> Vec3 {
        if stride.length_squared() > 1e-6 {
            self.step.position + stride
        } else {
            self.step.position
        }
    }

    pub fn resolve_ground_stride(&self, threshold: StepThreshold) -> Vec3 {
        self.resolve_paced_stride(self.step.stride, Vec3::ZERO, threshold)
    }

    pub fn resolve_paced_stride(
        &self,
        stride: Vec3,
        momentum: Vec3,
        threshold: StepThreshold,
    ) -> Vec3 {
        let effective_stride = if momentum.length_squared() > 1e-4 {
            stride + momentum * self.step.dt
        } else {
            stride
        };
        SurfaceStride::new(effective_stride, self.step.surface_up).resolve_ground(
            self.step.position,
            self.body.eye_height,
            self.body.envelope,
            threshold,
            self.permits_seam_fold(),
            self.terrain.world,
        )
    }

    pub fn reorient_edge_flight(&self, vertical_speed: f32) -> (Vec3, f32, Vec3) {
        let current_up = self.current_surface_up();
        if self.step.surface_up.dot(current_up) < 0.99 {
            let v_3d = self.step.surface_up * vertical_speed + self.commanded_velocity();
            let new_vertical_speed = v_3d.dot(current_up);
            let new_tangent_v = v_3d - current_up * new_vertical_speed;
            let effective_stride = new_tangent_v * self.step.dt;
            (current_up, new_vertical_speed, effective_stride)
        } else {
            (self.step.surface_up, vertical_speed, self.step.stride)
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
            .resolve_aerial(self.step.position, vertical_step, self.body.eye_height, self.body.envelope, threshold, self.terrain.world)
    }
}
