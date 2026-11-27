use glam::Vec3;
use world::{MeshTarget, PalmMolds};
use super::limb::{AxialLimb, FacetedSolid, LimbFrame, TaperedPrism};
use super::palette::{ComplexionSwatch, PigmentSwatch};
use super::pose::BodyPose;
use super::relic::HeldRelic;
use super::sight::SightMode;

/// Encapsulated dual-hand held relics, active palm molds, and open-satchel inspection posture.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HeldPalms {
    relics: (HeldRelic, HeldRelic),
    molds: PalmMolds,
    tray_open: bool,
}

impl HeldPalms {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(relics: (HeldRelic, HeldRelic), molds: PalmMolds, tray_open: bool) -> Self {
        Self { relics, molds, tray_open }
    }

    const fn relic_on(self, side: f32) -> HeldRelic {
        if side > 0.0 { self.relics.0 } else { self.relics.1 }
    }

    fn emit_in_palm(self, side: f32, frame: &LimbFrame, target: &mut dyn MeshTarget) {
        self.relic_on(side).figure(self.molds.on_side(side)).emit_in_palm(side, frame, target);
    }
}

/// Space-Age pressurized EVA wrist disconnect ring, forearm sleeve, and tactile gauntlet style.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SuitGauntlet {
    SealCuffs,
    TactileGloves,
    InsulatedWraps,
    TitaniumGauntlets,
    WeldMitts,
    TelemetrySleeves,
}

impl SuitGauntlet {
    pub fn next(self) -> Self {
        Self::from_code(self.code() + 1)
    }

    pub fn from_code(code: u8) -> Self {
        match code % 6 {
            0 => Self::SealCuffs,
            1 => Self::TactileGloves,
            2 => Self::InsulatedWraps,
            3 => Self::TitaniumGauntlets,
            4 => Self::WeldMitts,
            _ => Self::TelemetrySleeves,
        }
    }

    pub fn code(self) -> u8 {
        match self {
            Self::SealCuffs => 0,
            Self::TactileGloves => 1,
            Self::InsulatedWraps => 2,
            Self::TitaniumGauntlets => 3,
            Self::WeldMitts => 4,
            Self::TelemetrySleeves => 5,
        }
    }

