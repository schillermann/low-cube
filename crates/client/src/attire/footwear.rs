use glam::Vec3;
use world::MeshTarget;
use super::limb::{AxialLimb, FacetedSolid};
use super::palette::PigmentSwatch;
use super::pose::BodyPose;

/// Medieval footwear and boot style.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FootwearStyle {
    CuffedBoots,
    LacedBoots,
    Turnshoes,
}

impl FootwearStyle {
    pub fn next(self) -> Self {
        match self {
            Self::CuffedBoots => Self::LacedBoots,
            Self::LacedBoots => Self::Turnshoes,
            Self::Turnshoes => Self::CuffedBoots,
        }
    }

    pub fn from_code(code: u8) -> Self {
        match code % 3 {
            0 => Self::CuffedBoots,
            1 => Self::LacedBoots,
            _ => Self::Turnshoes,
        }
    }

    pub fn code(self) -> u8 {
        match self {
            Self::CuffedBoots => 0,
            Self::LacedBoots => 1,
            Self::Turnshoes => 2,
        }
    }

    /// Emits left and right chausses (thigh and upper shin) and medieval boots with clear knee articulation.
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

        for side in [-1.0f32, 1.0f32] {
            let (hip_pt, knee_pt, ankle_pt, toe_pt) = pose.leg_joints(side);
            let knee_above = hip_pt.lerp(knee_pt, 0.85);
            let calf_belly = knee_pt.lerp(ankle_pt, 0.36) + Vec3::new(side * 0.005, 0.0, -0.014);

            // Athletic thigh emerging cleanly beneath the tunic skirt and tapering into the knee
            AxialLimb::new(hip_pt, knee_above, pose.scale(0.078), pose.scale(0.066), hose_col)
                .emit(&frame, true, target);

            // Distinct knee joint & garter wrap marking the articulation point
            AxialLimb::new(knee_above, knee_pt, pose.scale(0.069), pose.scale(0.066), garter_col)
                .emit(&frame, true, target);

            // Upper shin expanding into a strong muscular calf belly (gastrocnemius)
            AxialLimb::new(knee_pt, calf_belly, pose.scale(0.066), pose.scale(0.076), hose_col)
                .emit(&frame, true, target);

            // Lower calf / boot shaft tapering from the calf belly down to the ankle
            let shaft_col = match self {
                Self::Turnshoes => hose_col,
                Self::CuffedBoots | Self::LacedBoots => boot_col,
            };
            AxialLimb::new(calf_belly, ankle_pt, pose.scale(0.076), pose.scale(0.062), shaft_col)
                .emit(&frame, true, target);

            match self {
                Self::CuffedBoots => {
                    let cuff_bottom = knee_pt.lerp(ankle_pt, 0.54);
                    AxialLimb::new(cuff_bottom, calf_belly, pose.scale(0.074), pose.scale(0.082), vest)
                        .emit(&frame, true, target);
                }
                Self::LacedBoots | Self::Turnshoes => {}
            }

            // Sturdy medieval boot foot extending from heel through instep to toe
            let heel_pt = ankle_pt + Vec3::new(0.0, -0.018, -0.028);
            AxialLimb::new(heel_pt, toe_pt, pose.scale(0.070), pose.scale(0.056), boot_col)
                .emit(&frame, true, target);
        }
    }
}
