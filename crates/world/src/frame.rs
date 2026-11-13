use glam::Vec3;
use crate::block::GridCell;

/// Orthogonal coordinate frame of a planetary face defining surface tangent space.
#[derive(Clone, Copy, Debug)]
pub struct SurfaceFrame {
    tangent: Vec3,
    bitangent: Vec3,
    normal: Vec3,
    half_size: f32,
}

impl SurfaceFrame {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(tangent: Vec3, bitangent: Vec3, normal: Vec3, half_size: f32) -> Self {
        Self {
            tangent,
            bitangent,
            normal,
            half_size,
        }
    }

    /// Evaluates a 3D position given tangential coordinates and elevation above base.
    pub fn point_at(&self, u: f32, v: f32, elevation: f32) -> Vec3 {
        self.normal * (self.half_size + elevation) + self.tangent * u + self.bitangent * v
    }

    /// Decomposes a 3D position into 2D tangential surface coordinates.
    pub fn decompose(&self, point: Vec3) -> (f32, f32) {
        (point.dot(self.tangent), point.dot(self.bitangent))
    }

    /// Resolves the discrete grid cell containing a 3D world point on this face.
    pub fn cell_at(&self, point: Vec3) -> GridCell {
        let (u, v) = self.decompose(point);
        GridCell::at_planar(u, v)
    }

    /// Converts discrete 0.5 m terrace steps into metric elevation above the face datum.
    pub fn step_elevation(&self, steps: i32) -> f32 {
        (steps as f32) * GridCell::SPAN
    }

    /// Projects height along the surface normal axis.
    pub fn height_of(&self, point: Vec3) -> f32 {
        point.dot(self.normal)
    }

    /// Projects elevation above the base face datum along the surface normal axis.
    pub fn elevation_of(&self, point: Vec3) -> f32 {
        point.dot(self.normal) - self.half_size
    }

    /// Clamps tangential coordinates inside the face perimeter with the given inward margin.
    pub fn clamp_interior(&self, u: f32, v: f32, margin: f32) -> (f32, f32) {
        let limit = (self.half_size - margin).max(0.1);
        (u.clamp(-limit, limit), v.clamp(-limit, limit))
    }

    pub fn normal(&self) -> Vec3 {
        self.normal
    }

    pub fn tangent(&self) -> Vec3 {
        self.tangent
    }

    pub fn bitangent(&self) -> Vec3 {
        self.bitangent
    }
}
