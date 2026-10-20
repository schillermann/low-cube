use glam::{Quat, Vec3};
use super::limb::LimbFrame;
use super::proportion::YouthPhysique;
use super::relic::MartialStyle;

/// Immutable gait rhythm tracking stride phase and walking-to-sprinting blend factor.
#[derive(Clone, Copy, Debug)]
pub struct StrideRhythm {
    phase: f32,
    blend: f32,
    cleave: f32,
}

impl StrideRhythm {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn with_cleave(phase: f32, blend: f32, cleave: f32) -> Self { Self { phase, blend, cleave } }
    pub fn new(phase: f32, blend: f32) -> Self { Self::with_cleave(phase, blend, 0.0) }
    pub fn idle() -> Self { Self::new(0.0, 0.0) }
    pub fn walking() -> Self { Self::new(0.0, 0.45) }
    pub fn running() -> Self { Self::new(0.0, 1.0) }
    pub fn trigger_strike(self) -> Self { Self::with_cleave(self.phase, self.blend, 0.001) }

    /// Smooth C1 swing curve: -1.0 at full wind-up (c=0.34), +1.0 at full cleave/slash follow-through (c=0.68).
    pub fn cleave_arc(self) -> f32 {
        if self.cleave <= 0.0 { 0.0 } else if self.cleave < 0.34 { -0.5 * (1.0 - (self.cleave / 0.34 * std::f32::consts::PI).cos()) }
        else if self.cleave < 0.68 { -((self.cleave - 0.34) / 0.34 * std::f32::consts::PI).cos() }
        else { 0.5 * (1.0 + ((self.cleave - 0.68) / 0.32 * std::f32::consts::PI).cos()) }
    }

    pub fn cleave_surge(self) -> f32 { if self.cleave > 0.0 { (self.cleave * std::f32::consts::PI).sin() } else { 0.0 } }

    /// Advances the stride phase and any active strike cycle at a constant angular pace.
    pub fn step_cycle(self, angular_rate: f32, dt: f32) -> Self {
        let next_cleave = if self.cleave > 0.0 && self.cleave + dt * 1.65 < 1.0 { self.cleave + dt * 1.65 } else { 0.0 };
        Self::with_cleave((self.phase + angular_rate * dt) % std::f32::consts::TAU, self.blend, next_cleave)
    }

    /// Returns a new StrideRhythm advanced by planar surface displacement over dt.
    pub fn advance(self, previous: Vec3, next: Vec3, terrain_up: Vec3, dt: f32) -> Self {
        let delta = next - previous;
        let tangent_dist = (delta - terrain_up * delta.dot(terrain_up)).length();
        let speed = if dt > 1e-5 { tangent_dist / dt } else { 0.0 };
        let target_blend = if speed > 6.3 { 0.45 + ((speed - 6.3) / 3.8).clamp(0.0, 0.55) } else { (speed / 6.0).clamp(0.0, 1.0) * 0.45 };
        let pace = if tangent_dist > 1e-4 { tangent_dist * 2.1 } else { dt * 1.5 };
        let next_phase = (self.phase + pace) % std::f32::consts::TAU;
        let next_blend = self.blend + (target_blend - self.blend) * (1.0 - (-7.0 * dt).exp());
        let next_cleave = if self.cleave > 0.0 && self.cleave + dt * 1.65 < 1.0 { self.cleave + dt * 1.65 } else { 0.0 };
        Self::with_cleave(next_phase, next_blend, next_cleave)
    }
}

/// Kinematic pose of an avatar body on a planetary surface for first-person and multiplayer display.
#[derive(Clone, Copy, Debug)]
pub struct BodyPose {
    position: Vec3,
    surface_rot: Quat,
    view_rot: Quat,
    eye_height: f32,
    rhythm: StrideRhythm,
    physique: YouthPhysique,
}

