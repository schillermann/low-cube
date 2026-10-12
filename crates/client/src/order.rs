use glam::Vec3;
use crate::controls::LocomotionTarget;
use crate::flight::GravitationalFlight;

/// Encapsulated navigational order commanded by input controls for a single simulation step.
pub struct LocomotionOrder {
    direction: Vec3,
    jump_requested: bool,
}

impl LocomotionOrder {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(direction: Vec3, jump_requested: bool) -> Self {
        Self {
            direction,
            jump_requested,
        }
    }

    /// Secondary constructor initializing with idle state.
    pub fn idle() -> Self {
        Self::new(Vec3::ZERO, false)
    }

    /// Computes the horizontal stride aligned with a surface normal for a given distance.
    pub fn surface_stride(&self, surface_up: Vec3, distance: f32) -> Vec3 {
        let tangent = self.direction - surface_up * self.direction.dot(surface_up);
        if tangent.length_squared() > 1e-4 {
            tangent.normalize() * distance
        } else {
            Vec3::ZERO
        }
    }

    /// Dispatches a jump command to a flight entity if jump was requested.
    pub fn dispatch_jump(&self, flight: &mut dyn GravitationalFlight) {
        if self.jump_requested {
            flight.jump();
        }
    }
}

impl LocomotionTarget for LocomotionOrder {
    fn travel(&mut self, direction: Vec3) {
        self.direction = direction;
    }

    fn jump(&mut self) {
        self.jump_requested = true;
    }
}
