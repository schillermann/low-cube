use glam::Vec3;
use super::{AttitudeGuidance, CockpitAxes, COMMAND_STEER_RATE, SAS_GAIN};

/// 6-DOF cockpit flight command uniting linear translation, angular torque demand, local spin, and `AttitudeGuidance`.
#[derive(Clone, Copy, Debug)]
pub struct PilotOrder {
    linear: Vec3,
    angular: Vec3,
    omega_local: Vec3,
    guidance: AttitudeGuidance,
}

impl PilotOrder {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn from_channels(
        linear: Vec3,
        angular: Vec3,
        omega_local: Vec3,
        guidance: AttitudeGuidance,
    ) -> Self {
        Self { linear, angular, omega_local, guidance }
    }

    /// Secondary constructor mapping legacy `(yaw_x, lift_y, pitch_z)` axes with engaged guidance.
    pub const fn new(burn_axes: Vec3) -> Self {
        Self::from_channels(
            Vec3::new(0.0, burn_axes.y, burn_axes.z * 0.90),
            Vec3::new(-burn_axes.z, -burn_axes.x, 0.0),
            Vec3::ZERO,
            AttitudeGuidance::Engaged,
        )
    }

    pub const fn idle() -> Self {
        Self::from_channels(Vec3::ZERO, Vec3::ZERO, Vec3::ZERO, AttitudeGuidance::Engaged)
    }

    pub const fn guidance_engaged(&self) -> bool {
        self.guidance.engaged()
    }

    pub const fn guidance(&self) -> AttitudeGuidance {
        self.guidance
    }

    pub const fn vertical_burn(&self) -> f32 {
        self.linear.y
    }

    fn effective_angular(&self) -> Vec3 {
        if self.guidance.engaged() {
            let target_rate = self.angular * COMMAND_STEER_RATE;
            ((target_rate - self.omega_local) * SAS_GAIN)
                .clamp(Vec3::splat(-1.4), Vec3::splat(1.4))
        } else {
            self.angular
        }
    }

    pub fn world_linear(&self, axes: CockpitAxes) -> Vec3 {
        axes.to_world(self.linear)
    }

    pub fn world_torque(&self, axes: CockpitAxes) -> Vec3 {
        axes.to_world(self.effective_angular())
    }

    /// Evaluates `[0.0..=1.0]` main-bell throttle along `world_thrust`.
    pub fn bell_throttle(&self, world_thrust: Vec3, axes: CockpitAxes) -> f32 {
        world_thrust.normalize_or_zero().dot(axes.up_axis() * self.linear.y.max(0.0)).clamp(0.0, 1.0)
    }

    pub fn active_burn(&self) -> bool {
        self.linear.length_squared() > 0.01 || self.effective_angular().length_squared() > 0.01
    }

    pub fn manual_input(&self) -> bool {
        self.linear.x.abs() > 0.05 || self.linear.z.abs() > 0.05 || self.angular.length_squared() > 0.01
    }

    pub fn console_stick(&self) -> f32 {
        let ang = self.angular;
        if ang.x.abs() >= ang.y.abs() && ang.x.abs() >= ang.z.abs() {
            -ang.x
        } else if ang.y.abs() >= ang.z.abs() {
            -ang.y
        } else {
            ang.z
        }
    }

    /// Packs 6-DOF thruster demand and `AttitudeGuidance` into `CraftPose`'s `(wheel_spin, steer_turn)` channels.
    pub fn pose_motion(&self) -> (f32, f32) {
        let q = |v: f32| ((v.clamp(-1.0, 1.0) + 1.0) * 7.0).round() as u32;
        let eff = self.effective_angular();
        let code = q(self.linear.x)
            | (q(self.linear.z) << 4)
            | (q(eff.x) << 8)
            | (q(eff.y) << 12)
            | (q(eff.z) << 16)
            | (u32::from(self.guidance.engaged()) << 20);
        (self.linear.y.clamp(-1.0, 1.0), 4.0 + (code as f32))
    }

    /// Reconstructs `PilotOrder` from `CraftPose`'s `(wheel_spin, steer_turn)` channels.
    pub fn from_pose_motion(spin: f32, turn: f32) -> Self {
        if turn >= 3.5 {
            let code = (turn - 4.0).round().max(0.0) as u32;
            let dq = |shift: u32| {
                let raw = ((code >> shift) & 0xF).min(14) as f32;
                let v = (raw / 7.0) - 1.0;
                if v.abs() < 0.08 { 0.0 } else { v }
            };
            let guidance = if ((code >> 20) & 1) != 0 {
                AttitudeGuidance::Engaged
            } else {
                AttitudeGuidance::Manual
            };
            Self::from_channels(
                Vec3::new(dq(0), spin.clamp(-1.0, 1.0), dq(4)),
                Vec3::new(dq(8), dq(12), dq(16)),
                Vec3::ZERO,
                guidance,
            )
        } else {
            Self::from_channels(
                Vec3::new(0.0, spin.clamp(-1.0, 1.0), turn.clamp(-1.0, 1.0) * 0.90),
                Vec3::new(-turn.clamp(-1.0, 1.0), 0.0, 0.0),
                Vec3::ZERO,
                AttitudeGuidance::Engaged,
            )
        }
    }
}
