use glam::Quat;

/// Encapsulated first-person look orientation and mouse sensitivity/inversion.
pub struct MouseLook {
    yaw: f32,
    pitch: f32,
    invert_y: bool,
    invert_x: bool,
}

impl MouseLook {
    /// Primary constructor initializing default angles.
    pub fn new() -> Self {
        Self {
            yaw: 0.0,
            pitch: 0.0,
            invert_y: false,
            invert_x: false,
        }
    }

    pub fn rotate(&mut self, dx: f32, dy: f32) {
        let sensitivity = 0.003;
        let x_sign = if self.invert_x { 1.0 } else { -1.0 };
        let y_sign = if self.invert_y { 1.0 } else { -1.0 };

        self.yaw += dx * sensitivity * x_sign;
        self.pitch = (self.pitch + dy * sensitivity * y_sign).clamp(-1.4, 1.4);
    }

    pub fn toggle_invert_y(&mut self) {
        self.invert_y = !self.invert_y;
        println!("[Controls] Invert Y (Pitch) is now: {}", self.invert_y);
    }

    pub fn toggle_invert_x(&mut self) {
        self.invert_x = !self.invert_x;
        println!("[Controls] Invert X (Yaw) is now: {}", self.invert_x);
    }

    pub fn rotation_quat(&self) -> Quat {
        Quat::from_rotation_y(self.yaw) * Quat::from_rotation_x(self.pitch)
    }
}
