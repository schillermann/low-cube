use glam::Vec3;
use winit::keyboard::KeyCode;
use crate::controls::{
    ActionBindings, ChordGrip, ControlAction, InputTrigger, KeyMotion, KeyboardControls,
    LocomotionTarget, MutedControls, SprintChord, SprintControls, SprintTarget,
};
use crate::gait::{LocomotionGait, SprintGait, WalkGait};
use crate::lens::PerspectiveLens;

struct MockLocomotionTarget {
    traveled: Option<Vec3>,
    jumped: bool,
    sneaked: bool,
    sprinted: bool,
}

impl MockLocomotionTarget {
    fn fresh() -> Self {
        Self {
            traveled: None,
            jumped: false,
            sneaked: false,
            sprinted: false,
        }
    }
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

    fn sprint(&mut self) {
        self.sprinted = true;
    }
}

#[test]
fn test_sprint_chord_and_target_decorator_composition() {
    let bindings = ActionBindings::seeded();
    let calm = SprintChord::calm();
    assert_eq!(calm, SprintChord::Calm(ChordGrip::released()));

    let with_ctrl = calm.transition(InputTrigger::Key(KeyCode::ControlLeft), true, &bindings);
    let latched = with_ctrl.transition(InputTrigger::Key(KeyCode::KeyW), true, &bindings);
    let sustained = latched.transition(InputTrigger::Key(KeyCode::ControlLeft), false, &bindings);
    assert!(matches!(sustained, SprintChord::Latched(_)));

    let mut plain = MockLocomotionTarget::fresh();
    {
        let mut promoted = SprintTarget::new(&mut plain);
        promoted.travel(Vec3::Z);
    }
    assert_eq!(plain.traveled, Some(Vec3::Z));
    assert!(plain.sprinted);

    // Pure KeyMotion remains strictly momentary without sprint-latch state
    let mut momentary = KeyMotion::idle();
    momentary.register_key(KeyCode::ControlLeft, true);
    momentary.register_key(KeyCode::KeyW, true);
    let mut raw_target = MockLocomotionTarget::fresh();
    momentary.drive_locomotion(Vec3::Z, Vec3::X, &mut raw_target);
    assert_eq!(raw_target.traveled, Some(Vec3::Z));
    assert!(!raw_target.sprinted);

    // MutedControls composes over SprintControls
    let mut sprint_ctrl = SprintControls::idle();
    sprint_ctrl.register_key(KeyCode::ControlLeft, true);
    sprint_ctrl.register_key(KeyCode::KeyW, true);
    let muted = MutedControls::new(Box::new(sprint_ctrl));
    let mut muted_target = MockLocomotionTarget::fresh();
    muted.drive_locomotion(Vec3::Z, Vec3::X, &mut muted_target);
    assert!(muted_target.traveled.is_none());
    assert!(!muted_target.sprinted);
}

#[test]
fn test_sprint_gait_pacing_and_lens_expansion() {
    let sprint = SprintGait::swift();
    let walk = WalkGait::upright();

    let base_distance = 1.0;
    assert!(sprint.pace_distance(base_distance) > walk.pace_distance(base_distance));
    assert_eq!(sprint.pace_distance(10.0), 16.0);

    let mut lens = PerspectiveLens::human_eye();
    assert!(!lens.expanded());

    for _ in 0..30 {
        lens = sprint.shape_lens(lens, 1.0 / 60.0);
    }
    assert!(lens.expanded());

    for _ in 0..30 {
        lens = walk.shape_lens(lens, 1.0 / 60.0);
    }
    assert!(!lens.expanded());
}

