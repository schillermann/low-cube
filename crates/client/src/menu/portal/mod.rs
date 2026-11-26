mod auto_punch;
mod beacon;
mod draft;
mod invite;
mod layout;
mod waiting;

use winit::keyboard::KeyCode;
use world::MeshTarget;
use super::clipboard::TextClipboard;
use super::view::{MenuView, TitledCard, VersionedView};
use super::widget::ScreenPoint;

pub use auto_punch::AutoPunchModal;
pub use beacon::{BeaconHud, PortalStatus, PunchBeacon, ReticleHud};
pub use draft::PortalDraft;
pub use invite::{PortalDirective, PortalInvite, PortalNotice};
pub use layout::{MultiplayerHubBody, PortalBounds, PortalLayout};
pub use waiting::{PunchWaitBody, WaitingBounds, WaitingChoice};

/// Encapsulated 12-char P2P portal banner, persistent NAT signal beacon, and server join studio.
pub struct PortalStudio {
    invite: PortalInvite,
    draft: PortalDraft,
    status: PortalStatus,
    clipboard: Box<dyn TextClipboard>,
}

impl PortalStudio {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(
        invite: PortalInvite,
        draft: PortalDraft,
        status: PortalStatus,
        clipboard: Box<dyn TextClipboard>,
    ) -> Self {
        Self { invite, draft, status, clipboard }
    }

    pub fn with_invite(invite: PortalInvite, clipboard: Box<dyn TextClipboard>) -> Self {
        let notice = invite.copy_at_startup(clipboard.as_ref());
        Self::new(invite, PortalDraft::empty(), PortalStatus::initial(notice), clipboard)
    }

    #[cfg(test)]
    pub fn with_clipboard(invite: PortalInvite, clipboard: Box<dyn TextClipboard>) -> Self {
        Self::new(invite, PortalDraft::empty(), PortalStatus::initial(PortalNotice::Idle), clipboard)
    }

    #[cfg(test)]
    pub const fn current_notice(&self) -> PortalNotice {
        self.status.notice()
    }

    #[cfg(test)]
    pub fn formatted_draft(&self) -> String {
        self.draft.formatted()
    }

    pub fn hosted_active(&self) -> bool {
        self.invite.active()
    }

    pub const fn beacon(&self) -> &PunchBeacon {
        self.status.beacon()
    }

    pub const fn awaiting_host_punch(&self) -> bool {
        self.status.listening()
    }

    pub fn step(&mut self, dt: f32) {
        self.status.advance(dt);
    }

    pub fn announce_hosted(&mut self, invite: PortalInvite) {
        self.status.silence_beacon();
        let notice = invite.copy_into(self.clipboard.as_ref());
        self.invite = invite;
        self.status.record_notice(notice);
    }

    pub fn announce_auto_punch(&mut self) {
        self.status.record_notice(PortalNotice::AutoPunchSent);
    }

    pub fn announce_punch_reply(&mut self, punch_code: String) {
        self.status.open_beacon(punch_code, self.clipboard.as_ref());
    }

    pub fn clipboard(&self) -> &dyn TextClipboard {
        self.clipboard.as_ref()
    }

    pub fn clear_dial_state(&mut self) {
        self.status.silence_beacon();
    }

    pub fn copy_hosted_ticket(&mut self) {
        if self.status.copy_beacon(self.clipboard.as_ref()) {
            return;
        }
        if self.invite.active() {
            let notice = self.invite.copy_into(self.clipboard.as_ref());
            self.status.record_notice(notice);
        }
    }

    pub fn refresh_startup_copy(&mut self) {
        if matches!(self.status.notice(), PortalNotice::StartupTicketCopied) {
            self.copy_hosted_ticket();
        }
    }

    pub fn report_connection_failure(&mut self) {
        self.status.silence_beacon();
        self.status.record_notice(PortalNotice::ConnectionFailed);
    }

    fn read_trimmed_clipboard(&self) -> String {
        let clean: String = self
            .clipboard
            .read_text()
            .chars()
            .filter(|ch| ch.is_ascii_graphic() || *ch == ' ')
            .take(220)
            .collect();
        clean.trim().to_string()
    }

    pub fn paste_guest_reply(&mut self) -> PortalDirective {
        let trimmed = self.read_trimmed_clipboard();
        if trimmed.is_empty() {
            self.status.record_notice(PortalNotice::ClipboardEmpty);
            return PortalDirective::StayInPortal;
        }
        if let Some(peer) = self.invite.verify_guest_punch(&trimmed) {
            self.draft.clear();
            self.status.record_notice(PortalNotice::NatPunchSent);
            PortalDirective::PunchGuest(peer)
        } else {
            self.status.record_notice(PortalNotice::InvalidTarget);
            PortalDirective::StayInPortal
        }
    }

