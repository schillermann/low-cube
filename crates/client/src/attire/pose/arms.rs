use glam::Vec3;
use super::super::limb::LimbFrame;
use super::super::rhythm::MartialStyle;
use super::BodyPose;

/// Decorator over `BodyPose` evaluating unilateral arm, wrist, and finger kinematics for a martial style.
#[derive(Clone, Copy)]
pub struct ArmKinematics<'a> {
    pose: &'a BodyPose,
    side: f32,
    martial: MartialStyle,
}

impl<'a> ArmKinematics<'a> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(pose: &'a BodyPose, side: f32, martial: MartialStyle) -> Self {
        Self { pose, side, martial }
    }

    pub fn external(self, holding: bool) -> (Vec3, Vec3, LimbFrame, f32) {
        let (pose, side, martial) = (self.pose, self.side, self.martial);
        let (rhythm, hover, leap) = pose.motion_factors();
        let (seated, steer) = (pose.seated_factor(), pose.steering_turn());
        let (torso, crouch) = (pose.torso_frame(), pose.crouch_compression() * (1.0 - seated));
        let ext = (pose.eye_height() + leap - 0.80).clamp(0.0, 0.40) * (1.0 - seated);
        let shoulder = Vec3::new(side * pose.scale(if pose.feminine() { 0.194 } else { 0.222 }), pose.torso_span(0.64), -0.066);
        let ground = (1.0 - ext * 2.0).clamp(0.0, 1.0) * (1.0 - hover) * (1.0 - seated);
        let offset = if side > 0.0 { 0.0 } else { std::f32::consts::PI };
        let (phase, walk, run) = rhythm.limb_gait(offset, ground);
        let (s, breath) = (phase.sin() * (1.0 - seated), rhythm.breath_wave(walk) * (1.0 - seated));
        let hold_factor = if (holding || pose.striking_on(side)) && seated < 0.5 { (1.0 - run).max(0.35) } else { 0.0 };
        let style = martial.active_for(pose.striking_on(side) && seated < 0.5);
        let imp = style.cleave_impulse(rhythm, side);
        let (abduct_add, swing_add, elbow_delta, elbow_bend_add, wrist_dx, wrist_dz) =
            style.external_drive(imp, crouch);
        let (upper_len, fore_len) = (pose.arm_size(0.26), pose.arm_size(0.24));
        let theta_abduct = 0.06 + 0.04 * walk + (0.28 + 0.04 * s.abs()) * run + (0.42 + 0.08 * s.abs()) * hover + 0.08 * hold_factor + (crouch + ext) * 0.10 + seated * 0.08 + abduct_add;
        let r_sag = upper_len * theta_abduct.cos();
        let steer_wave = seated * (0.94 - side * steer * 0.26);
        let upper_swing = (ext * 0.38 + crouch * 1.15 + (0.56 + 0.18 * s) * hover) * (1.0 - 0.55 * hold_factor) + steer_wave + swing_add;
        let theta_upper = -0.04 * (1.0 - run) + 0.22 * hold_factor + upper_swing + s * (0.26 * walk * (1.0 - 0.55 * hold_factor) + 0.16 * run);
        let elbow = Vec3::new(
            shoulder.x + side * (upper_len * theta_abduct.sin() + elbow_delta.x),
            shoulder.y - r_sag * theta_upper.cos() + elbow_delta.y,
            shoulder.z + r_sag * theta_upper.sin() + elbow_delta.z,
        );
        let theta_elbow = 0.20 + 0.14 * walk + 0.88 * run + 0.35 * hover + 0.92 * hold_factor + seated * 0.58 + (crouch * 1.35 + ext * 0.22) * (1.0 - 0.5 * hold_factor) + elbow_bend_add + s.max(0.0) * 0.16 * walk + s * 0.12 * run + breath * 0.03;
        let theta_forearm = theta_upper + theta_elbow;
        let wheel_dx = seated * (steer * 0.048 - side * steer.abs() * 0.022);
        let wheel_dy = seated * (-side * steer * 0.065);
        let wrist = Vec3::new(
            elbow.x + side * (wrist_dx - (0.010 + 0.014 * walk + 0.072 * run + 0.16 * crouch + 0.105 * seated + 0.035 * hold_factor)) + wheel_dx,
            elbow.y - fore_len * theta_forearm.cos() + wheel_dy,
            elbow.z + fore_len * theta_forearm.sin() + wrist_dz + 0.065 * seated,
        );
        let (shoulder_w, elbow_w, wrist_w) = (torso.point(shoulder.x, shoulder.y, shoulder.z), torso.point(elbow.x, elbow.y, elbow.z), torso.point(wrist.x, wrist.y, wrist.z));
        let (forearm_len, forearm_dir) = ((wrist_w - elbow_w).length(), (wrist_w - elbow_w).normalize());
        let hand_frame = style.external_wrist_orient(side, hold_factor, crouch, imp).orient_hand(&torso, wrist_w, forearm_dir);
        (shoulder_w, elbow_w, hand_frame, forearm_len)
    }

    pub fn first_person(self) -> (Vec3, Vec3, LimbFrame, f32) {
        let (pose, side, martial) = (self.pose, self.side, self.martial);
        let (rhythm, _, leap) = pose.motion_factors();
        let (up, view_fwd, right) = pose.view_axes();
        let style = martial.active_for(pose.striking_on(side));
        let imp = style.cleave_impulse(rhythm, side);
        let strike = imp.strike();
        let look_pitch = view_fwd.dot(up).clamp(-0.99, 0.99).asin();
        let pitch_follow = (if look_pitch >= 0.0 { 0.85 } else { 0.62 }) * (1.0 - strike) + strike;
        let eye_frame = LimbFrame::new(pose.eye_origin(), right, up, up.cross(right).normalize()).pitched(-look_pitch * pitch_follow);
        let (down, calm) = ((-view_fwd.dot(up)).clamp(0.0, 1.0) * (1.0 - strike), 1.0 - strike);
        let offset = if side > 0.0 { 0.0 } else { std::f32::consts::PI };
        let (phase, walk, run) = rhythm.limb_gait(offset, calm);
        let (primary, lagged) = (phase.sin(), (phase - 0.48).sin());
        let (sw_x, sw_y, sw_z, sw_yaw, elbow_add, aim_dip) = style.first_person_swing(imp);
        let spread_x = side * (pose.scale(0.195) + down * 0.05 + sw_x - (0.006 * walk + 0.010 * run + primary * (0.010 * walk + 0.014 * run)));
        let arc_y = 0.020 * run + sw_y + leap * 0.025 * calm + primary * (0.018 * walk + 0.024 * run) + rhythm.breath_wave(walk) * 0.004 * calm;
        let arc_z = -0.035 * run + sw_z + (primary * 0.022 + lagged * 0.008) * (0.55 * walk + 0.45 * run);
        let raw_frame = eye_frame.shifted(spread_x, -0.220 + arc_y, 0.30 + arc_z).yawed(side * (0.13 + 0.04 * walk + 0.10 * run + sw_yaw));
        let cam_y = (pose.eye_origin() - pose.foot_origin()).dot(up);
        let shoulder_w = pose.torso_frame().point(side * pose.scale(0.222), cam_y - 0.34 + (pose.torso_size(0.64) - 0.64), -0.066);
        let elbow_local = Vec3::new(
            side * (0.028 + 0.038 * run + elbow_add.x),
            -0.155 - 0.010 * run - elbow_add.y,
            -0.155 + 0.010 * run + 0.04 * imp.wind() + 0.02 * strike,
        );
        let (wrist_w, elbow_w) = (raw_frame.point(0.0, 0.0, 0.0), raw_frame.point(elbow_local.x, elbow_local.y, elbow_local.z));
        let (forearm_len, hand_dir) = ((wrist_w - elbow_w).length(), (wrist_w - raw_frame.point(side * 0.015, 0.035 - aim_dip, -0.22)).normalize());
        let hand_frame = style.first_person_wrist_orient(side, imp).orient_hand(&raw_frame, wrist_w, hand_dir);
        (shoulder_w, elbow_w, hand_frame, forearm_len)
    }

    pub fn finger_curl(self, holding_relic: bool) -> f32 {
        let (pose, side, martial) = (self.pose, self.side, self.martial);
        if matches!(martial, MartialStyle::PalmCradle) {
            return 0.34;
        }
        if holding_relic || pose.seated_factor() > 0.5 {
            return 0.88;
        }
        let (rhythm, _, _) = pose.motion_factors();
        let (arc, surge) = if pose.striking_on(side) { (rhythm.cleave_arc(side), rhythm.cleave_surge(side)) } else { (0.0, 0.0) };
        let offset = if side > 0.0 { 0.0 } else { std::f32::consts::PI };
        let (phase, walk, run) = rhythm.limb_gait(offset, 1.0);
        (0.18 + 0.32 * (1.0 - arc * arc) * surge + 0.76 * arc.max(0.0) + 0.07 * walk + 0.70 * run + phase.sin().max(0.0) * 0.06 * run).clamp(0.18, 0.96)
    }
}