#[test]
fn test_ctrl_w_latches_sprint_until_forward_released() {
    let mut motion = SprintControls::idle();
    let mut target = MockLocomotionTarget::fresh();

    // Hold Ctrl + W + A -> starts diagonal forward sprint
    motion.register_key(KeyCode::ControlLeft, true);
    motion.register_key(KeyCode::KeyW, true);
    motion.register_key(KeyCode::KeyA, true);
    motion.drive_locomotion(Vec3::Z, Vec3::X, &mut target);
    assert!(target.traveled.is_some());
    assert!(target.sprinted);

    // Releasing Ctrl and A while keeping W held keeps sprint latched
    motion.register_key(KeyCode::ControlLeft, false);
    motion.register_key(KeyCode::KeyA, false);
    target = MockLocomotionTarget::fresh();
    motion.drive_locomotion(Vec3::Z, Vec3::X, &mut target);
    assert_eq!(target.traveled, Some(Vec3::Z));
    assert!(target.sprinted);

    // Releasing W halts sprint
    motion.register_key(KeyCode::KeyW, false);
    target = MockLocomotionTarget::fresh();
    motion.drive_locomotion(Vec3::Z, Vec3::X, &mut target);
    assert!(!target.sprinted);

    // Next plain W press walks without sprinting
    motion.register_key(KeyCode::KeyW, true);
    target = MockLocomotionTarget::fresh();
    motion.drive_locomotion(Vec3::Z, Vec3::X, &mut target);
    assert_eq!(target.traveled, Some(Vec3::Z));
    assert!(!target.sprinted);
}

#[test]
fn test_tapping_ctrl_while_walking_forward_latches_sprint_and_backward_or_sneak_cancels() {
    let mut motion = SprintControls::idle();
    let mut target = MockLocomotionTarget::fresh();

    // Start walking forward with W, then tap and release Ctrl
    motion.register_key(KeyCode::KeyW, true);
    motion.drive_locomotion(Vec3::Z, Vec3::X, &mut target);
    assert!(!target.sprinted);

    motion.register_key(KeyCode::ControlLeft, true);
    motion.register_key(KeyCode::ControlLeft, false);
    target = MockLocomotionTarget::fresh();
    motion.drive_locomotion(Vec3::Z, Vec3::X, &mut target);
    assert!(target.sprinted);

    // Tapping S (MoveBackward) cancels unheld sprint latch
    motion.register_key(KeyCode::KeyS, true);
    motion.register_key(KeyCode::KeyS, false);
    target = MockLocomotionTarget::fresh();
    motion.drive_locomotion(Vec3::Z, Vec3::X, &mut target);
    assert!(!target.sprinted);

    // Re-latch sprint with Ctrl, then verify Sneak cancels unheld sprint latch
    motion.register_key(KeyCode::ControlLeft, true);
    motion.register_key(KeyCode::ControlLeft, false);
    motion.register_key(KeyCode::ShiftLeft, true);
    motion.register_key(KeyCode::ShiftLeft, false);
    target = MockLocomotionTarget::fresh();
    motion.drive_locomotion(Vec3::Z, Vec3::X, &mut target);
    assert!(!target.sprinted);
}

#[test]
fn test_double_tap_w_never_sprints_and_custom_sprint_binding_works() {
    let mut motion = SprintControls::idle();
    let mut target = MockLocomotionTarget::fresh();

    // Tapping and releasing Ctrl before pressing W does not sprint
    motion.register_key(KeyCode::ControlLeft, true);
    motion.register_key(KeyCode::ControlLeft, false);

    // Double-tapping W without Ctrl never triggers sprint
    motion.register_key(KeyCode::KeyW, true);
    motion.register_key(KeyCode::KeyW, false);
    motion.register_key(KeyCode::KeyW, true);
    motion.drive_locomotion(Vec3::Z, Vec3::X, &mut target);
    assert_eq!(target.traveled, Some(Vec3::Z));
    assert!(!target.sprinted);

    // Rebinding Sprint to KeyR works cleanly
    let custom = ActionBindings::seeded().with_key_binding(ControlAction::Sprint, KeyCode::KeyR);
    motion.apply_bindings(custom);
    motion.register_key(KeyCode::KeyW, true);
    motion.register_key(KeyCode::ControlLeft, true);
    target = MockLocomotionTarget::fresh();
    motion.drive_locomotion(Vec3::Z, Vec3::X, &mut target);
    assert!(!target.sprinted);

    motion.register_key(KeyCode::KeyR, true);
    target = MockLocomotionTarget::fresh();
    motion.drive_locomotion(Vec3::Z, Vec3::X, &mut target);
    assert!(target.sprinted);
}
