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
        if self.active_keys.contains(&KeyCode::KeyW) { direction += forward; }
        if self.active_keys.contains(&KeyCode::KeyS) { direction -= forward; }
        if self.active_keys.contains(&KeyCode::KeyD) { direction += right; }
        if self.active_keys.contains(&KeyCode::KeyA) { direction -= right; }

        if direction.length_squared() > 1e-4 {
            target.travel(direction.normalize());
        }

        if self.active_keys.contains(&KeyCode::Space) {
            target.jump();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockLocomotionTarget {
        traveled: Option<Vec3>,
        jumped: bool,
    }

    impl LocomotionTarget for MockLocomotionTarget {
        fn travel(&mut self, direction: Vec3) {
            self.traveled = Some(direction);
        }

        fn jump(&mut self) {
            self.jumped = true;
        }
    }

    #[test]
    fn test_unified_locomotion_messaging() {
        let mut motion = KeyMotion::idle();
        let mut target = MockLocomotionTarget {
            traveled: None,
            jumped: false,
        };

        // Idle state produces no messages
        motion.drive_locomotion(Vec3::Z, Vec3::X, &mut target);
        assert!(target.traveled.is_none());
        assert!(!target.jumped);

        // Pressing forward (W) and jump (Space) sends both travel and jump messages
        motion.register_key(KeyCode::KeyW, true);
        motion.register_key(KeyCode::Space, true);
        motion.drive_locomotion(Vec3::Z, Vec3::X, &mut target);

        assert_eq!(target.traveled, Some(Vec3::Z));
        assert!(target.jumped);

        // Releasing keys stops messages
        motion.register_key(KeyCode::KeyW, false);
        motion.register_key(KeyCode::Space, false);
        target.traveled = None;
        target.jumped = false;
        motion.drive_locomotion(Vec3::Z, Vec3::X, &mut target);

        assert!(target.traveled.is_none());
        assert!(!target.jumped);
    }
}
