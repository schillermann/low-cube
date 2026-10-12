use glam::Vec3;
use winit::keyboard::KeyCode;
use crate::controls::{KeyboardControls, KeyMotion, LocomotionTarget};

struct MockLocomotionTarget {
    traveled: Option<Vec3>,
    jumped: bool,
    sneaked: bool,
}

impl LocomotionTarget for MockLocomotionTarget {
    fn travel(&mut self, direction: Vec3) {
        self.traveled = Some(direction);
    }

    fn jump(&mut self) {
        self.jumped = true;
    }

    fn sneak(&mut self) {
        self.sneaked = true;
    }

    fn sprint(&mut self) {}
}

#[test]
fn test_unified_locomotion_messaging() {
    let mut motion = KeyMotion::idle();
    let mut target = MockLocomotionTarget {
        traveled: None,
        jumped: false,
        sneaked: false,
    };

    // Idle state produces no messages
    motion.drive_locomotion(Vec3::Z, Vec3::X, &mut target);
    assert!(target.traveled.is_none());
    assert!(!target.jumped);
    assert!(!target.sneaked);

    // Pressing forward (W), jump (Space), and sneak (ShiftLeft) sends all messages
    motion.register_key(KeyCode::KeyW, true);
    motion.register_key(KeyCode::Space, true);
    motion.register_key(KeyCode::ShiftLeft, true);
    motion.drive_locomotion(Vec3::Z, Vec3::X, &mut target);

    assert_eq!(target.traveled, Some(Vec3::Z));
    assert!(target.jumped);
    assert!(target.sneaked);

    // Releasing keys stops messages
    motion.register_key(KeyCode::KeyW, false);
    motion.register_key(KeyCode::Space, false);
    motion.register_key(KeyCode::ShiftLeft, false);
    target.traveled = None;
    target.jumped = false;
    target.sneaked = false;
    motion.drive_locomotion(Vec3::Z, Vec3::X, &mut target);

    assert!(target.traveled.is_none());
    assert!(!target.jumped);
    assert!(!target.sneaked);

    // ShiftRight also triggers sneak
    motion.register_key(KeyCode::ShiftRight, true);
    motion.drive_locomotion(Vec3::Z, Vec3::X, &mut target);
    assert!(target.sneaked);
}
