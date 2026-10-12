use glam::Mat4;

/// Perspective projection lens encapsulating field of view and clipping planes.
#[derive(Clone, Copy, Debug)]
pub struct PerspectiveLens {
    fov_radians: f32,
    near_plane: f32,
    far_plane: f32,
}

impl PerspectiveLens {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(fov_radians: f32, near_plane: f32, far_plane: f32) -> Self {
        Self {
            fov_radians,
            near_plane,
            far_plane,
        }
    }

    /// Secondary constructor with standard first-person field of view.
    pub fn human_eye() -> Self {
        Self::new(65.0f32.to_radians(), 0.1, 200.0)
    }

    pub fn projection_matrix(&self, aspect: f32) -> Mat4 {
        Mat4::perspective_rh(self.fov_radians, aspect, self.near_plane, self.far_plane)
    }

    /// Smoothly transitions field of view towards a target angle across elapsed time.
    pub fn transition_to(&self, target_fov_radians: f32, dt: f32) -> Self {
        let rate = 8.0;
        let factor = (rate * dt).clamp(0.0, 1.0);
        let updated = self.fov_radians + (target_fov_radians - self.fov_radians) * factor;
        Self::new(updated, self.near_plane, self.far_plane)
    }

    /// Verifies whether the lens is currently in an expanded high-speed state.
    #[cfg(test)]
    pub fn is_expanded(&self) -> bool {
        self.fov_radians > 70.0f32.to_radians()
    }
}
