use std::time::Instant;

/// Monotonic time abstraction decoupling clock reads from static platform calls.
pub trait MonotonicTime: Send + Sync {
    fn current_instant(&self) -> Instant;
}

/// Real-time platform clock reading hardware monotonic counter.
pub struct SystemClock;

impl SystemClock {
    /// Primary constructor doing nothing except creating instance.
    pub fn new() -> Self {
        Self
    }
}

impl MonotonicTime for SystemClock {
    fn current_instant(&self) -> Instant {
        Instant::now()
    }
}

/// Time decorator shifting the monotonic timeline by an encapsulated duration offset.
#[cfg(test)]
pub struct OffsetClock {
    clock: Box<dyn MonotonicTime>,
    offset: std::time::Duration,
}

#[cfg(test)]
impl OffsetClock {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(clock: Box<dyn MonotonicTime>, offset: std::time::Duration) -> Self {
        Self { clock, offset }
    }
}

#[cfg(test)]
impl MonotonicTime for OffsetClock {
    fn current_instant(&self) -> Instant {
        self.clock.current_instant() + self.offset
    }
}

/// Time decorator freezing the monotonic timeline at an anchored instant.
#[cfg(test)]
pub struct FrozenClock {
    instant: Instant,
}

#[cfg(test)]
impl FrozenClock {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(instant: Instant) -> Self {
        Self { instant }
    }

    /// Secondary constructor anchoring the frozen clock at an existing clock's current instant.
    pub fn at(clock: &dyn MonotonicTime) -> Self {
        Self::new(clock.current_instant())
    }
}

#[cfg(test)]
impl MonotonicTime for FrozenClock {
    fn current_instant(&self) -> Instant {
        self.instant
    }
}

/// Deterministic mock clock encapsulating a controllable instant for testing.
#[cfg(test)]
pub struct ControllableClock {
    current: std::sync::Arc<std::sync::Mutex<Instant>>,
}

#[cfg(test)]
impl ControllableClock {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(current: std::sync::Arc<std::sync::Mutex<Instant>>) -> Self {
        Self { current }
    }

    /// Secondary constructor anchoring the controllable clock at a given moment.
    pub fn starting_at(moment: Instant) -> Self {
        Self::new(std::sync::Arc::new(std::sync::Mutex::new(moment)))
    }

    /// Creates an attached replica sharing the synchronized timeline.
    pub fn replica(&self) -> Self {
        Self::new(self.current.clone())
    }

    /// Advances the shared timeline forward by elapsed duration.
    pub fn advance_by(&self, duration: std::time::Duration) {
        if let Ok(mut guard) = self.current.lock() {
            *guard += duration;
        }
    }
}

#[cfg(test)]
impl MonotonicTime for ControllableClock {
    fn current_instant(&self) -> Instant {
        self.current.lock().map(|g| *g).unwrap_or_else(|_| Instant::now())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_clock_decorators() {
        let base_instant = Instant::now();
        let frozen = FrozenClock::new(base_instant);
        assert_eq!(frozen.current_instant(), base_instant);

        let from_clock = FrozenClock::at(&frozen);
        assert_eq!(from_clock.current_instant(), base_instant);

        let offset_clock = OffsetClock::new(Box::new(frozen), Duration::from_millis(500));
        assert_eq!(offset_clock.current_instant(), base_instant + Duration::from_millis(500));
    }
}
