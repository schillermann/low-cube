use glam::Vec3;
use world::PlanetaryWorld;
use crate::gait::LocomotionGait;
use crate::stride::SurfaceStride;
use crate::threshold::{StepThreshold, SurfaceClearance};
use super::FlightPhase;

/// Unified physical simulation context for a flight phase transition.
pub struct FlightContext<'a> {
    dt: f32,
    position: Vec3,
    stride: Vec3,
    surface_up: Vec3,
    eye_height: f32,
    gravity: f32,
    gait: &'a dyn LocomotionGait,
    world: &'a dyn PlanetaryWorld,
}

impl<'a> FlightContext<'a> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(
        dt: f32,
        position: Vec3,
        stride: Vec3,
        surface_up: Vec3,
        eye_height: f32,
        gravity: f32,
        gait: &'a dyn LocomotionGait,
        world: &'a dyn PlanetaryWorld,
    ) -> Self {
        Self {
            dt,
            position,
            stride,
            surface_up,
            eye_height,
            gravity,
            gait,
            world,
        }
    }

    pub fn active_stride(&self) -> bool {
        self.stride.length_squared() > 1e-6
    }

    pub fn commanded_velocity(&self) -> Vec3 {
        if self.dt > 1e-6 {
            self.stride / self.dt
        } else {
            Vec3::ZERO
        }
    }

    pub fn candidate_horizontal(&self) -> Vec3 {
        if self.active_stride() {
            self.position + self.stride
        } else {
            self.position
        }
    }

    pub fn constrain_surface(&self, point: Vec3) -> Vec3 {
        self.world.constrain_to_surface(point, self.eye_height)
    }

    pub fn current_surface_up(&self) -> Vec3 {
        self.world.surface_up_at(self.position)
    }

    pub fn clearance(&self, point: Vec3, surface_point: Vec3) -> SurfaceClearance {
        SurfaceClearance::between(self.position, point, surface_point, self.surface_up, self.world)
    }

    pub fn resolve_ground_stride(&self, threshold: StepThreshold) -> Vec3 {
        SurfaceStride::new(self.stride, self.surface_up)
            .resolve_ground(self.position, self.eye_height, threshold, self.world)
    }

    pub fn resolve_precipice(&self, threshold: StepThreshold, horizontal: Vec3) -> FlightProgress {
        self.gait.resolve_precipice(self, threshold, horizontal)
    }

    pub fn gravity_velocity_delta(&self) -> f32 {
        self.gravity * self.dt
    }

    pub fn dt(&self) -> f32 {
        self.dt
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
            .resolve_aerial(self.position, vertical_step, self.eye_height, threshold, self.world)
    }
}

/// Outcome of a flight phase transition containing the subsequent phase and updated position.
pub struct FlightProgress {
    phase: FlightPhase,
    position: Vec3,
}

impl FlightProgress {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(phase: FlightPhase, position: Vec3) -> Self {
        Self { phase, position }
    }

    /// Mutates the caller's active phase reference and yields the updated spatial position.
    pub fn apply(self, flight: &mut FlightPhase) -> Vec3 {
        *flight = self.phase;
        self.position
    }
}
