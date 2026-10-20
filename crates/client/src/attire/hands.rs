use glam::Vec3;
use world::MeshTarget;
use super::limb::{AxialLimb, FacetedSolid, LimbFrame};
use super::palette::{ComplexionSwatch, PigmentSwatch};
use super::pose::BodyPose;
use super::relic::HeldRelic;
use super::sight::SightMode;

/// Medieval forearm and glove style visible in the first-person field of view and multiplayer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VambraceStyle {
    BareForearms,
    RawhideWraps,
    StuddedVambraces,
}

impl VambraceStyle {
    pub fn next(self) -> Self {
        match self {
            Self::BareForearms => Self::RawhideWraps,
            Self::RawhideWraps => Self::StuddedVambraces,
            Self::StuddedVambraces => Self::BareForearms,
        }
    }

    pub fn from_code(code: u8) -> Self {
        match code % 3 {
            0 => Self::BareForearms,
            1 => Self::RawhideWraps,
            _ => Self::StuddedVambraces,
        }
    }

    pub fn code(self) -> u8 {
        match self {
            Self::BareForearms => 0,
            Self::RawhideWraps => 1,
            Self::StuddedVambraces => 2,
        }
    }

    /// Emits both left and right upper arms, forearms, wrist gear, curved hands, and held relics in both hands.
    pub fn emit(
        self,
        pose: &BodyPose,
        sight: SightMode,
        complexion: ComplexionSwatch,
        pigment: PigmentSwatch,
        relics: (HeldRelic, HeldRelic),
        target: &mut dyn MeshTarget,
    ) {
        let skin = complexion.skin();
        let hide = Vec3::new(0.42, 0.26, 0.15);
        let dark_hide = Vec3::new(0.28, 0.17, 0.10);
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

            AxialLimb::new(
                shoulder_world,
                elbow_world + upper_dir * 0.012,
                pose.scale(0.056),
                pose.scale(0.044),
                pigment.cloth(),
            )
            .emit(&world_frame, true, target);

            // Polymorphic forearm and bracer: wristband stops cleanly behind the wrist (-0.018) with a wrist bridge into the palm
            let (wrist_col, palm_col) = match self {
                Self::BareForearms => {
                    AxialLimb::new(Vec3::new(0.0, 0.0, -forearm_len - 0.010), Vec3::new(0.0, 0.0, -0.014), pose.scale(0.042), pose.scale(0.027), skin)
                        .emit(&forearm_frame, true, target);
                    (skin, skin)
                }
                Self::RawhideWraps => {
                    AxialLimb::new(Vec3::new(0.0, 0.0, -forearm_len - 0.010), Vec3::new(0.0, 0.0, -forearm_len * 0.56), pose.scale(0.042), pose.scale(0.035), skin)
                        .emit(&forearm_frame, true, target);
                    AxialLimb::new(Vec3::new(0.0, 0.0, -forearm_len * 0.58), Vec3::new(0.0, 0.0, -0.018), pose.scale(0.039), pose.scale(0.029), dark_hide)
                        .emit(&forearm_frame, true, target);
                    (skin, skin)
                }
                Self::StuddedVambraces => {
                    AxialLimb::new(Vec3::new(0.0, 0.0, -forearm_len - 0.010), Vec3::new(0.0, 0.0, -forearm_len * 0.56), pose.scale(0.042), pose.scale(0.035), pigment.trim())
                        .emit(&forearm_frame, true, target);
                    AxialLimb::new(Vec3::new(0.0, 0.0, -forearm_len * 0.58), Vec3::new(0.0, 0.0, -0.018), pose.scale(0.039), pose.scale(0.029), hide)
                        .emit(&forearm_frame, true, target);
                    (hide, hide)
                }
            };

            // Exposed wrist joint (-0.020..0.004) connecting forearm/bracer seamlessly into the tapered palm (0.0..0.064)
            AxialLimb::new(Vec3::new(0.0, 0.0, -0.020), Vec3::new(0.0, 0.0, 0.004), pose.scale(0.026), pose.scale(0.022), wrist_col)
                .emit(&forearm_frame, true, target);
            AxialLimb::new(
                Vec3::new(0.0, 0.0, 0.0),
                Vec3::new(0.0, 0.0, 0.064),
                pose.scale(0.022),
                pose.scale(0.010),
                palm_col,
            ).emit(&arm_frame, true, target);
            AxialLimb::new(
                Vec3::new(0.0, 0.011, -0.004),
                Vec3::new(0.0, 0.019, 0.065),
                pose.scale(0.021),
                pose.scale(0.010),
                palm_col,
            ).emit(&arm_frame, true, target);
            AxialLimb::new(
                Vec3::new(0.0, -0.011, -0.004),
                Vec3::new(0.0, -0.017, 0.060),
                pose.scale(0.020),
                pose.scale(0.009),
                palm_col,
            ).emit(&arm_frame, true, target);

            // Tapered thumb (~4cm) narrowing toward the tip and wrapping around the inner side of the grip channel
            let curl = pose.finger_curl(side, holding);
            let open = (1.0 - curl).clamp(0.0, 1.0);
            let thumb_knuckle = Vec3::new(-side * 0.004, 0.022, 0.028);
            let thumb_mid = thumb_knuckle + Vec3::new(-side * (0.006 * open + 0.026 * curl), 0.018 * open - 0.002 * curl, 0.014);
            let thumb_tip = thumb_mid + Vec3::new(-side * (0.010 * open + 0.014 * curl), 0.014 * open - 0.014 * curl, 0.014);
            AxialLimb::new(thumb_knuckle, thumb_mid, 0.0110, 0.0078, palm_col).emit(&arm_frame, false, target);
            AxialLimb::new(thumb_mid, thumb_tip, 0.0078, 0.0042, skin).emit(&arm_frame, true, target);

            // 4 tapered fingers narrowing from knuckle (0.0094) through mid-joint (0.0066) to slender fingertips (0.0036)
            for f in 0..4 {
                let t = (f as f32) / 3.0;
                let len_scale = if f == 1 || f == 2 { 1.0 } else { 0.90 };
                let knuckle = Vec3::new(0.0, 0.023 - t * 0.042, 0.064 - t * 0.005);
                let mid_joint = knuckle + Vec3::new(-side * (0.008 + 0.020 * curl), 0.002 * open, (0.048 * open + 0.014 * curl) * len_scale);
                let tip = mid_joint + Vec3::new(-side * (0.018 * open + 0.018 * curl), -0.004 * open, (0.036 * open - 0.026 * curl) * len_scale);
                AxialLimb::new(knuckle, mid_joint, 0.0094, 0.0066, skin).emit(&arm_frame, true, target);
                AxialLimb::new(mid_joint, tip, 0.0066, 0.0036, skin).emit(&arm_frame, true, target);
            }

            hand_relic.emit_in_palm(side, &arm_frame, target);
        }
    }
}
