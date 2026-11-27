use std::fs;
use crate::toml_document::TomlDocument;
use super::proportion::CrewPhysique;
use super::EvaAttire;

/// Focused interface for persistent profile storage of Space-Age EVA attire.
pub trait AttireProfile {
    fn load_or_astronaut(&self) -> EvaAttire;
    fn save(&self, attire: &EvaAttire);
}

/// Persistent TOML storage for customizable EVA suit attire (`1` field).
pub struct FileAttire {
    path: String,
}

impl FileAttire {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(path: String) -> Self {
        Self { path }
    }

    /// Secondary constructor targeting the local `character.toml` profile.
    pub fn profile() -> Self {
        Self::new("character.toml".to_string())
    }
}

impl AttireProfile for FileAttire {
    /// Loads customized EVA attire from file or returns `EvaAttire::astronaut()` when no profile exists yet.
    fn load_or_astronaut(&self) -> EvaAttire {
        let Some(doc) = TomlDocument::from_file(&self.path) else {
            return EvaAttire::astronaut();
        };
        let bytes = [
            doc.parsed_property("shell").unwrap_or(0),
            doc.parsed_property("gauntlet").unwrap_or(1),
            doc.parsed_property("harness").unwrap_or(2),
            doc.parsed_property("boots").unwrap_or(0),
            doc.parsed_property("helm").unwrap_or(0),
            doc.parsed_property("relic").unwrap_or(0),
            doc.parsed_property("pigment").unwrap_or(0),
            doc.parsed_property("complexion").unwrap_or(0),
            doc.parsed_property("physique").unwrap_or(0),
        ];
        let mut attire = EvaAttire::from_multiplayer_bytes(bytes);
        if let (Some(c), Some(t), Some(a), Some(l)) = (
            doc.parsed_property("crown_body"),
            doc.parsed_property("torso_body"),
            doc.parsed_property("arm_body"),
            doc.parsed_property("leg_body"),
        ) {
            attire = attire.with_physique(CrewPhysique::from_region_bytes([c, t, a, l]));
        }
        attire
    }

    /// Persists the player's customized EVA suit attire to the TOML profile, failing fast on I/O errors.
    fn save(&self, attire: &EvaAttire) {
        let existing_name = TomlDocument::from_file(&self.path)
            .and_then(|doc| doc.property("name"))
            .unwrap_or_default();
        let name_line = if existing_name.is_empty() {
            String::new()
        } else {
            format!("name = \"{existing_name}\"\n")
        };
        let bytes = attire.multiplayer_bytes();
        let regions = attire.region_bytes();
        let text = format!(
            "# Space-Age EVA Crew Profile\n[character]\n{name_line}shell = {}\ngauntlet = {}\nharness = {}\nboots = {}\nhelm = {}\nrelic = {}\npigment = {}\ncomplexion = {}\nphysique = {}\ncrown_body = {}\ntorso_body = {}\narm_body = {}\nleg_body = {}\n",
            bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7], bytes[8],
            regions[0], regions[1], regions[2], regions[3]
        );
        fs::write(&self.path, text).expect("Failed to persist EvaAttire profile to disk");
    }
}
