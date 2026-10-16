use crate::aurora_cadence::{
    AuroraCadence, DormantAuroraCadence, NightlyAuroraCadence, StochasticAuroraCadence,
};

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
        if let Some(mode) = Self::flag_value(args, "--aurora") {
            let cadence = Self::cadence_for_mode(&mode, day_duration);
            Self::new(cadence)
        } else {
            Self::new(fallback)
        }
    }

    fn flag_value(args: &[String], flag: &str) -> Option<String> {
        args.windows(2)
            .find(|pair| pair[0] == flag)
            .map(|pair| pair[1].clone())
            .or_else(|| {
                args.iter()
                    .find(|arg| arg.starts_with(flag) && arg[flag.len()..].starts_with('='))
                    .and_then(|arg| arg.split_once('='))
                    .map(|(_, val)| val.to_string())
            })
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
                "Invalid --aurora argument '{other}'. Supported modes: 'off', 'nightly', 'rare', 'normal', 'frequent'."
            ),
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