    /// Emits both pressurized suit arms, anodized wrist disconnect rings, EVA gloves, and held tools/blocks.
    pub fn emit(
        self,
        pose: &BodyPose,
        sight: SightMode,
        swatches: (ComplexionSwatch, PigmentSwatch),
        palms: HeldPalms,
        target: &mut dyn MeshTarget,
    ) {
        let (_complexion, pigment) = swatches;
        let carbon = Vec3::new(0.22, 0.26, 0.32);
        let silicone_grip = Vec3::new(0.30, 0.35, 0.42);
        let titanium = Vec3::new(0.68, 0.74, 0.82);
        let anodized_ring = Vec3::new(0.24, 0.64, 0.92);
        let amber_foil = Vec3::new(0.88, 0.66, 0.22);
        let world_frame = LimbFrame::new(Vec3::ZERO, Vec3::X, Vec3::Y, Vec3::Z);

        for side in [-1.0f32, 1.0f32] {
            let hand_relic = palms.relic_on(side);
            let holding = hand_relic.gripped();
            let martial = hand_relic.martial_style();
            let (shoulder_world, elbow_world, arm_frame, forearm_len) =
                sight.arm_joints(pose, side, holding, martial, palms.tray_open);
            let upper_dir = (elbow_world - shoulder_world).normalize_or_zero();
            let wrist_world = arm_frame.point(0.0, 0.0, 0.0);
            let forearm_dir = (wrist_world - elbow_world).normalize_or_zero();
            let forearm_up = (arm_frame.point(1.0, 0.0, 0.0) - wrist_world).cross(forearm_dir).normalize_or_zero();
            let forearm_frame = LimbFrame::new(wrist_world, forearm_dir.cross(forearm_up).normalize_or_zero(), forearm_up, forearm_dir);

            let (upper_r0, upper_r1, upper_col) = match self {
                Self::TitaniumGauntlets => (0.062, 0.050, titanium),
                Self::TelemetrySleeves => (0.068, 0.052, pigment.cloth()),
                _ => (0.058, 0.046, pigment.cloth()),
            };
            AxialLimb::new(
                shoulder_world,
                elbow_world + upper_dir * 0.012,
                pose.arm_girth(upper_r0),
                pose.arm_girth(upper_r1),
                upper_col,
            )
            .emit(&world_frame, true, target);

            let (ring_col, palm_col, finger_col) = match self {
                Self::SealCuffs => {
                    AxialLimb::new(Vec3::new(0.0, 0.0, -forearm_len - 0.012), Vec3::new(0.0, 0.0, -0.024), pose.arm_girth(0.050), pose.arm_girth(0.036), pigment.cloth())
                        .emit(&forearm_frame, true, target);
                    (anodized_ring, silicone_grip, carbon)
                }
                Self::TactileGloves => {
                    AxialLimb::new(Vec3::new(0.0, 0.0, -forearm_len - 0.010), Vec3::new(0.0, 0.0, -forearm_len * 0.48), pose.arm_girth(0.048), pose.arm_girth(0.040), pigment.cloth())
                        .emit(&forearm_frame, true, target);
                    AxialLimb::new(Vec3::new(0.0, 0.0, -forearm_len * 0.50), Vec3::new(0.0, 0.0, -0.020), pose.arm_girth(0.042), pose.arm_girth(0.034), carbon)
                        .emit(&forearm_frame, true, target);
                    (anodized_ring, silicone_grip, silicone_grip)
                }
                Self::InsulatedWraps => {
                    AxialLimb::new(Vec3::new(0.0, 0.0, -forearm_len - 0.010), Vec3::new(0.0, 0.0, -0.020), pose.arm_girth(0.048), pose.arm_girth(0.036), pigment.trim())
                        .emit(&forearm_frame, true, target);
                    (amber_foil, pigment.cloth() * 0.88, silicone_grip)
                }
                Self::TitaniumGauntlets => {
                    AxialLimb::new(Vec3::new(0.0, 0.0, -forearm_len - 0.018), Vec3::new(0.0, 0.0, -0.018), pose.arm_girth(0.054), pose.arm_girth(0.040), titanium)
                        .emit(&forearm_frame, true, target);
                    (amber_foil, titanium, silicone_grip)
                }
                Self::WeldMitts => {
                    AxialLimb::new(Vec3::new(0.0, 0.0, -forearm_len - 0.010), Vec3::new(0.0, 0.0, -0.018), pose.arm_girth(0.052), pose.arm_girth(0.040), amber_foil)
                        .emit(&forearm_frame, true, target);
                    (titanium, carbon, carbon)
                }
                Self::TelemetrySleeves => {
                    AxialLimb::new(Vec3::new(0.0, 0.0, -forearm_len - 0.010), Vec3::new(0.0, 0.0, -0.024), pose.arm_girth(0.048), pose.arm_girth(0.036), pigment.cloth())
                        .emit(&forearm_frame, true, target);
                    (anodized_ring, pigment.cloth(), silicone_grip)
                }
            };

            let (hs, hg) = if pose.feminine() { (1.0, 1.0) } else { (1.18, 1.16) };
            let hand_frame = arm_frame.scaled(hs);
            let palm_frame = arm_frame.pitched(std::f32::consts::FRAC_PI_2).scaled(hs);
            let cradle = !pose.piloting_rocket() && matches!(martial, super::relic::MartialStyle::PalmCradle);
            let cup = if cradle { side * 0.003 } else { 0.0 };
            AxialLimb::new(Vec3::new(0.0, 0.0, -0.024), Vec3::new(0.0, 0.0, 0.004), pose.arm_girth(0.031 * hg), pose.arm_girth(0.028 * hg), ring_col).emit(&forearm_frame, true, target);
            TaperedPrism::new(8, Vec3::new(0.0, -0.004, 0.0), Vec3::new(cup, 0.032, 0.0), pose.arm_girth(0.014), pose.arm_girth(0.026 * hg), pose.arm_girth(0.012), pose.arm_girth(0.034 * hg), palm_col).emit(&palm_frame, false, target);
            TaperedPrism::new(8, Vec3::new(cup, 0.032, 0.0), Vec3::new(cup, 0.068, 0.0), pose.arm_girth(0.012), pose.arm_girth(0.034 * hg), pose.arm_girth(0.010), pose.arm_girth(0.035 * hg), palm_col).emit(&palm_frame, true, target);

            let curl = pose.arm(side, martial).finger_curl(holding);
            let open = (1.0 - curl).clamp(0.0, 1.0);
            let thumb_knuckle = Vec3::new(-side * 0.002, pose.arm_girth((if cradle { 0.031 } else { 0.027 }) * hg), 0.024);
            let thumb_mid = thumb_knuckle + Vec3::new(-side * (0.006 * open + 0.024 * curl), 0.018 * open - 0.002 * curl, 0.016);
            let thumb_tip = thumb_mid + Vec3::new(-side * (0.010 * open + 0.014 * curl), 0.014 * open - 0.014 * curl, 0.016);
            AxialLimb::new(thumb_knuckle, thumb_mid, pose.arm_girth(0.0125 * hg), pose.arm_girth(0.0094 * hg), palm_col).emit(&hand_frame, false, target);
            AxialLimb::new(thumb_mid, thumb_tip, pose.arm_girth(0.0094 * hg), pose.arm_girth(0.0062 * hg), finger_col).emit(&hand_frame, true, target);

            for f in 0..4 {
                let t = (f as f32) / 3.0;
                let len_scale = if f == 1 || f == 2 { 1.0 } else { 0.90 };
                let knuckle = Vec3::new(cup, pose.arm_girth((0.022 - t * 0.044) * hg), 0.062 - t * 0.003);
                let mid_joint = knuckle + Vec3::new(-side * (0.007 + 0.020 * curl), 0.002 * open, (0.048 * open + 0.015 * curl) * len_scale);
                let tip = mid_joint + Vec3::new(-side * (0.016 * open + 0.018 * curl), -0.004 * open, (0.036 * open - 0.026 * curl) * len_scale);
                AxialLimb::new(knuckle, mid_joint, pose.arm_girth(0.0105 * hg), pose.arm_girth(0.0080 * hg), finger_col).emit(&hand_frame, true, target);
                AxialLimb::new(mid_joint, tip, pose.arm_girth(0.0080 * hg), pose.arm_girth(0.0054 * hg), finger_col).emit(&hand_frame, true, target);
            }

            let relic_frame = if cradle && !pose.feminine() { arm_frame.shifted(-side * 0.003, 0.0, 0.008) } else { arm_frame };
            palms.emit_in_palm(side, &relic_frame, target);
        }
    }
}
