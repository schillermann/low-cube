use glam::Vec3;
use world::PlanetaryWorld;
use crate::threshold::{StepThreshold, SurfaceClearance, SurfaceStep};

/// Evaluated ground trial step outcome.
struct WalkTrial {
    destination: Vec3,
    passable: bool,
}

impl WalkTrial {
    fn evaluate(origin: Vec3, step: Vec3, up: Vec3, eye: f32, limit: StepThreshold, world: &dyn PlanetaryWorld) -> Self {
        if step.length_squared() <= 1e-6 {
            return Self { destination: origin, passable: false };
        }
        let candidate = origin + step;
        let surface_pos = world.constrain_to_surface(candidate, eye);
        let clearance = SurfaceClearance::between(candidate, surface_pos, up);
        let passable = !clearance.is_precipice() && SurfaceStep::new(origin, surface_pos, up).is_walkable_with(limit);
        Self { destination: surface_pos, passable }
    }
}

/// Evaluated mid-air trial step outcome.
struct AerialTrial {
    destination: Vec3,
    unobstructed: bool,
}

impl AerialTrial {
    fn evaluate(
        origin: Vec3,
        tangent: Vec3,
        vertical: Vec3,
        up: Vec3,
        eye: f32,
        limit: StepThreshold,
        world: &dyn PlanetaryWorld,
    ) -> Self {
        if tangent.length_squared() <= 1e-6 {
            return Self { destination: origin + vertical, unobstructed: false };
        }
        let candidate = origin + tangent + vertical;
        let surface_pos = world.constrain_to_surface(candidate, eye);
        let clearance = SurfaceClearance::between(candidate, surface_pos, up);
        Self { destination: candidate, unobstructed: !clearance.is_cliff_impact(limit) }
    }
}

/// Tangential stride across a planetary surface resolving ground walking and aerial obstacle deflection.
#[derive(Clone, Copy, Debug)]
pub struct SurfaceStride {
    stride: Vec3,
    surface_up: Vec3,
}

impl SurfaceStride {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(stride: Vec3, surface_up: Vec3) -> Self {
        Self { stride, surface_up }
    }

    fn axial_decomposition(&self) -> (Vec3, Vec3) {
        let n = self.surface_up.abs();
        let (a, b) = if n.y >= n.x && n.y >= n.z {
            (Vec3::new(self.stride.x, 0.0, 0.0), Vec3::new(0.0, 0.0, self.stride.z))
        } else if n.x >= n.y && n.x >= n.z {
            (Vec3::new(0.0, self.stride.y, 0.0), Vec3::new(0.0, 0.0, self.stride.z))
        } else {
            (Vec3::new(self.stride.x, 0.0, 0.0), Vec3::new(0.0, self.stride.y, 0.0))
        };
        if a.length_squared() >= b.length_squared() { (a, b) } else { (b, a) }
    }

    /// Resolves ground traversal destination, smoothly sliding along obstacles if direct stride is obstructed.
    pub fn resolve_ground(&self, origin: Vec3, eye: f32, limit: StepThreshold, world: &dyn PlanetaryWorld) -> Vec3 {
        let direct = WalkTrial::evaluate(origin, self.stride, self.surface_up, eye, limit, world);
        if direct.passable {
            return direct.destination;
        }

        let (first, second) = self.axial_decomposition();
        let primary = WalkTrial::evaluate(origin, first, self.surface_up, eye, limit, world);
        if primary.passable {
            return primary.destination;
        }

        let lateral = WalkTrial::evaluate(origin, second, self.surface_up, eye, limit, world);
        if lateral.passable {
            return lateral.destination;
        }

        world.constrain_to_surface(origin, eye)
    }

    /// Resolves aerial motion destination, deflecting laterally along vertical cliff faces.
    pub fn resolve_aerial(&self, origin: Vec3, vert: Vec3, eye: f32, limit: StepThreshold, world: &dyn PlanetaryWorld) -> Vec3 {
        let direct = AerialTrial::evaluate(origin, self.stride, vert, self.surface_up, eye, limit, world);
        if direct.unobstructed {
            return direct.destination;
        }

        let (first, second) = self.axial_decomposition();
        let primary = AerialTrial::evaluate(origin, first, vert, self.surface_up, eye, limit, world);
        if primary.unobstructed {
            return primary.destination;
        }

        let lateral = AerialTrial::evaluate(origin, second, vert, self.surface_up, eye, limit, world);
        if lateral.unobstructed {
            return lateral.destination;
        }

        origin + vert
    }
}
