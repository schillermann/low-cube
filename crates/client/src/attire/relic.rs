use glam::Vec3;
use world::MeshTarget;
use super::limb::{FacetedSolid, LimbFrame, TaperedPrism};

/// Object held in the avatar's right hand visible in first-person and multiplayer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HeldRelic {
    EmptyPalm,
    QuarriedStone,
    TimberTorch,
    WoodenMallet,
}

impl HeldRelic {
    pub fn next(self) -> Self {
        match self {
            Self::EmptyPalm => Self::QuarriedStone,
            Self::QuarriedStone => Self::TimberTorch,
            Self::TimberTorch => Self::WoodenMallet,
            Self::WoodenMallet => Self::EmptyPalm,
        }
    }

    pub fn from_code(code: u8) -> Self {
        match code % 4 {
            0 => Self::EmptyPalm,
            1 => Self::QuarriedStone,
            2 => Self::TimberTorch,
            _ => Self::WoodenMallet,
        }
    }

    pub fn code(self) -> u8 {
        match self {
            Self::EmptyPalm => 0,
            Self::QuarriedStone => 1,
            Self::TimberTorch => 2,
            Self::WoodenMallet => 3,
        }
    }

    pub fn gripped(self) -> bool {
        match self {
            Self::EmptyPalm => false,
            Self::QuarriedStone | Self::TimberTorch | Self::WoodenMallet => true,
        }
    }

    pub fn emit_in_palm(self, frame: &LimbFrame, target: &mut dyn MeshTarget) {
        match self {
            Self::EmptyPalm => {}
            Self::QuarriedStone => {
                let stone_col = Vec3::new(0.55, 0.56, 0.58);
                TaperedPrism::new(
                    4,
                    Vec3::new(0.0, -0.02, 0.05),
                    Vec3::new(0.0, 0.09, 0.05),
                    0.055,
                    0.055,
                    0.055,
                    0.055,
                    stone_col,
                )
                .emit(frame, true, target);
            }
            Self::TimberTorch => {
                let wood_col = Vec3::new(0.45, 0.28, 0.14);
                let ember_col = Vec3::new(0.96, 0.52, 0.12);
                TaperedPrism::hexagonal(
                    Vec3::new(0.0, -0.08, 0.04),
                    Vec3::new(0.0, 0.14, 0.06),
                    0.016,
                    0.020,
                    wood_col,
                )
                .emit(frame, false, target);
                TaperedPrism::hexagonal(
                    Vec3::new(0.0, 0.14, 0.06),
                    Vec3::new(0.0, 0.21, 0.065),
                    0.024,
                    0.008,
                    ember_col,
                )
                .emit(frame, true, target);
            }
            Self::WoodenMallet => {
                let shaft_col = Vec3::new(0.48, 0.31, 0.16);
                let head_col = Vec3::new(0.36, 0.24, 0.13);
                TaperedPrism::hexagonal(
                    Vec3::new(0.0, -0.08, 0.04),
                    Vec3::new(0.0, 0.14, 0.05),
                    0.015,
                    0.016,
                    shaft_col,
                )
                .emit(frame, true, target);
                TaperedPrism::new(
                    6,
                    Vec3::new(0.0, 0.11, 0.01),
                    Vec3::new(0.0, 0.17, 0.09),
                    0.035,
                    0.055,
                    0.035,
                    0.055,
                    head_col,
                )
                .emit(frame, true, target);
            }
        }
    }
}
