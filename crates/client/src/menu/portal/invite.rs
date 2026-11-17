use std::net::SocketAddr;
use sanctuary::{JoinTarget, PortalCode};
use crate::menu::clipboard::TextClipboard;

/// Directive returned from interactive Co-op / Server join & NAT hole-punch studio actions.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PortalDirective {
    StayInPortal,
    OpenLivePortal,
    OpenJoinScreen,
    ReturnToMultiplayer,
    ReturnToPauseMenu,
    ResumeWaitingInWorld,
    CancelPunchDial,
    ConnectToWorld(JoinTarget),
    PunchGuest(SocketAddr),
}

/// Polymorphic status feedback eliminating nullable error strings.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PortalNotice {
    Idle,
    StartupTicketCopied,
    TicketCopied,
    ReplyCopied,
    ClipboardUnavailable,
    ClipboardEmpty,
    InvalidTarget,
    ConnectionFailed,
    NatPunchSent,
    AutoPunchSent,
}

/// Active 12-character host portal code and `LC1-...` ticket credentials.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PortalInvite {
    code: String,
    ticket: String,
}

impl PortalInvite {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(code: String, ticket: String) -> Self {
        Self { code, ticket }
    }

    pub const fn hosted(code: String, ticket: String) -> Self {
        Self::new(code, ticket)
    }

    #[cfg(test)]
    pub const fn empty() -> Self {
        Self::new(String::new(), String::new())
    }

    pub fn active(&self) -> bool {
        !self.code.is_empty() || !self.ticket.is_empty()
    }

    pub fn verify_guest_punch(&self, input: &str) -> Option<SocketAddr> {
        let host_code = PortalCode::parse(&self.code)?;
        host_code.verify_punch(input)
    }

    pub fn copy_into(&self, clipboard: &dyn TextClipboard) -> PortalNotice {
        let payload = if !self.ticket.is_empty() {
            &self.ticket
        } else {
            &self.code
        };
        if clipboard.store_text(payload) {
            PortalNotice::TicketCopied
        } else {
            PortalNotice::ClipboardUnavailable
        }
    }

    pub fn copy_at_startup(&self, clipboard: &dyn TextClipboard) -> PortalNotice {
        if !self.active() {
            PortalNotice::Idle
        } else if matches!(self.copy_into(clipboard), PortalNotice::TicketCopied) {
            PortalNotice::StartupTicketCopied
        } else {
            PortalNotice::ClipboardUnavailable
        }
    }

    pub fn banner_headline(&self) -> String {
        if !self.code.is_empty() {
            format!("CODE: {}", self.code)
        } else {
            let preview: String = self.ticket.chars().take(18).collect();
            format!("TICKET: {preview}...")
        }
    }
}
