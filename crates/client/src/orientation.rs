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

    /// Secondary constructor wrapping orientation with default calibrated sensitivity (0.003).
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

/// Polymorphic axis polarity without primitive boolean branching.
pub trait AxisPolarity: Send + Sync {
    fn apply(&self, delta: f32) -> f32;
    fn invert(&self) -> Box<dyn AxisPolarity>;
}

/// Direct unmodified directional axis motion.
pub struct DirectPolarity;

impl AxisPolarity for DirectPolarity {
    fn apply(&self, delta: f32) -> f32 {
        delta
    }

    fn invert(&self) -> Box<dyn AxisPolarity> {
        Box::new(ReversePolarity)
    }
}

/// Inverted negated directional axis motion.
pub struct ReversePolarity;

impl AxisPolarity for ReversePolarity {
    fn apply(&self, delta: f32) -> f32 {
        -delta
    }

    fn invert(&self) -> Box<dyn AxisPolarity> {
        Box::new(DirectPolarity)
    }
}

/// Decorator managing polymorphic axis inversion states without primitive flags.
pub struct InvertibleLook {
    look: Box<dyn ViewOrientation>,
    x_polarity: Box<dyn AxisPolarity>,
    y_polarity: Box<dyn AxisPolarity>,
}

impl InvertibleLook {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(
        look: Box<dyn ViewOrientation>,
        x_polarity: Box<dyn AxisPolarity>,
        y_polarity: Box<dyn AxisPolarity>,
    ) -> Self {
        Self {
            look,
            x_polarity,
            y_polarity,
        }
    }

    /// Secondary constructor initializing direct un-inverted axis polarities.
    pub fn direct(look: Box<dyn ViewOrientation>) -> Self {
        Self::new(look, Box::new(DirectPolarity), Box::new(DirectPolarity))
    }

    /// Secondary constructor initializing inverted pitch polarity.
    #[cfg(test)]
    pub fn inverted_pitch(look: Box<dyn ViewOrientation>) -> Self {
        Self::new(look, Box::new(DirectPolarity), Box::new(ReversePolarity))
    }

    /// Secondary constructor initializing inverted yaw polarity.
    #[cfg(test)]
    pub fn inverted_yaw(look: Box<dyn ViewOrientation>) -> Self {
        Self::new(look, Box::new(ReversePolarity), Box::new(DirectPolarity))
    }

    /// Inverts the horizontal yaw axis polarity.
    pub fn toggle_x(&mut self) {
        self.x_polarity = self.x_polarity.invert();
    }

    /// Inverts the vertical pitch axis polarity.
    pub fn toggle_y(&mut self) {
        self.y_polarity = self.y_polarity.invert();
    }
}

impl ViewOrientation for InvertibleLook {
    fn rotate(&mut self, dx: f32, dy: f32) {
        self.look.rotate(
            self.x_polarity.apply(dx),
            self.y_polarity.apply(dy),
        );
    }

    fn combine_with_surface(&self, surface_rot: Quat) -> Quat {
        self.look.combine_with_surface(surface_rot)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invertible_look_polarities() {
        let base = Box::new(MouseLook::idle());
        let mut inverted_pitch = InvertibleLook::inverted_pitch(base);
        inverted_pitch.rotate(0.0, 1.0);
        let q_pitch = inverted_pitch.combine_with_surface(Quat::IDENTITY);

        let base_yaw = Box::new(MouseLook::idle());
        let mut inverted_yaw = InvertibleLook::inverted_yaw(base_yaw);
        inverted_yaw.rotate(1.0, 0.0);
        let q_yaw = inverted_yaw.combine_with_surface(Quat::IDENTITY);

        assert_ne!(q_pitch, Quat::IDENTITY);
        assert_ne!(q_yaw, Quat::IDENTITY);

        let mut invertible = InvertibleLook::direct(Box::new(MouseLook::idle()));
        invertible.toggle_y();
        invertible.toggle_x();
        invertible.rotate(1.0, 1.0);
        let q_inv = invertible.combine_with_surface(Quat::IDENTITY);
        assert_ne!(q_inv, Quat::IDENTITY);
    }
}
