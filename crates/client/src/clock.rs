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
