use std::time::{Duration, Instant};
use glam::Vec3;
use winit::keyboard::KeyCode;
use crate::cadence::{PrimedCadence, RestingCadence, SprintCadence};
use crate::controls::{ActionBindings, KeyboardControls, KeyMotion, LocomotionTarget};
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
fn test_cadence_state_transitions() {
    let now = Instant::now();
    let mut target = MockLocomotionTarget::fresh();

    let resting: Box<dyn SprintCadence> = Box::new(RestingCadence::brisk());
    resting.dispatch_sprint(&mut target);
    assert!(!target.sprinted);

    // First press transitions to EngagedCadence
    let engaged = resting.on_press(now);
    engaged.dispatch_sprint(&mut target);
    assert!(!target.sprinted);

    // Release within 100ms transitions to PrimedCadence
    let primed = engaged.on_release(now + Duration::from_millis(100));
    primed.dispatch_sprint(&mut target);
    assert!(!target.sprinted);

    // Second press 150ms after release (under 300ms threshold) transitions to SprintingCadence
    let sprinting = primed.on_press(now + Duration::from_millis(250));
    sprinting.dispatch_sprint(&mut target);
    assert!(target.sprinted);

    // Releasing forward key returns to RestingCadence
    let released = sprinting.on_release(now + Duration::from_millis(500));
    let mut reset_target = MockLocomotionTarget::fresh();
    released.dispatch_sprint(&mut reset_target);
    assert!(!reset_target.sprinted);
}

#[test]
fn test_cadence_timeout_rejects_slow_double_tap() {
    let now = Instant::now();
    let mut target = MockLocomotionTarget::fresh();

    let primed: Box<dyn SprintCadence> = Box::new(PrimedCadence::new(now, Duration::from_millis(300)));

    // Press after 400ms (exceeding 300ms timeout) returns regular EngagedCadence instead of sprinting
    let engaged = primed.on_press(now + Duration::from_millis(400));
    engaged.dispatch_sprint(&mut target);
    assert!(!target.sprinted);
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
fn test_key_motion_double_w_triggers_sprint() {
    let mut motion = KeyMotion::idle();
    let mut target = MockLocomotionTarget::fresh();

    // First tap W
    motion.register_key(KeyCode::KeyW, true);
    motion.drive_locomotion(Vec3::Z, Vec3::X, &mut target);
    assert!(target.traveled.is_some());
    assert!(!target.sprinted);

    // Release W
    motion.register_key(KeyCode::KeyW, false);

    // Immediate second tap W (within double tap window)
    motion.register_key(KeyCode::KeyW, true);
    target = MockLocomotionTarget::fresh();
    motion.drive_locomotion(Vec3::Z, Vec3::X, &mut target);
    assert!(target.traveled.is_some());
    assert!(target.sprinted);

    // Release stops sprint
    motion.register_key(KeyCode::KeyW, false);
    target = MockLocomotionTarget::fresh();
    motion.drive_locomotion(Vec3::Z, Vec3::X, &mut target);
    assert!(!target.sprinted);
}

#[test]
fn test_key_motion_double_w_with_controllable_clock() {
    use std::collections::HashSet;
    use crate::clock::ControllableClock;

    let start = Instant::now();
    let clock = ControllableClock::starting_at(start);
    let mut motion = KeyMotion::new(
        HashSet::new(),
        ActionBindings::seeded(),
        Box::new(RestingCadence::brisk()),
        Box::new(clock.replica()),
    );
    let mut target = MockLocomotionTarget::fresh();

    // First press at t = 0
    motion.register_key(KeyCode::KeyW, true);
    motion.drive_locomotion(Vec3::Z, Vec3::X, &mut target);
    assert!(!target.sprinted);

    // Release at t = 50ms
    clock.advance_by(Duration::from_millis(50));
    motion.register_key(KeyCode::KeyW, false);

    // Second press at t = 200ms (150ms after release, well under 300ms)
    clock.advance_by(Duration::from_millis(150));
    motion.register_key(KeyCode::KeyW, true);
    target = MockLocomotionTarget::fresh();
    motion.drive_locomotion(Vec3::Z, Vec3::X, &mut target);
    assert!(target.sprinted);
}

#[test]
fn test_key_motion_slow_double_w_rejected_by_clock() {
    use std::collections::HashSet;
    use crate::clock::ControllableClock;

    let start = Instant::now();
    let clock = ControllableClock::starting_at(start);
    let mut motion = KeyMotion::new(
        HashSet::new(),
        ActionBindings::seeded(),
        Box::new(RestingCadence::brisk()),
        Box::new(clock.replica()),
    );
    let mut target = MockLocomotionTarget::fresh();

    // First press at t = 0
    motion.register_key(KeyCode::KeyW, true);
    // Release at t = 50ms
    clock.advance_by(Duration::from_millis(50));
    motion.register_key(KeyCode::KeyW, false);

    // Second press at t = 450ms (400ms after release, exceeding 300ms timeout)
    clock.advance_by(Duration::from_millis(400));
    motion.register_key(KeyCode::KeyW, true);
    motion.drive_locomotion(Vec3::Z, Vec3::X, &mut target);
    assert!(!target.sprinted);
}
