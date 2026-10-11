use glam::Vec3;
use winit::keyboard::KeyCode;

/// Interface representing player keyboard navigation state.
pub trait KeyboardControls {
    fn set_key(&mut self, code: KeyCode, pressed: bool);
    fn resolve_motion(&self, forward: Vec3, right: Vec3) -> Vec3;
}

/// Encapsulated keyboard state implementing KeyboardControls.
pub struct KeyState {
    forward: bool,
    backward: bool,
    left: bool,
    right: bool,
}

impl KeyState {
    pub fn new() -> Self {
        Self {
            forward: false,
            backward: false,
            left: false,
            right: false,
        }
    }
}

impl KeyboardControls for KeyState {
    fn set_key(&mut self, code: KeyCode, pressed: bool) {
        match code {
            KeyCode::KeyW => self.forward = pressed,
            KeyCode::KeyS => self.backward = pressed,
            KeyCode::KeyA => self.left = pressed,
            KeyCode::KeyD => self.right = pressed,
            _ => {}
        }
    }

    fn resolve_motion(&self, forward: Vec3, right: Vec3) -> Vec3 {
        let mut move_vec = Vec3::ZERO;
        if self.forward { move_vec += forward; }
        if self.backward { move_vec -= forward; }
        if self.right { move_vec += right; }
        if self.left { move_vec -= right; }
        move_vec
    }
}
