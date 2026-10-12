use glam::Vec3;
use crate::frame::SurfaceFrame;

/// Distinct face orientation of a cubic planetary body.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CubeFace {
    Top,
    Bottom,
    Right,
    Left,
    Front,
    Back,
}

impl CubeFace {
    /// Outward unit surface normal vector.
    pub fn normal(&self) -> Vec3 {
        match self {
            Self::Top => Vec3::Y,
            Self::Bottom => Vec3::NEG_Y,
            Self::Right => Vec3::X,
            Self::Left => Vec3::NEG_X,
            Self::Front => Vec3::Z,
            Self::Back => Vec3::NEG_Z,
        }
    }

    /// Primary surface tangent vector (+U axis).
    pub fn tangent(&self) -> Vec3 {
        match self {
            Self::Top => Vec3::X,
            Self::Bottom => Vec3::X,
            Self::Right => Vec3::NEG_Z,
            Self::Left => Vec3::Z,
            Self::Front => Vec3::X,
            Self::Back => Vec3::NEG_X,
        }
    }

    /// Secondary surface bitangent vector (+V axis).
    pub fn bitangent(&self) -> Vec3 {
        match self {
            Self::Top => Vec3::Z,
            Self::Bottom => Vec3::NEG_Z,
            Self::Right => Vec3::Y,
            Self::Left => Vec3::Y,
            Self::Front => Vec3::Y,
            Self::Back => Vec3::Y,
        }
    }

    /// Constructs the orthogonal surface frame for this face at given planetary half size.
    pub fn surface_frame(&self, half_size: f32) -> SurfaceFrame {
        SurfaceFrame::new(self.tangent(), self.bitangent(), self.normal(), half_size)
    }

    /// All six cubic planetary faces.
    pub fn all() -> [Self; 6] {
        [
            Self::Top,
            Self::Bottom,
            Self::Right,
            Self::Left,
            Self::Front,
            Self::Back,
        ]
    }

    /// Dense ordinal index (0..6) for direct array indexing without linear search.
    pub fn index(&self) -> usize {
        match self {
            Self::Top => 0,
            Self::Bottom => 1,
            Self::Right => 2,
            Self::Left => 3,
            Self::Front => 4,
            Self::Back => 5,
        }
    }

    /// Resolves the dominant face for a given 3D position or direction vector.
    pub fn dominant(point: Vec3) -> Self {
        let abs = point.abs();
        if abs.y >= abs.x && abs.y >= abs.z {
            if point.y >= 0.0 { Self::Top } else { Self::Bottom }
        } else if abs.x >= abs.y && abs.x >= abs.z {
            if point.x >= 0.0 { Self::Right } else { Self::Left }
        } else {
            if point.z >= 0.0 { Self::Front } else { Self::Back }
        }
    }
}
