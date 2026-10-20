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
        let dir = tangent.normalize();
        let mid_surface = world.constrain_to_surface(candidate + dir * (self.radius * 0.5), eye);
        let probe_surface = world.constrain_to_surface(candidate + dir * self.radius, eye);
        if up.dot(world.surface_up_at(probe_surface)) < 0.7 {
            return true;
        }
        SurfaceStep::new(surface_pos, mid_surface, up).walkable_with(limit)
            && SurfaceStep::new(mid_surface, probe_surface, up).walkable_with(limit)
            && (probe_surface - surface_pos).dot(up) < self.height
    }

    fn unobstructed_aerial(
        &self,
        _origin: Vec3,
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
        let candidate_surf = world.constrain_to_surface(candidate, eye);
        let base = if (candidate_surf - candidate).dot(up) > 0.0 { candidate_surf } else { candidate };
        let dir = tangent.normalize();
        let mid = base + dir * (self.radius * 0.5);
        let probe = base + dir * self.radius;
        let mid_surface = world.constrain_to_surface(mid, eye);
        let probe_surface = world.constrain_to_surface(probe, eye);
        let clearance = SurfaceClearance::between(base, probe, probe_surface, up, world);
        if !clearance.cliff_impact(limit) {
            return true;
        }
        SurfaceStep::new(candidate_surf, mid_surface, up).walkable_with(limit)
            && SurfaceStep::new(mid_surface, probe_surface, up).walkable_with(limit)
    }
}
