use glam::Vec3;
use crate::frame::SurfaceFrame;

/// Distinct climate zone surface of the cubic planetoid.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ClimateZone {
    Temperate,
    Volcanic,
    Desert,
    Arctic,
    Jungle,
    Steppe,
}

impl ClimateZone {
    /// Outward unit surface normal vector.
    pub fn normal(&self) -> Vec3 {
        match self {
            Self::Temperate => Vec3::Y,
            Self::Volcanic => Vec3::NEG_Y,
            Self::Desert => Vec3::X,
            Self::Arctic => Vec3::NEG_X,
            Self::Jungle => Vec3::Z,
            Self::Steppe => Vec3::NEG_Z,
        }
    }

    /// Primary surface tangent vector (+U axis).
    pub fn tangent(&self) -> Vec3 {
        match self {
            Self::Temperate => Vec3::X,
            Self::Volcanic => Vec3::X,
            Self::Desert => Vec3::NEG_Z,
            Self::Arctic => Vec3::Z,
            Self::Jungle => Vec3::X,
            Self::Steppe => Vec3::NEG_X,
        }
    }

    /// Secondary surface bitangent vector (+V axis).
    pub fn bitangent(&self) -> Vec3 {
        match self {
            Self::Temperate => Vec3::Z,
            Self::Volcanic => Vec3::NEG_Z,
            Self::Desert => Vec3::Y,
            Self::Arctic => Vec3::Y,
            Self::Jungle => Vec3::Y,
            Self::Steppe => Vec3::Y,
        }
    }

    /// Constructs the orthogonal surface frame for this climate zone at given planetary half size.
    pub fn surface_frame(&self, half_size: f32) -> SurfaceFrame {
        SurfaceFrame::new(self.tangent(), self.bitangent(), self.normal(), half_size)
    }

    /// All six cubic planetary climate zones.
    pub fn all() -> [Self; 6] {
        [
            Self::Temperate,
            Self::Volcanic,
            Self::Desert,
            Self::Arctic,
            Self::Jungle,
            Self::Steppe,
        ]
    }

    /// Dense ordinal index (0..6) for direct array indexing without linear search.
    pub fn index(&self) -> usize {
        match self {
            Self::Temperate => 0,
            Self::Volcanic => 1,
            Self::Desert => 2,
            Self::Arctic => 3,
            Self::Jungle => 4,
            Self::Steppe => 5,
        }
    }

    /// Resolves the dominant climate zone for a given 3D position or direction vector.
    pub fn dominant(point: Vec3) -> Self {
        let abs = point.abs();
        if abs.y >= abs.x && abs.y >= abs.z {
            if point.y >= 0.0 { Self::Temperate } else { Self::Volcanic }
        } else if abs.x >= abs.y && abs.x >= abs.z {
            if point.x >= 0.0 { Self::Desert } else { Self::Arctic }
        } else {
            if point.z >= 0.0 { Self::Jungle } else { Self::Steppe }
        }
    }
}
