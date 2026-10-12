use std::f32::consts::FRAC_PI_2;
use glam::Vec3;
use super::cube_face::CubeFace;
use super::seam::SeamFold;

/// Interface representing continuous normal transitions and geometric bevel zones across planetary seams.
pub trait BevelTransition: Send + Sync {
    fn contains(&self, point: Vec3) -> bool;
    fn normal_at(&self, point: Vec3) -> Vec3;
    fn blend_factor(&self, point: Vec3) -> f32;
}

/// Encapsulated seam bevel geometry providing smooth continuous surface normals across a 90° edge.
#[derive(Clone, Copy, Debug)]
pub struct SeamBevel {
    source: CubeFace,
    destination: CubeFace,
    seam_center: Vec3,
    half_size: f32,
    margin: f32,
}

impl SeamBevel {
    /// Primary constructor assigning geometry arguments directly to fields.
    pub fn new(
        source: CubeFace,
        destination: CubeFace,
        seam_center: Vec3,
        half_size: f32,
        margin: f32,
    ) -> Self {
        Self {
            source,
            destination,
            seam_center,
            half_size,
            margin,
        }
    }

    /// Secondary constructor deriving seam center from adjacent faces and half size.
    pub fn between(
        source: CubeFace,
        destination: CubeFace,
        half_size: f32,
        margin: f32,
    ) -> Self {
        let seam_center = (source.normal() + destination.normal()) * half_size;
        Self::new(source, destination, seam_center, half_size, margin)
    }

    /// Secondary constructor deriving bevel from an existing SeamFold.
    pub fn from_fold(fold: &SeamFold, half_size: f32, margin: f32) -> Self {
        fold.bevel(half_size, margin)
    }

    /// Continuous interpolation parameter t in [0.0, 1.0] across the bevel zone.
    pub fn blend_factor(&self, point: Vec3) -> f32 {
        let h1 = point.dot(self.source.normal());
        let h2 = point.dot(self.destination.normal());
        let delta = h2 - h1;
        ((delta + self.margin) / (2.0 * self.margin)).clamp(0.0, 1.0)
    }

    /// Predicate verifying whether a spatial position lies within the bevel transition zone.
    pub fn contains(&self, point: Vec3) -> bool {
        let h1 = point.dot(self.source.normal());
        let h2 = point.dot(self.destination.normal());
        let threshold = self.half_size - self.margin;
        h1 >= threshold && h2 >= threshold && (h2 - h1).abs() <= self.margin
    }

    /// Evaluates continuous surface normal transitioning smoothly across the 90° edge.
    pub fn normal_at(&self, point: Vec3) -> Vec3 {
        let t = self.blend_factor(point);
        let angle = t * FRAC_PI_2;
        let n1 = self.source.normal();
        let n2 = self.destination.normal();
        n1 * angle.cos() + n2 * angle.sin()
    }

    /// Distance from a spatial position to the continuous linear seam axis.
    pub fn distance_to_seam_line(&self, point: Vec3) -> f32 {
        let axis = self.source.normal().cross(self.destination.normal()).normalize();
        let offset = point - self.seam_center;
        let axial = offset.dot(axis);
        (offset - axis * axial).length()
    }

    /// Evaluates surface normal and blend factor into a consumer without extracting raw fields ("Tell, Don't Ask").
    pub fn evaluate<T>(&self, point: Vec3, consumer: impl FnOnce(Vec3, f32) -> T) -> T {
        consumer(self.normal_at(point), self.blend_factor(point))
    }
}

impl BevelTransition for SeamBevel {
    fn contains(&self, point: Vec3) -> bool {
        self.contains(point)
    }

    fn normal_at(&self, point: Vec3) -> Vec3 {
        self.normal_at(point)
    }

    fn blend_factor(&self, point: Vec3) -> f32 {
        self.blend_factor(point)
    }
}
