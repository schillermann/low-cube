use glam::Vec3;
use winit::keyboard::KeyCode;
use super::{KeyboardControls, LocomotionTarget};

/// Decorator scaling travel locomotion direction vectors by an encapsulated damping factor.
#[cfg(test)]
pub struct DampedLocomotionTarget<'a> {
    target: &'a mut dyn LocomotionTarget,
    damping: f32,
}

#[cfg(test)]
impl<'a> DampedLocomotionTarget<'a> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(target: &'a mut dyn LocomotionTarget, damping: f32) -> Self {
        Self { target, damping }
    }
}

#[cfg(test)]
impl<'a> LocomotionTarget for DampedLocomotionTarget<'a> {
    fn travel(&mut self, direction: Vec3) {
        self.target.travel(direction * self.damping);
    }

    fn jump(&mut self) {
        self.target.jump();
    }

    fn sneak(&mut self) {
        self.target.sneak();
    }

    fn sprint(&mut self) {
        self.target.sprint();
    }
}

/// Decorator suppressing jump intents during ground-locked locomotion.
#[cfg(test)]
pub struct GroundedLocomotionTarget<'a> {
    target: &'a mut dyn LocomotionTarget,
}

#[cfg(test)]
impl<'a> GroundedLocomotionTarget<'a> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(target: &'a mut dyn LocomotionTarget) -> Self {
        Self { target }
    }
}

#[cfg(test)]
impl<'a> LocomotionTarget for GroundedLocomotionTarget<'a> {
    fn travel(&mut self, direction: Vec3) {
        self.target.travel(direction);
    }

    fn jump(&mut self) {
        // Suppressed
    }

    fn sneak(&mut self) {
        self.target.sneak();
    }

    fn sprint(&mut self) {
        self.target.sprint();
    }
}

/// Encapsulated tally of dispatched locomotion actions.
#[cfg(test)]
struct LocomotionTally {
    travels: usize,
    jumps: usize,
    sneaks: usize,
    sprints: usize,
}

#[cfg(test)]
impl LocomotionTally {
    fn zero() -> Self {
        Self {
            travels: 0,
            jumps: 0,
            sneaks: 0,
            sprints: 0,
        }
    }
}

/// Decorator counting dispatched locomotion instructions for testing and telemetry.
#[cfg(test)]
pub struct TallyLocomotionTarget<'a> {
    target: &'a mut dyn LocomotionTarget,
    tally: LocomotionTally,
}

#[cfg(test)]
impl<'a> TallyLocomotionTarget<'a> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(target: &'a mut dyn LocomotionTarget) -> Self {
        Self {
            target,
            tally: LocomotionTally::zero(),
        }
    }

    pub fn travels(&self) -> usize {
        self.tally.travels
    }

    pub fn jumps(&self) -> usize {
        self.tally.jumps
    }

    pub fn sneaks(&self) -> usize {
        self.tally.sneaks
    }

    pub fn sprints(&self) -> usize {
        self.tally.sprints
    }
}

#[cfg(test)]
impl<'a> LocomotionTarget for TallyLocomotionTarget<'a> {
    fn travel(&mut self, direction: Vec3) {
        self.tally.travels += 1;
        self.target.travel(direction);
    }

    fn jump(&mut self) {
        self.tally.jumps += 1;
        self.target.jump();
    }

    fn sneak(&mut self) {
        self.tally.sneaks += 1;
        self.target.sneak();
    }

    fn sprint(&mut self) {
        self.tally.sprints += 1;
        self.target.sprint();
    }
}

/// Decorator muting player locomotion drive (e.g. while paused or in inventory).
#[cfg(test)]
pub struct MutedControls {
    inner: Box<dyn KeyboardControls>,
}

#[cfg(test)]
impl MutedControls {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(inner: Box<dyn KeyboardControls>) -> Self {
        Self { inner }
    }
}

#[cfg(test)]
impl KeyboardControls for MutedControls {
    fn register_key(&mut self, code: KeyCode, pressed: bool) {
        self.inner.register_key(code, pressed);
    }

    fn drive_locomotion(
        &self,
        _forward: Vec3,
        _right: Vec3,
        _target: &mut dyn LocomotionTarget,
    ) {
        // Muted
    }
}
