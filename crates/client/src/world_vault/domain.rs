use std::path::{Path, PathBuf};

/// Immutable sanitized player world title eliminating raw string helper functions (`1` field).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorldTitle {
    text: String,
}

impl WorldTitle {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(text: String) -> Self {
        Self { text }
    }

    pub fn sanitized(raw: &str) -> Self {
        let mut out = String::with_capacity(20);
        let mut prev_space = true;
        for ch in raw.chars() {
            if out.len() >= 20 {
                break;
            }
            let up = ch.to_ascii_uppercase();
            if up.is_ascii_alphanumeric() || matches!(up, '-' | '_' | '#') {
                out.push(up);
                prev_space = false;
            } else if ch == ' ' && !prev_space {
                out.push(' ');
                prev_space = true;
            }
        }
        Self::new(out.trim().to_string())
    }

    pub fn numbered(ordinal: usize) -> Self {
        Self::new(format!("WORLD {}", ordinal.max(1)))
    }

    pub fn blank(&self) -> bool {
        self.text.is_empty()
    }

    pub fn resolved(&self, fallback_ordinal: usize) -> Self {
        if self.blank() {
            Self::numbered(fallback_ordinal)
        } else {
            self.clone()
        }
    }

    pub fn appended(&self, raw: &str) -> Self {
        Self::sanitized(&format!("{}{raw}", self.text))
    }

    pub fn popped(&self) -> Self {
        let mut next = self.text.clone();
        next.pop();
        Self::new(next)
    }

    pub fn derive_slug(&self) -> WorldSlug {
        let mut out = String::with_capacity(20);
        let mut prev_sep = true;
        for ch in self.text.chars() {
            if out.len() >= 20 {
                break;
            }
            let low = ch.to_ascii_lowercase();
            if low.is_ascii_alphanumeric() {
                out.push(low);
                prev_sep = false;
            } else if !prev_sep {
                out.push('_');
                prev_sep = true;
            }
        }
        let trimmed = out.trim_matches('_').to_string();
        WorldSlug::new(if trimmed.is_empty() { "world".to_string() } else { trimmed })
    }

    pub fn field_caption(&self, focused: bool) -> String {
        if focused {
            format!("{}_", self.text)
        } else if self.blank() {
            "TYPE WORLD NAME".to_string()
        } else {
            self.text.clone()
        }
    }

    pub fn short_label(&self) -> String {
        let short: String = self.text.chars().take(16).collect();
        if short.is_empty() { "WORLD".to_string() } else { short }
    }

    pub fn inscribe_toml(&self, out: &mut String) {
        out.push_str(&format!("name = \"{}\"\n", self.text));
    }

    pub fn announce_portal_banner(&self) {
        println!("=== LOW-CUBE CO-OP PORTAL OPEN ({}) ===", self.text);
    }

    #[cfg(test)]
    pub fn matches_text(&self, expected: &str) -> bool {
        self.text == expected
    }
}

/// Validated filesystem directory identifier for a saved world under `worlds/<slug>/` (`1` field).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorldSlug {
    text: String,
}

impl WorldSlug {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(text: String) -> Self {
        Self { text }
    }

    pub const fn absent() -> Self {
        Self::new(String::new())
    }

    pub fn validated(raw: String) -> Option<Self> {
        let ok = !raw.is_empty()
            && raw.len() <= 32
            && raw
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '-');
        ok.then_some(Self::new(raw))
    }

    pub fn disambiguated(&self, ordinal: u32) -> Self {
        Self::new(format!("{}_{ordinal}", self.text))
    }

    pub fn matches(&self, other: &Self) -> bool {
        !self.text.is_empty() && self.text == other.text
    }

    pub fn directory_in(&self, root: &Path) -> PathBuf {
        root.join(&self.text)
    }

    pub fn journal_in(&self, root: &Path) -> PathBuf {
        self.directory_in(root).join("chronicle.wal")
    }

    pub fn compare_key(&self, other: &Self) -> std::cmp::Ordering {
        self.text.cmp(&other.text)
    }
}
