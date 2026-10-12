use glam::Mat4;

/// Interface for perspective and orthographic camera projection lenses ("Tell, Don't Ask").
pub trait ProjectionLens: Send + Sync {
    fn projection_matrix(&self, aspect: f32) -> Mat4;
}

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

    /// Smoothly transitions field of view towards a target angle across elapsed time.
    pub fn transition_to(&self, target_fov_radians: f32, dt: f32) -> Self {
        let rate = 8.0;
        let factor = (rate * dt).clamp(0.0, 1.0);
        let updated = self.fov_radians + (target_fov_radians - self.fov_radians) * factor;
        Self::new(updated, self.near_plane, self.far_plane)
    }

    /// Verifies whether the lens is currently in an expanded high-speed state.
    #[cfg(test)]
    pub fn expanded(&self) -> bool {
        self.fov_radians > 70.0f32.to_radians()
    }
}

impl ProjectionLens for PerspectiveLens {
    fn projection_matrix(&self, aspect: f32) -> Mat4 {
        Mat4::perspective_rh(self.fov_radians, aspect, self.near_plane, self.far_plane)
    }
}

/// Decorator applying optical zoom magnification to an underlying projection lens.
#[cfg(test)]
pub struct MagnifiedLens<'a> {
    origin: &'a dyn ProjectionLens,
    zoom_factor: f32,
}

#[cfg(test)]
impl<'a> MagnifiedLens<'a> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(origin: &'a dyn ProjectionLens, zoom_factor: f32) -> Self {
        Self { origin, zoom_factor }
    }
}

#[cfg(test)]
impl<'a> ProjectionLens for MagnifiedLens<'a> {
    fn projection_matrix(&self, aspect: f32) -> Mat4 {
        let base = self.origin.projection_matrix(aspect);
        Mat4::from_scale(glam::Vec3::new(self.zoom_factor, self.zoom_factor, 1.0)) * base
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_magnified_lens_decorates_projection_matrix() {
        let lens = PerspectiveLens::human_eye();
        let zoomed = MagnifiedLens::new(&lens, 2.0);
        let base_proj = lens.projection_matrix(16.0 / 9.0);
        let zoomed_proj = zoomed.projection_matrix(16.0 / 9.0);
        assert_ne!(base_proj, zoomed_proj);
    }
}
