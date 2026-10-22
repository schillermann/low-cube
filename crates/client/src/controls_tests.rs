use glam::Vec3;
use winit::keyboard::KeyCode;
use crate::controls::{
    ActionBindings, BindingProfile, ControlAction, FileBindings, InputTrigger, KeyboardControls,
    KeyMotion, LocomotionTarget,
};

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

    // Only one keybinding per action: ShiftRight does not trigger sneak while Sneak is bound to ShiftLeft
    motion.register_key(KeyCode::ShiftRight, true);
    motion.drive_locomotion(Vec3::Z, Vec3::X, &mut target);
    assert!(!target.sneaked, "Secondary unconfigured key must not trigger Sneak");
    motion.register_key(KeyCode::ShiftRight, false);

    // Rebinding Sneak to ShiftRight and MoveForward to ArrowUp updates controls cleanly
    let custom = ActionBindings::seeded()
        .with_key_binding(ControlAction::Sneak, KeyCode::ShiftRight)
        .with_key_binding(ControlAction::MoveForward, KeyCode::ArrowUp);
    motion.apply_bindings(custom);
    motion.register_key(KeyCode::ShiftLeft, true);
    motion.register_key(KeyCode::KeyW, true);
    motion.drive_locomotion(Vec3::Z, Vec3::X, &mut target);
    assert!(!target.sneaked && target.traveled.is_none(), "Old keys must no longer trigger rebound actions");

    motion.register_key(KeyCode::ShiftLeft, false);
    motion.register_key(KeyCode::KeyW, false);
    motion.register_key(KeyCode::ShiftRight, true);
    motion.register_key(KeyCode::ArrowUp, true);
    motion.drive_locomotion(Vec3::Z, Vec3::X, &mut target);
    assert!(target.sneaked && target.traveled == Some(Vec3::Z));

    // Assigning an already bound key swaps bindings so every action retains a single unique key
    let swapped = ActionBindings::seeded().with_key_binding(ControlAction::MoveForward, KeyCode::KeyS);
    assert_eq!(swapped.trigger_for(ControlAction::MoveForward), InputTrigger::Key(KeyCode::KeyS));
    assert_eq!(swapped.trigger_for(ControlAction::MoveBackward), InputTrigger::Key(KeyCode::KeyW));
    assert_eq!(swapped.trigger_for(ControlAction::MoldForm), InputTrigger::WheelClick);
    assert_eq!(swapped.trigger_for(ControlAction::MoldLeft), InputTrigger::WheelUp);
    assert_eq!(swapped.trigger_for(ControlAction::MoldRight), InputTrigger::WheelDown);

    let temp_path = std::env::temp_dir().join("test_low_cube_controls_profile.toml");
    let store = FileBindings::new(temp_path.to_str().unwrap().to_string());
    store.save(&swapped);
    let loaded = store.load_or_seeded();
    let _ = std::fs::remove_file(&temp_path);
    assert_eq!(swapped, loaded);
}

#[test]
fn test_locomotion_and_control_decorators() {
    use crate::controls::{DampedLocomotionTarget, GroundedLocomotionTarget, MutedControls, TallyLocomotionTarget};

    let mut base_target = MockLocomotionTarget {
        traveled: None,
        jumped: false,
        sneaked: false,
    };

    // Test Tally, Grounded, and Damped decorator composition
    {
        let mut grounded = GroundedLocomotionTarget::new(&mut base_target);
        let mut tally = TallyLocomotionTarget::new(&mut grounded);
        let mut damped = DampedLocomotionTarget::new(&mut tally, 0.5);

        damped.travel(Vec3::new(10.0, 0.0, 0.0));
        damped.jump();
        damped.sneak();
        damped.sprint();

        assert_eq!(tally.travels(), 1);
        assert_eq!(tally.jumps(), 1);
        assert_eq!(tally.sneaks(), 1);
        assert_eq!(tally.sprints(), 1);
    }

    assert_eq!(base_target.traveled, Some(Vec3::new(5.0, 0.0, 0.0)));
    assert!(!base_target.jumped); // Grounded suppressed jump
    assert!(base_target.sneaked);

    // Test MutedControls decorator
    let mut motion = KeyMotion::idle();
    motion.register_key(KeyCode::KeyW, true);
    let muted = MutedControls::new(Box::new(motion));
    base_target.traveled = None;
    muted.drive_locomotion(Vec3::Z, Vec3::X, &mut base_target);
    assert!(base_target.traveled.is_none());
}
