use glam::{Mat4, Quat, Vec3};
use crate::attire::{BodyPose, StrideRhythm, YouthPhysique};
use crate::horizon::ViewFrame;

/// Kinematic posture defining the vertical eye level offset for standing, crouching, and jumping.
#[derive(Clone, Copy, Debug)]
pub struct AvatarPosture {
    elevation: f32,
    leap_flex: f32,
    phase_timer: f32,
}

impl AvatarPosture {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn with_leap(elevation: f32, leap_flex: f32, phase_timer: f32) -> Self {
        Self { elevation, leap_flex, phase_timer }
    }

    pub fn new(elevation: f32) -> Self {
        Self::with_leap(elevation, 0.0, 0.0)
    }

    /// Secondary constructor for upright standing posture.
    pub fn standing() -> Self {
        Self::new(0.8)
    }

    /// Secondary constructor for crouched posture in die Hocke.
    #[cfg(test)]
    pub fn crouching() -> Self {
        Self::new(0.36)
    }

    /// Smoothly transitions posture towards a target elevation across elapsed time.
    pub fn transition_to(&self, target_elevation: f32, dt: f32) -> Self {
        let rate = 15.0;
        let factor = (rate * dt).clamp(0.0, 1.0);
        let updated = self.elevation + (target_elevation - self.elevation) * factor;
        Self::with_leap(updated, self.leap_flex, self.phase_timer)
    }

    /// Smoothly advances takeoff knee bend, airborne leg extension, and landing knee compression without discontinuities.
    pub fn advance_leap(self, previous: Vec3, next: Vec3, terrain_up: Vec3, dt: f32) -> Self {
        let v_up = if dt > 1e-5 { (next - previous).dot(terrain_up) / dt } else { 0.0 };
        let (target, rate, next_timer) = if self.phase_timer < 0.0 || v_up.abs() > 0.35 {
            let air_time = if self.phase_timer < 0.0 { -self.phase_timer + dt } else { dt };
            let landed = v_up.abs() <= 1e-3 && (air_time > 0.42 || self.leap_flex > 0.08);
            if landed {
                (-0.40, 26.0, 0.16)
            } else if air_time < 0.08 && v_up > 2.2 {
                (-0.38, 32.0, -air_time)
            } else {
                (0.36, 12.0, -air_time)
            }
        } else if self.phase_timer > 0.0 {
            (-0.40, 26.0, (self.phase_timer - dt).max(0.0))
        } else {
            (0.0, 10.0, 0.0)
        };
        let next_flex = self.leap_flex + (target - self.leap_flex) * (rate * dt).clamp(0.0, 1.0);
        Self::with_leap(self.elevation, next_flex, next_timer)
    }

    /// Computes view matrix using view frame and encapsulated eye elevation offset.
    pub fn view_matrix(&self, frame: &dyn ViewFrame, position: Vec3) -> Mat4 {
        frame.view_matrix_at(position, self.elevation)
    }

    /// Assembles a kinematic BodyPose using this posture's encapsulated eye elevation and jump flex.
    pub fn body_pose(
        &self,
        position: Vec3,
        surface_rot: Quat,
        view_rot: Quat,
        rhythm: StrideRhythm,
    ) -> BodyPose {
        BodyPose::new(
            position,
            surface_rot,
            view_rot,
            self.elevation,
            rhythm,
            YouthPhysique::SlenderSquire,
        )
        .with_leap(self.leap_flex)
    }

    /// Verifies whether the avatar is currently in a crouched state.
    #[cfg(test)]
    pub fn crouched(&self) -> bool {
        self.elevation < 0.6
    }
}
