use std::fs;
use crate::aurora_cadence::{
    AuroraCadence, DormantAuroraCadence, NightlyAuroraCadence, StochasticAuroraCadence,
};

/// Configuration file-driven aurora cadence decorator.
pub struct FileAurora {
    cadence: Box<dyn AuroraCadence>,
}

impl FileAurora {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(cadence: Box<dyn AuroraCadence>) -> Self {
        Self { cadence }
    }

    /// Secondary constructor targeting the server.toml configuration file.
    pub fn server(day_duration: f32, fallback: Box<dyn AuroraCadence>) -> Self {
        Self::from_path("server.toml", day_duration, fallback)
    }

    /// Secondary constructor loading aurora configuration from an explicit file path.
    pub fn from_path(path: &str, day_duration: f32, fallback: Box<dyn AuroraCadence>) -> Self {
        let content = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => return Self::new(fallback),
        };
        if let Some(mode) = Self::extract_mode(&content) {
            let cadence = Self::cadence_for_mode(&mode, day_duration);
            Self::new(cadence)
        } else {
            Self::new(fallback)
        }
    }

    fn extract_mode(content: &str) -> Option<String> {
        let mut in_aurora = false;
        for line in content.lines().map(str::trim) {
            if line.starts_with('[') && line.ends_with(']') {
                let section = line[1..line.len() - 1].trim();
                in_aurora = section.eq_ignore_ascii_case("aurora");
                continue;
            }
            if in_aurora && !line.is_empty() && !line.starts_with('#') {
                if let Some((k, v)) = line.split_once('=') {
                    if k.trim().eq_ignore_ascii_case("mode") {
                        let raw = v.split('#').next().unwrap_or("").trim();
                        let unquoted = raw.trim_matches('"').trim_matches('\'').trim();
                        return Some(unquoted.to_string());
                    }
                }
            }
        }
        None
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
                "Invalid aurora mode '{other}' in configuration. Supported modes: 'off', 'nightly', 'rare', 'normal', 'frequent'."
            ),
        }
    }
}

impl AuroraCadence for FileAurora {
    fn activity(&self, progress: f32, elapsed_seconds: f64, night_factor: f32) -> f32 {
        self.cadence.activity(progress, elapsed_seconds, night_factor)
    }

    fn clone_cadence(&self) -> Box<dyn AuroraCadence> {
        Box::new(Self::new(self.cadence.clone_cadence()))
    }
}
