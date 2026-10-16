use crate::aurora_cadence::{
    AuroraCadence, DormantAuroraCadence, NightlyAuroraCadence, StochasticAuroraCadence,
};

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
            let cadence = Self::cadence_for_mode(&raw_mode, day_duration);
            Self::new(cadence)
        } else {
            Self::new(fallback)
        }
    }

    fn cadence_for_mode(mode: &str, day_duration: f32) -> Box<dyn AuroraCadence> {
        let trimmed = mode.trim().to_ascii_lowercase();
        match trimmed.as_str() {
            "off" | "aus" | "disabled" | "dormant" | "none" | "false" | "0" => {
                Box::new(DormantAuroraCadence::new())
            }
            "nightly" | "jede_nacht" | "every_night" | "always" | "true" | "1" => {
                Box::new(NightlyAuroraCadence::new())
            }
            "rare" | "selten" | "seldom" => {
                Box::new(StochasticAuroraCadence::rare(day_duration, Box::new(NightlyAuroraCadence::new())))
            }
            "normal" | "regular" | "moderate" | "random" | "zufall" | "stochastic" => {
                Box::new(StochasticAuroraCadence::normal(day_duration, Box::new(NightlyAuroraCadence::new())))
            }
            "frequent" | "often" | "haeufig" => {
                Box::new(StochasticAuroraCadence::frequent(day_duration, Box::new(NightlyAuroraCadence::new())))
            }
            _ if mode.eq_ignore_ascii_case("häufig") => {
                Box::new(StochasticAuroraCadence::frequent(day_duration, Box::new(NightlyAuroraCadence::new())))
            }
            other => panic!(
                "Invalid LOW_CUBE_AURORA mode '{other}'. Supported modes: 'off', 'nightly', 'rare', 'normal', 'frequent'."
            ),
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
