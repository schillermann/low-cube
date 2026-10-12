use glam::Mat4;

/// Perspective projection lens encapsulating field of view and clipping planes.
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
}
