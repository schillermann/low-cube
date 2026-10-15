use glam::Vec3;
use world::PlanetaryWorld;
use crate::threshold::StepThreshold;

mod trial;
#[cfg(test)]
mod tests;

pub use trial::{
    AerialTrial, DeflectedStride, RestingStride, StrideResolution, WalkTrial,
};

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

    /// Resolves ground traversal destination, smoothly deflecting along obstacles via decorator chain.
    pub fn resolve_ground(
        &self,
        origin: Vec3,
        eye: f32,
        limit: StepThreshold,
        seam_allowed: bool,
        world: &dyn PlanetaryWorld,
    ) -> Vec3 {
        let (first, second) = self.axial_decomposition();
        let resting = RestingStride::new(world.constrain_to_surface(origin, eye));
        let trial_second = WalkTrial::new(origin, second, self.surface_up, eye, limit, seam_allowed, world);
        let lateral = DeflectedStride::new(&trial_second, &resting);
        let trial_first = WalkTrial::new(origin, first, self.surface_up, eye, limit, seam_allowed, world);
        let primary = DeflectedStride::new(&trial_first, &lateral);
        let trial_direct = WalkTrial::new(origin, self.stride, self.surface_up, eye, limit, seam_allowed, world);
        let direct = DeflectedStride::new(&trial_direct, &primary);
        direct.resolve()
    }

    /// Resolves aerial motion destination, deflecting laterally along vertical cliff faces via decorator chain.
    pub fn resolve_aerial(&self, origin: Vec3, vert: Vec3, eye: f32, limit: StepThreshold, world: &dyn PlanetaryWorld) -> Vec3 {
        let (first, second) = self.axial_decomposition();
        let resting = RestingStride::new(origin + vert);
        let trial_second = AerialTrial::new(origin, second, vert, self.surface_up, eye, limit, world);
        let lateral = DeflectedStride::new(&trial_second, &resting);
        let trial_first = AerialTrial::new(origin, first, vert, self.surface_up, eye, limit, world);
        let primary = DeflectedStride::new(&trial_first, &lateral);
        let trial_direct = AerialTrial::new(origin, self.stride, vert, self.surface_up, eye, limit, world);
        let direct = DeflectedStride::new(&trial_direct, &primary);
        direct.resolve()
    }
}
