mod arms;

use glam::{Quat, Vec3};
use world::SurfaceSightRay;
use super::limb::LimbFrame;
use super::proportion::YouthPhysique;
use super::rhythm::{MartialStyle, StrideRhythm};
pub use arms::ArmKinematics;

#[derive(Clone, Copy, Debug)]
struct PoseStance {
    position: Vec3,
    surface_rot: Quat,
    view_rot: Quat,
    eye_height: f32,
}

#[derive(Clone, Copy, Debug)]
struct PoseMotion {
    leap: f32,
    hover: f32,
    rhythm: StrideRhythm,
    physique: YouthPhysique,
}

/// Kinematic pose of an avatar body on a planetary surface for first-person and multiplayer display.
#[derive(Clone, Copy, Debug)]
pub struct BodyPose {
    stance: PoseStance,
    motion: PoseMotion,
}

impl BodyPose {
    /// Primary constructor doing nothing except assigning arguments to fields.
    #[allow(clippy::too_many_arguments)]
    pub fn with_hover_pose(
        position: Vec3,
        surface_rot: Quat,
        view_rot: Quat,
        eye_height: f32,
        leap: f32,
        hover: f32,
        rhythm: StrideRhythm,
        physique: YouthPhysique,
    ) -> Self {
        Self {
            stance: PoseStance { position, surface_rot, view_rot, eye_height },
            motion: PoseMotion { leap, hover, rhythm, physique },
        }
    }

    pub fn with_leap_pose(
        position: Vec3,
        surface_rot: Quat,
        view_rot: Quat,
        eye_height: f32,
        leap: f32,
        rhythm: StrideRhythm,
        physique: YouthPhysique,
    ) -> Self {
        Self::with_hover_pose(position, surface_rot, view_rot, eye_height, leap, 0.0, rhythm, physique)
    }

    pub fn new(
        position: Vec3,
        surface_rot: Quat,
        view_rot: Quat,
        eye_height: f32,
        rhythm: StrideRhythm,
        physique: YouthPhysique,
    ) -> Self {
        Self::with_leap_pose(position, surface_rot, view_rot, eye_height, 0.0, rhythm, physique)
    }

    pub fn with_leap(self, leap: f32) -> Self {
        Self { motion: PoseMotion { leap, ..self.motion }, ..self }
    }

    pub fn with_hover(self, hover: f32) -> Self {
        Self { motion: PoseMotion { hover, ..self.motion }, ..self }
    }

    pub fn with_seated(self, seated: f32) -> Self {
        Self { motion: PoseMotion { hover: -2.0 * seated.clamp(0.0, 1.0), ..self.motion }, ..self }
    }

    pub fn with_seated_steer(self, steer: f32) -> Self {
        Self { motion: PoseMotion { hover: -(2.0 + steer.clamp(-1.0, 1.0)), ..self.motion }, ..self }
    }

    pub fn with_surface_rotation(self, surface_rot: Quat) -> Self {
        Self { stance: PoseStance { surface_rot, ..self.stance }, ..self }
    }

    pub fn with_physique(self, physique: YouthPhysique) -> Self {
        Self { motion: PoseMotion { physique, ..self.motion }, ..self }
    }

    pub const fn hover_factor(&self) -> f32 { self.motion.hover.max(0.0) }
    pub const fn seated_factor(&self) -> f32 {
        if self.motion.hover <= -0.95 { 1.0 } else { (-self.motion.hover).clamp(0.0, 1.0) }
    }
    pub const fn steering_turn(&self) -> f32 {
        if self.motion.hover <= -0.95 { (-self.motion.hover - 2.0).clamp(-1.0, 1.0) } else { 0.0 }
    }
    pub(super) const fn eye_height(&self) -> f32 { self.stance.eye_height }
    pub(super) const fn motion_factors(&self) -> (StrideRhythm, f32, f32) { (self.motion.rhythm, self.hover_factor(), self.motion.leap) }
    pub(super) fn view_axes(&self) -> (Vec3, Vec3, Vec3) {
        (
            (self.stance.surface_rot * Vec3::Y).normalize(),
            (self.stance.view_rot * Vec3::NEG_Z).normalize(),
            (self.stance.view_rot * Vec3::X).normalize(),
        )
    }

