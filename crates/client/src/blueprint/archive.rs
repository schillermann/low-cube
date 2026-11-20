use std::fs;
use world::CraftBlueprint;
use crate::toml_document::TomlDocument;
use super::SavedBlueprint;

/// Focused persistent storage contract for named player vehicle blueprints.
pub trait BlueprintArchive {
    fn load_saved(&self) -> Vec<SavedBlueprint>;
    fn save_all(&mut self, entries: &[SavedBlueprint]);
}

/// Persistent TOML storage saving and loading named vehicle blueprints in `blueprints.toml`.
pub struct FileBlueprints {
    path: String,
}

impl FileBlueprints {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(path: String) -> Self {
        Self { path }
    }

    /// Secondary constructor targeting the local `blueprints.toml` profile.
    pub fn profile() -> Self {
        Self::new("blueprints.toml".to_string())
    }

    fn hex_encode(bytes: &[u8]) -> String {
        const HEX: &[u8; 16] = b"0123456789abcdef";
        let mut out = String::with_capacity(bytes.len() * 2);
        for &b in bytes {
            out.push(HEX[(b >> 4) as usize] as char);
            out.push(HEX[(b & 0x0F) as usize] as char);
        }
        out
    }

    fn hex_nibble(ch: u8) -> Option<u8> {
        match ch {
            b'0'..=b'9' => Some(ch - b'0'),
            b'a'..=b'f' => Some(ch - b'a' + 10),
            b'A'..=b'F' => Some(ch - b'A' + 10),
            _ => None,
        }
    }

    fn hex_decode(raw: &str) -> Option<Vec<u8>> {
        let bytes = raw.trim().as_bytes();
        if bytes.is_empty() || bytes.len() % 2 != 0 {
            return None;
        }
        let mut out = Vec::with_capacity(bytes.len() / 2);
        for pair in bytes.chunks_exact(2) {
            let hi = Self::hex_nibble(pair[0])?;
            let lo = Self::hex_nibble(pair[1])?;
            out.push((hi << 4) | lo);
        }
        Some(out)
    }

    fn sanitize_title(raw: &str) -> String {
        raw.chars()
            .filter(|&c| c != '"' && c != '\'' && c != '\n' && c != '\r')
            .take(24)
            .collect()
    }
}

impl BlueprintArchive for FileBlueprints {
    fn load_saved(&self) -> Vec<SavedBlueprint> {
        let Some(doc) = TomlDocument::from_file(&self.path) else {
            return Vec::new();
        };
        let count: usize = doc.parsed_property("count").unwrap_or(0).min(128);
        let mut loaded = Vec::with_capacity(count);
        for idx in 0..count {
            let section = format!("blueprint_{idx}");
            let Some(name) = doc.section_property(&section, "name", false) else {
                continue;
            };
            let Some(hex_payload) = doc.section_property(&section, "payload", false) else {
                continue;
            };
            let Some(raw_bytes) = Self::hex_decode(&hex_payload) else {
                continue;
            };
            if let Some((bp, _)) = CraftBlueprint::decode(&raw_bytes) {
                loaded.push(SavedBlueprint::new(Self::sanitize_title(&name), bp));
            }
        }
        loaded
    }

    fn save_all(&mut self, entries: &[SavedBlueprint]) {
        let mut text = format!("# Saved Vehicle Blueprints\ncount = {}\n", entries.len());
        let mut wire = Vec::new();
        for (idx, entry) in entries.iter().enumerate() {
            wire.clear();
            entry.blueprint().inscribe(&mut wire);
            let safe_name = Self::sanitize_title(entry.title_text());
            let hex = Self::hex_encode(&wire);
            text.push_str(&format!("\n[blueprint_{idx}]\nname = \"{safe_name}\"\npayload = \"{hex}\"\n"));
        }
        fs::write(&self.path, text).expect("Failed to persist vehicle blueprints to disk");
    }
}

/// Isolated in-memory `BlueprintArchive` Fake Object for deterministic offline unit tests.
#[cfg(test)]
pub struct MemoryBlueprints {
    entries: Vec<SavedBlueprint>,
}

#[cfg(test)]
impl MemoryBlueprints {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(entries: Vec<SavedBlueprint>) -> Self {
        Self { entries }
    }

    pub const fn empty() -> Self {
        Self::new(Vec::new())
    }
}

#[cfg(test)]
impl BlueprintArchive for MemoryBlueprints {
    fn load_saved(&self) -> Vec<SavedBlueprint> {
        self.entries.clone()
    }

    fn save_all(&mut self, entries: &[SavedBlueprint]) {
        self.entries = entries.to_vec();
    }
}
