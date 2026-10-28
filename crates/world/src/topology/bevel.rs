use std::f32::consts::FRAC_PI_2;
use glam::Vec3;
use super::climate_zone::ClimateZone;
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
    zones: (ClimateZone, ClimateZone),
    seam_center: Vec3,
    half_size: f32,
    margin: f32,
}

impl SeamBevel {
    /// Primary constructor assigning geometry arguments directly to fields.
    pub fn new(
        source: ClimateZone,
        destination: ClimateZone,
        seam_center: Vec3,
        half_size: f32,
        margin: f32,
    ) -> Self {
        Self {
            zones: (source, destination),
            seam_center,
            half_size,
            margin,
        }
    }

    /// Secondary constructor deriving seam center from adjacent climate zones and half size.
    pub fn between(
        source: ClimateZone,
        destination: ClimateZone,
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

    fn delta_at(&self, point: Vec3) -> f32 {
        let (source, destination) = self.zones;
        let h1 = point.dot(source.normal());
        let h2 = point.dot(destination.normal());
        if h1 >= self.half_size && h2 <= self.half_size {
            h2 - self.half_size
        } else if h2 >= self.half_size && h1 <= self.half_size {
            self.half_size - h1
        } else {
            h2 - h1
        }
    }

    /// Continuous interpolation parameter t in [0.0, 1.0] across the bevel zone with C^2 smootherstep.
    pub fn blend_factor(&self, point: Vec3) -> f32 {
        let delta = self.delta_at(point);
        let linear = ((delta + self.margin) / (2.0 * self.margin)).clamp(0.0, 1.0);
        linear * linear * linear * (linear * (linear * 6.0 - 15.0) + 10.0)
    }

    /// Predicate verifying whether a spatial position lies within the bevel transition zone.
    pub fn contains(&self, point: Vec3) -> bool {
        let (source, destination) = self.zones;
        let h1 = point.dot(source.normal());
        let h2 = point.dot(destination.normal());
        let threshold = self.half_size - self.margin;
        let delta = self.delta_at(point);
        h1 >= threshold && h2 >= threshold && delta.abs() <= self.margin
    }

    /// Evaluates continuous surface normal transitioning smoothly across the 90° edge.
    pub fn normal_at(&self, point: Vec3) -> Vec3 {
        let (source, destination) = self.zones;
        let t = self.blend_factor(point);
        let angle = t * FRAC_PI_2;
        let n1 = source.normal();
        let n2 = destination.normal();
        n1 * angle.cos() + n2 * angle.sin()
    }

    /// Distance from a spatial position to the continuous linear seam axis.
    pub fn distance_to_seam_line(&self, point: Vec3) -> f32 {
        let (source, destination) = self.zones;
        let axis = source.normal().cross(destination.normal()).normalize();
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
