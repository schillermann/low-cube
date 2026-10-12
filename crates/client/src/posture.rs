use glam::{Mat4, Vec3};
use crate::basis::OrientationBasis;

/// Kinematic posture defining the vertical eye level offset for standing and crouching.
#[derive(Clone, Copy, Debug)]
pub struct AvatarPosture {
    elevation: f32,
}

impl AvatarPosture {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(elevation: f32) -> Self {
        Self { elevation }
    }

    /// Secondary constructor for upright standing posture.
    pub fn standing() -> Self {
        Self::new(0.8)
    }

    /// Secondary constructor for crouched posture in die Hocke.
    #[cfg(test)]
    pub fn crouching() -> Self {
        Self::new(0.45)
    }

    /// Smoothly transitions posture towards a target elevation across elapsed time.
    pub fn transition_to(&self, target_elevation: f32, dt: f32) -> Self {
        let rate = 15.0;
        let factor = (rate * dt).clamp(0.0, 1.0);
        let updated = self.elevation + (target_elevation - self.elevation) * factor;
        Self::new(updated)
    }

    /// Computes view matrix using orientation basis and encapsulated eye elevation offset.
    pub fn view_matrix(&self, basis: &OrientationBasis, position: Vec3) -> Mat4 {
        basis.view_matrix_at(position, self.elevation)
    }

    /// Verifies whether the avatar is currently in a crouched state.
    #[cfg(test)]
    pub fn is_crouching(&self) -> bool {
        self.elevation < 0.6
    }
}
