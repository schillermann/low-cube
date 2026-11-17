use std::net::SocketAddr;
use glam::Vec3;
use sanctuary::{JoinPreview, JoinTarget};
use world::MeshTarget;
use crate::menu::glyph::PixelStamp;
use crate::menu::widget::ScreenBounds;
use super::invite::PortalInvite;

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

    pub fn blank(&self) -> bool {
        self.raw.is_empty()
    }

    pub fn clear(&mut self) {
        self.raw.clear();
    }

    pub fn replace(&mut self, next: String) {
        self.raw = next;
    }

    pub fn verify_punch_for(&self, invite: &PortalInvite) -> Option<SocketAddr> {
        invite.verify_guest_punch(&self.raw)
    }

    pub fn parse_target(&self) -> Option<JoinTarget> {
        JoinTarget::parse(&self.raw)
    }

    pub fn preview(&self) -> JoinPreview {
        JoinTarget::preview(&self.raw)
    }

    fn code_grouping_symbols(&self, text: &str) -> Option<String> {
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
            if let Some(symbols) = self.code_grouping_symbols(&self.raw) {
                if symbols.len() == 4 || symbols.len() == 8 {
                    let _ = self.raw.pop();
                }
            }
        }
    }

    /// Formats 12-char code candidates with automatic `XXXX-XXXX-XXXX` hyphens while preserving IPs/tickets.
    pub fn formatted(&self) -> String {
        let Some(symbols) = self.code_grouping_symbols(&self.raw) else {
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

    pub fn emit_box(&self, bounds: ScreenBounds, target: &mut dyn MeshTarget) {
        bounds.emit_framed(
            Vec3::new(0.04, 0.06, 0.10),
            Vec3::new(0.42, 0.58, 0.78),
            0.006,
            target,
        );
        let center_y = bounds.center_y();
        if self.blank() {
            PixelStamp::caption(Vec3::new(0.48, 0.55, 0.66))
                .emit_centered("TYPE 12 CHARS OR CLICK PASTE|", 0.0, center_y, target);
            return;
        }
        let formatted = self.formatted();
        let chars: Vec<char> = formatted.chars().collect();
        let shown = if chars.len() > 44 {
            let tail: String = chars[chars.len() - 41..].iter().collect();
            format!("...{tail}|")
        } else {
            format!("{formatted}|")
        };
        let stamp = if chars.len() <= 20 {
            PixelStamp::title(Vec3::new(0.94, 0.97, 1.0))
        } else {
            PixelStamp::caption(Vec3::new(0.94, 0.97, 1.0))
        };
        stamp.emit_centered(&shown, 0.0, center_y, target);
    }
}
