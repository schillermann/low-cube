use crate::aurora_cadence::{AuroraCadence, StochasticAuroraCadence};
use crate::command_flags::CommandFlags;

/// Command-line argument-driven aurora cadence decorator.
pub struct ArgumentAurora {
    cadence: Box<dyn AuroraCadence>,
}

impl ArgumentAurora {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(cadence: Box<dyn AuroraCadence>) -> Self {
        Self { cadence }
    }

    /// Secondary constructor capturing process command-line arguments.
    pub fn process(day_duration: f32, fallback: Box<dyn AuroraCadence>) -> Self {
        let args: Vec<String> = std::env::args().collect();
        Self::from_arguments(&args, day_duration, fallback)
    }

    /// Secondary constructor parsing an explicit arguments slice.
    pub fn from_arguments(args: &[String], day_duration: f32, fallback: Box<dyn AuroraCadence>) -> Self {
        if let Some(mode) = CommandFlags::from_slice(args).value_of("--aurora") {
            Self::new(StochasticAuroraCadence::from_mode(&mode, day_duration))
        } else {
            Self::new(fallback)
        }
    }
}

impl AuroraCadence for ArgumentAurora {
    fn activity(&self, progress: f32, elapsed_seconds: f64, night_factor: f32) -> f32 {
        self.cadence.activity(progress, elapsed_seconds, night_factor)
    }

    fn clone_cadence(&self) -> Box<dyn AuroraCadence> {
        Box::new(Self::new(self.cadence.clone_cadence()))
    }
}
