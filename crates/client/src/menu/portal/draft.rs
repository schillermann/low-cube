/// Encapsulated join/punch input draft auto-grouping 12-character codes into `XXXX-XXXX-XXXX` blocks.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PortalDraft {
    raw: String,
}

impl PortalDraft {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(raw: String) -> Self {
        Self { raw }
    }

    pub const fn empty() -> Self {
        Self::new(String::new())
    }

    pub fn as_str(&self) -> &str {
        &self.raw
    }

    pub fn blank(&self) -> bool {
        self.raw.is_empty()
    }

    pub fn clear(&mut self) {
        self.raw.clear();
    }

    pub fn replace(&mut self, next: String) {
        self.raw = next;
    }

    fn code_grouping_symbols(text: &str) -> Option<String> {
        let trimmed = text.trim();
        if trimmed.is_empty()
            || trimmed.contains('.')
            || trimmed.contains(':')
            || trimmed.contains('/')
            || (trimmed.len() >= 4 && trimmed[..4].eq_ignore_ascii_case("lc1-"))
        {
            return None;
        }
        let lower = trimmed.to_ascii_lowercase();
        if "localhost".starts_with(&lower) {
            return None;
        }
        if !trimmed
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | ' '))
        {
            return None;
        }
        let symbols: String = trimmed
            .chars()
            .filter(|ch| ch.is_ascii_alphanumeric())
            .map(|ch| ch.to_ascii_uppercase())
            .collect();
        if symbols.is_empty() || symbols.len() > 12 {
            None
        } else {
            Some(symbols)
        }
    }

    pub fn append(&mut self, typed: &str) -> bool {
        let mut appended = false;
        for ch in typed.chars() {
            if ch == ' ' {
                if !self.raw.is_empty() && !self.raw.ends_with('-') && self.raw.len() < 160 {
                    self.raw.push('-');
                    appended = true;
                }
                continue;
            }
            let allowed = ch.is_ascii_alphanumeric() || matches!(ch, '-' | ':' | '.' | '_' | '/');
            if allowed && self.raw.len() < 160 {
                if ch == '-' && self.raw.ends_with('-') {
                    continue;
                }
                self.raw.push(ch);
                appended = true;
            }
        }
        appended
    }

    pub fn backspace(&mut self) {
        let popped = self.raw.pop();
        if matches!(popped, Some('-' | '_')) {
            if let Some(symbols) = Self::code_grouping_symbols(&self.raw) {
                if symbols.len() == 4 || symbols.len() == 8 {
                    let _ = self.raw.pop();
                }
            }
        }
    }

    /// Formats 12-char code candidates with automatic `XXXX-XXXX-XXXX` hyphens while preserving IPs/tickets.
    pub fn formatted(&self) -> String {
        let Some(symbols) = Self::code_grouping_symbols(&self.raw) else {
            return self.raw.clone();
        };
        let mut grouped = String::with_capacity(14);
        for (idx, ch) in symbols.chars().enumerate() {
            if idx == 4 || idx == 8 {
                grouped.push('-');
            }
            grouped.push(ch);
        }
        if symbols.len() == 4 || symbols.len() == 8 {
            grouped.push('-');
        }
        grouped
    }
}
