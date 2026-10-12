use glam::Vec3;
use world::PlanetaryWorld;
use crate::threshold::{StepThreshold, SurfaceClearance, SurfaceStep};

/// Evaluated ground trial step outcome.
struct WalkTrial<'a> {
    origin: Vec3,
    step: Vec3,
    up: Vec3,
    eye: f32,
    limit: StepThreshold,
    world: &'a dyn PlanetaryWorld,
}

impl<'a> WalkTrial<'a> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(
        origin: Vec3,
        step: Vec3,
        up: Vec3,
        eye: f32,
        limit: StepThreshold,
        world: &'a dyn PlanetaryWorld,
    ) -> Self {
        Self {
            origin,
            step,
            up,
            eye,
            limit,
            world,
        }
    }

    /// Predicate verifying whether this ground step is traversable.
    pub fn is_passable(&self) -> bool {
        if self.step.length_squared() <= 1e-6 {
            return false;
        }
        let candidate = self.origin + self.step;
        let surface_pos = self.world.constrain_to_surface(candidate, self.eye);
        let clearance = SurfaceClearance::between(candidate, surface_pos, self.up);
        !clearance.is_precipice()
            && SurfaceStep::new(self.origin, surface_pos, self.up).is_walkable_with(self.limit)
    }

    /// Resolved surface destination position for this ground trial step.
    pub fn destination(&self) -> Vec3 {
        if self.step.length_squared() <= 1e-6 {
            return self.origin;
        }
        self.world.constrain_to_surface(self.origin + self.step, self.eye)
    }
}

/// Evaluated mid-air trial step outcome.
struct AerialTrial<'a> {
    origin: Vec3,
    tangent: Vec3,
    vertical: Vec3,
    up: Vec3,
    eye: f32,
    limit: StepThreshold,
    world: &'a dyn PlanetaryWorld,
}

impl<'a> AerialTrial<'a> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(
        origin: Vec3,
        tangent: Vec3,
        vertical: Vec3,
        up: Vec3,
        eye: f32,
        limit: StepThreshold,
        world: &'a dyn PlanetaryWorld,
    ) -> Self {
        Self {
            origin,
            tangent,
            vertical,
            up,
            eye,
            limit,
            world,
        }
    }

    /// Predicate verifying whether aerial trajectory is clear of vertical cliff collisions.
    pub fn is_unobstructed(&self) -> bool {
        if self.tangent.length_squared() <= 1e-6 {
            return false;
        }
        let candidate = self.origin + self.tangent + self.vertical;
        let surface_pos = self.world.constrain_to_surface(candidate, self.eye);
        let clearance = SurfaceClearance::between(candidate, surface_pos, self.up);
        !clearance.is_cliff_impact(self.limit)
    }

    /// Candidate aerial destination position for this flight trial step.
    pub fn destination(&self) -> Vec3 {
        self.origin + self.tangent + self.vertical
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
        let direct = WalkTrial::new(origin, self.stride, self.surface_up, eye, limit, world);
        if direct.is_passable() {
            return direct.destination();
        }

        let (first, second) = self.axial_decomposition();
        let primary = WalkTrial::new(origin, first, self.surface_up, eye, limit, world);
        if primary.is_passable() {
            return primary.destination();
        }

        let lateral = WalkTrial::new(origin, second, self.surface_up, eye, limit, world);
        if lateral.is_passable() {
            return lateral.destination();
        }

        world.constrain_to_surface(origin, eye)
    }

    /// Resolves aerial motion destination, deflecting laterally along vertical cliff faces.
    pub fn resolve_aerial(&self, origin: Vec3, vert: Vec3, eye: f32, limit: StepThreshold, world: &dyn PlanetaryWorld) -> Vec3 {
        let direct = AerialTrial::new(origin, self.stride, vert, self.surface_up, eye, limit, world);
        if direct.is_unobstructed() {
            return direct.destination();
        }

        let (first, second) = self.axial_decomposition();
        let primary = AerialTrial::new(origin, first, vert, self.surface_up, eye, limit, world);
        if primary.is_unobstructed() {
            return primary.destination();
        }

        let lateral = AerialTrial::new(origin, second, vert, self.surface_up, eye, limit, world);
        if lateral.is_unobstructed() {
            return lateral.destination();
        }

        origin + vert
    }
}
