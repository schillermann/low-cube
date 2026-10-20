use glam::Vec3;
use world::MeshTarget;
use super::limb::{FacetedSolid, LimbFrame, TaperedPrism};

/// Polymorphic martial action style bound to the held relic.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MartialStyle {
    Unarmed,
    LumberjackCleave,
    SwordplaySlash,
}

/// Object held in the avatar's right hand visible in first-person and multiplayer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HeldRelic {
    EmptyPalm,
    QuarriedStone,
    TimberTorch,
    WoodenMallet,
    WoodmanAxe,
    KnightlySword,
}

impl HeldRelic {
    pub fn next(self) -> Self {
        match self {
            Self::EmptyPalm => Self::QuarriedStone,
            Self::QuarriedStone => Self::TimberTorch,
            Self::TimberTorch => Self::WoodenMallet,
            Self::WoodenMallet => Self::WoodmanAxe,
            Self::WoodmanAxe => Self::KnightlySword,
            Self::KnightlySword => Self::EmptyPalm,
        }
    }

    pub fn from_code(code: u8) -> Self {
        match code % 6 {
            0 => Self::EmptyPalm,
            1 => Self::QuarriedStone,
            2 => Self::TimberTorch,
            3 => Self::WoodenMallet,
            4 => Self::WoodmanAxe,
            _ => Self::KnightlySword,
        }
    }

    pub fn code(self) -> u8 {
        match self {
            Self::EmptyPalm => 0,
            Self::QuarriedStone => 1,
            Self::TimberTorch => 2,
            Self::WoodenMallet => 3,
            Self::WoodmanAxe => 4,
            Self::KnightlySword => 5,
        }
    }

    pub fn gripped(self) -> bool {
        !matches!(self, Self::EmptyPalm)
    }

    pub fn martial_style(self) -> MartialStyle {
        match self {
            Self::WoodmanAxe => MartialStyle::LumberjackCleave,
            Self::KnightlySword => MartialStyle::SwordplaySlash,
            Self::EmptyPalm | Self::QuarriedStone | Self::TimberTorch | Self::WoodenMallet => MartialStyle::Unarmed,
        }
    }