impl BodyPose {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(position: Vec3, surface_rot: Quat, view_rot: Quat, eye_height: f32, rhythm: StrideRhythm, physique: YouthPhysique) -> Self {
        Self { position, surface_rot, view_rot, eye_height, rhythm, physique }
    }

    pub fn with_physique(self, physique: YouthPhysique) -> Self {
        Self::new(self.position, self.surface_rot, self.view_rot, self.eye_height, self.rhythm, physique)
    }

    pub fn scale(&self, measure: f32) -> f32 { self.physique.scale(measure) }
    pub fn crown_size(&self, measure: f32) -> f32 { self.physique.crown_size(measure) }
    pub fn crown_girth(&self, measure: f32) -> f32 { self.physique.crown_girth(measure) }
    pub fn crown_span(&self, measure: f32) -> f32 { self.physique.crown_span(measure) }
    pub fn torso_size(&self, measure: f32) -> f32 { self.physique.torso_size(measure) }
    pub fn torso_depth(&self, measure: f32) -> f32 { self.physique.torso_depth(measure) }
    pub fn arm_size(&self, measure: f32) -> f32 { self.physique.arm_size(measure) }
    pub fn arm_girth(&self, measure: f32) -> f32 { self.physique.arm_girth(measure) }
    pub fn leg_size(&self, measure: f32) -> f32 { self.physique.leg_size(measure) }
    pub fn leg_girth(&self, measure: f32) -> f32 { self.physique.leg_girth(measure) }

    pub fn hip_elevation(&self) -> f32 { (0.80 - self.crouch_compression() * 0.38) + (self.leg_size(0.68) - 0.68) }
    pub fn torso_span(&self, offset_above_hip: f32) -> f32 { self.hip_elevation() + self.torso_size(offset_above_hip) }
    pub fn foot_origin(&self) -> Vec3 { self.position - (self.surface_rot * Vec3::Y) * 0.98 }

    /// Eye position in world space matching CameraHorizon (pulled forward in front of the torso when looking down).
    pub fn eye_origin(&self) -> Vec3 {
        let (up, view_fwd, right) = ((self.surface_rot * Vec3::Y).normalize(), (self.view_rot * Vec3::NEG_Z).normalize(), (self.view_rot * Vec3::X).normalize());
        let down = (-view_fwd.dot(up)).clamp(0.0, 1.0);
        self.position + up * self.eye_height + up.cross(right).normalize() * (0.09 + down * 0.07)
    }

    /// Horizontal heading frame on the planetary surface for torso, belt, legs, and boots.
    pub fn torso_frame(&self) -> LimbFrame {
        let (up, raw_fwd, fallback) = ((self.surface_rot * Vec3::Y).normalize(), self.view_rot * Vec3::NEG_Z, self.surface_rot * Vec3::NEG_Z);
        let flat_fwd = raw_fwd - up * raw_fwd.dot(up);
        let forward = if flat_fwd.length_squared() > 1e-4 { flat_fwd.normalize() } else { (fallback - up * fallback.dot(up)).normalize() };
        let origin = self.foot_origin() - up * ((2.0 * self.rhythm.phase).cos() * 0.012 * self.rhythm.blend);
        let twist = self.rhythm.phase.sin() * 0.045 * self.rhythm.blend - 0.22 * (-self.rhythm.cleave_arc()).max(0.0);
        LimbFrame::new(origin, forward.cross(up).normalize(), up, forward).yawed(twist)
    }

    /// Head and neck frame aligned with horizontal torso and vertical look pitch for multiplayer/mirror view.
    pub fn crown_frame(&self) -> LimbFrame {
        let base = self.torso_frame();
        let neck_y = self.torso_span(0.36 + self.eye_height * 0.65 + self.crouch_compression() * 0.38);
        let neck_frame = base.shifted(0.0, neck_y, -0.056);
        let view_fwd = (self.view_rot * Vec3::NEG_Z).normalize();
        let up = (self.surface_rot * Vec3::Y).normalize();
        let pitch = view_fwd.dot(up).clamp(-0.8, 0.8).asin();
        neck_frame.pitched(-pitch * 0.65)
    }

