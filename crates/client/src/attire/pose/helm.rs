use glam::Vec3;
use super::super::limb::LimbFrame;
use super::super::rhythm::WristPosture;
use super::BodyPose;

/// 6-DOF rocket cockpit control deflection: 4-way flight stick `(lateral, fore_aft)` and thrust lever `[-1.0..=1.0]`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RocketControls {
    stick: (f32, f32),
    thrust: f32,
}

impl RocketControls {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(stick: (f32, f32), thrust: f32) -> Self {
        Self { stick, thrust }
    }

    pub const fn neutral() -> Self {
        Self::new((0.0, 0.0), 0.0)
    }

    pub fn clamped(stick_x: f32, stick_z: f32, thrust: f32) -> Self {
        Self::new(
            (stick_x.clamp(-1.0, 1.0), stick_z.clamp(-1.0, 1.0)),
            thrust.clamp(-1.0, 1.0),
        )
    }

    pub fn blended(self, target: Self, factor: f32) -> Self {
        let t = factor.clamp(0.0, 1.0);
        Self::new(
            (
                self.stick.0 + (target.stick.0 - self.stick.0) * t,
                self.stick.1 + (target.stick.1 - self.stick.1) * t,
            ),
            self.thrust + (target.thrust - self.thrust) * t,
        )
    }

    pub fn plume_channels(self) -> (f32, f32) {
        let planar = if self.stick.1.abs() >= self.stick.0.abs() {
            self.stick.1
        } else {
            self.stick.0
        };
        (self.thrust, planar)
    }

    pub fn studio_showcase(cycle_progress: f32) -> Self {
        let u = cycle_progress.rem_euclid(1.0) * 4.0;
        let phase_idx = (u as usize) % 4;
        let pulse = (u.fract() * std::f32::consts::PI).sin();
        match phase_idx {
            0 => Self::new((0.0, pulse), pulse),
            1 => Self::new((0.0, -pulse), -pulse),
            2 => Self::new((-pulse, 0.0), 0.80 * pulse),
            _ => Self::new((pulse, 0.0), -0.80 * pulse),
        }
    }

    pub(super) fn external_arm(
        self,
        pose: &BodyPose,
        side: f32,
        torso: &LimbFrame,
    ) -> (Vec3, Vec3, LimbFrame, f32) {
        let shoulder = Vec3::new(
            side * pose.scale(if pose.feminine() { 0.194 } else { 0.222 }),
            pose.torso_span(0.64),
            -0.066,
        );
        let (upper_len, fore_len) = (pose.arm_size(0.26), pose.arm_size(0.24));
        let (elbow, wrist, posture) = if side > 0.0 {
            let (sx, sz) = (self.stick.0.clamp(-1.0, 1.0), self.stick.1.clamp(-1.0, 1.0));
            let abduct = 0.10 + 0.08 * sx;
            let r_sag = upper_len * abduct.cos();
            let theta_upper = 0.74 + 0.26 * sz;
            let elbow = Vec3::new(
                shoulder.x + upper_len * abduct.sin() + pose.arm_size(0.018) * sx,
                shoulder.y - r_sag * theta_upper.cos(),
                shoulder.z + r_sag * theta_upper.sin(),
            );
            let theta_fore = theta_upper + (0.82 - 0.24 * sz);
            let wrist_x = elbow.x - pose.arm_size(0.135) + pose.arm_size(0.085) * sx;
            let dx = wrist_x - elbow.x;
            let r_fore = (fore_len * fore_len - dx * dx).max(0.0064).sqrt();
            let arc_drop = pose.arm_size(0.014) * (sx * sx + sz * sz).min(1.0);
            let wrist = Vec3::new(
                wrist_x,
                elbow.y - r_fore * theta_fore.cos() - arc_drop,
                elbow.z + r_fore * theta_fore.sin() + pose.arm_size(0.036) * sz,
            );
            let orient = WristPosture::new(1.0, 0.45 * sx, -0.28 * sx, 0.12 + 0.30 * sz);
            (elbow, wrist, orient)
        } else {
            let th = self.thrust.clamp(-1.0, 1.0);
            let abduct = 0.13 - 0.03 * th;
            let r_sag = upper_len * abduct.cos();
            let theta_upper = 0.68 + 0.34 * th;
            let elbow = Vec3::new(
                shoulder.x - upper_len * abduct.sin(),
                shoulder.y - r_sag * theta_upper.cos(),
                shoulder.z + r_sag * theta_upper.sin(),
            );
            let theta_fore = theta_upper + (0.86 - 0.24 * th);
            let wrist_x = -pose.scale(0.145);
            let dx = wrist_x - elbow.x;
            let r_fore = (fore_len * fore_len - dx * dx).max(0.0064).sqrt();
            let wrist = Vec3::new(
                wrist_x,
                elbow.y - r_fore * theta_fore.cos() + pose.arm_size(0.018) * th,
                elbow.z + r_fore * theta_fore.sin() + pose.arm_size(0.044) * th,
            );
            let orient = WristPosture::new(0.14, side, 0.08, -0.10 + 0.24 * th);
            (elbow, wrist, orient)
        };
        let shoulder_w = torso.point(shoulder.x, shoulder.y, shoulder.z);
        let elbow_w = torso.point(elbow.x, elbow.y, elbow.z);
        let wrist_w = torso.point(wrist.x, wrist.y, wrist.z);
        let forearm_vec = wrist_w - elbow_w;
        let forearm_len = forearm_vec.length();
        let hand_frame = posture.orient_hand(torso, wrist_w, forearm_vec.normalize_or_zero());
        (shoulder_w, elbow_w, hand_frame, forearm_len)
    }
}

/// Polymorphic seated and helm control posture replacing magic float sentinels.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum CockpitPosture {
    Standing,
    Seated(f32),
    WheelSteer(f32),
    RocketPilot(RocketControls),
}

impl CockpitPosture {
    pub(super) const fn seated_factor(self) -> f32 {
        match self {
            Self::Standing => 0.0,
            Self::Seated(factor) => factor,
            Self::WheelSteer(_) | Self::RocketPilot(_) => 1.0,
        }
    }

    pub(super) const fn driving_factor(self) -> f32 {
        match self {
            Self::WheelSteer(_) | Self::RocketPilot(_) => 1.0,
            Self::Standing | Self::Seated(_) => 0.0,
        }
    }

    pub(super) const fn steering_turn(self) -> f32 {
        match self {
            Self::WheelSteer(steer) => steer,
            Self::Standing | Self::Seated(_) | Self::RocketPilot(_) => 0.0,
        }
    }

    pub(super) const fn rocket_controls(self) -> Option<RocketControls> {
        match self {
            Self::RocketPilot(controls) => Some(controls),
            Self::Standing | Self::Seated(_) | Self::WheelSteer(_) => None,
        }
    }

    pub(super) const fn piloting_rocket(self) -> bool {
        matches!(self, Self::RocketPilot(_))
    }
}
