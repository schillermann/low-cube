use glam::Vec3;

/// Narrow boundary interface governing spherical radial normals and crust approach envelope.
pub trait SeamBoundary: Send + Sync {
    fn blend_normal(&self, position: Vec3, face_up: Vec3) -> Vec3;
    fn fold_constrained(&self, face_index: usize, raw_point: Vec3, constrained: Vec3) -> Vec3;
    fn within_approach(&self, _position: Vec3) -> bool {
        true
    }
    fn half_span(&self) -> f32 {
        64.0
    }
}

/// Null-Object `SeamBoundary` leaving planar single-face coordinates and normals untouched.
pub struct OpenBoundary;

impl OpenBoundary {
    /// Primary constructor initializing the planar Null-Object boundary.
    pub const fn new() -> Self {
        Self
    }
}

impl SeamBoundary for OpenBoundary {
    fn blend_normal(&self, _position: Vec3, face_up: Vec3) -> Vec3 {
        face_up
    }

    fn fold_constrained(&self, _face_index: usize, _raw_point: Vec3, constrained: Vec3) -> Vec3 {
        constrained
    }
}

/// True spherical `QuadSphere` boundary resolving $C^\infty$ radial surface-up vectors and spherical approach envelopes (`1` field).
pub struct ShellBoundary {
    radius: f32,
}

impl ShellBoundary {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(radius: f32) -> Self {
        Self { radius }
    }
}

impl SeamBoundary for ShellBoundary {
    fn blend_normal(&self, position: Vec3, face_up: Vec3) -> Vec3 {
        position.try_normalize().unwrap_or(face_up)
    }

    fn fold_constrained(&self, _face_index: usize, _raw_point: Vec3, constrained: Vec3) -> Vec3 {
        constrained
    }

    fn within_approach(&self, position: Vec3) -> bool {
        let band = (self.radius * 0.45).clamp(36.0, 160.0);
        position.length() <= self.radius + band
    }

    fn half_span(&self) -> f32 {
        self.radius
    }
}