    /// Contralateral 3D leg joint chain (hip, knee, ankle, toe) emerging cleanly below the tunic skirt hem.
    pub fn leg_joints(&self, side: f32) -> (Vec3, Vec3, Vec3, Vec3) {
        let phase = if side > 0.0 { self.rhythm.phase + std::f32::consts::PI } else { self.rhythm.phase };
        let (s, pass, crouch) = (phase.sin(), (phase + 0.25).cos().max(0.0), self.crouch_compression());
        let (walk, run) = ((self.rhythm.blend / 0.45).clamp(0.0, 1.0), ((self.rhythm.blend - 0.48) / 0.52).clamp(0.0, 1.0));
        let hip = Vec3::new(side * self.scale(0.088), self.hip_elevation() - 0.02, -0.030);
        let fwd_swing = if s >= 0.0 { s * (0.32 * walk + 0.20 * run) + pass * (0.05 * walk + 0.12 * run) } else { s * (0.26 * walk + 0.16 * run) };
        let theta_hip = 0.04 + crouch * 0.52 + fwd_swing;
        let theta_knee = 0.08 + crouch * 0.90 + (0.08 * walk + 0.16 * run) + pass * (0.44 * walk + 0.50 * run);
        let (theta_shin, thigh_len, shin_len) = (theta_hip - theta_knee, self.leg_size(0.34), self.leg_size(0.34));
        let knee_out = side * (self.scale(0.008) + pass * (0.016 * walk + 0.012 * run) + crouch * 0.018);
        let knee = Vec3::new(hip.x + knee_out, hip.y - thigh_len * theta_hip.cos(), hip.z + thigh_len * theta_hip.sin());
        let ankle_in = -side * (self.scale(0.005) + pass * (0.012 * walk + 0.010 * run));
        let ankle = Vec3::new(knee.x + ankle_in, (knee.y - shin_len * theta_shin.cos()).max(0.10), knee.z + shin_len * theta_shin.sin());
        let theta_foot = (theta_shin + 1.38).clamp(1.10, 1.54);
        let foot_len = self.leg_size(0.16);
        let toe = Vec3::new(ankle.x + side * 0.006, (ankle.y - foot_len * theta_foot.cos()).max(0.04), ankle.z + foot_len * theta_foot.sin());
        (hip, knee, ankle, toe)
    }

