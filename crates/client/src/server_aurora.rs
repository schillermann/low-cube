use crate::argument_aurora::ArgumentAurora;
use crate::aurora_cadence::{AuroraCadence, NightlyAuroraCadence};
use crate::environment_aurora::EnvironmentAurora;
use crate::file_aurora::FileAurora;

/// Server launch aurora cadence decorator coordinating CLI arguments, environment variables, config file, and Nightly baseline.
pub struct ServerAurora {
    cadence: Box<dyn AuroraCadence>,
}

impl ServerAurora {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(cadence: Box<dyn AuroraCadence>) -> Self {
        Self { cadence }
    }

    /// Secondary constructor assembling the full decorator chain in priority order.
    pub fn assembled(day_duration: f32) -> Self {
        Self::new(Box::new(ArgumentAurora::process(
            day_duration,
            Box::new(EnvironmentAurora::process(
                day_duration,
                Box::new(FileAurora::server(
                    day_duration,
                    Box::new(NightlyAuroraCadence::new()),
                )),
            )),
        )))
    }
}

impl AuroraCadence for ServerAurora {
    fn activity(&self, progress: f32, elapsed_seconds: f64, night_factor: f32) -> f32 {
        self.cadence.activity(progress, elapsed_seconds, night_factor)
    }

    fn clone_cadence(&self) -> Box<dyn AuroraCadence> {
        Box::new(Self::new(self.cadence.clone_cadence()))
    }
}
