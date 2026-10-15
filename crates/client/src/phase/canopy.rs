/// Kinematic braking entity modeling aerodynamic leaf drag within tree crowns.
#[derive(Clone, Copy, Debug)]
pub struct CanopyBraking {
    terminal_velocity: f32,
    drag_coefficient: f32,
}

impl CanopyBraking {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(terminal_velocity: f32, drag_coefficient: f32) -> Self {
        Self {
            terminal_velocity,
            drag_coefficient,
        }
    }

    /// Secondary constructor calibrated for foliage cushioning.
    pub fn calibrated() -> Self {
        Self::new(1.5, 12.0)
    }

    /// Applies aerodynamic drag to descending vertical speed when inside foliage.
    pub fn brake_fall(&self, vertical_speed: f32, drag: f32, dt: f32) -> f32 {
        if drag > 0.0 && vertical_speed < -self.terminal_velocity {
            let target = -self.terminal_velocity;
            let blend = (drag * self.drag_coefficient * dt).min(1.0);
            vertical_speed + (target - vertical_speed) * blend
        } else {
            vertical_speed
        }
    }
}
