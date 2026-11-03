use std::fs;

/// Encapsulated TOML configuration text resolving unquoted section and top-level properties.
#[derive(Clone, Debug)]
pub struct TomlDocument {
    content: String,
}

impl TomlDocument {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(content: String) -> Self {
        Self { content }
    }

    /// Secondary constructor reading configuration text from a file path if readable.
    pub fn from_file(path: &str) -> Option<Self> {
        fs::read_to_string(path).ok().map(Self::new)
    }

    /// Resolves an unquoted value for `key` across any section in the document.
    pub fn property(&self, key: &str) -> Option<String> {
        for line in self.content.lines().map(str::trim) {
            if line.is_empty() || line.starts_with('#') || (line.starts_with('[') && line.ends_with(']')) {
                continue;
            }
            if let Some((k, v)) = line.split_once('=') {
                if k.trim().eq_ignore_ascii_case(key) {
                    return Some(Self::clean_value(v));
                }
            }
        }
        None
    }

    /// Resolves and parses an unquoted value for `key` across any section in the document.
    pub fn parsed_property<T: std::str::FromStr>(&self, key: &str) -> Option<T> {
        self.property(key)?.parse::<T>().ok()
    }

    /// Resolves an unquoted value for `key` within `section` (or before any section header when `top_level_allowed` is true).
    pub fn section_property(&self, section: &str, key: &str, top_level_allowed: bool) -> Option<String> {
        let mut active = top_level_allowed;
        for line in self.content.lines().map(str::trim) {
            if line.starts_with('[') && line.ends_with(']') {
                let header = line[1..line.len() - 1].trim();
                active = header.eq_ignore_ascii_case(section);
                continue;
            }
            if active && !line.is_empty() && !line.starts_with('#') {
                if let Some((k, v)) = line.split_once('=') {
                    if k.trim().eq_ignore_ascii_case(key) {
                        return Some(Self::clean_value(v));
                    }
                }
            }
        }
        None
    }

    fn clean_value(raw_val: &str) -> String {
        let uncommented = raw_val.split('#').next().unwrap_or("").trim();
        uncommented.trim_matches('"').trim_matches('\'').trim().to_string()
    }
}