    pub fn striking_on(&self, side: f32) -> bool { self.motion.rhythm.hand_cleave(side) > 0.0 }
    pub fn feminine(&self) -> bool { self.motion.physique.feminine() }
    pub fn scale(&self, measure: f32) -> f32 { self.motion.physique.scale(measure) }
    pub fn crown_size(&self, measure: f32) -> f32 { self.motion.physique.crown_size(measure) }
    pub fn crown_girth(&self, measure: f32) -> f32 { self.motion.physique.crown_girth(measure) }
    pub fn crown_span(&self, measure: f32) -> f32 { self.motion.physique.crown_span(measure) }
    pub fn torso_size(&self, measure: f32) -> f32 { self.motion.physique.torso_size(measure) }
    pub fn torso_depth(&self, measure: f32) -> f32 { self.motion.physique.torso_depth(measure) }
    pub fn arm_size(&self, measure: f32) -> f32 { self.motion.physique.arm_size(measure) }
    pub fn arm_girth(&self, measure: f32) -> f32 { self.motion.physique.arm_girth(measure) }
    pub fn leg_size(&self, measure: f32) -> f32 { self.motion.physique.leg_size(measure) }
    pub fn leg_girth(&self, measure: f32) -> f32 { self.motion.physique.leg_girth(measure) }
    pub fn hip_elevation(&self) -> f32 {
        let seated = self.seated_factor();
        let drop = self.crouch_compression() * 0.78 * (1.0 - seated) + seated * 0.34;
        (0.80 - drop) + (self.leg_size(0.68) - 0.68)
    }
    pub fn torso_span(&self, offset_above_hip: f32) -> f32 { self.hip_elevation() + self.torso_size(offset_above_hip) }
    pub fn foot_origin(&self) -> Vec3 { self.stance.position - (self.stance.surface_rot * Vec3::Y) * 0.98 }

    pub fn eye_origin(&self) -> Vec3 {
        let (up, view_fwd, right) = self.view_axes();
        let cranium = self.stance.position + up * self.stance.eye_height.clamp(0.36 * (1.0 - self.hover_factor()), 0.80);
        SurfaceSightRay::new(cranium, view_fwd).brow_origin(up, up.cross(right))
    }

    pub fn view_frame(&self) -> LimbFrame {
        LimbFrame::new(
            self.eye_origin(),
            (self.stance.view_rot * Vec3::X).normalize(),
            (self.stance.view_rot * Vec3::Y).normalize(),
            (self.stance.view_rot * Vec3::NEG_Z).normalize(),
        )
    }

    pub fn stance_frame(&self) -> LimbFrame {
        let up = (self.stance.surface_rot * Vec3::Y).normalize();
        let fallback = self.stance.surface_rot * Vec3::NEG_Z;
        let seated = self.seated_factor();
        let raw_fwd = if seated > 0.5 { fallback } else { self.stance.view_rot * Vec3::NEG_Z };
        let flat_fwd = raw_fwd - up * raw_fwd.dot(up);
        let forward = if flat_fwd.length_squared() > 1e-4 {
            flat_fwd.normalize()
        } else {
            (fallback - up * fallback.dot(up)).normalize()
        };
        let (rhythm, hover) = (self.motion.rhythm, self.hover_factor());
        let origin = self.foot_origin() - up * (rhythm.stance_bob(hover) * (1.0 - seated));
        LimbFrame::new(origin, forward.cross(up).normalize(), up, forward)
            .yawed(rhythm.stance_twist(hover) * (1.0 - seated))
            .shifted(0.0, 0.98, -0.36 * hover)
            .pitched(hover * 1.42)
            .shifted(0.0, -0.98 - 0.42 * hover, 0.0)
    }

    pub fn torso_frame(&self) -> LimbFrame {
        let (hip_y, crouch, seated) = (self.hip_elevation(), self.crouch_compression(), self.seated_factor());
        self.stance_frame()
            .shifted(0.0, hip_y, -0.030 - crouch * 0.22 * (1.0 - seated) - seated * 0.08)
            .pitched(crouch * 0.85 * (1.0 - seated) - seated * 0.12)
            .shifted(0.0, -hip_y, 0.030)
    }

    pub fn neck_frame(&self) -> LimbFrame {
        self.torso_frame().shifted(0.0, self.torso_span(0.88), -0.056)
    }

