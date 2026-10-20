use glam::Vec3;
use world::MeshTarget;
use super::limb::{AxialLimb, FacetedSolid};
use super::palette::PigmentSwatch;
use super::pose::BodyPose;

/// Polygon-style footwear, boots, gaiters, and armored greaves.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FootwearStyle {
    CuffedBoots,
    LacedBoots,
    Turnshoes,
    PlateGreaves,
    ForgeBoots,
    GarrisonBoots,
}

impl FootwearStyle {
    pub fn next(self) -> Self {
        Self::from_code(self.code() + 1)
    }

    pub fn from_code(code: u8) -> Self {
        match code % 6 {
            0 => Self::CuffedBoots,
            1 => Self::LacedBoots,
            2 => Self::Turnshoes,
            3 => Self::PlateGreaves,
            4 => Self::ForgeBoots,
            _ => Self::GarrisonBoots,
        }
    }

    pub fn code(self) -> u8 {
        match self {
            Self::CuffedBoots => 0,
            Self::LacedBoots => 1,
            Self::Turnshoes => 2,
            Self::PlateGreaves => 3,
            Self::ForgeBoots => 4,
            Self::GarrisonBoots => 5,
        }
    }

    /// Emits left and right thighs, knees, calves, and Polygon-style boots or greaves.
    pub fn emit(
        self,
        pose: &BodyPose,
        pigment: PigmentSwatch,
        target: &mut dyn MeshTarget,
    ) {
        let frame = pose.torso_frame();
        let hose_col = pigment.chausses();
        let garter_col = pigment.trim();
        let vest = Vec3::new(0.46, 0.28, 0.16);
        let boot_col = Vec3::new(0.30, 0.19, 0.11);
        let dark_boot = Vec3::new(0.18, 0.12, 0.08);
        let steel = Vec3::new(0.66, 0.70, 0.76);
        let brass = Vec3::new(0.84, 0.68, 0.24);

        for side in [-1.0f32, 1.0f32] {
            let (hip_pt, knee_pt, ankle_pt, toe_pt) = pose.leg_joints(side);
            let knee_above = hip_pt.lerp(knee_pt, 0.84);
            let calf_belly = knee_pt.lerp(ankle_pt, 0.36) + Vec3::new(side * 0.005, 0.0, -0.014);

            let thigh_col = match self {
                Self::GarrisonBoots if side > 0.0 => pigment.cloth(),
                Self::ForgeBoots => dark_boot,
                _ => hose_col,
            };
            AxialLimb::new(hip_pt, knee_above, pose.leg_girth(0.078), pose.leg_girth(0.066), thigh_col)
                .emit(&frame, true, target);

            let (knee_r0, knee_r1, knee_col) = match self {
                Self::PlateGreaves => (0.080, 0.074, brass),
                Self::ForgeBoots => (0.078, 0.074, steel),
                Self::LacedBoots => (0.076, 0.072, dark_boot),
                _ => (0.069, 0.066, garter_col),
            };
            AxialLimb::new(knee_above, knee_pt, pose.leg_girth(knee_r0), pose.leg_girth(knee_r1), knee_col)
                .emit(&frame, true, target);

            let (calf_col, shaft_col, foot_col) = match self {
                Self::PlateGreaves => (steel, steel, steel),
                Self::Turnshoes => (hose_col, vest, boot_col),
                Self::ForgeBoots => (dark_boot, vest, steel),
                Self::GarrisonBoots => (steel * 0.85, dark_boot, dark_boot),
                Self::LacedBoots => (hose_col, dark_boot, dark_boot),
                Self::CuffedBoots => (hose_col, boot_col, boot_col),
            };

            AxialLimb::new(knee_pt, calf_belly, pose.leg_girth(0.066), pose.leg_girth(0.078), calf_col)
                .emit(&frame, true, target);
            AxialLimb::new(calf_belly, ankle_pt, pose.leg_girth(0.078), pose.leg_girth(0.062), shaft_col)
                .emit(&frame, true, target);

            match self {
                Self::CuffedBoots => {
                    let cuff_bottom = knee_pt.lerp(ankle_pt, 0.50);
                    AxialLimb::new(cuff_bottom, calf_belly, pose.leg_girth(0.078), pose.leg_girth(0.088), brass)
                        .emit(&frame, true, target);
                }
                Self::LacedBoots | Self::GarrisonBoots => {
                    let strap_top = knee_pt.lerp(ankle_pt, 0.56);
                    AxialLimb::new(strap_top, ankle_pt, pose.leg_girth(0.080), pose.leg_girth(0.066), vest)
                        .emit(&frame, true, target);
                }
                Self::PlateGreaves | Self::Turnshoes | Self::ForgeBoots => {}
            }

            let heel_pt = ankle_pt + Vec3::new(0.0, -0.018, -0.028);
            AxialLimb::new(heel_pt, toe_pt, pose.leg_girth(0.072), pose.leg_girth(0.056), foot_col)
                .emit(&frame, true, target);
        }
    }
}
