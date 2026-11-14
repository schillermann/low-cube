mod draft;
mod invite;
mod layout;

use sanctuary::JoinTarget;
use winit::keyboard::KeyCode;
use world::MeshTarget;
use super::clipboard::TextClipboard;
use super::widget::{ScreenBounds, ScreenPoint};

pub use draft::PortalDraft;
pub use invite::{PortalDirective, PortalInvite, PortalNotice};
pub use layout::PortalLayout;

/// Encapsulated 12-char P2P portal banner, FRITZ!Box NAT punch reply, and server join studio.
pub struct PortalStudio {
    invite: PortalInvite,
    draft: PortalDraft,
    notice: PortalNotice,
    clipboard: Box<dyn TextClipboard>,
}

impl PortalStudio {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(
        invite: PortalInvite,
        draft: PortalDraft,
        notice: PortalNotice,
        clipboard: Box<dyn TextClipboard>,
    ) -> Self {
        Self { invite, draft, notice, clipboard }
    }

    pub fn with_invite(invite: PortalInvite, clipboard: Box<dyn TextClipboard>) -> Self {
        let notice = if !invite.active() {
            PortalNotice::Idle
        } else if clipboard.store_text(invite.clipboard_payload()) {
            PortalNotice::StartupTicketCopied
        } else {
            PortalNotice::ClipboardUnavailable
        };
        Self::new(invite, PortalDraft::empty(), notice, clipboard)
    }

    #[cfg(test)]
    pub fn with_clipboard(invite: PortalInvite, clipboard: Box<dyn TextClipboard>) -> Self {
        Self::new(invite, PortalDraft::empty(), PortalNotice::Idle, clipboard)
    }

    #[cfg(test)]
    pub const fn current_notice(&self) -> PortalNotice {
        self.notice
    }

    #[cfg(test)]
    pub fn formatted_draft(&self) -> String {
        self.draft.formatted()
    }

    pub fn portal_button() -> ScreenBounds {
        PortalLayout::portal_button()
    }

    pub fn join_portal_button() -> ScreenBounds {
        PortalLayout::join_portal_button()
    }

    pub fn hub_back_button() -> ScreenBounds {
        PortalLayout::hub_back_button()
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
        self.invite.record_punch_reply(punch_code);
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
            self.draft.replace(trimmed.to_string());
            self.notice = PortalNotice::Idle;
        }
    }

    pub fn append_text(&mut self, typed: &str) {
        if self.draft.append(typed) {
            self.notice = PortalNotice::Idle;
        }
    }

    fn submit_draft(&mut self) -> PortalDirective {
        if let Some(peer) = self.invite.verify_guest_punch(self.draft.as_str()) {
            self.draft.clear();
            self.notice = PortalNotice::NatPunchSent;
            return PortalDirective::PunchGuest(peer);
        }
        if let Some(target) = JoinTarget::parse(self.draft.as_str()) {
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
                self.draft.backspace();
                self.notice = PortalNotice::Idle;
                PortalDirective::StayInPortal
            }
            KeyCode::Enter | KeyCode::NumpadEnter => self.submit_draft(),
            _ => PortalDirective::StayInPortal,
        }
    }

    pub fn press_hub_pointer(&mut self, pointer: ScreenPoint) -> PortalDirective {
        if Self::portal_button().contains(pointer) {
            if self.hosted_active() {
                self.copy_hosted_ticket();
                PortalDirective::StayInPortal
            } else {
                PortalDirective::OpenLivePortal
            }
        } else if Self::join_portal_button().contains(pointer) {
            PortalDirective::OpenJoinScreen
        } else if Self::hub_back_button().contains(pointer) {
            PortalDirective::ReturnToPauseMenu
        } else {
            PortalDirective::StayInPortal
        }
    }

    pub fn press_pointer(&mut self, pointer: ScreenPoint) -> PortalDirective {
        if Self::paste_button().contains(pointer) {
            self.paste_from_clipboard();
            PortalDirective::StayInPortal
        } else if Self::clear_button().contains(pointer) {
            self.draft.clear();
            self.clear_dial_state();
            self.notice = PortalNotice::Idle;
            PortalDirective::StayInPortal
        } else if Self::join_button().contains(pointer) {
            self.submit_draft()
        } else if Self::back_button().contains(pointer) {
            self.clear_dial_state();
            PortalDirective::ReturnToMultiplayer
        } else {
            PortalDirective::StayInPortal
        }
    }

    pub fn emit_multiplayer_hub(&self, pointer: ScreenPoint, target: &mut dyn MeshTarget) {
        PortalLayout::new(&self.invite, &self.draft, self.notice, pointer)
            .emit_multiplayer_hub(target);
    }

    pub fn emit_overlay(&self, pointer: ScreenPoint, target: &mut dyn MeshTarget) {
        PortalLayout::new(&self.invite, &self.draft, self.notice, pointer)
            .emit_overlay(target);
    }
}
