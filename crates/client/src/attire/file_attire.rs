use std::fs;
use crate::toml_document::TomlDocument;
use super::proportion::YouthPhysique;
use super::YouthAttire;

/// Focused interface for persistent profile storage of medieval attire.
pub trait AttireProfile {
    fn load_or_squire(&self) -> YouthAttire;
    fn save(&self, attire: &YouthAttire);
}

/// Persistent TOML storage for customizable medieval attire.
pub struct FileAttire {
    path: String,
}

impl FileAttire {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(path: String) -> Self {
        Self { path }
    }

    /// Secondary constructor targeting the local character.toml profile.
    pub fn profile() -> Self {
        Self::new("character.toml".to_string())
    }
}

impl AttireProfile for FileAttire {
    /// Loads customized attire from file or returns squire attire when no profile exists yet.
    fn load_or_squire(&self) -> YouthAttire {
        let Some(doc) = TomlDocument::from_file(&self.path) else {
            return YouthAttire::squire();
        };
        let bytes = [
            doc.parsed_property("tunic").unwrap_or(0),
            doc.parsed_property("vambrace").unwrap_or(1),
            doc.parsed_property("girdle").unwrap_or(0),
            doc.parsed_property("footwear").unwrap_or(0),
            doc.parsed_property("crown").unwrap_or(0),
            doc.parsed_property("relic").unwrap_or(0),
            doc.parsed_property("pigment").unwrap_or(0),
            doc.parsed_property("complexion").unwrap_or(0),
            doc.parsed_property("physique").unwrap_or(0),
        ];
        let mut attire = YouthAttire::from_multiplayer_bytes(bytes);
        if let (Some(c), Some(t), Some(a), Some(l)) = (
            doc.parsed_property("crown_body"),
            doc.parsed_property("torso_body"),
            doc.parsed_property("arm_body"),
            doc.parsed_property("leg_body"),
        ) {
            attire = attire.with_physique(YouthPhysique::from_region_bytes([c, t, a, l]));
        }
        attire
    }

    /// Persists the player's customized medieval attire to the TOML profile, failing fast on I/O errors.
    fn save(&self, attire: &YouthAttire) {
        let bytes = attire.multiplayer_bytes();
        let regions = attire.region_bytes();
        let text = format!(
            "# Medieval Youth Avatar Profile\n[character]\ntunic = {}\nvambrace = {}\ngirdle = {}\nfootwear = {}\ncrown = {}\nrelic = {}\npigment = {}\ncomplexion = {}\nphysique = {}\ncrown_body = {}\ntorso_body = {}\narm_body = {}\nleg_body = {}\n",
            bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7], bytes[8],
            regions[0], regions[1], regions[2], regions[3]
        );
        fs::write(&self.path, text).expect("Failed to persist YouthAttire profile to disk");
    }
}