    /// True 3D anatomical arm joint chain with forward upper-arm drive and distinct axe (lateral felling arc) vs. sword martial kinematics.
    pub fn external_arm(&self, side: f32, holding: bool, martial: MartialStyle) -> (Vec3, Vec3, LimbFrame, f32) {
        let (torso, crouch) = (self.torso_frame(), self.crouch_compression());
        let shoulder = Vec3::new(side * self.scale(0.222), self.torso_span(0.64 - crouch * 0.38), -0.066);
        let phase = if side > 0.0 { self.rhythm.phase } else { self.rhythm.phase + std::f32::consts::PI };
        let (s, walk, run) = (phase.sin(), (self.rhythm.blend / 0.45).clamp(0.0, 1.0), ((self.rhythm.blend - 0.48) / 0.52).clamp(0.0, 1.0));
        let (breath, hold_factor) = (self.rhythm.phase.sin() * (1.0 - walk), if holding { (1.0 - run).max(0.35) } else { 0.0 });
        let (arc, surge) = if !matches!(martial, MartialStyle::Unarmed) { (self.rhythm.cleave_arc(), self.rhythm.cleave_surge()) } else { (0.0, 0.0) };
        let (wind, strike, sword) = ((-arc).max(0.0), arc.max(0.0), matches!(martial, MartialStyle::SwordplaySlash));
        let theta_abduct = 0.06 + 0.04 * walk + (0.28 + 0.04 * s.abs()) * run + 0.08 * hold_factor + if sword { 0.35 * wind + 0.42 * strike } else { 0.78 * wind };
        let (upper_len, fore_len) = (self.arm_size(0.26), self.arm_size(0.24));
        let r_sag = upper_len * theta_abduct.cos();
        let upper_swing = if sword { 0.55 * wind + 0.75 * strike + 0.35 * surge } else { -0.25 * wind + 0.52 * strike + 0.25 * surge };
        let theta_upper = -0.04 * (1.0 - run) + 0.22 * hold_factor + upper_swing + s * (0.26 * walk * (1.0 - 0.55 * hold_factor) + 0.16 * run);
        let elbow_x = shoulder.x + side * (upper_len * theta_abduct.sin() + if sword { -0.12 * wind + 0.08 * strike } else { 0.05 * wind - 0.06 * strike });
        let elbow = Vec3::new(elbow_x, shoulder.y - r_sag * theta_upper.cos() + 0.06 * wind, shoulder.z + r_sag * theta_upper.sin() + 0.12 * strike + 0.08 * surge);
        let theta_elbow = 0.20 + 0.14 * walk + 0.88 * run + 0.92 * hold_factor + (if sword { 0.35 * wind - 0.55 * strike } else { 0.35 * wind - 0.52 * strike - 0.22 * surge }) + s.max(0.0) * 0.16 * walk + s * 0.12 * run + breath * 0.03;
        let theta_forearm = theta_upper + theta_elbow;
        let wrist_dx = if sword { -0.22 * wind + 0.16 * strike } else { 0.14 * wind - 0.02 * strike };
        let wrist = Vec3::new(elbow.x + side * (wrist_dx - (0.010 + 0.014 * walk + 0.072 * run + 0.035 * hold_factor)), elbow.y - fore_len * theta_forearm.cos(), elbow.z + fore_len * theta_forearm.sin());
        let (shoulder_w, elbow_w, wrist_w) = (torso.point(shoulder.x, shoulder.y, shoulder.z), torso.point(elbow.x, elbow.y, elbow.z), torso.point(wrist.x, wrist.y, wrist.z));
        let (forearm_len, forearm_dir) = ((wrist_w - elbow_w).length(), (wrist_w - elbow_w).normalize());
        let roll_y = side * if sword { 0.55 * wind - 0.65 * strike } else { 0.12 * hold_factor + 0.14 * wind + 0.24 * strike + 0.10 * surge };
        let hand_up = (torso.point(1.0, roll_y, if sword { 0.65 * strike } else { 0.0 }) - torso.point(0.0, 0.0, 0.0)).normalize().cross(forearm_dir).normalize();
        (shoulder_w, elbow_w, LimbFrame::new(wrist_w, forearm_dir.cross(hand_up).normalize(), hand_up, forearm_dir), forearm_len)
    }