    /// Emits the relic seated inside the hollow grip pocket of the left or right palm (gx = -side * 0.025, gz = 0.054).
    pub fn emit_in_palm(self, side: f32, frame: &LimbFrame, target: &mut dyn MeshTarget) {
        let (gx, gz) = (-side * 0.025, 0.054);
        match self {
            Self::EmptyPalm => {}
            Self::QuarriedStone => {
                TaperedPrism::new(4, Vec3::new(gx - 0.015, -0.02, gz), Vec3::new(gx - 0.015, 0.09, gz), 0.048, 0.048, 0.048, 0.048, Vec3::new(0.55, 0.56, 0.58))
                    .emit(frame, true, target);
            }
            Self::TimberTorch => {
                TaperedPrism::hexagonal(Vec3::new(gx, -0.10, gz), Vec3::new(gx, 0.18, gz + 0.01), 0.013, 0.017, Vec3::new(0.45, 0.28, 0.14))
                    .emit(frame, false, target);
                TaperedPrism::hexagonal(Vec3::new(gx, 0.18, gz + 0.01), Vec3::new(gx, 0.25, gz + 0.015), 0.022, 0.008, Vec3::new(0.96, 0.52, 0.12))
                    .emit(frame, true, target);
            }
            Self::WoodenMallet => {
                TaperedPrism::hexagonal(Vec3::new(gx, -0.10, gz), Vec3::new(gx, 0.18, gz), 0.013, 0.014, Vec3::new(0.48, 0.31, 0.16))
                    .emit(frame, true, target);
                TaperedPrism::new(6, Vec3::new(gx, 0.14, gz - 0.04), Vec3::new(gx, 0.20, gz + 0.05), 0.032, 0.052, 0.032, 0.052, Vec3::new(0.36, 0.24, 0.13))
                    .emit(frame, true, target);
            }
            Self::WoodmanAxe => {
                let (ash_wood, worn_ash, forged_iron, blade_steel, bright_edge) = (
                    Vec3::new(0.54, 0.37, 0.20), Vec3::new(0.42, 0.28, 0.15), Vec3::new(0.28, 0.31, 0.35),
                    Vec3::new(0.52, 0.56, 0.61), Vec3::new(0.84, 0.88, 0.93),
                );
                TaperedPrism::hexagonal(Vec3::new(gx, -0.155, gz - 0.008), Vec3::new(gx, -0.115, gz), 0.016, 0.012, worn_ash).emit(frame, true, target);
                TaperedPrism::hexagonal(Vec3::new(gx, -0.115, gz), Vec3::new(gx, 0.045, gz), 0.012, 0.013, worn_ash).emit(frame, false, target);
                TaperedPrism::hexagonal(Vec3::new(gx, 0.045, gz), Vec3::new(gx, 0.252, gz + 0.004), 0.013, 0.015, ash_wood).emit(frame, true, target);
                TaperedPrism::new(6, Vec3::new(gx, 0.184, gz - 0.004), Vec3::new(gx, 0.244, gz - 0.002), 0.017, 0.025, 0.018, 0.026, forged_iron).emit(frame, true, target);
                TaperedPrism::new(4, Vec3::new(gx, 0.194, gz + 0.032), Vec3::new(gx, 0.242, gz + 0.034), 0.013, 0.024, 0.014, 0.026, forged_iron).emit(frame, true, target);
                TaperedPrism::new(4, Vec3::new(gx, 0.128, gz + 0.064), Vec3::new(gx, 0.194, gz + 0.062), 0.008, 0.020, 0.011, 0.026, blade_steel).emit(frame, true, target);
                TaperedPrism::new(4, Vec3::new(gx, 0.194, gz + 0.062), Vec3::new(gx, 0.254, gz + 0.064), 0.011, 0.026, 0.010, 0.024, blade_steel).emit(frame, true, target);
                TaperedPrism::new(4, Vec3::new(gx, 0.124, gz + 0.084), Vec3::new(gx, 0.172, gz + 0.094), 0.003, 0.010, 0.004, 0.014, bright_edge).emit(frame, true, target);
                TaperedPrism::new(4, Vec3::new(gx, 0.172, gz + 0.094), Vec3::new(gx, 0.218, gz + 0.096), 0.004, 0.014, 0.004, 0.014, bright_edge).emit(frame, true, target);
                TaperedPrism::new(4, Vec3::new(gx, 0.218, gz + 0.096), Vec3::new(gx, 0.258, gz + 0.086), 0.004, 0.014, 0.003, 0.010, bright_edge).emit(frame, true, target);
            }
            Self::KnightlySword => {
                let (grip_leather, guard_iron, fuller_steel, honed_edge) = (
                    Vec3::new(0.26, 0.15, 0.09), Vec3::new(0.32, 0.35, 0.40), Vec3::new(0.56, 0.60, 0.66), Vec3::new(0.84, 0.88, 0.94),
                );
                // Wheel pommel (Scheibenknauf) and leather-bound grip seated inside the closed palm
                TaperedPrism::new(8, Vec3::new(gx, -0.106, gz), Vec3::new(gx, -0.068, gz), 0.015, 0.024, 0.015, 0.024, guard_iron).emit(frame, true, target);
                TaperedPrism::new(6, Vec3::new(gx, -0.068, gz), Vec3::new(gx, 0.048, gz), 0.011, 0.015, 0.012, 0.016, grip_leather).emit(frame, false, target);
                // Cruciform crossguard (Parierstange) extending across the plane of the double-edged blade
                TaperedPrism::new(6, Vec3::new(gx, 0.046, gz), Vec3::new(gx, 0.066, gz), 0.013, 0.082, 0.012, 0.076, guard_iron).emit(frame, true, target);
                // Double-edged medieval blade with central fuller (Hohlkehle) tapering to a sharp point at y = 0.68
                TaperedPrism::new(4, Vec3::new(gx, 0.066, gz), Vec3::new(gx, 0.420, gz), 0.005, 0.025, 0.004, 0.020, honed_edge).emit(frame, false, target);
                TaperedPrism::new(4, Vec3::new(gx, 0.068, gz), Vec3::new(gx, 0.390, gz), 0.006, 0.011, 0.005, 0.008, fuller_steel).emit(frame, true, target);
                TaperedPrism::new(4, Vec3::new(gx, 0.420, gz), Vec3::new(gx, 0.680, gz), 0.004, 0.020, 0.001, 0.002, honed_edge).emit(frame, true, target);
            }
        }
    }
}
