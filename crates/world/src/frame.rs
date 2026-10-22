use glam::Vec3;

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

    /// Projects height along the surface normal axis.
    pub fn height_of(&self, point: Vec3) -> f32 {
        point.dot(self.normal)
    }

    /// Projects elevation above the base face datum along the surface normal axis.
    pub fn elevation_of(&self, point: Vec3) -> f32 {
        point.dot(self.normal) - self.half_size
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
