use glam::{Quat, Vec3};
use super::limb::LimbFrame;
use super::palette::YouthPhysique;

/// Immutable gait rhythm tracking stride phase and walking-to-sprinting blend factor.
#[derive(Clone, Copy, Debug)]
pub struct StrideRhythm {
    phase: f32,
    blend: f32,
}

impl StrideRhythm {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(phase: f32, blend: f32) -> Self {
        Self { phase, blend }
    }

    /// Secondary constructor for a resting avatar gait.
    pub fn idle() -> Self {
        Self::new(0.0, 0.0)
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
        Self::new(next_phase, next_blend)
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

    /// Returns a new BodyPose decorated with the given YouthPhysique.
    pub fn with_physique(self, physique: YouthPhysique) -> Self {
        Self::new(self.position, self.surface_rot, self.view_rot, self.eye_height, self.rhythm, physique)
    }

    /// Scales a lateral anatomical measure by the pose's encapsulated YouthPhysique.
    pub fn scale(&self, measure: f32) -> f32 {
        self.physique.scale(measure)
    }

    /// Ground foot anchor in world space (surface constrained position is 1.0 above ground).
    pub fn foot_origin(&self) -> Vec3 {
        let up = self.surface_rot * Vec3::Y;
        self.position - up * 0.98
    }

    /// Eye position in world space matching CameraHorizon.
    pub fn eye_origin(&self) -> Vec3 {
        let up = self.surface_rot * Vec3::Y;
        self.position + up * self.eye_height
    }

    /// Horizontal heading frame on the planetary surface for torso, belt, legs, and boots.
    pub fn torso_frame(&self) -> LimbFrame {
        let up = (self.surface_rot * Vec3::Y).normalize();
        let raw_fwd = self.view_rot * Vec3::NEG_Z;
        let flat_fwd = raw_fwd - up * raw_fwd.dot(up);
        let fallback = self.surface_rot * Vec3::NEG_Z;
        let forward = if flat_fwd.length_squared() > 1e-4 { flat_fwd.normalize() } else { (fallback - up * fallback.dot(up)).normalize() };
        let right = forward.cross(up).normalize();
        let bob = -(2.0 * self.rhythm.phase).cos() * 0.012 * self.rhythm.blend;
        let origin = self.foot_origin() + up * bob;
        let twist = self.rhythm.phase.sin() * 0.045 * self.rhythm.blend;
        LimbFrame::new(origin, right, up, forward).yawed(twist)
    }

    /// View-aligned frame for the first-person hands and forearms (damped when looking down so arms stay on torso).
    pub fn hand_frame(&self) -> LimbFrame {
        let up = (self.surface_rot * Vec3::Y).normalize();
        let view_fwd = (self.view_rot * Vec3::NEG_Z).normalize();
        let right = (self.view_rot * Vec3::X).normalize();
        let flat_fwd = up.cross(right).normalize();
        let look_pitch = view_fwd.dot(up).clamp(-0.99, 0.99).asin();
        let arm_pitch = if look_pitch >= 0.0 { look_pitch * 0.85 } else { look_pitch * 0.62 };
        LimbFrame::new(self.eye_origin(), right, up, flat_fwd).pitched(-arm_pitch)
    }

    /// Exact 3D world-space shoulder socket high on the upper torso.
    pub fn shoulder_socket(&self, side: f32) -> Vec3 {
        let shoulder_y = 1.50 - self.crouch_compression() * 0.80;
        self.torso_frame().point(side * self.scale(0.125), shoulder_y, -0.078)
    }

    /// Downward gaze factor [0.0..1.0] used to part forearms slightly outward when looking down at the torso.
    pub fn downward_gaze(&self) -> f32 {
        let up = (self.surface_rot * Vec3::Y).normalize();
        let view_fwd = (self.view_rot * Vec3::NEG_Z).normalize();
        (-view_fwd.dot(up)).clamp(0.0, 1.0)
    }

    /// Head and neck frame aligned with horizontal torso and vertical look pitch for multiplayer/mirror view.
    pub fn crown_frame(&self) -> LimbFrame {
        let base = self.torso_frame();
        let neck_y = 1.16 + self.eye_height * 0.65;
        let neck_frame = base.shifted(0.0, neck_y, -0.04);
        let view_fwd = (self.view_rot * Vec3::NEG_Z).normalize();
        let up = (self.surface_rot * Vec3::Y).normalize();
        let pitch = view_fwd.dot(up).clamp(-0.8, 0.8).asin();
        neck_frame.pitched(-pitch * 0.65)
    }

    /// Contralateral 3D leg joint chain (hip, knee, ankle, toe) with natural human swing-through and forward stride extension.
    pub fn leg_joints(&self, side: f32) -> (Vec3, Vec3, Vec3, Vec3) {
        let phase = if side > 0.0 { self.rhythm.phase + std::f32::consts::PI } else { self.rhythm.phase };
        let s = phase.sin();
        let walk = (self.rhythm.blend / 0.45).clamp(0.0, 1.0);
        let run = ((self.rhythm.blend - 0.48) / 0.52).clamp(0.0, 1.0);
        let pass = (phase + 0.25).cos().max(0.0);
        let crouch = self.crouch_compression();
        let hip = Vec3::new(side * self.scale(0.096), 0.86 - crouch * 0.40, -0.026);
        let fwd_swing = if s >= 0.0 { s * (0.34 * walk + 0.22 * run) + pass * (0.06 * walk + 0.14 * run) } else { s * (0.28 * walk + 0.18 * run) };
        let theta_hip = 0.05 + crouch * 0.55 + fwd_swing;
        let theta_knee = 0.09 + crouch * 0.95 + (0.08 * walk + 0.16 * run) + pass * (0.46 * walk + 0.52 * run);
        let theta_shin = theta_hip - theta_knee;
        let (thigh_len, shin_len) = (0.37, 0.36);
        let knee_out = side * (self.scale(0.008) + pass * (0.018 * walk + 0.014 * run) + crouch * 0.020);
        let knee = Vec3::new(hip.x + knee_out, hip.y - thigh_len * theta_hip.cos(), hip.z + thigh_len * theta_hip.sin());
        let ankle_in = -side * (self.scale(0.005) + pass * (0.014 * walk + 0.010 * run));
        let ankle = Vec3::new(knee.x + ankle_in, (knee.y - shin_len * theta_shin.cos()).max(0.10), knee.z + shin_len * theta_shin.sin());
        let theta_foot = (theta_shin + 1.38).clamp(1.10, 1.54);
        let toe = Vec3::new(ankle.x + side * 0.006, (ankle.y - 0.16 * theta_foot.cos()).max(0.04), ankle.z + 0.16 * theta_foot.sin());
        (hip, knee, ankle, toe)
    }

    /// Forearm kinematics: stays low during walking (`walk`), pulls closer and rises with inward bend ONLY when running (`run`).
    pub fn forearm_swing(&self, side: f32) -> (f32, f32, f32, f32, f32) {
        let phase = if side > 0.0 { self.rhythm.phase } else { self.rhythm.phase + std::f32::consts::PI };
        let primary = phase.sin();
        let lagged = (phase - 0.48).sin();
        let dip = (2.0 * phase).cos();
        let walk = (self.rhythm.blend / 0.45).clamp(0.0, 1.0);
        let run = ((self.rhythm.blend - 0.48) / 0.52).clamp(0.0, 1.0);
        let breath = self.rhythm.phase.sin() * (1.0 - walk);
        let sway_x = -side * (0.006 * walk + 0.010 * run + primary * (0.010 * walk + 0.016 * run));
        let stance_asymmetry = side * (0.006 * walk + 0.010 * run);
        let arc_y = stance_asymmetry + 0.060 * run + primary * (0.022 * walk + 0.040 * run) + lagged * 0.012 * run - dip * 0.008 * walk + breath * 0.004;
        let arc_z = -0.045 * run + (primary * 0.024 + lagged * 0.010) * (0.55 * walk + 0.45 * run);
        let swing_pitch = -0.08 * run - (primary * (0.06 * walk + 0.10 * run) + lagged * 0.04 * run) + breath * 0.010;
        let swing_yaw = side * (0.05 * walk + 0.13 * run + primary * (0.04 * walk + 0.06 * run));
        (sway_x, arc_y, arc_z, swing_pitch, swing_yaw)
    }

    /// Finger flexion: stays open/relaxed when standing (0.18) and walking (0.25), clenches into a fist (0.96) ONLY when running.
    pub fn finger_curl(&self, side: f32, holding_relic: bool) -> f32 {
        if holding_relic { return 0.88; }
        let walk = (self.rhythm.blend / 0.45).clamp(0.0, 1.0);
        let run = ((self.rhythm.blend - 0.48) / 0.52).clamp(0.0, 1.0);
        let phase = if side > 0.0 { self.rhythm.phase } else { self.rhythm.phase + std::f32::consts::PI };
        let pump = phase.sin().max(0.0) * 0.06 * run;
        (0.18 + 0.07 * walk + 0.70 * run + pump).clamp(0.18, 0.96)
    }

    /// Local elbow offset relative to wrist: stays low (-0.065m) when walking, lifts moderately (-0.020m) when running.
    pub fn elbow_offset(&self, side: f32) -> Vec3 {
        let run = ((self.rhythm.blend - 0.48) / 0.52).clamp(0.0, 1.0);
        let phase = if side > 0.0 { self.rhythm.phase } else { self.rhythm.phase + std::f32::consts::PI };
        let pump = phase.sin() * 0.022 * run;
        Vec3::new(side * (0.022 + 0.026 * run), -0.065 + 0.045 * run + pump, -0.155 + 0.010 * run)
    }

    /// Vertical torso height compression when crouching in sneak gait.
    pub fn crouch_compression(&self) -> f32 {
        (0.8 - self.eye_height).clamp(0.0, 0.4)
    }
}
