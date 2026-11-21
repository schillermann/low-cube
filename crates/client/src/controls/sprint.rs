use glam::Vec3;
use super::{ActionBindings, ControlAction, InputTrigger, KeyMotion, KeyboardControls, LocomotionTarget};

const FORWARD_BIT: u8 = 1 << 0;
const BACKWARD_BIT: u8 = 1 << 1;
const SNEAK_BIT: u8 = 1 << 2;
const SPRINT_BIT: u8 = 1 << 3;

/// Immutable four-action chord bitmask tracking forward, backward, sneak, and sprint triggers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ChordGrip {
    mask: u8,
}

impl ChordGrip {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(mask: u8) -> Self {
        Self { mask }
    }

    /// Secondary constructor representing all chord keys released.
    pub const fn released() -> Self {
        Self::new(0)
    }

    pub fn updated(self, trigger: InputTrigger, pressed: bool, bindings: &ActionBindings) -> Self {
        let mut bit = 0;
        if bindings.matches(ControlAction::MoveForward, trigger) {
            bit |= FORWARD_BIT;
        }
        if bindings.matches(ControlAction::MoveBackward, trigger) {
            bit |= BACKWARD_BIT;
        }
        if bindings.matches(ControlAction::Sneak, trigger) {
            bit |= SNEAK_BIT;
        }
        if bindings.matches(ControlAction::Sprint, trigger) {
            bit |= SPRINT_BIT;
        }
        if pressed {
            Self::new(self.mask | bit)
        } else {
            Self::new(self.mask & !bit)
        }
    }

    pub const fn ignited(self) -> bool {
        self.mask == (FORWARD_BIT | SPRINT_BIT)
    }

    pub const fn obstructed(self) -> bool {
        (self.mask & (FORWARD_BIT | BACKWARD_BIT | SNEAK_BIT)) != FORWARD_BIT
    }
}

/// LocomotionTarget decorator promoting active travel into a sprint stride.
pub struct SprintTarget<'a> {
    origin: &'a mut dyn LocomotionTarget,
}

impl<'a> SprintTarget<'a> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(origin: &'a mut dyn LocomotionTarget) -> Self {
        Self { origin }
    }
}

impl<'a> LocomotionTarget for SprintTarget<'a> {
    fn travel(&mut self, direction: Vec3) {
        self.origin.travel(direction);
        self.origin.sprint();
    }

    fn jump(&mut self) {
        self.origin.jump();
    }

    fn sneak(&mut self) {
        self.origin.sneak();
    }

    fn sprint(&mut self) {
        self.origin.sprint();
    }

    fn roll(&mut self, roll_dir: f32) {
        self.origin.roll(roll_dir);
    }

    fn strafe_shift(&mut self) {
        self.origin.strafe_shift();
    }
}

/// Polymorphic self-governing sprint-chord momentum state (Calm vs. Latched).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SprintChord {
    Calm(ChordGrip),
    Latched(ChordGrip),
}

impl SprintChord {
    /// Secondary constructor initializing calm walking chord state.
    pub const fn calm() -> Self {
        Self::Calm(ChordGrip::released())
    }

    pub fn transition(self, trigger: InputTrigger, pressed: bool, bindings: &ActionBindings) -> Self {
        match self {
            Self::Calm(grip) => {
                let next = grip.updated(trigger, pressed, bindings);
                if next.ignited() { Self::Latched(next) } else { Self::Calm(next) }
            }
            Self::Latched(grip) => {
                let next = grip.updated(trigger, pressed, bindings);
                if next.obstructed() { Self::Calm(next) } else { Self::Latched(next) }
            }
        }
    }

    pub fn drive(
        self,
        origin: &dyn KeyboardControls,
        forward: Vec3,
        right: Vec3,
        target: &mut dyn LocomotionTarget,
    ) {
        match self {
            Self::Calm(_) => origin.drive_locomotion(forward, right, target),
            Self::Latched(_) => {
                let mut promoted = SprintTarget::new(target);
                origin.drive_locomotion(forward, right, &mut promoted);
            }
        }
    }
}

/// KeyboardControls decorator latching forward sprint momentum upon the bound Sprint + MoveForward chord.
pub struct SprintControls<C: KeyboardControls = KeyMotion> {
    origin: C,
    bindings: ActionBindings,
    chord: SprintChord,
}

impl<C: KeyboardControls> SprintControls<C> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(origin: C, bindings: ActionBindings, chord: SprintChord) -> Self {
        Self { origin, bindings, chord }
    }
}

impl SprintControls<KeyMotion> {
    /// Secondary constructor decorating momentary `KeyMotion` with sprint-chord latching.
    pub fn with_bindings(bindings: ActionBindings) -> Self {
        Self::new(KeyMotion::with_bindings(bindings), bindings, SprintChord::calm())
    }

    /// Secondary constructor initializing seeded sprint-decorated controls.
    #[cfg(test)]
    pub fn idle() -> Self {
        Self::with_bindings(ActionBindings::seeded())
    }
}

impl<C: KeyboardControls> KeyboardControls for SprintControls<C> {
    fn register_trigger(&mut self, trigger: InputTrigger, pressed: bool) {
        self.origin.register_trigger(trigger, pressed);
        self.chord = self.chord.transition(trigger, pressed, &self.bindings);
    }

    fn clear_triggers(&mut self) {
        self.origin.clear_triggers();
        self.chord = SprintChord::calm();
    }

    fn apply_bindings(&mut self, bindings: ActionBindings) {
        self.origin.apply_bindings(bindings);
        self.bindings = bindings;
        self.chord = SprintChord::calm();
    }

    fn drive_locomotion(
        &self,
        forward: Vec3,
        right: Vec3,
        target: &mut dyn LocomotionTarget,
    ) {
        self.chord.drive(&self.origin, forward, right, target);
    }
}
