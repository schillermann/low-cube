use std::collections::HashSet;
use glam::Vec3;
#[cfg(test)]
use winit::keyboard::KeyCode;

mod bindings;
mod file_bindings;
#[cfg(test)]
mod modulation;
mod sprint;

pub use bindings::{ActionBindings, ControlAction, InputTrigger};
pub use file_bindings::{BindingProfile, FileBindings};
#[cfg(test)]
pub use modulation::{
    DampedLocomotionTarget, GroundedLocomotionTarget, MutedControls, TallyLocomotionTarget,
};
pub use sprint::SprintControls;
#[cfg(test)]
pub use sprint::{ChordGrip, SprintChord, SprintTarget};

/// Interface for receiving unified locomotion messages ("Tell, Don't Ask").
pub trait LocomotionTarget {
    fn travel(&mut self, direction: Vec3);
    fn jump(&mut self);
    fn sneak(&mut self);
    fn sprint(&mut self);
}

/// Interface representing player keyboard navigation state and bound action matching.
pub trait KeyboardControls {
    #[cfg(test)]
    fn register_key(&mut self, code: KeyCode, pressed: bool) {
        self.register_trigger(InputTrigger::Key(code), pressed);
    }
    fn register_trigger(&mut self, _trigger: InputTrigger, _pressed: bool) {}
    fn clear_triggers(&mut self) {}
    fn apply_bindings(&mut self, _bindings: ActionBindings) {}
    fn drive_locomotion(
        &self,
        forward: Vec3,
        right: Vec3,
        target: &mut dyn LocomotionTarget,
    );
}

/// Momentary trigger registry mapping held directional, jump, and sneak actions onto a `LocomotionTarget`.
pub struct KeyMotion {
    active_triggers: HashSet<InputTrigger>,
    bindings: ActionBindings,
}

impl KeyMotion {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(active_triggers: HashSet<InputTrigger>, bindings: ActionBindings) -> Self {
        Self {
            active_triggers,
            bindings,
        }
    }

    /// Secondary constructor initializing with an empty trigger registry and given bindings.
    pub fn with_bindings(bindings: ActionBindings) -> Self {
        Self::new(HashSet::new(), bindings)
    }

    /// Secondary constructor initializing with an empty trigger registry and seeded bindings.
    #[cfg(test)]
    pub fn idle() -> Self {
        Self::with_bindings(ActionBindings::seeded())
    }

    fn held(&self, action: ControlAction) -> bool {
        self.bindings.active_in(action, &self.active_triggers)
    }
}

impl KeyboardControls for KeyMotion {
    fn register_trigger(&mut self, trigger: InputTrigger, pressed: bool) {
        if pressed {
            self.active_triggers.insert(trigger);
        } else {
            self.active_triggers.remove(&trigger);
        }
    }

    fn clear_triggers(&mut self) {
        self.active_triggers.clear();
    }

    fn apply_bindings(&mut self, bindings: ActionBindings) {
        self.bindings = bindings;
        self.clear_triggers();
    }

    fn drive_locomotion(
        &self,
        forward: Vec3,
        right: Vec3,
        target: &mut dyn LocomotionTarget,
    ) {
        let mut direction = Vec3::ZERO;
        if self.held(ControlAction::MoveForward) {
            direction += forward;
        }
        if self.held(ControlAction::MoveBackward) {
            direction -= forward;
        }
        if self.held(ControlAction::MoveRight) {
            direction += right;
        }
        if self.held(ControlAction::MoveLeft) {
            direction -= right;
        }

        if direction.length_squared() > 1e-4 {
            target.travel(direction.normalize());
        }

        if self.held(ControlAction::Jump) {
            target.jump();
        }

        if self.held(ControlAction::Sneak) {
            target.sneak();
        }
    }
}