    pub fn paste_from_clipboard(&mut self) -> PortalDirective {
        let trimmed = self.read_trimmed_clipboard();
        if trimmed.is_empty() {
            self.status.record_notice(PortalNotice::ClipboardEmpty);
            return PortalDirective::StayInPortal;
        }
        if let Some(peer) = self.invite.verify_guest_punch(&trimmed) {
            self.draft.clear();
            self.status.record_notice(PortalNotice::NatPunchSent);
            return PortalDirective::PunchGuest(peer);
        }
        self.draft.replace(trimmed);
        self.status.record_notice(PortalNotice::Idle);
        PortalDirective::StayInPortal
    }

    pub fn append_text(&mut self, typed: &str) {
        if !self.status.listening() && self.draft.append(typed) {
            self.status.record_notice(PortalNotice::Idle);
        }
    }

    fn submit_draft(&mut self) -> PortalDirective {
        if let Some(peer) = self.draft.verify_punch_for(&self.invite) {
            self.draft.clear();
            self.status.record_notice(PortalNotice::NatPunchSent);
            return PortalDirective::PunchGuest(peer);
        }
        if let Some(target) = self.draft.parse_target() {
            self.status.silence_beacon();
            self.status.record_notice(PortalNotice::Idle);
            PortalDirective::ConnectToWorld(target)
        } else {
            self.status.record_notice(PortalNotice::InvalidTarget);
            PortalDirective::StayInPortal
        }
    }

    pub fn handle_key(&mut self, code: KeyCode, ctrl_held: bool) -> PortalDirective {
        if self.status.listening() {
            if ctrl_held && code == KeyCode::KeyC {
                let _ = self.status.copy_beacon(self.clipboard.as_ref());
                return PortalDirective::StayInPortal;
            }
            if matches!(code, KeyCode::Enter | KeyCode::NumpadEnter) {
                return PortalDirective::ResumeWaitingInWorld;
            }
            return PortalDirective::StayInPortal;
        }
        if ctrl_held && code == KeyCode::KeyV {
            return self.paste_from_clipboard();
        }
        if ctrl_held && code == KeyCode::KeyC {
            self.copy_hosted_ticket();
            return PortalDirective::StayInPortal;
        }
        match code {
            KeyCode::Backspace => {
                self.draft.backspace();
                self.status.record_notice(PortalNotice::Idle);
                PortalDirective::StayInPortal
            }
            KeyCode::Enter | KeyCode::NumpadEnter => self.submit_draft(),
            _ => PortalDirective::StayInPortal,
        }
    }

    pub fn press_hub_pointer(&mut self, pointer: ScreenPoint) -> PortalDirective {
        let bounds = PortalBounds::new(self.hosted_active() || self.awaiting_host_punch());
        if bounds.portal_button().contains(pointer) {
            if self.hosted_active() {
                self.copy_hosted_ticket();
                PortalDirective::StayInPortal
            } else {
                PortalDirective::OpenLivePortal
            }
        } else if bounds.join_portal_button().contains(pointer) {
            PortalDirective::OpenJoinScreen
        } else if self.hosted_active() && bounds.hub_paste_button().contains(pointer) {
            self.paste_guest_reply()
        } else if bounds.hub_back_button().contains(pointer) {
            PortalDirective::ReturnToPauseMenu
        } else {
            PortalDirective::StayInPortal
        }
    }

    pub fn press_pointer(&mut self, pointer: ScreenPoint) -> PortalDirective {
        if self.status.listening() {
            let Some(choice) = WaitingBounds::new().select(pointer) else {
                return PortalDirective::StayInPortal;
            };
            match choice {
                WaitingChoice::CopyReply => {
                    let _ = self.status.copy_beacon(self.clipboard.as_ref());
                }
                WaitingChoice::CancelConnection => self.status.silence_beacon(),
                WaitingChoice::PlayWhileWaiting => {}
            }
            return choice.into_directive();
        }
        let bounds = PortalBounds::compact();
        if bounds.paste_button().contains(pointer) {
            self.paste_from_clipboard()
        } else if bounds.clear_button().contains(pointer) {
            self.draft.clear();
            self.status.record_notice(PortalNotice::Idle);
            PortalDirective::StayInPortal
        } else if bounds.join_button().contains(pointer) {
            self.submit_draft()
        } else if bounds.back_button().contains(pointer) {
            PortalDirective::ReturnToMultiplayer
        } else {
            PortalDirective::StayInPortal
        }
    }

    pub fn hub_view(&self, pointer: ScreenPoint) -> TitledCard<'static, MultiplayerHubBody<'_>> {
        PortalLayout::new(&self.invite, &self.draft, &self.status, pointer).hub_view()
    }

    pub fn emit_join_screen(&self, pointer: ScreenPoint, target: &mut dyn MeshTarget) {
        if self.status.listening() {
            VersionedView::current(PunchWaitBody::new(&self.status, pointer).card()).emit(target);
        } else {
            VersionedView::current(
                PortalLayout::new(&self.invite, &self.draft, &self.status, pointer).join_view(),
            )
            .emit(target);
        }
    }
}
