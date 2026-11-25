use glam::Vec3;
use super::Callsign;

pub const MAX_UTTERANCE_BYTES: usize = 140;

/// Spatial text utterance spoken at a world coordinate and heard only by nearby companions (`<= 4` fields).
#[derive(Clone, Debug, PartialEq)]
pub struct ProximityUtterance {
    speaker: u32,
    callsign: Callsign,
    origin: Vec3,
    text: String,
}

impl ProximityUtterance {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn with_callsign(speaker: u32, callsign: Callsign, origin: Vec3, text: String) -> Self {
        Self { speaker, callsign, origin, text }
    }

    /// Secondary constructor initializing an utterance without an explicit callsign.
    pub const fn new(speaker: u32, origin: Vec3, text: String) -> Self {
        Self::with_callsign(speaker, Callsign::unnamed(), origin, text)
    }

    /// Secondary constructor sanitizing printable text and attaching a server-resolved `Callsign`.
    pub fn named(speaker: u32, callsign: Callsign, origin: Vec3, raw: &str) -> Option<Self> {
        let mut cleaned = String::with_capacity(raw.len().min(MAX_UTTERANCE_BYTES));
        for ch in raw.trim().chars() {
            if ch.is_control() {
                continue;
            }
            if cleaned.len() + ch.len_utf8() > MAX_UTTERANCE_BYTES {
                break;
            }
            cleaned.push(ch);
        }
        let trimmed = cleaned.trim();
        if trimmed.is_empty() || !origin.is_finite() {
            None
        } else {
            Some(Self::with_callsign(speaker, callsign, origin, trimmed.to_string()))
        }
    }

    /// Secondary constructor sanitizing printable text and rejecting blank utterances.
    pub fn sanitized(speaker: u32, origin: Vec3, raw: &str) -> Option<Self> {
        Self::named(speaker, Callsign::unnamed(), origin, raw)
    }

    /// Evaluates whether a listener at `listener_pos` stands within `earshot_meters` of the speaker.
    pub fn audible_at(&self, listener_pos: Vec3, earshot_meters: f32) -> bool {
        (listener_pos - self.origin).length_squared() <= earshot_meters * earshot_meters
    }

    /// Formats the speaker's callsign (or `P<slot>` fallback) and spoken line for HUD chat overlays.
    pub fn formatted_line(&self) -> String {
        format!("{}: {}", self.callsign.speaker_label(self.speaker), self.text)
    }

    /// True when prefixed with `~` to mute suit radio transceivers in vacuum (helmet-contact only).
    pub fn radio_silent(&self) -> bool {
        self.text.starts_with('~')
    }

    /// Transfers `(speaker_slot, origin, text)` to a collaborator without exposing fields.
    pub fn transfer<R>(&self, f: impl FnOnce(u32, Vec3, &str) -> R) -> R {
        f(self.speaker, self.origin, &self.text)
    }

    pub fn inscribe(&self, out: &mut Vec<u8>) {
        out.extend_from_slice(&self.speaker.to_le_bytes());
        for c in [self.origin.x, self.origin.y, self.origin.z] {
            out.extend_from_slice(&c.to_le_bytes());
        }
        let bytes = self.text.as_bytes();
        let len = bytes.len().min(MAX_UTTERANCE_BYTES);
        out.push(len as u8);
        out.extend_from_slice(&bytes[..len]);
        self.callsign.inscribe(out);
    }

    pub fn decode(bytes: &[u8]) -> Option<(Self, usize)> {
        if bytes.len() < 17 {
            return None;
        }
        let speaker = u32::from_le_bytes(bytes[0..4].try_into().ok()?);
        let ox = f32::from_le_bytes(bytes[4..8].try_into().ok()?);
        let oy = f32::from_le_bytes(bytes[8..12].try_into().ok()?);
        let oz = f32::from_le_bytes(bytes[12..16].try_into().ok()?);
        let text_len = usize::from(bytes[16]);
        if text_len == 0 || text_len > MAX_UTTERANCE_BYTES || bytes.len() < 17 + text_len {
            return None;
        }
        let raw_str = std::str::from_utf8(&bytes[17..17 + text_len]).ok()?;
        let head_end = 17 + text_len;
        let (callsign, extra) = if bytes.len() > head_end {
            Callsign::decode(&bytes[head_end..])?
        } else {
            (Callsign::unnamed(), 0)
        };
        let utterance = Self::named(speaker, callsign, Vec3::new(ox, oy, oz), raw_str)?;
        Some((utterance, head_end + extra))
    }
}
