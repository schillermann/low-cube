mod invite;
mod layout;

use sanctuary::JoinTarget;
use winit::keyboard::KeyCode;
use world::MeshTarget;
use super::clipboard::TextClipboard;
use super::widget::{ScreenBounds, ScreenPoint};

pub use invite::{PortalDirective, PortalInvite, PortalNotice};
pub use layout::PortalLayout;

/// Encapsulated 12-char P2P portal banner, FRITZ!Box NAT punch reply, and server join studio.
pub struct PortalStudio {
    invite: PortalInvite,
    join_draft: String,
    notice: PortalNotice,
    clipboard: Box<dyn TextClipboard>,
}

impl PortalStudio {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(
        invite: PortalInvite,
        join_draft: String,
        notice: PortalNotice,
        clipboard: Box<dyn TextClipboard>,
    ) -> Self {
        Self { invite, join_draft, notice, clipboard }
    }

    pub fn with_invite(invite: PortalInvite, clipboard: Box<dyn TextClipboard>) -> Self {
        let notice = if !invite.active() {
            PortalNotice::Idle
        } else if clipboard.store_text(invite.clipboard_payload()) {
            PortalNotice::StartupTicketCopied
        } else {
            PortalNotice::ClipboardUnavailable
        };
        Self::new(invite, String::new(), notice, clipboard)
    }

    #[cfg(test)]
    pub fn with_clipboard(invite: PortalInvite, clipboard: Box<dyn TextClipboard>) -> Self {
        Self::new(invite, String::new(), PortalNotice::Idle, clipboard)
    }

    #[cfg(test)]
    pub const fn current_notice(&self) -> PortalNotice {
        self.notice
    }

    pub fn paste_button() -> ScreenBounds {
        PortalLayout::paste_button()
    }

    pub fn clear_button() -> ScreenBounds {
        PortalLayout::clear_button()
    }

    pub fn join_button() -> ScreenBounds {
        PortalLayout::join_button()
    }

    pub fn back_button() -> ScreenBounds {
        PortalLayout::back_button()
    }

    pub fn hosted_active(&self) -> bool {
        self.invite.active()
    }

    pub fn announce_hosted(&mut self, invite: PortalInvite) {
        let copied = self.clipboard.store_text(invite.clipboard_payload());
        self.invite = invite;
        self.notice = if copied {
            PortalNotice::TicketCopied
        } else {
            PortalNotice::ClipboardUnavailable
        };
    }

    pub fn announce_punch_reply(&mut self, punch_code: String) {
        let _ = self.clipboard.store_text(&punch_code);
        self.invite.set_punch_reply(punch_code);
        self.notice = PortalNotice::AwaitingHostPunch;
    }

    pub fn clear_dial_state(&mut self) {
        self.invite.clear_punch_reply();
        if matches!(self.notice, PortalNotice::AwaitingHostPunch) {
            self.notice = PortalNotice::Idle;
        }
    }

    pub fn copy_hosted_ticket(&mut self) {
        if self.invite.active() || !self.invite.punch_reply().is_empty() {
            self.notice = if self.clipboard.store_text(self.invite.clipboard_payload()) {
                PortalNotice::TicketCopied
            } else {
                PortalNotice::ClipboardUnavailable
            };
        }
    }

    pub fn refresh_startup_copy(&mut self) {
        if matches!(self.notice, PortalNotice::StartupTicketCopied) {
            self.copy_hosted_ticket();
        }
    }

    pub fn report_connection_failure(&mut self) {
        self.invite.clear_punch_reply();
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
            if ch == ' ' {
                if !self.join_draft.is_empty()
                    && !self.join_draft.ends_with('-')
                    && self.join_draft.len() < 160
                {
                    self.join_draft.push('-');
                    appended = true;
                }
                continue;
            }
            let allowed = ch.is_ascii_alphanumeric() || matches!(ch, '-' | ':' | '.' | '_' | '/');
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
        if let Some(peer) = self.invite.verify_guest_punch(&self.join_draft) {
            self.join_draft.clear();
            self.notice = PortalNotice::NatPunchSent;
            return PortalDirective::PunchGuest(peer);
        }
        if let Some(target) = JoinTarget::parse(&self.join_draft) {
            self.invite.clear_punch_reply();
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
            self.clear_dial_state();
            self.notice = PortalNotice::Idle;
            PortalDirective::StayInPortal
        } else if Self::join_button().contains(pointer) {
            self.submit_draft()
        } else if Self::back_button().contains(pointer) {
            self.clear_dial_state();
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
        PortalLayout::new(&self.invite, &self.join_draft, self.notice, pointer)
            .emit_pause_buttons(host_bounds, join_bounds, target);
    }

    pub fn emit_overlay(&self, pointer: ScreenPoint, target: &mut dyn MeshTarget) {
        PortalLayout::new(&self.invite, &self.join_draft, self.notice, pointer)
            .emit_overlay(target);
    }
}
