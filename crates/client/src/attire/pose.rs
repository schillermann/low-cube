use glam::{Quat, Vec3};
use world::SurfaceSightRay;
use super::limb::LimbFrame;
use super::proportion::YouthPhysique;
use super::rhythm::{MartialStyle, StrideRhythm};

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

    pub fn with_physique(self, physique: YouthPhysique) -> Self {
        Self { motion: PoseMotion { physique, ..self.motion }, ..self }
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
    pub fn hip_elevation(&self) -> f32 { (0.80 - self.crouch_compression() * 0.78) + (self.leg_size(0.68) - 0.68) }
    pub fn torso_span(&self, offset_above_hip: f32) -> f32 { self.hip_elevation() + self.torso_size(offset_above_hip) }
    pub fn foot_origin(&self) -> Vec3 { self.stance.position - (self.stance.surface_rot * Vec3::Y) * 0.98 }

    pub fn eye_origin(&self) -> Vec3 {
        let up = (self.stance.surface_rot * Vec3::Y).normalize();
        let view_fwd = (self.stance.view_rot * Vec3::NEG_Z).normalize();
        let right = (self.stance.view_rot * Vec3::X).normalize();
        let cranium = self.stance.position + up * self.stance.eye_height.clamp(0.36 * (1.0 - self.motion.hover), 0.80);
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
        let raw_fwd = self.stance.view_rot * Vec3::NEG_Z;
        let fallback = self.stance.surface_rot * Vec3::NEG_Z;
        let flat_fwd = raw_fwd - up * raw_fwd.dot(up);
        let forward = if flat_fwd.length_squared() > 1e-4 {
            flat_fwd.normalize()
        } else {
            (fallback - up * fallback.dot(up)).normalize()
        };
        let (rhythm, hover) = (self.motion.rhythm, self.motion.hover);
        let origin = self.foot_origin() - up * rhythm.stance_bob(hover);
        LimbFrame::new(origin, forward.cross(up).normalize(), up, forward)
            .yawed(rhythm.stance_twist(hover))
            .shifted(0.0, 0.98, -0.36 * hover)
            .pitched(hover * 1.42)
            .shifted(0.0, -0.98 - 0.42 * hover, 0.0)
    }

    pub fn torso_frame(&self) -> LimbFrame {
        let (hip_y, crouch) = (self.hip_elevation(), self.crouch_compression());
        self.stance_frame()
            .shifted(0.0, hip_y, -0.030 - crouch * 0.22)
            .pitched(crouch * 0.85)
            .shifted(0.0, -hip_y, 0.030)
    }

    pub fn neck_frame(&self) -> LimbFrame {
        self.torso_frame().shifted(0.0, self.torso_span(0.88), -0.056)
    }

    pub fn crown_frame(&self) -> LimbFrame {
        let view_fwd = (self.stance.view_rot * Vec3::NEG_Z).normalize();
        let up = (self.stance.surface_rot * Vec3::Y).normalize();
        let pitch = -view_fwd.dot(up).clamp(-0.8, 0.8).asin() * 0.65 - self.crouch_compression() * 0.55 - self.motion.hover * 1.12;
        self.neck_frame().pitched(pitch)
    }

    pub fn leg_joints(&self, side: f32) -> (Vec3, Vec3, Vec3, Vec3) {
        let (rhythm, hover) = (self.motion.rhythm, self.motion.hover);
        let crouch = self.crouch_compression();
        let ext = (self.stance.eye_height + self.motion.leap - 0.80).clamp(0.0, 0.40);
        let ground = (1.0 - ext * 2.2).clamp(0.0, 1.0) * (1.0 - hover);
        let offset = if side > 0.0 { std::f32::consts::PI } else { 0.0 };
        let (phase, walk, run) = rhythm.limb_gait(offset, ground);
        let (s, pass) = (phase.sin(), (phase + 0.25).cos().max(0.0));
        let hip = Vec3::new(side * self.scale(0.088), self.hip_elevation() - 0.02, -0.030 - crouch * 0.22);
        let stride_swing = if s >= 0.0 {
            s * (0.32 * walk + 0.20 * run) + pass * (0.05 * walk + 0.12 * run)
        } else {
            s * (0.26 * walk + 0.16 * run)
        };
        let theta_hip = 0.04 + crouch * 2.35 - ext * 0.95 + stride_swing + s * 0.24 * hover;
        let theta_knee = (0.08 + crouch * 4.25 - ext * 0.25 + (0.08 * walk + 0.16 * run) + pass * (0.44 * walk + 0.50 * run) + (0.26 + 0.20 * pass) * hover).max(0.0);
        let (theta_shin, thigh_len, shin_len) = (theta_hip - theta_knee, self.leg_size(0.34), self.leg_size(0.34));
        let knee_out = side * (self.scale(0.008) + pass * (0.016 * walk + 0.012 * run) + crouch * 0.042 + hover * 0.028);
        let knee = Vec3::new(hip.x + knee_out, hip.y - thigh_len * theta_hip.cos(), hip.z + thigh_len * theta_hip.sin());
        let ankle_in = -side * (self.scale(0.005) + pass * (0.012 * walk + 0.010 * run));
        let ankle = Vec3::new(knee.x + ankle_in, (knee.y - shin_len * theta_shin.cos()).max(0.10), knee.z + shin_len * theta_shin.sin());
        let theta_foot = (theta_shin + 1.38 - ext * 0.85 - hover * 0.45).clamp(0.65, 1.54);
        let foot_len = self.leg_size(0.16);
        let toe = Vec3::new(ankle.x + side * 0.006, (ankle.y - foot_len * theta_foot.cos()).max(0.02), ankle.z + foot_len * theta_foot.sin());
        (hip, knee, ankle, toe)
    }

    pub fn external_arm(&self, side: f32, holding: bool, martial: MartialStyle) -> (Vec3, Vec3, LimbFrame, f32) {
        let (rhythm, hover) = (self.motion.rhythm, self.motion.hover);
        let (torso, crouch) = (self.torso_frame(), self.crouch_compression());
        let ext = (self.stance.eye_height + self.motion.leap - 0.80).clamp(0.0, 0.40);
        let shoulder = Vec3::new(side * self.scale(if self.feminine() { 0.194 } else { 0.222 }), self.torso_span(0.64), -0.066);
        let ground = (1.0 - ext * 2.0).clamp(0.0, 1.0) * (1.0 - hover);
        let offset = if side > 0.0 { 0.0 } else { std::f32::consts::PI };
        let (phase, walk, run) = rhythm.limb_gait(offset, ground);
        let (s, breath) = (phase.sin(), rhythm.breath_wave(walk));
        let hold_factor = if holding || self.striking_on(side) { (1.0 - run).max(0.35) } else { 0.0 };
        let style = martial.active_for(self.striking_on(side));
        let (wind, strike, bow, surge) = style.cleave_impulse(rhythm, side);
        let (abduct_add, swing_add, elbow_delta, elbow_bend_add, wrist_dx, wrist_dz) =
            style.external_drive(wind, strike, bow, surge, crouch);
        let (upper_len, fore_len) = (self.arm_size(0.26), self.arm_size(0.24));
        let theta_abduct = 0.06 + 0.04 * walk + (0.28 + 0.04 * s.abs()) * run + (0.42 + 0.08 * s.abs()) * hover + 0.08 * hold_factor + (crouch + ext) * 0.10 + abduct_add;
        let r_sag = upper_len * theta_abduct.cos();
        let upper_swing = (ext * 0.38 + crouch * 1.15 + (0.56 + 0.18 * s) * hover) * (1.0 - 0.55 * hold_factor) + swing_add;
        let theta_upper = -0.04 * (1.0 - run) + 0.22 * hold_factor + upper_swing + s * (0.26 * walk * (1.0 - 0.55 * hold_factor) + 0.16 * run);
        let elbow = Vec3::new(
            shoulder.x + side * (upper_len * theta_abduct.sin() + elbow_delta.x),
            shoulder.y - r_sag * theta_upper.cos() + elbow_delta.y,
            shoulder.z + r_sag * theta_upper.sin() + elbow_delta.z,
        );
        let theta_elbow = 0.20 + 0.14 * walk + 0.88 * run + 0.35 * hover + 0.92 * hold_factor + (crouch * 1.35 + ext * 0.22) * (1.0 - 0.5 * hold_factor) + elbow_bend_add + s.max(0.0) * 0.16 * walk + s * 0.12 * run + breath * 0.03;
        let theta_forearm = theta_upper + theta_elbow;
        let wrist = Vec3::new(
            elbow.x + side * (wrist_dx - (0.010 + 0.014 * walk + 0.072 * run + 0.16 * crouch + 0.035 * hold_factor)),
            elbow.y - fore_len * theta_forearm.cos(),
            elbow.z + fore_len * theta_forearm.sin() + wrist_dz,
        );
        let (shoulder_w, elbow_w, wrist_w) = (torso.point(shoulder.x, shoulder.y, shoulder.z), torso.point(elbow.x, elbow.y, elbow.z), torso.point(wrist.x, wrist.y, wrist.z));
        let (forearm_len, forearm_dir) = ((wrist_w - elbow_w).length(), (wrist_w - elbow_w).normalize());
        let (rx, ry, wrist_yaw, wrist_snap) = style.external_wrist_orient(side, hold_factor, crouch, wind, strike, bow, surge);
        let hand_up = (torso.point(rx, ry, 0.0) - torso.point(0.0, 0.0, 0.0)).normalize().cross(forearm_dir).normalize();
        let hand_frame = LimbFrame::new(wrist_w, forearm_dir.cross(hand_up).normalize(), hand_up, forearm_dir)
            .yawed(wrist_yaw)
            .pitched(wrist_snap);
        (shoulder_w, elbow_w, hand_frame, forearm_len)
    }

    pub fn first_person_arm(&self, side: f32, martial: MartialStyle) -> (Vec3, Vec3, LimbFrame, f32) {
        let rhythm = self.motion.rhythm;
        let (up, view_fwd, right) = (
            (self.stance.surface_rot * Vec3::Y).normalize(),
            (self.stance.view_rot * Vec3::NEG_Z).normalize(),
            (self.stance.view_rot * Vec3::X).normalize(),
        );
        let style = martial.active_for(self.striking_on(side));
        let (wind, strike, bow, surge) = style.cleave_impulse(rhythm, side);
        let look_pitch = view_fwd.dot(up).clamp(-0.99, 0.99).asin();
        let pitch_follow = (if look_pitch >= 0.0 { 0.85 } else { 0.62 }) * (1.0 - strike) + strike;
        let eye_frame = LimbFrame::new(self.eye_origin(), right, up, up.cross(right).normalize()).pitched(-look_pitch * pitch_follow);
        let (down, calm) = ((-view_fwd.dot(up)).clamp(0.0, 1.0) * (1.0 - strike), 1.0 - strike);
        let offset = if side > 0.0 { 0.0 } else { std::f32::consts::PI };
        let (phase, walk, run) = rhythm.limb_gait(offset, calm);
        let (primary, lagged) = (phase.sin(), (phase - 0.48).sin());
        let (sw_x, sw_y, sw_z, sw_yaw, elbow_add, aim_dip) = style.first_person_swing(wind, strike, bow, surge);
        let spread_x = side * (self.scale(0.195) + down * 0.05 + sw_x - (0.006 * walk + 0.010 * run + primary * (0.010 * walk + 0.014 * run)));
        let arc_y = 0.020 * run + sw_y + self.motion.leap * 0.025 * calm + primary * (0.018 * walk + 0.024 * run) + rhythm.breath_wave(walk) * 0.004 * calm;
        let arc_z = -0.035 * run + sw_z + (primary * 0.022 + lagged * 0.008) * (0.55 * walk + 0.45 * run);
        let raw_frame = eye_frame.shifted(spread_x, -0.220 + arc_y, 0.30 + arc_z).yawed(side * (0.13 + 0.04 * walk + 0.10 * run + sw_yaw));
        let cam_y = (self.eye_origin() - self.foot_origin()).dot(up);
        let shoulder_w = self.torso_frame().point(side * self.scale(0.222), cam_y - 0.34 + (self.torso_size(0.64) - 0.64), -0.066);
        let elbow_local = Vec3::new(
            side * (0.028 + 0.038 * run + elbow_add.x),
            -0.155 - 0.010 * run - elbow_add.y,
            -0.155 + 0.010 * run + 0.04 * wind + 0.02 * strike,
        );
        let (wrist_w, elbow_w) = (raw_frame.point(0.0, 0.0, 0.0), raw_frame.point(elbow_local.x, elbow_local.y, elbow_local.z));
        let (forearm_len, hand_dir) = ((wrist_w - elbow_w).length(), (wrist_w - raw_frame.point(side * 0.015, 0.035 - aim_dip, -0.22)).normalize());
        let (rx, ry, wrist_yaw, wrist_snap) = style.first_person_wrist_orient(side, wind, strike, bow);
        let hand_up = (raw_frame.point(rx, ry, 0.0) - wrist_w).cross(hand_dir).normalize();
        let hand_frame = LimbFrame::new(wrist_w, hand_dir.cross(hand_up).normalize(), hand_up, hand_dir)
            .yawed(wrist_yaw)
            .pitched(wrist_snap);
        (shoulder_w, elbow_w, hand_frame, forearm_len)
    }

    pub fn finger_curl(&self, side: f32, holding_relic: bool, martial: MartialStyle) -> f32 {
        if matches!(martial, MartialStyle::PalmCradle) {
            return 0.34;
        }
        if holding_relic {
            return 0.88;
        }
        let rhythm = self.motion.rhythm;
        let (arc, surge) = if self.striking_on(side) { (rhythm.cleave_arc(side), rhythm.cleave_surge(side)) } else { (0.0, 0.0) };
        let offset = if side > 0.0 { 0.0 } else { std::f32::consts::PI };
        let (phase, walk, run) = rhythm.limb_gait(offset, 1.0);
        (0.18 + 0.32 * (1.0 - arc * arc) * surge + 0.76 * arc.max(0.0) + 0.07 * walk + 0.70 * run + phase.sin().max(0.0) * 0.06 * run).clamp(0.18, 0.96)
    }

    pub fn crouch_compression(&self) -> f32 {
        (0.8 - (self.stance.eye_height + self.motion.leap)).clamp(0.0, 0.55) * (1.0 - self.motion.hover)
    }
}
