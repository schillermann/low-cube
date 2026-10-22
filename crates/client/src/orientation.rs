use glam::Quat;

/// Interface for viewpoint rotation and surface alignment.
pub trait ViewOrientation {
    fn rotate(&mut self, dx: f32, dy: f32);
    fn combine_with_surface(&self, surface_rot: Quat) -> Quat;
}

/// Encapsulated first-person look orientation without internal flags or hardcoded coefficients.
pub struct MouseLook {
    yaw: f32,
    pitch: f32,
}

impl MouseLook {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(yaw: f32, pitch: f32) -> Self {
        Self { yaw, pitch }
    }

    /// Secondary constructor initializing idle zero angles.
    pub fn idle() -> Self {
        Self::new(0.0, 0.0)
    }
}

impl ViewOrientation for MouseLook {
    fn rotate(&mut self, dx: f32, dy: f32) {
        self.yaw -= dx;
        self.pitch = (self.pitch - dy).clamp(-1.4, 1.4);
    }

    fn combine_with_surface(&self, surface_rot: Quat) -> Quat {
        let local_rot = Quat::from_rotation_y(self.yaw) * Quat::from_rotation_x(self.pitch);
        surface_rot * local_rot
    }
}

/// Decorator scaling mouse rotational delta by an encapsulated sensitivity coefficient.
pub struct SensitivityLook {
    look: Box<dyn ViewOrientation>,
    sensitivity: f32,
}

impl SensitivityLook {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(look: Box<dyn ViewOrientation>, sensitivity: f32) -> Self {
        Self { look, sensitivity }
    }

    /// Secondary constructor wrapping orientation with calibrated sensitivity (0.003).
    pub fn calibrated(look: Box<dyn ViewOrientation>) -> Self {
        Self::new(look, 0.003)
    }
}

impl ViewOrientation for SensitivityLook {
    fn rotate(&mut self, dx: f32, dy: f32) {
        self.look.rotate(dx * self.sensitivity, dy * self.sensitivity);
    }

    fn combine_with_surface(&self, surface_rot: Quat) -> Quat {
        self.look.combine_with_surface(surface_rot)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sensitivity_look_rotation() {
        let mut look = SensitivityLook::calibrated(Box::new(MouseLook::idle()));
        look.rotate(10.0, 5.0);
        let q = look.combine_with_surface(Quat::IDENTITY);
        assert_ne!(q, Quat::IDENTITY);
    }
}
