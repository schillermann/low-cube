use std::fs;
use super::crown::CrownStyle;
use super::footwear::FootwearStyle;
use super::garments::{GirdleStyle, TunicStyle};
use super::hands::VambraceStyle;
use super::palette::{ComplexionSwatch, PigmentSwatch, YouthPhysique};
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

    fn code_in_text(&self, content: &str, key_name: &str) -> Option<u8> {
        content
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty() && !line.starts_with('#') && !line.starts_with('['))
            .find_map(|line| {
                let (key, val) = line.split_once('=')?;
                if key.trim() == key_name {
                    val.split('#').next()?.trim().parse::<u8>().ok()
                } else {
                    None
                }
            })
    }
}

impl AttireProfile for FileAttire {
    /// Loads customized attire from file or returns squire attire when no profile exists yet.
    fn load_or_squire(&self) -> YouthAttire {
        let Ok(content) = fs::read_to_string(&self.path) else {
            return YouthAttire::squire();
        };
        let bytes = [
            self.code_in_text(&content, "tunic").unwrap_or(0),
            self.code_in_text(&content, "vambrace").unwrap_or(1),
            self.code_in_text(&content, "girdle").unwrap_or(0),
            self.code_in_text(&content, "footwear").unwrap_or(0),
            self.code_in_text(&content, "crown").unwrap_or(0),
            self.code_in_text(&content, "relic").unwrap_or(0),
            self.code_in_text(&content, "pigment").unwrap_or(0),
            self.code_in_text(&content, "complexion").unwrap_or(0),
            self.code_in_text(&content, "physique").unwrap_or(0),
        ];
        self.decode_multiplayer(bytes)
    }

    /// Persists the player's customized medieval attire to the TOML profile, failing fast on I/O errors.
    fn save(&self, attire: &YouthAttire) {
        let bytes = self.encode_multiplayer(attire);
        let text = format!(
            "# Medieval Youth Avatar Profile\n[character]\ntunic = {}\nvambrace = {}\ngirdle = {}\nfootwear = {}\ncrown = {}\nrelic = {}\npigment = {}\ncomplexion = {}\nphysique = {}\n",
            bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7], bytes[8]
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
            HeldRelic::from_code(bytes[5]),
            PigmentSwatch::from_code(bytes[6]),
            ComplexionSwatch::from_code(bytes[7]),
            YouthPhysique::from_code(bytes[8]),
        )
    }
}
