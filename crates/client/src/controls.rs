use std::collections::HashSet;
use glam::Vec3;
use winit::keyboard::KeyCode;

/// Interface for receiving unified locomotion messages ("Tell, Don't Ask").
pub trait LocomotionTarget {
    fn travel(&mut self, direction: Vec3);
    fn jump(&mut self);
}

/// Interface representing player keyboard navigation state.
pub trait KeyboardControls {
    fn register_key(&mut self, code: KeyCode, pressed: bool);
    fn drive_locomotion(
        &self,
        forward: Vec3,
        right: Vec3,
        target: &mut dyn LocomotionTarget,
    );
}

/// Encapsulated active key registry implementing KeyboardControls without primitive flags.
pub struct KeyMotion {
    active_keys: HashSet<KeyCode>,
}

impl KeyMotion {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(active_keys: HashSet<KeyCode>) -> Self {
        Self { active_keys }
    }

    /// Secondary constructor initializing with an empty key registry.
    pub fn idle() -> Self {
        Self::new(HashSet::new())
    }
}

impl KeyboardControls for KeyMotion {
    fn register_key(&mut self, code: KeyCode, pressed: bool) {
        if pressed {
            self.active_keys.insert(code);
        } else {
            self.active_keys.remove(&code);
        }
    }

    fn drive_locomotion(
        &self,
        forward: Vec3,
        right: Vec3,
        target: &mut dyn LocomotionTarget,
    ) {
        let mut direction = Vec3::ZERO;
        if self.active_keys.contains(&KeyCode::KeyW) {
            direction += forward;
        }
        if self.active_keys.contains(&KeyCode::KeyS) {
            direction -= forward;
        }
        if self.active_keys.contains(&KeyCode::KeyD) {
            direction += right;
        }
        if self.active_keys.contains(&KeyCode::KeyA) {
            direction -= right;
        }

        if direction.length_squared() > 1e-4 {
            target.travel(direction.normalize());
        }

        if self.active_keys.contains(&KeyCode::Space) {
            target.jump();
        }
    }
}
