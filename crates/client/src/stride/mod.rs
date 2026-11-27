use glam::Vec3;
use world::PlanetaryWorld;
use crate::envelope::BodyEnvelope;
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
        let n = self.surface_up;
        let na = n.abs();
        let raw_a = if na.y >= na.x && na.y >= na.z {
            Vec3::X
        } else if na.x >= na.y && na.x >= na.z {
            Vec3::Y
        } else {
            Vec3::Z
        };
        let axis_a = (raw_a - n * raw_a.dot(n)).normalize_or_zero();
        let axis_b = n.cross(axis_a).normalize_or_zero();
        let (a, b) = (axis_a * self.stride.dot(axis_a), axis_b * self.stride.dot(axis_b));
        if a.length_squared() >= b.length_squared() { (a, b) } else { (b, a) }
    }

    /// Resolves ground traversal destination, smoothly deflecting along obstacles via decorator chain.
    pub fn resolve_ground(
        &self,
        origin: Vec3,
        eye: f32,
        envelope: BodyEnvelope,
        limit: StepThreshold,
        seam_allowed: bool,
        world: &dyn PlanetaryWorld,
    ) -> Vec3 {
        let (first, second) = self.axial_decomposition();
        let orig_surf = envelope.constrain_with_height(origin, eye, world);
        let resting_pos = if crate::threshold::SurfaceClearance::between(origin, origin, orig_surf, self.surface_up, world).passable_with(seam_allowed)
            && crate::threshold::SurfaceStep::new(origin, orig_surf, self.surface_up).walkable_with(limit)
        {
            orig_surf
        } else {
            origin
        };
        let resting = RestingStride::new(resting_pos);
        let trial_second = WalkTrial::new(origin, second, self.surface_up, eye, envelope, limit, seam_allowed, world);
        let lateral = DeflectedStride::new(&trial_second, &resting);
        let trial_first = WalkTrial::new(origin, first, self.surface_up, eye, envelope, limit, seam_allowed, world);
        let primary = DeflectedStride::new(&trial_first, &lateral);
        let trial_direct = WalkTrial::new(origin, self.stride, self.surface_up, eye, envelope, limit, seam_allowed, world);
        let direct = DeflectedStride::new(&trial_direct, &primary);
        direct.resolve()
    }

    /// Resolves aerial motion destination, deflecting laterally along vertical cliff faces via decorator chain.
    pub fn resolve_aerial(&self, origin: Vec3, vert: Vec3, eye: f32, envelope: BodyEnvelope, limit: StepThreshold, world: &dyn PlanetaryWorld) -> Vec3 {
        let (first, second) = self.axial_decomposition();
        let resting = RestingStride::new(origin + vert);
        let trial_second = AerialTrial::new(origin, second, vert, self.surface_up, eye, envelope, limit, world);
        let lateral = DeflectedStride::new(&trial_second, &resting);
        let trial_first = AerialTrial::new(origin, first, vert, self.surface_up, eye, envelope, limit, world);
        let primary = DeflectedStride::new(&trial_first, &lateral);
        let trial_direct = AerialTrial::new(origin, self.stride, vert, self.surface_up, eye, envelope, limit, world);
        let direct = DeflectedStride::new(&trial_direct, &primary);
        direct.resolve()
    }
}
