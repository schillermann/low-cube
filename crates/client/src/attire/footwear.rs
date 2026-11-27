use glam::Vec3;
use world::MeshTarget;
use super::limb::{AxialLimb, FacetedSolid};
use super::palette::PigmentSwatch;
use super::pose::BodyPose;

/// Space-Age EVA magnetic-sole boots, regolith gaiters, and armored leg greaves.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MagBootStyle {
    MagLockBoots,
    TreadedGaiters,
    DeckSlipons,
    ArmoredSabatons,
    ThermalBoots,
    ClampBoots,
}

impl MagBootStyle {
    pub fn next(self) -> Self {
        Self::from_code(self.code() + 1)
    }

    pub fn from_code(code: u8) -> Self {
        match code % 6 {
            0 => Self::MagLockBoots,
            1 => Self::TreadedGaiters,
            2 => Self::DeckSlipons,
            3 => Self::ArmoredSabatons,
            4 => Self::ThermalBoots,
            _ => Self::ClampBoots,
        }
    }

    pub fn code(self) -> u8 {
        match self {
            Self::MagLockBoots => 0,
            Self::TreadedGaiters => 1,
            Self::DeckSlipons => 2,
            Self::ArmoredSabatons => 3,
            Self::ThermalBoots => 4,
            Self::ClampBoots => 5,
        }
    }

    /// Emits left and right pressurized suit legs, knee bellows, and magnetic-sole boots.
    pub fn emit(
        self,
        pose: &BodyPose,
        pigment: PigmentSwatch,
        target: &mut dyn MeshTarget,
    ) {
        let frame = pose.stance_frame();
        let suit_leg = pigment.trousers();
        let trim_col = pigment.trim();
        let carbon = Vec3::new(0.22, 0.26, 0.32);
        let mag_sole = Vec3::new(0.16, 0.19, 0.24);
        let titanium = Vec3::new(0.68, 0.74, 0.82);
        let amber = Vec3::new(0.92, 0.70, 0.22);

        for side in [-1.0f32, 1.0f32] {
            let (hip_pt, knee_pt, ankle_pt, toe_pt) = pose.leg_joints(side);
            let knee_above = hip_pt.lerp(knee_pt, 0.84);
            let calf_belly = knee_pt.lerp(ankle_pt, 0.36) + Vec3::new(side * 0.005, 0.0, -0.014);

            let thigh_col = match self {
                Self::ClampBoots if side > 0.0 => pigment.cloth(),
                Self::ThermalBoots => pigment.cloth(),
                _ => suit_leg,
            };
            AxialLimb::new(hip_pt, knee_above, pose.leg_girth(0.078), pose.leg_girth(0.066), thigh_col)
                .emit(&frame, true, target);

            let (knee_r0, knee_r1, knee_col) = match self {
                Self::ArmoredSabatons => (0.080, 0.074, titanium),
                Self::ThermalBoots => (0.078, 0.074, amber),
                Self::TreadedGaiters => (0.076, 0.072, carbon),
                _ => (0.069, 0.066, trim_col),
            };
            AxialLimb::new(knee_above, knee_pt, pose.leg_girth(knee_r0), pose.leg_girth(knee_r1), knee_col)
                .emit(&frame, true, target);

            let (calf_col, shaft_col, foot_col) = match self {
                Self::ArmoredSabatons => (titanium, titanium, mag_sole),
                Self::DeckSlipons => (suit_leg, carbon, mag_sole),
                Self::ThermalBoots => (pigment.cloth(), carbon, titanium),
                Self::ClampBoots => (titanium * 0.85, carbon, mag_sole),
                Self::TreadedGaiters => (suit_leg, carbon, mag_sole),
                Self::MagLockBoots => (suit_leg, titanium * 0.78, mag_sole),
            };

            AxialLimb::new(knee_pt, calf_belly, pose.leg_girth(0.066), pose.leg_girth(0.078), calf_col)
                .emit(&frame, true, target);
            AxialLimb::new(calf_belly, ankle_pt, pose.leg_girth(0.078), pose.leg_girth(0.062), shaft_col)
                .emit(&frame, true, target);

            match self {
                Self::MagLockBoots => {
                    let cuff_bottom = knee_pt.lerp(ankle_pt, 0.50);
                    AxialLimb::new(cuff_bottom, calf_belly, pose.leg_girth(0.078), pose.leg_girth(0.086), amber)
                        .emit(&frame, true, target);
                }
                Self::TreadedGaiters | Self::ClampBoots => {
                    let strap_top = knee_pt.lerp(ankle_pt, 0.56);
                    AxialLimb::new(strap_top, ankle_pt, pose.leg_girth(0.080), pose.leg_girth(0.066), trim_col)
                        .emit(&frame, true, target);
                }
                Self::ArmoredSabatons | Self::DeckSlipons | Self::ThermalBoots => {}
            }

            let heel_pt = ankle_pt + Vec3::new(0.0, -0.018, -0.028);
            AxialLimb::new(heel_pt, toe_pt, pose.leg_girth(0.074), pose.leg_girth(0.058), foot_col)
                .emit(&frame, true, target);
        }
    }
}
