pub const MAX_CALLSIGN_BYTES: usize = 24;
const MAX_STEM_BYTES: usize = 16;

/// Stack-allocated companion callsign (`<= 24` UTF-8 bytes) disambiguating survivor identities without heap allocation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Callsign {
    bytes: [u8; MAX_CALLSIGN_BYTES],
    len: u8,
}

impl Callsign {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(bytes: [u8; MAX_CALLSIGN_BYTES], len: u8) -> Self {
        Self { bytes, len }
    }

    /// Secondary constructor representing an unnamed visitor prior to server assignment.
    pub const fn unnamed() -> Self {
        Self::new([0; MAX_CALLSIGN_BYTES], 0)
    }

    /// Secondary constructor representing an unnamed local P2P host.
    pub fn host() -> Self {
        Self::from_trusted("HOST")
    }

    /// Secondary constructor numbering an unnamed survivor (`Survivor#1`, `Survivor#2`, ...).
    pub fn numbered_survivor(ordinal: u32) -> Self {
        Self::from_trusted(&format!("Survivor#{}", ordinal.max(1)))
    }

    fn from_trusted(text: &str) -> Self {
        let mut bytes = [0u8; MAX_CALLSIGN_BYTES];
        let mut used = 0usize;
        for ch in text.chars() {
            let span = ch.len_utf8();
            if used + span > MAX_CALLSIGN_BYTES {
                break;
            }
            ch.encode_utf8(&mut bytes[used..used + span]);
            used += span;
        }
        Self::new(bytes, used as u8)
    }

    /// Sanitizes client-submitted text, stripping control characters and `#` so only the server assigns `#N` suffixes.
    pub fn sanitized(raw: &str) -> Self {
        let mut cleaned = String::with_capacity(MAX_STEM_BYTES);
        let mut prev_space = false;
        for ch in raw.trim().chars() {
            if ch.is_control() || ch == '#' || ch == '|' || ch == '"' || ch == '\'' {
                continue;
            }
            let normalized = if ch.is_whitespace() { ' ' } else { ch };
            if normalized == ' ' && prev_space {
                continue;
            }
            prev_space = normalized == ' ';
            if cleaned.len() + normalized.len_utf8() > MAX_STEM_BYTES {
                break;
            }
            cleaned.push(normalized);
        }
        let trimmed = cleaned.trim();
        if trimmed.is_empty() {
            Self::unnamed()
        } else {
            Self::from_trusted(trimmed)
        }
    }

    /// Appends `#ordinal` (`Tom#2`, `Tom#3`) when `ordinal >= 2`, or mints `Survivor#ordinal` when unnamed.
    pub fn disambiguated(self, ordinal: u32) -> Self {
        if !self.present() {
            return Self::numbered_survivor(ordinal);
        }
        if ordinal <= 1 {
            return self;
        }
        let suffix = format!("#{ordinal}");
        let budget = MAX_CALLSIGN_BYTES.saturating_sub(suffix.len());
        let mut stem = String::new();
        for ch in self.as_str().chars() {
            if stem.len() + ch.len_utf8() > budget {
                break;
            }
            stem.push(ch);
        }
        stem.push_str(&suffix);
        Self::from_trusted(&stem)
    }

    pub const fn present(&self) -> bool {
        self.len > 0
    }

    pub fn as_str(&self) -> &str {
        let end = (self.len as usize).min(MAX_CALLSIGN_BYTES);
        std::str::from_utf8(&self.bytes[..end]).unwrap_or("")
    }

    pub fn matches_case_insensitive(&self, other: Self) -> bool {
        self.present()
            && other.present()
            && self.as_str().to_lowercase() == other.as_str().to_lowercase()
    }

    pub fn speaker_label(&self, fallback_slot: u32) -> String {
        if self.present() {
            self.as_str().to_string()
        } else {
            format!("P{fallback_slot}")
        }
    }

    pub fn inscribe(&self, out: &mut Vec<u8>) {
        let slice = self.as_str().as_bytes();
        let count = slice.len().min(MAX_CALLSIGN_BYTES);
        out.push(count as u8);
        out.extend_from_slice(&slice[..count]);
    }

    pub fn decode(bytes: &[u8]) -> Option<(Self, usize)> {
        let count = usize::from(*bytes.first()?);
        if count > MAX_CALLSIGN_BYTES || bytes.len() < 1 + count {
            return None;
        }
        if count == 0 {
            return Some((Self::unnamed(), 1));
        }
        let text = std::str::from_utf8(&bytes[1..1 + count]).ok()?;
        if text.chars().any(char::is_control) {
            return None;
        }
        Some((Self::from_trusted(text), 1 + count))
    }
}
