use glam::Vec3;
use world::PlanetaryWorld;
use crate::threshold::{StepThreshold, SurfaceClearance, SurfaceStep};

/// Interface for physical avatar collision volumes preventing wall penetration and camera clipping.
pub trait PhysicalEnvelope {
    #[allow(clippy::too_many_arguments)]
    fn unobstructed_ground(
        &self,
        candidate: Vec3,
        surface_pos: Vec3,
        step: Vec3,
        up: Vec3,
        eye: f32,
        limit: StepThreshold,
        world: &dyn PlanetaryWorld,
    ) -> bool;

    #[allow(clippy::too_many_arguments)]
    fn unobstructed_aerial(
        &self,
        origin: Vec3,
        candidate: Vec3,
        tangent: Vec3,
        up: Vec3,
        eye: f32,
        limit: StepThreshold,
        world: &dyn PlanetaryWorld,
    ) -> bool;
}

/// Physical avatar body volume preventing wall penetration and camera clipping.
#[derive(Clone, Copy, Debug)]
pub struct BodyEnvelope {
    radius: f32,
    height: f32,
}

impl BodyEnvelope {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(radius: f32, height: f32) -> Self {
        Self { radius, height }
    }

    /// Secondary constructor calibrated for a youthful medieval avatar stature.
    pub fn youth() -> Self {
        Self::new(0.32, 1.8)
    }
}

impl PhysicalEnvelope for BodyEnvelope {
    fn unobstructed_ground(
        &self,
        candidate: Vec3,
        surface_pos: Vec3,
        step: Vec3,
        up: Vec3,
        eye: f32,
        limit: StepThreshold,
        world: &dyn PlanetaryWorld,
    ) -> bool {
        let tangent = step - up * step.dot(up);
        if tangent.length_squared() <= 1e-6 {
            return true;
        }
        let probe = candidate + tangent.normalize() * self.radius;
        let probe_surface = world.constrain_to_surface(probe, eye);
        let probe_up = world.surface_up_at(probe_surface);
        if up.dot(probe_up) < 0.7 {
            return true;
        }
        let step_ahead = SurfaceStep::new(surface_pos, probe_surface, up);
        step_ahead.walkable_with(limit) && (probe_surface - surface_pos).dot(up) < self.height
    }

    fn unobstructed_aerial(
        &self,
        origin: Vec3,
        candidate: Vec3,
        tangent: Vec3,
        up: Vec3,
        eye: f32,
        limit: StepThreshold,
        world: &dyn PlanetaryWorld,
    ) -> bool {
        if tangent.length_squared() <= 1e-6 {
            return true;
        }
        let probe = candidate + tangent.normalize() * self.radius;
        let probe_surface = world.constrain_to_surface(probe, eye);
        let clearance = SurfaceClearance::between(origin, probe, probe_surface, up, world);
        if !clearance.cliff_impact(limit) {
            return true;
        }
        let origin_surf = world.constrain_to_surface(origin, eye);
        SurfaceStep::new(origin_surf, probe_surface, up).walkable_with(limit)
    }
}
