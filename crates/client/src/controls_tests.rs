use glam::Vec3;
use winit::keyboard::KeyCode;
use crate::controls::{KeyboardControls, KeyMotion, LocomotionTarget};

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
