use glam::Vec3;
use sanctuary::{JoinPreview, JoinTarget};
use world::MeshTarget;
use crate::menu::glyph::PixelStamp;
use crate::menu::widget::{ScreenBounds, ScreenPoint};
use super::draft::PortalDraft;
use super::invite::{PortalInvite, PortalNotice};

/// Declarative screen layout and status presenter for the in-game Co-op / NAT Punch studio.
pub struct PortalLayout<'a> {
    invite: &'a PortalInvite,
    draft: &'a PortalDraft,
    notice: PortalNotice,
    pointer: ScreenPoint,
}

impl<'a> PortalLayout<'a> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(
        invite: &'a PortalInvite,
        draft: &'a PortalDraft,
        notice: PortalNotice,
        pointer: ScreenPoint,
    ) -> Self {
        Self { invite, draft, notice, pointer }
    }

    pub fn portal_button() -> ScreenBounds {
        ScreenBounds::new(-0.34, 0.12, 0.34, 0.24)
    }

    pub fn join_portal_button() -> ScreenBounds {
        ScreenBounds::new(-0.34, -0.06, 0.34, 0.06)
    }

    pub fn hub_back_button() -> ScreenBounds {
        ScreenBounds::new(-0.34, -0.50, 0.34, -0.38)
    }

    pub fn paste_button() -> ScreenBounds {
        ScreenBounds::new(-0.64, -0.14, -0.03, -0.01)
    }

    pub fn clear_button() -> ScreenBounds {
        ScreenBounds::new(0.03, -0.14, 0.64, -0.01)
    }

    pub fn join_button() -> ScreenBounds {
        ScreenBounds::new(-0.64, -0.34, 0.64, -0.20)
    }

    pub fn back_button() -> ScreenBounds {
        ScreenBounds::new(-0.34, -0.60, 0.34, -0.48)
    }

    pub fn emit_multiplayer_hub(&self, target: &mut dyn MeshTarget) {
        let bg = Vec3::new(0.08, 0.10, 0.15);
        let rim = Vec3::new(0.32, 0.39, 0.50);
        let gold = Vec3::new(0.96, 0.82, 0.34);
        let mint = Vec3::new(0.56, 0.86, 0.68);
        let silver = Vec3::new(0.76, 0.82, 0.90);
        let active = self.invite.active();
        let host_label = if active { "COPY INVITE TICKET" } else { "OPEN CO-OP PORTAL" };
        let join_label = if active { "PUNCH NAT / JOIN" } else { "JOIN CO-OP / SERVER" };
        let host_col = if active { Vec3::new(0.22, 0.52, 0.32) } else { Vec3::new(0.18, 0.44, 0.46) };
        ScreenBounds::new(-0.58, -0.60, 0.58, 0.44).emit_framed(bg, rim, 0.008, target);
        PixelStamp::title(gold).emit_centered("MULTIPLAYER", 0.0, 0.35, target);
        Self::portal_button().emit_button(host_label, self.pointer, host_col, active, target);
        Self::join_portal_button().emit_button(join_label, self.pointer, Vec3::new(0.20, 0.36, 0.56), false, target);
        if active {
            PixelStamp::title(gold).emit_centered(&self.invite.banner_headline(), 0.0, -0.17, target);
            let sub = match self.notice {
                PortalNotice::TicketCopied | PortalNotice::StartupTicketCopied => "SHARE 12-CHAR CODE OR PASTE TICKET",
                _ => "SHARE 12-CHAR CODE WITH FRIEND",
            };
            PixelStamp::caption(mint).emit_centered(sub, 0.0, -0.27, target);
        } else {
            PixelStamp::caption(silver).emit_centered("HOST A 12-CHAR PORTAL OR JOIN A WORLD", 0.0, -0.22, target);
        }
        Self::hub_back_button().emit_button("BACK TO MAIN MENU", self.pointer, Vec3::new(0.28, 0.32, 0.40), false, target);
    }

    fn status_line(&self, silver: Vec3) -> (String, Vec3) {
        let green = Vec3::new(0.38, 0.88, 0.52);
        let amber = Vec3::new(0.96, 0.72, 0.28);
        let red = Vec3::new(0.94, 0.36, 0.34);
        match self.notice {
            PortalNotice::AwaitingHostPunch => (
                format!("NAT LOCKED - TELL HOST: {}", self.invite.punch_reply()),
                amber,
            ),
            PortalNotice::NatPunchSent => ("NAT PUNCH SENT - GUEST LINKING...".to_string(), green),
            PortalNotice::ClipboardEmpty => ("CLIPBOARD EMPTY".to_string(), amber),
            PortalNotice::InvalidTarget => ("INVALID 12-CHAR CODE, TICKET, OR IP".to_string(), red),
            PortalNotice::ConnectionFailed => ("CONNECTION FAILED - CHECK CODE OR IP".to_string(), red),
            PortalNotice::Idle
            | PortalNotice::StartupTicketCopied
            | PortalNotice::TicketCopied
            | PortalNotice::ClipboardUnavailable => {
                let raw = self.draft.as_str();
                if self.invite.verify_guest_punch(raw).is_some() {
                    return ("READY: PUNCH GUEST NAT - PRESS ENTER".to_string(), green);
                }
                match JoinTarget::preview(raw) {
                    JoinPreview::CodeReady(code) => (format!("READY: {} - PRESS ENTER", code.encode()), green),
                    JoinPreview::CodeTypo => ("TYPO IN 12-CHAR CODE - CHECK LETTERS".to_string(), amber),
                    JoinPreview::TicketOrAddressReady => ("READY TO CONNECT - PRESS ENTER".to_string(), green),
                    JoinPreview::Empty | JoinPreview::Malformed => {
                        let hint = if self.invite.active() {
                            "HOSTING: TYPE 12 GUEST PUNCH CHARS OR SERVER"
                        } else {
                            "TYPE 12 CODE CHARS (AUTO-GROUPED) OR PASTE"
                        };
                        (hint.to_string(), silver)
                    }
                }
            }
        }
    }

    pub fn emit_overlay(&self, target: &mut dyn MeshTarget) {
        let bg = Vec3::new(0.08, 0.10, 0.15);
        let rim = Vec3::new(0.32, 0.39, 0.50);
        let gold = Vec3::new(0.96, 0.82, 0.34);
        let silver = Vec3::new(0.76, 0.82, 0.90);
        ScreenBounds::new(-0.72, -0.68, 0.72, 0.50).emit_framed(bg, rim, 0.008, target);
        let title = if self.invite.active() { "NAT PUNCH OR JOIN SERVER" } else { "JOIN CO-OP OR SERVER" };
        PixelStamp::title(gold).emit_centered(title, 0.0, 0.39, target);
        PixelStamp::caption(silver).emit_centered(
            "TYPE 12 CODE CHARS (AUTO-GROUPED), TICKET, OR IP",
            0.0,
            0.27,
            target,
        );

        let box_bounds = ScreenBounds::new(-0.64, 0.06, 0.64, 0.20);
        box_bounds.emit_framed(Vec3::new(0.04, 0.06, 0.10), Vec3::new(0.42, 0.58, 0.78), 0.006, target);
        if self.draft.blank() {
            PixelStamp::caption(Vec3::new(0.48, 0.55, 0.66))
                .emit_centered("TYPE 12 CHARS OR CLICK PASTE|", 0.0, 0.13, target);
        } else {
            let formatted = self.draft.formatted();
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
            stamp.emit_centered(&shown, 0.0, 0.13, target);
        }

        let action_label = if self.invite.verify_guest_punch(self.draft.as_str()).is_some() {
            "PUNCH GUEST NAT (ENTER)"
        } else {
            "JOIN WORLD (ENTER)"
        };
        Self::paste_button().emit_button("PASTE (CTRL+V)", self.pointer, Vec3::new(0.22, 0.38, 0.56), false, target);
        Self::clear_button().emit_button("CLEAR", self.pointer, Vec3::new(0.36, 0.26, 0.32), false, target);
        Self::join_button().emit_button(action_label, self.pointer, Vec3::new(0.18, 0.48, 0.28), false, target);
        Self::back_button().emit_button("BACK TO MULTIPLAYER", self.pointer, Vec3::new(0.28, 0.32, 0.40), false, target);

        let (status_text, status_col) = self.status_line(silver);
        PixelStamp::caption(status_col).emit_centered(&status_text, 0.0, -0.41, target);
    }
}
