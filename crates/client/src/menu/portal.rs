use glam::Vec3;
use sanctuary::JoinTarget;
use winit::keyboard::KeyCode;
use world::MeshTarget;
use super::clipboard::{SystemClipboard, TextClipboard};
use super::glyph::PixelStamp;
use super::widget::{ScreenBounds, ScreenPoint};

/// Directive returned from interactive Co-op / Server join studio actions.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PortalDirective {
    StayInPortal,
    ReturnToPauseMenu,
    ConnectToWorld(JoinTarget),
}

/// Polymorphic status feedback eliminating nullable error strings.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PortalNotice {
    Idle,
    TicketCopied,
    ClipboardEmpty,
    InvalidTarget,
    ConnectionFailed,
}

/// Encapsulated Co-op hosting ticket banner and in-game server/ticket join studio.
pub struct PortalStudio {
    hosted_ticket: String,
    join_draft: String,
    notice: PortalNotice,
    clipboard: Box<dyn TextClipboard>,
}

impl PortalStudio {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(
        hosted_ticket: String,
        join_draft: String,
        notice: PortalNotice,
        clipboard: Box<dyn TextClipboard>,
    ) -> Self {
        Self {
            hosted_ticket,
            join_draft,
            notice,
            clipboard,
        }
    }

    pub fn with_ticket(hosted_ticket: String) -> Self {
        let clipboard = Box::new(SystemClipboard::new());
        let notice = if hosted_ticket.is_empty() {
            PortalNotice::Idle
        } else {
            let _ = clipboard.store_text(&hosted_ticket);
            PortalNotice::TicketCopied
        };
        Self::new(hosted_ticket, String::new(), notice, clipboard)
    }

