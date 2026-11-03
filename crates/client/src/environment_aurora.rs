use crate::aurora_cadence::{AuroraCadence, StochasticAuroraCadence};

/// Environment variable-driven aurora cadence decorator.
pub struct EnvironmentAurora {
    cadence: Box<dyn AuroraCadence>,
}

impl EnvironmentAurora {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(cadence: Box<dyn AuroraCadence>) -> Self {
        Self { cadence }
    }

    /// Secondary constructor reading process environment variables.
    pub fn process(day_duration: f32, fallback: Box<dyn AuroraCadence>) -> Self {
        if let Ok(raw_mode) = std::env::var("LOW_CUBE_AURORA") {
            Self::new(StochasticAuroraCadence::from_mode(&raw_mode, day_duration))
        } else {
            Self::new(fallback)
        }
    }
}

impl AuroraCadence for EnvironmentAurora {
    fn activity(&self, progress: f32, elapsed_seconds: f64, night_factor: f32) -> f32 {
        self.cadence.activity(progress, elapsed_seconds, night_factor)
    }

    fn clone_cadence(&self) -> Box<dyn AuroraCadence> {
        Box::new(Self::new(self.cadence.clone_cadence()))
    }
}
