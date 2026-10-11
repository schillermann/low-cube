use glam::Quat;

/// Interface for viewpoint rotation and surface alignment.
pub trait ViewOrientation {
    fn rotate(&mut self, dx: f32, dy: f32);
    fn toggle_invert_y(&mut self);
    fn toggle_invert_x(&mut self);
    fn combine_with_surface(&self, surface_rot: Quat) -> Quat;
}

/// Encapsulated first-person look orientation and mouse sensitivity/inversion.
pub struct MouseLook {
    yaw: f32,
    pitch: f32,
    invert_y: bool,
    invert_x: bool,
}

impl MouseLook {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(yaw: f32, pitch: f32, invert_y: bool, invert_x: bool) -> Self {
        Self {
            yaw,
            pitch,
            invert_y,
            invert_x,
        }
    }

    /// Secondary constructor initializing idle zero angles.
    pub fn idle() -> Self {
        Self::new(0.0, 0.0, false, false)
    }
}

impl ViewOrientation for MouseLook {
    fn rotate(&mut self, dx: f32, dy: f32) {
        let sensitivity = 0.003;
        let x_sign = if self.invert_x { 1.0 } else { -1.0 };
        let y_sign = if self.invert_y { 1.0 } else { -1.0 };

        self.yaw += dx * sensitivity * x_sign;
        self.pitch = (self.pitch + dy * sensitivity * y_sign).clamp(-1.4, 1.4);
    }

    fn toggle_invert_y(&mut self) {
        self.invert_y = !self.invert_y;
    }

    fn toggle_invert_x(&mut self) {
        self.invert_x = !self.invert_x;
    }

    fn combine_with_surface(&self, surface_rot: Quat) -> Quat {
        let local_rot = Quat::from_rotation_y(self.yaw) * Quat::from_rotation_x(self.pitch);
        surface_rot * local_rot
    }
}
