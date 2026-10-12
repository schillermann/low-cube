use std::time::{Duration, Instant};
use crate::controls::LocomotionTarget;

/// Polymorphic double-tap rhythm tracking rapid forward key strokes.
pub trait SprintCadence: Send + Sync {
    fn on_press(self: Box<Self>, now: Instant) -> Box<dyn SprintCadence>;
    fn on_release(self: Box<Self>, now: Instant) -> Box<dyn SprintCadence>;
    fn dispatch_sprint(&self, target: &mut dyn LocomotionTarget);
}

/// Transient state used during atomic cadence state transition.
pub struct TransitioningCadence;

impl SprintCadence for TransitioningCadence {
    fn on_press(self: Box<Self>, _now: Instant) -> Box<dyn SprintCadence> {
        self
    }

    fn on_release(self: Box<Self>, _now: Instant) -> Box<dyn SprintCadence> {
        self
    }

    fn dispatch_sprint(&self, _target: &mut dyn LocomotionTarget) {}
}

/// Quiescent state with no recent forward key activity.
pub struct RestingCadence {
    timeout: Duration,
}

impl RestingCadence {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(timeout: Duration) -> Self {
        Self { timeout }
    }

    /// Secondary constructor with brisk double-tap window.
    pub fn brisk() -> Self {
        Self::new(Duration::from_millis(300))
    }
}

impl SprintCadence for RestingCadence {
    fn on_press(self: Box<Self>, _now: Instant) -> Box<dyn SprintCadence> {
        Box::new(EngagedCadence::new(self.timeout))
    }

    fn on_release(self: Box<Self>, _now: Instant) -> Box<dyn SprintCadence> {
        self
    }

    fn dispatch_sprint(&self, _target: &mut dyn LocomotionTarget) {}
}

/// Initial press state where forward key is held for regular walking.
pub struct EngagedCadence {
    timeout: Duration,
}

impl EngagedCadence {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(timeout: Duration) -> Self {
        Self { timeout }
    }
}

impl SprintCadence for EngagedCadence {
    fn on_press(self: Box<Self>, _now: Instant) -> Box<dyn SprintCadence> {
        self
    }

    fn on_release(self: Box<Self>, now: Instant) -> Box<dyn SprintCadence> {
        Box::new(PrimedCadence::new(now, self.timeout))
    }

    fn dispatch_sprint(&self, _target: &mut dyn LocomotionTarget) {}
}

/// Primed state where forward key was recently released, awaiting potential double-tap.
pub struct PrimedCadence {
    released_at: Instant,
    timeout: Duration,
}

impl PrimedCadence {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(released_at: Instant, timeout: Duration) -> Self {
        Self {
            released_at,
            timeout,
        }
    }
}

impl SprintCadence for PrimedCadence {
    fn on_press(self: Box<Self>, now: Instant) -> Box<dyn SprintCadence> {
        if now.duration_since(self.released_at) <= self.timeout {
            Box::new(SprintingCadence::new(self.timeout))
        } else {
            Box::new(EngagedCadence::new(self.timeout))
        }
    }

    fn on_release(self: Box<Self>, _now: Instant) -> Box<dyn SprintCadence> {
        self
    }

    fn dispatch_sprint(&self, _target: &mut dyn LocomotionTarget) {}
}

/// Active sprinting state sustained as long as forward key remains held down.
pub struct SprintingCadence {
    timeout: Duration,
}

impl SprintingCadence {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(timeout: Duration) -> Self {
        Self { timeout }
    }
}

impl SprintCadence for SprintingCadence {
    fn on_press(self: Box<Self>, _now: Instant) -> Box<dyn SprintCadence> {
        self
    }

    fn on_release(self: Box<Self>, _now: Instant) -> Box<dyn SprintCadence> {
        Box::new(RestingCadence::new(self.timeout))
    }

    fn dispatch_sprint(&self, target: &mut dyn LocomotionTarget) {
        target.sprint();
    }
}
