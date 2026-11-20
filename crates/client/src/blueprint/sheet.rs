use std::fs;
use std::path::Path;
use world::CraftBlueprint;
use crate::toml_document::TomlDocument;
use super::SavedBlueprint;

/// Sanitized blueprint title stripped of quote and line-break characters.
pub struct CleanTitle<'a> {
    raw: &'a str,
}

impl<'a> CleanTitle<'a> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(raw: &'a str) -> Self {
        Self { raw }
    }

    pub fn text(&self) -> String {
        self.raw
            .chars()
            .filter(|&c| c != '"' && c != '\'' && c != '\n' && c != '\r')
            .take(24)
            .collect()
    }
}

/// Portable lowercase filename stem derived from a player-assigned blueprint title.
pub struct BlueprintSlug<'a> {
    title: &'a str,
}

impl<'a> BlueprintSlug<'a> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(title: &'a str) -> Self {
        Self { title }
    }

    pub fn stem(&self) -> String {
        let mut out = String::with_capacity(self.title.len());
        for ch in self.title.chars() {
            if ch.is_ascii_alphanumeric() {
                out.push(ch.to_ascii_lowercase());
            } else if ch == '-' {
                if !out.is_empty() && !out.ends_with('_') && !out.ends_with('-') {
                    out.push('-');
                }
            } else if !out.is_empty() && !out.ends_with('_') && !out.ends_with('-') {
                out.push('_');
            }
        }
        let trimmed = out.trim_matches(|c| c == '_' || c == '-');
        if trimmed.is_empty() {
            return "vehicle".to_string();
        }
        if self.reserved_device(trimmed) {
            return format!("bp_{trimmed}");
        }
        trimmed.to_string()
    }

    fn reserved_device(&self, name: &str) -> bool {
        matches!(
            name,
            "con"
                | "prn"
                | "aux"
                | "nul"
                | "com1"
                | "com2"
                | "com3"
                | "com4"
                | "com5"
                | "com6"
                | "com7"
                | "com8"
                | "com9"
                | "lpt1"
                | "lpt2"
                | "lpt3"
                | "lpt4"
                | "lpt5"
                | "lpt6"
                | "lpt7"
                | "lpt8"
                | "lpt9"
        )
    }
}

struct WireHex<'a> {
    bytes: &'a [u8],
}

impl<'a> WireHex<'a> {
    const fn new(bytes: &'a [u8]) -> Self {
        Self { bytes }
    }

    fn encoded(&self) -> String {
        const HEX: &[u8; 16] = b"0123456789abcdef";
        let mut out = String::with_capacity(self.bytes.len() * 2);
        for &b in self.bytes {
            out.push(HEX[(b >> 4) as usize] as char);
            out.push(HEX[(b & 0x0F) as usize] as char);
        }
        out
    }
}

struct HexText<'a> {
    raw: &'a str,
}

impl<'a> HexText<'a> {
    const fn new(raw: &'a str) -> Self {
        Self { raw }
    }

    fn decoded(&self) -> Option<Vec<u8>> {
        let bytes = self.raw.trim().as_bytes();
        if bytes.is_empty() || bytes.len() % 2 != 0 {
            return None;
        }
        let mut out = Vec::with_capacity(bytes.len() / 2);
        for pair in bytes.chunks_exact(2) {
            let hi = self.nibble(pair[0])?;
            let lo = self.nibble(pair[1])?;
            out.push((hi << 4) | lo);
        }
        Some(out)
    }

    const fn nibble(&self, ch: u8) -> Option<u8> {
        match ch {
            b'0'..=b'9' => Some(ch - b'0'),
            b'a'..=b'f' => Some(ch - b'a' + 10),
            b'A'..=b'F' => Some(ch - b'A' + 10),
            _ => None,
        }
    }
}

/// Single-vehicle TOML document persisted as an individual `.toml` file inside `blueprints/`.
pub struct BlueprintSheet {
    content: String,
}

impl BlueprintSheet {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(content: String) -> Self {
        Self { content }
    }

    pub fn from_file(path: &Path) -> Option<Self> {
        fs::read_to_string(path).ok().map(Self::new)
    }

    pub fn from_saved(entry: &SavedBlueprint) -> Self {
        let mut wire = Vec::new();
        entry.blueprint().inscribe(&mut wire);
        let safe_name = CleanTitle::new(entry.title_text()).text();
        let hex = WireHex::new(&wire).encoded();
        let text = format!(
            "# Saved Vehicle Blueprint\n[blueprint]\nname = \"{safe_name}\"\npayload = \"{hex}\"\n"
        );
        Self::new(text)
    }

    pub fn write_to(&self, path: &Path) {
        fs::write(path, &self.content).expect("Failed to persist vehicle blueprint TOML file");
    }

    pub fn decode_saved(&self) -> Option<SavedBlueprint> {
        let doc = TomlDocument::new(self.content.clone());
        let name = doc.section_property("blueprint", "name", true)?;
        let clean = CleanTitle::new(&name).text();
        if clean.is_empty() {
            return None;
        }
        let hex_payload = doc.section_property("blueprint", "payload", true)?;
        let raw_bytes = HexText::new(&hex_payload).decoded()?;
        let (bp, _) = CraftBlueprint::decode(&raw_bytes)?;
        Some(SavedBlueprint::new(clean, bp))
    }
}
