use glam::Vec3;
use world::MeshTarget;
use super::limb::{AxialLimb, FacetedSolid, LimbFrame};
use super::palette::{ComplexionSwatch, PigmentSwatch};
use super::pose::BodyPose;
use super::relic::HeldRelic;

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

    /// Emits both left and right forearms, wrist gear, relaxed curved hands, and held relic.
    pub fn emit(
        self,
        pose: &BodyPose,
        complexion: ComplexionSwatch,
        pigment: PigmentSwatch,
        relic: HeldRelic,
        target: &mut dyn MeshTarget,
    ) {
        let base_frame = pose.hand_frame();
        let skin = complexion.skin();
        let hide = Vec3::new(0.42, 0.26, 0.15);
        let dark_hide = Vec3::new(0.28, 0.17, 0.10);
        let down = pose.downward_gaze();
        let world_frame = LimbFrame::new(Vec3::ZERO, Vec3::X, Vec3::Y, Vec3::Z);

        for side in [-1.0f32, 1.0f32] {
            let (sway_x, arc_y, arc_z, swing_pitch, swing_yaw) = pose.forearm_swing(side);
            let spread_x = side * (pose.scale(0.195) + down * 0.05) + sway_x;
            let raw_frame = base_frame
                .shifted(spread_x, -0.220 + arc_y, 0.30 + arc_z)
                .yawed(side * 0.13 + swing_yaw)
                .pitched(-0.04 + swing_pitch);

            // Align arm_frame with the exact forearm vector so the hand never bends outward at the wrist
            let elbow_pt = pose.elbow_offset(side);
            let shoulder_world = pose.shoulder_socket(side);
            let wrist_world = raw_frame.point(0.0, 0.0, 0.0);
            let elbow_world = raw_frame.point(elbow_pt.x, elbow_pt.y, elbow_pt.z);
            let forearm_vec = wrist_world - elbow_world;
            let forearm_len = forearm_vec.length();
            let forearm_dir = forearm_vec / forearm_len;
            let raw_up = raw_frame.point(0.0, 1.0, 0.0) - wrist_world;
            let hand_right = forearm_dir.cross(raw_up).normalize();
            let hand_up = hand_right.cross(forearm_dir).normalize();
            let arm_frame = LimbFrame::new(wrist_world, hand_right, hand_up, forearm_dir);

            AxialLimb::new(
                shoulder_world,
                elbow_world,
                pose.scale(0.048),
                pose.scale(0.039),
                pigment.cloth(),
            )
            .emit(&world_frame, true, target);

            // Polymorphic forearm and bracer emission with zero overlapping skin geometry
            let palm_col = match self {
                Self::BareForearms => {
                    AxialLimb::new(
                        Vec3::new(0.0, 0.0, -forearm_len),
                        Vec3::new(0.0, 0.0, -0.004),
                        pose.scale(0.039),
                        pose.scale(0.021),
                        skin,
                    )
                    .emit(&arm_frame, true, target);
                    skin
                }
                Self::RawhideWraps => {
                    AxialLimb::new(
                        Vec3::new(0.0, 0.0, -forearm_len),
                        Vec3::new(0.0, 0.0, -forearm_len * 0.56),
                        pose.scale(0.039),
                        pose.scale(0.032),
                        skin,
                    )
                    .emit(&arm_frame, true, target);
                    AxialLimb::new(
                        Vec3::new(0.0, 0.0, -forearm_len * 0.58),
                        Vec3::new(0.0, 0.0, -0.004),
                        pose.scale(0.036),
                        pose.scale(0.024),
                        dark_hide,
                    )
                    .emit(&arm_frame, true, target);
                    skin
                }
                Self::StuddedVambraces => {
                    AxialLimb::new(
                        Vec3::new(0.0, 0.0, -forearm_len),
                        Vec3::new(0.0, 0.0, -forearm_len * 0.56),
                        pose.scale(0.039),
                        pose.scale(0.032),
                        pigment.trim(),
                    )
                    .emit(&arm_frame, true, target);
                    AxialLimb::new(
                        Vec3::new(0.0, 0.0, -forearm_len * 0.58),
                        Vec3::new(0.0, 0.0, -0.004),
                        pose.scale(0.036),
                        pose.scale(0.024),
                        hide,
                    )
                    .emit(&arm_frame, true, target);
                    hide
                }
            };

            // Flared vertical palm stepping out in height (+/-Y) from the narrow wrist, pointing straight along +Z
            AxialLimb::new(
                Vec3::new(0.0, 0.016, 0.0),
                Vec3::new(0.0, 0.020, 0.066),
                pose.scale(0.020),
                pose.scale(0.018),
                palm_col,
            ).emit(&arm_frame, true, target);
            AxialLimb::new(
                Vec3::new(0.0, -0.015, 0.0),
                Vec3::new(0.0, -0.018, 0.062),
                pose.scale(0.019),
                pose.scale(0.016),
                palm_col,
            ).emit(&arm_frame, true, target);

            // Compact thumb (~4cm) sticking out upward/inward when not running, and locking over the fist when running
            let curl = pose.finger_curl(side, side > 0.0 && relic.gripped());
            let open = (1.0 - curl).clamp(0.0, 1.0);
            let thumb_knuckle = Vec3::new(side * 0.004, 0.022, 0.032);
            let thumb_mid = thumb_knuckle + Vec3::new(-side * (0.006 * open + 0.010 * curl), 0.018 * open + 0.004 * curl, 0.016);
            let thumb_tip = thumb_mid + Vec3::new(-side * (0.010 * open + 0.018 * curl), 0.014 * open - 0.012 * curl, 0.014 * open + 0.006 * curl);
            AxialLimb::new(thumb_knuckle, thumb_mid, 0.0115, 0.010, palm_col).emit(&arm_frame, false, target);
            AxialLimb::new(thumb_mid, thumb_tip, 0.010, 0.0085, skin).emit(&arm_frame, true, target);

            // 4 longer vertical fingers aligned straight with the palm, folding inward into a fist when running
            for f in 0..4 {
                let t = (f as f32) / 3.0;
                let len_scale = if f == 1 || f == 2 { 1.0 } else { 0.90 };
                let knuckle = Vec3::new(0.0, 0.024 - t * 0.044, 0.066 - t * 0.005);
                let mid_joint = knuckle + Vec3::new(-side * (0.008 + 0.028 * curl), 0.002 * (1.0 - curl), 0.048 * len_scale * (1.0 - 0.68 * curl));
                let tip = mid_joint + Vec3::new(-side * 0.018 * (1.0 - 0.65 * curl), -0.005 * (0.20 + curl), 0.042 * len_scale * (0.85 - 1.50 * curl));
                AxialLimb::new(knuckle, mid_joint, 0.0096, 0.0084, skin).emit(&arm_frame, true, target);
                AxialLimb::new(mid_joint, tip, 0.0084, 0.0072, skin).emit(&arm_frame, true, target);
            }

            if side > 0.0 {
                relic.emit_in_palm(&arm_frame, target);
            }
        }
    }
}