    #[cfg(test)]
    pub fn with_clipboard(hosted_ticket: String, clipboard: Box<dyn TextClipboard>) -> Self {
        Self::new(hosted_ticket, String::new(), PortalNotice::Idle, clipboard)
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

    pub fn hosted_active(&self) -> bool {
        !self.hosted_ticket.is_empty()
    }

    pub fn announce_hosted(&mut self, ticket: String) {
        let _ = self.clipboard.store_text(&ticket);
        self.hosted_ticket = ticket;
        self.notice = PortalNotice::TicketCopied;
    }

    pub fn copy_hosted_ticket(&mut self) {
        if !self.hosted_ticket.is_empty() {
            let _ = self.clipboard.store_text(&self.hosted_ticket);
            self.notice = PortalNotice::TicketCopied;
        }
    }

    pub fn report_connection_failure(&mut self) {
        self.notice = PortalNotice::ConnectionFailed;
    }

    pub fn paste_from_clipboard(&mut self) {
        let pasted: String = self
            .clipboard
            .read_text()
            .chars()
            .filter(|ch| ch.is_ascii_graphic() || *ch == ' ')
            .take(200)
            .collect();
        let trimmed = pasted.trim();
        if trimmed.is_empty() {
            self.notice = PortalNotice::ClipboardEmpty;
        } else {
            self.join_draft = trimmed.to_string();
            self.notice = PortalNotice::Idle;
        }
    }

    pub fn append_text(&mut self, typed: &str) {
        let mut appended = false;
        for ch in typed.chars() {
            let allowed = ch.is_ascii_alphanumeric()
                || matches!(ch, '-' | ':' | '.' | '_' | '/');
            if allowed && self.join_draft.len() < 160 {
                self.join_draft.push(ch);
                appended = true;
            }
        }
        if appended {
            self.notice = PortalNotice::Idle;
        }
    }

    fn submit_draft(&mut self) -> PortalDirective {
        if let Some(target) = JoinTarget::parse(&self.join_draft) {
            self.notice = PortalNotice::Idle;
            PortalDirective::ConnectToWorld(target)
        } else {
            self.notice = PortalNotice::InvalidTarget;
            PortalDirective::StayInPortal
        }
    }

    pub fn handle_key(&mut self, code: KeyCode, ctrl_held: bool) -> PortalDirective {
        if ctrl_held && code == KeyCode::KeyV {
            self.paste_from_clipboard();
            return PortalDirective::StayInPortal;
        }
        if ctrl_held && code == KeyCode::KeyC {
            self.copy_hosted_ticket();
            return PortalDirective::StayInPortal;
        }
        match code {
            KeyCode::Backspace => {
                let _ = self.join_draft.pop();
                self.notice = PortalNotice::Idle;
                PortalDirective::StayInPortal
            }
            KeyCode::Enter | KeyCode::NumpadEnter => self.submit_draft(),
            _ => PortalDirective::StayInPortal,
        }
    }

    pub fn press_pointer(&mut self, pointer: ScreenPoint) -> PortalDirective {
        if Self::paste_button().contains(pointer) {
            self.paste_from_clipboard();
            PortalDirective::StayInPortal
        } else if Self::clear_button().contains(pointer) {
            self.join_draft.clear();
            self.notice = PortalNotice::Idle;
            PortalDirective::StayInPortal
        } else if Self::join_button().contains(pointer) {
            self.submit_draft()
        } else if Self::back_button().contains(pointer) {
            PortalDirective::ReturnToPauseMenu
        } else {
            PortalDirective::StayInPortal
        }
    }

    pub fn emit_pause_portal_buttons(
        &self,
        host_bounds: ScreenBounds,
        join_bounds: ScreenBounds,
        pointer: ScreenPoint,
        target: &mut dyn MeshTarget,
    ) {
        let gold = Vec3::new(0.96, 0.82, 0.34);
        let host_label = if self.hosted_ticket.is_empty() {
            "OPEN CO-OP PORTAL"
        } else {
            "COPY INVITE TICKET"
        };
        let host_col = if self.hosted_ticket.is_empty() {
            Vec3::new(0.18, 0.44, 0.46)
        } else {
            Vec3::new(0.22, 0.52, 0.32)
        };
        host_bounds.emit_button(host_label, pointer, host_col, !self.hosted_ticket.is_empty(), target);
        join_bounds.emit_button("JOIN CO-OP / SERVER", pointer, Vec3::new(0.20, 0.36, 0.56), false, target);
        if !self.hosted_ticket.is_empty() {
            let preview: String = self.hosted_ticket.chars().take(18).collect();
            let msg = if matches!(self.notice, PortalNotice::TicketCopied) {
                format!("COPIED: {preview}...")
            } else {
                format!("{preview}...")
            };
            PixelStamp::caption(gold).emit_centered(&msg, 0.0, -0.82, target);
        }
    }

    pub fn emit_overlay(&self, pointer: ScreenPoint, target: &mut dyn MeshTarget) {
        let bg = Vec3::new(0.08, 0.10, 0.15);
        let rim = Vec3::new(0.32, 0.39, 0.50);
        let gold = Vec3::new(0.96, 0.82, 0.34);
        let silver = Vec3::new(0.76, 0.82, 0.90);
        ScreenBounds::new(-0.72, -0.68, 0.72, 0.50).emit_framed(bg, rim, 0.008, target);
        PixelStamp::title(gold).emit_centered("JOIN CO-OP OR SERVER", 0.0, 0.39, target);
        PixelStamp::caption(silver).emit_centered(
            "PASTE INVITE TICKET (LC1-...) OR ENTER SERVER IP:PORT",
            0.0,
            0.27,
            target,
        );

        let box_bounds = ScreenBounds::new(-0.64, 0.06, 0.64, 0.20);
        box_bounds.emit_framed(Vec3::new(0.04, 0.06, 0.10), Vec3::new(0.42, 0.58, 0.78), 0.006, target);
        if self.join_draft.is_empty() {
            PixelStamp::caption(Vec3::new(0.48, 0.55, 0.66))
                .emit_centered("CLICK PASTE OR TYPE ADDRESS|", 0.0, 0.13, target);
        } else {
            let chars: Vec<char> = self.join_draft.chars().collect();
            let shown = if chars.len() > 44 {
                let tail: String = chars[chars.len() - 41..].iter().collect();
                format!("...{tail}|")
            } else {
                format!("{}|", self.join_draft)
            };
            PixelStamp::caption(Vec3::new(0.94, 0.97, 1.0)).emit_centered(&shown, 0.0, 0.13, target);
        }

        Self::paste_button().emit_button("PASTE (CTRL+V)", pointer, Vec3::new(0.22, 0.38, 0.56), false, target);
        Self::clear_button().emit_button("CLEAR", pointer, Vec3::new(0.36, 0.26, 0.32), false, target);
        Self::join_button().emit_button("JOIN WORLD (ENTER)", pointer, Vec3::new(0.18, 0.48, 0.28), false, target);
        Self::back_button().emit_button("BACK TO MAIN MENU", pointer, Vec3::new(0.28, 0.32, 0.40), false, target);

        let (status_text, status_col) = match self.notice {
            PortalNotice::ClipboardEmpty => ("CLIPBOARD EMPTY", Vec3::new(0.94, 0.62, 0.26)),
            PortalNotice::InvalidTarget => ("INVALID TICKET OR IP:PORT", Vec3::new(0.94, 0.36, 0.34)),
            PortalNotice::ConnectionFailed => ("CONNECTION FAILED - CHECK SERVER", Vec3::new(0.94, 0.36, 0.34)),
            PortalNotice::Idle | PortalNotice::TicketCopied => {
                if !self.join_draft.is_empty() && JoinTarget::parse(&self.join_draft).is_some() {
                    ("READY TO CONNECT - PRESS ENTER", Vec3::new(0.38, 0.88, 0.52))
                } else {
                    ("NO ROUTER SETUP NEEDED WITH LC1 TICKET", silver)
                }
            }
        };
        PixelStamp::caption(status_col).emit_centered(status_text, 0.0, -0.41, target);
    }
}
