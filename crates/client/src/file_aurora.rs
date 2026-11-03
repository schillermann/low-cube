use crate::aurora_cadence::{AuroraCadence, StochasticAuroraCadence};
use crate::toml_document::TomlDocument;

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
        let Some(doc) = TomlDocument::from_file(path) else {
            return Self::new(fallback);
        };
        if let Some(mode) = doc.section_property("aurora", "mode", false) {
            Self::new(StochasticAuroraCadence::from_mode(&mode, day_duration))
        } else {
            Self::new(fallback)
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