    /// View-aligned First-Person arm with pure lateral woodcutter swing (seitlicher Schwung) for axe and diagonal slash for sword.
    pub fn first_person_arm(&self, side: f32, martial: MartialStyle) -> (Vec3, Vec3, LimbFrame, f32) {
        let (up, view_fwd, right) = ((self.surface_rot * Vec3::Y).normalize(), (self.view_rot * Vec3::NEG_Z).normalize(), (self.view_rot * Vec3::X).normalize());
        let look_pitch = view_fwd.dot(up).clamp(-0.99, 0.99).asin();
        let eye_frame = LimbFrame::new(self.eye_origin(), right, up, up.cross(right).normalize()).pitched(-if look_pitch >= 0.0 { look_pitch * 0.85 } else { look_pitch * 0.62 });
        let (down, phase) = ((-view_fwd.dot(up)).clamp(0.0, 1.0), if side > 0.0 { self.rhythm.phase } else { self.rhythm.phase + std::f32::consts::PI });
        let (primary, lagged, walk, run) = (phase.sin(), (phase - 0.48).sin(), (self.rhythm.blend / 0.45).clamp(0.0, 1.0), ((self.rhythm.blend - 0.48) / 0.52).clamp(0.0, 1.0));
        let (arc, surge) = if !matches!(martial, MartialStyle::Unarmed) { (self.rhythm.cleave_arc(), self.rhythm.cleave_surge()) } else { (0.0, 0.0) };
        let (wind, strike, sword) = ((-arc).max(0.0), arc.max(0.0), matches!(martial, MartialStyle::SwordplaySlash));
        let (sw_x, sw_y, sw_z, sw_yaw) = if sword { (-0.22 * wind + 0.16 * strike, 0.19 * wind - 0.11 * strike, 0.05 * wind + 0.22 * strike + 0.10 * surge, 0.42 * wind - 0.36 * strike) }
            else { (0.18 * wind - 0.045 * strike, 0.06 * wind - 0.01 * strike, -0.04 * wind + 0.08 * strike + 0.05 * surge, -0.18 * wind + 0.10 * strike + 0.05 * surge) };
        let spread_x = side * (self.scale(0.195) + down * 0.05 + sw_x - (0.006 * walk + 0.010 * run + primary * (0.010 * walk + 0.014 * run)));
        let arc_y = 0.020 * run + sw_y + primary * (0.018 * walk + 0.024 * run) + self.rhythm.phase.sin() * (1.0 - walk) * 0.004;
        let arc_z = -0.035 * run + sw_z + (primary * 0.022 + lagged * 0.008) * (0.55 * walk + 0.45 * run);
        let raw_frame = eye_frame.shifted(spread_x, -0.220 + arc_y, 0.30 + arc_z).yawed(side * (0.13 + 0.04 * walk + 0.10 * run + sw_yaw));
        let shoulder_w = self.torso_frame().point(side * self.scale(0.222), self.torso_span(0.64 - self.crouch_compression() * 0.38), -0.066);
        let elbow_local = Vec3::new(side * (0.028 + 0.038 * run + 0.08 * wind + 0.02 * strike), -0.155 - 0.010 * run + 0.03 * wind + 0.02 * strike, -0.155 + 0.010 * run - 0.02 * wind + 0.02 * strike);
        let (wrist_w, elbow_w) = (raw_frame.point(0.0, 0.0, 0.0), raw_frame.point(elbow_local.x, elbow_local.y, elbow_local.z));
        let (forearm_len, hand_dir) = ((wrist_w - elbow_w).length(), (wrist_w - raw_frame.point(side * 0.015, 0.035, -0.22)).normalize());
        let roll_y = side * if sword { 0.65 * wind - 0.75 * strike } else { 0.10 + 0.12 * wind + 0.22 * strike + 0.10 * surge };
        let hand_up = (raw_frame.point(1.0, roll_y, if sword { 0.70 * strike } else { 0.0 }) - wrist_w).cross(hand_dir).normalize();
        (shoulder_w, elbow_w, LimbFrame::new(wrist_w, hand_dir.cross(hand_up).normalize(), hand_up, hand_dir), forearm_len)
    }

    pub fn finger_curl(&self, side: f32, holding_relic: bool) -> f32 {
        if holding_relic { return 0.88; }
        let (walk, run) = ((self.rhythm.blend / 0.45).clamp(0.0, 1.0), ((self.rhythm.blend - 0.48) / 0.52).clamp(0.0, 1.0));
        let phase = if side > 0.0 { self.rhythm.phase } else { self.rhythm.phase + std::f32::consts::PI };
        (0.18 + 0.07 * walk + 0.70 * run + phase.sin().max(0.0) * 0.06 * run).clamp(0.18, 0.96)
    }

    pub fn crouch_compression(&self) -> f32 { (0.8 - self.eye_height).clamp(0.0, 0.4) }
}
