use glam::Vec3;
use world::MeshTarget;
use super::limb::{AxialLimb, FacetedSolid, LimbFrame};
use super::palette::{ComplexionSwatch, PigmentSwatch};
use super::pose::BodyPose;
use super::pouch::BeltSatchel;
use super::relic::HeldRelic;
use super::sight::SightMode;

/// Polygon-style forearm, sleeve, bracer, and gauntlet style.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VambraceStyle {
    BareForearms,
    RawhideWraps,
    StuddedVambraces,
    PlateGauntlets,
    ForgeGloves,
    PuffedSleeves,
}

impl VambraceStyle {
    pub fn next(self) -> Self {
        Self::from_code(self.code() + 1)
    }

    pub fn from_code(code: u8) -> Self {
        match code % 6 {
            0 => Self::BareForearms,
            1 => Self::RawhideWraps,
            2 => Self::StuddedVambraces,
            3 => Self::PlateGauntlets,
            4 => Self::ForgeGloves,
            _ => Self::PuffedSleeves,
        }
    }

    pub fn code(self) -> u8 {
        match self {
            Self::BareForearms => 0,
            Self::RawhideWraps => 1,
            Self::StuddedVambraces => 2,
            Self::PlateGauntlets => 3,
            Self::ForgeGloves => 4,
            Self::PuffedSleeves => 5,
        }
    }

