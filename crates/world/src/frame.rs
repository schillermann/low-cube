use glam::Vec3;
use crate::block::GridCell;

/// Surface projection topology distinguishing planar decks from round `QuadSphere` planetary shells.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FrameShape {
    Planar,
    QuadSphere,
}

/// Orthogonal local coordinate frame of a planetary shell or deck (`4` strictly private fields).
#[derive(Clone, Copy, Debug)]
pub struct SurfaceFrame {
    axes: (Vec3, Vec3),
    normal: Vec3,
    half_size: f32,
    shape: FrameShape,
}

impl SurfaceFrame {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn with_shape(
        axes: (Vec3, Vec3),
        normal: Vec3,
        half_size: f32,
        shape: FrameShape,
    ) -> Self {
        Self { axes, normal, half_size, shape }
    }

    /// Secondary constructor for local planar coordinate frames (`FrameShape::Planar`).
    pub const fn new(tangent: Vec3, bitangent: Vec3, normal: Vec3, half_size: f32) -> Self {
        Self::with_shape((tangent, bitangent), normal, half_size, FrameShape::Planar)
    }

    /// Secondary constructor projecting `[-radius, radius]²` smoothly onto a true sphere of `radius`.
    pub const fn quad_sphere(tangent: Vec3, bitangent: Vec3, normal: Vec3, radius: f32) -> Self {
        Self::with_shape((tangent, bitangent), normal, radius, FrameShape::QuadSphere)
    }

    pub const fn spherical(&self) -> bool {
        matches!(self.shape, FrameShape::QuadSphere)
    }

    /// Verifies whether `point` lies inside this shell's `90°` frustum cone.
    pub fn matches_cone(&self, point: Vec3) -> bool {
        let n = point.dot(self.normal);
        let tu = point.dot(self.axes.0).abs();
        let tv = point.dot(self.axes.1).abs();
        n > 0.0 && n >= tu && n >= tv
    }

    /// Evaluates a 3D position given tangential coordinates `(u, v)` and elevation above datum.
    pub fn point_at(&self, u: f32, v: f32, elevation: f32) -> Vec3 {
        let (tangent, bitangent) = self.axes;
        match self.shape {
            FrameShape::Planar => {
                self.normal * (self.half_size + elevation) + tangent * u + bitangent * v
            }
            FrameShape::QuadSphere => {
                let cube_pt = self.normal * self.half_size + tangent * u + bitangent * v;
                let radial = cube_pt.try_normalize().unwrap_or(self.normal);
                radial * (self.half_size + elevation)
            }
        }
    }

    /// Decomposes a 3D position into 2D tangential shell coordinates `(u, v)`.
    pub fn decompose(&self, point: Vec3) -> (f32, f32) {
        let (tangent, bitangent) = self.axes;
        match self.shape {
            FrameShape::Planar => (point.dot(tangent), point.dot(bitangent)),
            FrameShape::QuadSphere => {
                let denom = point.dot(self.normal).max(1e-4);
                let scale = self.half_size / denom;
                (point.dot(tangent) * scale, point.dot(bitangent) * scale)
            }
        }
    }

    /// Decomposes a free 3D direction vector onto `(tangent, bitangent)` without point division.
    pub fn decompose_vector(&self, vector: Vec3) -> (f32, f32) {
        (vector.dot(self.axes.0), vector.dot(self.axes.1))
    }

    /// Resolves the discrete `0.5 m` grid cell containing `point` on this shell.
    pub fn cell_at(&self, point: Vec3) -> GridCell {
        let (u, v) = self.decompose(point);
        GridCell::at_planar(u, v)
    }

    /// Converts discrete `0.5 m` terrace steps into metric elevation above datum.
    pub fn step_elevation(&self, steps: i32) -> f32 {
        (steps as f32) * GridCell::SPAN
    }

    /// Projects radial/normal height from planet center (`half_size + elevation`).
    pub fn height_of(&self, point: Vec3) -> f32 {
        match self.shape {
            FrameShape::Planar => point.dot(self.normal),
            FrameShape::QuadSphere => point.length(),
        }
    }

    /// Projects elevation above the shell datum (`radius` on `QuadSphere`, `half_size` plane on `Planar`).
    pub fn elevation_of(&self, point: Vec3) -> f32 {
        self.height_of(point) - self.half_size
    }

    /// Local outward surface-up unit vector (`radial` on `QuadSphere`, `normal` on `Planar`).
    pub fn up_at(&self, point: Vec3) -> Vec3 {
        match self.shape {
            FrameShape::Planar => self.normal,
            FrameShape::QuadSphere => point.try_normalize().unwrap_or(self.normal),
        }
    }

    /// Local unit `(tangent, bitangent)` basis orthogonal to `up_at(point)`.
    pub fn tangent_at(&self, point: Vec3) -> (Vec3, Vec3) {
        match self.shape {
            FrameShape::Planar => self.axes,
            FrameShape::QuadSphere => {
                let up = self.up_at(point);
                let tang = (self.axes.0 - up * self.axes.0.dot(up))
                    .try_normalize()
                    .unwrap_or(self.axes.0);
                let bitan = (self.axes.1 - up * self.axes.1.dot(up))
                    .try_normalize()
                    .unwrap_or(self.axes.1);
                (tang, bitan)
            }
        }
    }

    /// Clamps tangential coordinates inside the shell perimeter with the given inward margin.
    pub fn clamp_interior(&self, u: f32, v: f32, margin: f32) -> (f32, f32) {
        let limit = (self.half_size - margin).max(0.1);
        (u.clamp(-limit, limit), v.clamp(-limit, limit))
    }

    pub const fn normal(&self) -> Vec3 {
        self.normal
    }

    pub const fn tangent(&self) -> Vec3 {
        self.axes.0
    }

    pub const fn bitangent(&self) -> Vec3 {
        self.axes.1
    }
}
