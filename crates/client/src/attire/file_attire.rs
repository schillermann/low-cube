use std::fs;
use crate::toml_document::TomlDocument;
use super::crown::CrownStyle;
use super::footwear::FootwearStyle;
use super::garments::{GirdleStyle, TunicStyle};
use super::hands::VambraceStyle;
use super::palette::{ComplexionSwatch, PigmentSwatch};
use super::proportion::YouthPhysique;
use super::relic::HeldRelic;
use super::YouthAttire;

/// Interface for persistent profile storage and multiplayer wire encoding of medieval attire.
pub trait AttireProfile {
    fn load_or_squire(&self) -> YouthAttire;
    fn save(&self, attire: &YouthAttire);
    fn encode_multiplayer(&self, attire: &YouthAttire) -> [u8; 9];
    fn decode_multiplayer(&self, bytes: [u8; 9]) -> YouthAttire;
}

/// Persistent TOML storage and multiplayer packet codec for customizable medieval attire.
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
        let mut attire = self.decode_multiplayer(bytes);
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
        let bytes = self.encode_multiplayer(attire);
        let regions = attire.region_bytes();
        let text = format!(
            "# Medieval Youth Avatar Profile\n[character]\ntunic = {}\nvambrace = {}\ngirdle = {}\nfootwear = {}\ncrown = {}\nrelic = {}\npigment = {}\ncomplexion = {}\nphysique = {}\ncrown_body = {}\ntorso_body = {}\narm_body = {}\nleg_body = {}\n",
            bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7], bytes[8],
            regions[0], regions[1], regions[2], regions[3]
        );
        fs::write(&self.path, text).expect("Failed to persist YouthAttire profile to disk");
    }

    /// Encodes attire into a compact 9-byte array for multiplayer synchronization.
    fn encode_multiplayer(&self, attire: &YouthAttire) -> [u8; 9] {
        attire.multiplayer_bytes()
    }

    /// Decodes a compact 9-byte multiplayer packet into a complete YouthAttire instance.
    fn decode_multiplayer(&self, bytes: [u8; 9]) -> YouthAttire {
        YouthAttire::new(
            TunicStyle::from_code(bytes[0]),
            VambraceStyle::from_code(bytes[1]),
            GirdleStyle::from_code(bytes[2]),
            FootwearStyle::from_code(bytes[3]),
            CrownStyle::from_code(bytes[4]),
            HeldRelic::from_code(bytes[5] % 16),
            HeldRelic::from_code(bytes[5] / 16),
            PigmentSwatch::from_code(bytes[6]),
            ComplexionSwatch::from_code(bytes[7]),
            YouthPhysique::from_code(bytes[8]),
        )
    }
}