    /// Emits both arms, hands, and held relics with the active palm block mold.
    #[allow(clippy::too_many_arguments)]
    pub fn emit(
        self,
        pose: &BodyPose,
        sight: SightMode,
        complexion: ComplexionSwatch,
        pigment: PigmentSwatch,
        relics: (HeldRelic, HeldRelic),
        satchel: BeltSatchel,
        target: &mut dyn MeshTarget,
    ) {
        let skin = complexion.skin();
        let hide = Vec3::new(0.42, 0.26, 0.15);
        let dark_hide = Vec3::new(0.28, 0.17, 0.10);
        let steel = Vec3::new(0.66, 0.70, 0.76);
        let brass = Vec3::new(0.84, 0.68, 0.24);
        let world_frame = LimbFrame::new(Vec3::ZERO, Vec3::X, Vec3::Y, Vec3::Z);

        for side in [-1.0f32, 1.0f32] {
            let hand_relic = if side > 0.0 { relics.0 } else { relics.1 };
            let holding = hand_relic.gripped();
            let martial = hand_relic.martial_style();
            let (shoulder_world, elbow_world, arm_frame, forearm_len) = sight.arm_joints(pose, side, holding, martial);
            let upper_dir = (elbow_world - shoulder_world).normalize_or_zero();
            let wrist_world = arm_frame.point(0.0, 0.0, 0.0);
            let forearm_dir = (wrist_world - elbow_world).normalize_or_zero();
            let forearm_up = (arm_frame.point(1.0, 0.0, 0.0) - wrist_world).cross(forearm_dir).normalize_or_zero();
            let forearm_frame = LimbFrame::new(wrist_world, forearm_dir.cross(forearm_up).normalize_or_zero(), forearm_up, forearm_dir);

            let (upper_r0, upper_r1, upper_col) = match self {
                Self::PlateGauntlets => (0.062, 0.050, steel),
                Self::PuffedSleeves => (0.078, 0.052, pigment.cloth()),
                Self::BareForearms => (0.060, 0.052, pigment.cloth()),
                _ => (0.056, 0.044, pigment.cloth()),
            };
            AxialLimb::new(
                shoulder_world,
                elbow_world + upper_dir * 0.012,
                pose.arm_girth(upper_r0),
                pose.arm_girth(upper_r1),
                upper_col,
            )
            .emit(&world_frame, true, target);

            let (wrist_col, palm_col) = match self {
                Self::BareForearms => {
                    // Rolled shirt cuff at elbow + muscular bare forearm + leather wrist band
                    AxialLimb::new(Vec3::new(0.0, 0.0, -forearm_len - 0.012), Vec3::new(0.0, 0.0, -forearm_len * 0.78), pose.arm_girth(0.056), pose.arm_girth(0.054), pigment.trim())
                        .emit(&forearm_frame, true, target);
                    AxialLimb::new(Vec3::new(0.0, 0.0, -forearm_len * 0.80), Vec3::new(0.0, 0.0, -0.040), pose.arm_girth(0.048), pose.arm_girth(0.032), skin)
                        .emit(&forearm_frame, true, target);
                    AxialLimb::new(Vec3::new(0.0, 0.0, -0.042), Vec3::new(0.0, 0.0, -0.016), pose.arm_girth(0.036), pose.arm_girth(0.032), dark_hide)
                        .emit(&forearm_frame, true, target);
                    (skin, skin)
                }
                Self::RawhideWraps => {
                    AxialLimb::new(Vec3::new(0.0, 0.0, -forearm_len - 0.010), Vec3::new(0.0, 0.0, -forearm_len * 0.56), pose.arm_girth(0.042), pose.arm_girth(0.035), skin)
                        .emit(&forearm_frame, true, target);
                    AxialLimb::new(Vec3::new(0.0, 0.0, -forearm_len * 0.58), Vec3::new(0.0, 0.0, -0.018), pose.arm_girth(0.041), pose.arm_girth(0.031), dark_hide)
                        .emit(&forearm_frame, true, target);
                    (skin, skin)
                }
                Self::StuddedVambraces => {
                    AxialLimb::new(Vec3::new(0.0, 0.0, -forearm_len - 0.010), Vec3::new(0.0, 0.0, -forearm_len * 0.56), pose.arm_girth(0.044), pose.arm_girth(0.038), pigment.trim())
                        .emit(&forearm_frame, true, target);
                    AxialLimb::new(Vec3::new(0.0, 0.0, -forearm_len * 0.58), Vec3::new(0.0, 0.0, -0.018), pose.arm_girth(0.042), pose.arm_girth(0.032), hide)
                        .emit(&forearm_frame, true, target);
                    (hide, hide)
                }
                Self::PlateGauntlets => {
                    // Faceted elbow couter + flared steel vambrace + armored gauntlet
                    AxialLimb::new(Vec3::new(0.0, 0.0, -forearm_len - 0.018), Vec3::new(0.0, 0.0, -forearm_len * 0.72), pose.arm_girth(0.056), pose.arm_girth(0.046), brass)
                        .emit(&forearm_frame, true, target);
                    AxialLimb::new(Vec3::new(0.0, 0.0, -forearm_len * 0.74), Vec3::new(0.0, 0.0, -0.016), pose.arm_girth(0.046), pose.arm_girth(0.038), steel)
                        .emit(&forearm_frame, true, target);
                    (steel, steel)
                }
                Self::ForgeGloves => {
                    // Heavy heat-shielded flared leather blacksmith gauntlets over bare upper forearms
                    AxialLimb::new(Vec3::new(0.0, 0.0, -forearm_len - 0.010), Vec3::new(0.0, 0.0, -forearm_len * 0.52), pose.arm_girth(0.048), pose.arm_girth(0.042), skin)
                        .emit(&forearm_frame, true, target);
                    AxialLimb::new(Vec3::new(0.0, 0.0, -forearm_len * 0.55), Vec3::new(0.0, 0.0, -0.014), pose.arm_girth(0.056), pose.arm_girth(0.038), dark_hide)
                        .emit(&forearm_frame, true, target);
                    (dark_hide, dark_hide)
                }
                Self::PuffedSleeves => {
                    AxialLimb::new(Vec3::new(0.0, 0.0, -forearm_len - 0.010), Vec3::new(0.0, 0.0, -0.032), pose.arm_girth(0.046), pose.arm_girth(0.034), pigment.trim())
                        .emit(&forearm_frame, true, target);
                    AxialLimb::new(Vec3::new(0.0, 0.0, -0.034), Vec3::new(0.0, 0.0, -0.014), pose.arm_girth(0.040), pose.arm_girth(0.036), brass)
                        .emit(&forearm_frame, true, target);
                    (skin, skin)
                }
            };

            // Wrist joint and hollowed/cupped palm when cradling a block
            let cradle = matches!(martial, super::relic::MartialStyle::PalmCradle);
            let cup = if cradle { -side * 0.006 } else { 0.0 };
            AxialLimb::new(Vec3::new(0.0, 0.0, -0.020), Vec3::new(0.0, 0.0, 0.004), pose.arm_girth(0.026), pose.arm_girth(0.022), wrist_col).emit(&forearm_frame, true, target);
            AxialLimb::new(Vec3::new(0.0, 0.0, 0.0), Vec3::new(-cup * 0.5, 0.0, 0.064), pose.arm_girth(0.022), pose.arm_girth(0.010), palm_col).emit(&arm_frame, true, target);
            AxialLimb::new(Vec3::new(0.0, 0.011, -0.004), Vec3::new(cup, 0.021, 0.065), pose.arm_girth(0.021), pose.arm_girth(0.010), palm_col).emit(&arm_frame, true, target);
            AxialLimb::new(Vec3::new(0.0, -0.011, -0.004), Vec3::new(cup, -0.019, 0.060), pose.arm_girth(0.020), pose.arm_girth(0.009), palm_col).emit(&arm_frame, true, target);

            let curl = pose.finger_curl(side, holding, martial);
            let open = (1.0 - curl).clamp(0.0, 1.0);
            let thumb_knuckle = Vec3::new(-side * 0.004, if cradle { 0.028 } else { 0.022 }, 0.028);
            let thumb_mid = thumb_knuckle + Vec3::new(-side * (0.006 * open + 0.026 * curl), 0.018 * open - 0.002 * curl, 0.014);
            let thumb_tip = thumb_mid + Vec3::new(-side * (0.010 * open + 0.014 * curl), 0.014 * open - 0.014 * curl, 0.014);
            AxialLimb::new(thumb_knuckle, thumb_mid, 0.0110, 0.0078, palm_col).emit(&arm_frame, false, target);
            AxialLimb::new(thumb_mid, thumb_tip, 0.0078, 0.0042, skin).emit(&arm_frame, true, target);

            for f in 0..4 {
                let t = (f as f32) / 3.0;
                let len_scale = if f == 1 || f == 2 { 1.0 } else { 0.90 };
                let knuckle = Vec3::new(cup, 0.023 - t * 0.042, 0.064 - t * 0.005);
                let mid_joint = knuckle + Vec3::new(-side * (0.008 + 0.020 * curl), 0.002 * open, (0.048 * open + 0.014 * curl) * len_scale);
                let tip = mid_joint + Vec3::new(-side * (0.018 * open + 0.018 * curl), -0.004 * open, (0.036 * open - 0.026 * curl) * len_scale);
                AxialLimb::new(knuckle, mid_joint, 0.0094, 0.0066, skin).emit(&arm_frame, true, target);
                AxialLimb::new(mid_joint, tip, 0.0066, 0.0036, skin).emit(&arm_frame, true, target);
            }

            satchel.emit_in_palm(side, hand_relic, &arm_frame, target);
        }
    }
}
