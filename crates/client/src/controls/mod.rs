use std::collections::HashSet;
use glam::Vec3;
use winit::keyboard::KeyCode;
use crate::cadence::{RestingCadence, SprintCadence, TransitioningCadence};
use crate::clock::{MonotonicTime, SystemClock};

#[cfg(test)]
mod decorator;

#[cfg(test)]
pub use decorator::{
    DampedLocomotionTarget, GroundedLocomotionTarget, MutedControls, TallyLocomotionTarget,
};

/// Interface for receiving unified locomotion messages ("Tell, Don't Ask").
pub trait LocomotionTarget {
    fn travel(&mut self, direction: Vec3);
    fn jump(&mut self);
    fn sneak(&mut self);
    fn sprint(&mut self);
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
    cadence: Box<dyn SprintCadence>,
    clock: Box<dyn MonotonicTime>,
}

impl KeyMotion {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(
        active_keys: HashSet<KeyCode>,
        cadence: Box<dyn SprintCadence>,
        clock: Box<dyn MonotonicTime>,
    ) -> Self {
        Self {
            active_keys,
            cadence,
            clock,
        }
    }

    /// Secondary constructor initializing with an empty key registry, brisk cadence, and system clock.
    pub fn idle() -> Self {
        Self::new(
            HashSet::new(),
            Box::new(RestingCadence::brisk()),
            Box::new(SystemClock::new()),
        )
    }
}

impl KeyboardControls for KeyMotion {
    fn register_key(&mut self, code: KeyCode, pressed: bool) {
        if pressed {
            self.active_keys.insert(code);
        } else {
            self.active_keys.remove(&code);
        }

        if code == KeyCode::KeyW {
            let now = self.clock.current_instant();
            let current = std::mem::replace(&mut self.cadence, Box::new(TransitioningCadence));
            self.cadence = if pressed {
                current.on_press(now)
            } else {
                current.on_release(now)
            };
        } else if code == KeyCode::KeyS && pressed {
            self.cadence = Box::new(RestingCadence::brisk());
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
            if self.active_keys.contains(&KeyCode::KeyW) {
                self.cadence.dispatch_sprint(target);
            }
        }

        if self.active_keys.contains(&KeyCode::Space) {
            target.jump();
        }

        if self.active_keys.contains(&KeyCode::ShiftLeft)
            || self.active_keys.contains(&KeyCode::ShiftRight)
        {
            target.sneak();
        }
    }
}