    pub fn crown_frame(&self) -> LimbFrame {
        let view_fwd = (self.stance.view_rot * Vec3::NEG_Z).normalize();
        let up = (self.stance.surface_rot * Vec3::Y).normalize();
        let seated = self.seated_factor();
        let neck = self.neck_frame();
        let neck_origin = neck.point(0.0, 0.0, 0.0);
        let torso_fwd = (neck.point(0.0, 0.0, 1.0) - neck_origin).normalize_or_zero();
        let torso_right = (neck.point(1.0, 0.0, 0.0) - neck_origin).normalize_or_zero();
        let flat_view = (view_fwd - up * view_fwd.dot(up)).normalize_or_zero();
        let head_yaw = if flat_view.length_squared() > 1e-4 {
            flat_view.dot(torso_right).atan2(flat_view.dot(torso_fwd)).clamp(-1.42, 1.42)
        } else {
            0.0
        };
        let pitch = -view_fwd.dot(up).clamp(-0.8, 0.8).asin() * 0.65 - self.crouch_compression() * 0.55 * (1.0 - seated) + seated * 0.10 - self.hover_factor() * 1.12;
        neck.yawed(-head_yaw).pitched(pitch)
    }

    pub fn leg_joints(&self, side: f32) -> (Vec3, Vec3, Vec3, Vec3) {
        let (rhythm, hover, seated) = (self.motion.rhythm, self.hover_factor(), self.seated_factor());
        let crouch = self.crouch_compression() * (1.0 - seated);
        let ext = (self.stance.eye_height + self.motion.leap - 0.80).clamp(0.0, 0.40) * (1.0 - seated);
        let ground = (1.0 - ext * 2.2).clamp(0.0, 1.0) * (1.0 - hover) * (1.0 - seated);
        let offset = if side > 0.0 { std::f32::consts::PI } else { 0.0 };
        let (phase, walk, run) = rhythm.limb_gait(offset, ground);
        let (s, pass) = (phase.sin(), (phase + 0.25).cos().max(0.0));
        let hip = Vec3::new(side * self.scale(0.088 + 0.008 * seated), self.hip_elevation() - 0.02, -0.030 - crouch * 0.22 - seated * 0.08);
        let stride_swing = if s >= 0.0 {
            s * (0.32 * walk + 0.20 * run) + pass * (0.05 * walk + 0.12 * run)
        } else {
            s * (0.26 * walk + 0.16 * run)
        };
        let theta_hip = 0.04 + crouch * 2.35 + seated * 1.38 - ext * 0.95 + stride_swing + s * 0.24 * hover;
        let theta_knee = (0.08 + crouch * 4.25 + seated * 1.42 - ext * 0.25 + (0.08 * walk + 0.16 * run) + pass * (0.44 * walk + 0.50 * run) + (0.26 + 0.20 * pass) * hover).max(0.0);
        let (theta_shin, thigh_len, shin_len) = (theta_hip - theta_knee, self.leg_size(0.34), self.leg_size(0.34));
        let knee_out = side * (self.scale(0.008) + pass * (0.016 * walk + 0.012 * run) + crouch * 0.042 + seated * 0.018 + hover * 0.028);
        let knee = Vec3::new(hip.x + knee_out, hip.y - thigh_len * theta_hip.cos(), hip.z + thigh_len * theta_hip.sin());
        let ankle_in = -side * (self.scale(0.005) + pass * (0.012 * walk + 0.010 * run));
        let ankle = Vec3::new(knee.x + ankle_in, (knee.y - shin_len * theta_shin.cos()).max(0.10), knee.z + shin_len * theta_shin.sin());
        let theta_foot = (theta_shin + 1.38 - ext * 0.85 - hover * 0.45).clamp(0.65, 1.54);
        let foot_len = self.leg_size(0.16);
        let toe = Vec3::new(ankle.x + side * 0.006, (ankle.y - foot_len * theta_foot.cos()).max(0.02), ankle.z + foot_len * theta_foot.sin());
        (hip, knee, ankle, toe)
    }

    pub fn crouch_compression(&self) -> f32 {
        (0.8 - (self.stance.eye_height + self.motion.leap)).clamp(0.0, 0.55) * (1.0 - self.hover_factor())
    }

    pub const fn arm<'a>(&'a self, side: f32, martial: MartialStyle) -> ArmKinematics<'a> {
        ArmKinematics::new(self, side, martial)
    }
}
