use std::collections::HashSet;
use glam::Vec3;
#[cfg(test)]
use winit::keyboard::KeyCode;
use crate::cadence::{RestingCadence, SprintCadence, TransitioningCadence};
use crate::clock::{MonotonicTime, SystemClock};

mod bindings;
mod file_bindings;
#[cfg(test)]
mod modulation;

pub use bindings::{ActionBindings, ControlAction, InputTrigger, TriggerNotation};
pub use file_bindings::{BindingProfile, FileBindings};
#[cfg(test)]
pub use modulation::{
    DampedLocomotionTarget, GroundedLocomotionTarget, MutedControls, TallyLocomotionTarget,
};

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
    fn apply_bindings(&mut self, _bindings: ActionBindings) {}
    fn matches_action(&self, _action: ControlAction, _trigger: InputTrigger) -> bool { false }
    fn drive_locomotion(
        &self,
        forward: Vec3,
        right: Vec3,
        target: &mut dyn LocomotionTarget,
    );
}

/// Encapsulated active trigger registry implementing KeyboardControls with single-trigger bindings.
pub struct KeyMotion {
    active_triggers: HashSet<InputTrigger>,
    bindings: ActionBindings,
    cadence: Box<dyn SprintCadence>,
    clock: Box<dyn MonotonicTime>,
}

impl KeyMotion {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(
        active_triggers: HashSet<InputTrigger>,
        bindings: ActionBindings,
        cadence: Box<dyn SprintCadence>,
        clock: Box<dyn MonotonicTime>,
    ) -> Self {
        Self {
            active_triggers,
            bindings,
            cadence,
            clock,
        }
    }

    /// Secondary constructor initializing with an empty trigger registry, given bindings, brisk cadence, and system clock.
    pub fn with_bindings(bindings: ActionBindings) -> Self {
        Self::new(
            HashSet::new(),
            bindings,
            Box::new(RestingCadence::brisk()),
            Box::new(SystemClock::new()),
        )
    }

    /// Secondary constructor initializing with an empty trigger registry, seeded bindings, brisk cadence, and system clock.
    #[cfg(test)]
    pub fn idle() -> Self {
        Self::with_bindings(ActionBindings::seeded())
    }
}

impl KeyboardControls for KeyMotion {
    fn register_trigger(&mut self, trigger: InputTrigger, pressed: bool) {
        if pressed {
            self.active_triggers.insert(trigger);
        } else {
            self.active_triggers.remove(&trigger);
        }

        if self.bindings.matches(ControlAction::MoveForward, trigger) {
            let now = self.clock.current_instant();
            let current = std::mem::replace(&mut self.cadence, Box::new(TransitioningCadence));
            self.cadence = if pressed {
                current.on_press(now)
            } else {
                current.on_release(now)
            };
        } else if self.bindings.matches(ControlAction::MoveBackward, trigger) && pressed {
            self.cadence = Box::new(RestingCadence::brisk());
        }
    }

    fn apply_bindings(&mut self, bindings: ActionBindings) {
        self.bindings = bindings;
        self.active_triggers.clear();
        self.cadence = Box::new(RestingCadence::brisk());
    }

    fn matches_action(&self, action: ControlAction, trigger: InputTrigger) -> bool {
        self.bindings.matches(action, trigger)
    }

    fn drive_locomotion(
        &self,
        forward: Vec3,
        right: Vec3,
        target: &mut dyn LocomotionTarget,
    ) {
        let mut direction = Vec3::ZERO;
        if self.active_triggers.contains(&self.bindings.trigger_for(ControlAction::MoveForward)) {
            direction += forward;
        }
        if self.active_triggers.contains(&self.bindings.trigger_for(ControlAction::MoveBackward)) {
            direction -= forward;
        }
        if self.active_triggers.contains(&self.bindings.trigger_for(ControlAction::MoveRight)) {
            direction += right;
        }
        if self.active_triggers.contains(&self.bindings.trigger_for(ControlAction::MoveLeft)) {
            direction -= right;
        }

        if direction.length_squared() > 1e-4 {
            target.travel(direction.normalize());
            if self.active_triggers.contains(&self.bindings.trigger_for(ControlAction::MoveForward)) {
                self.cadence.dispatch_sprint(target);
            }
        }

        if self.active_triggers.contains(&self.bindings.trigger_for(ControlAction::Jump)) {
            target.jump();
        }

        if self.active_triggers.contains(&self.bindings.trigger_for(ControlAction::Sneak)) {
            target.sneak();
        }
    }
}
