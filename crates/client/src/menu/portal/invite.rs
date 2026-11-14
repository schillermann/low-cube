use std::net::SocketAddr;
use sanctuary::{JoinTarget, PortalCode};

/// Directive returned from interactive Co-op / Server join & NAT hole-punch studio actions.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PortalDirective {
    StayInPortal,
    OpenLivePortal,
    OpenJoinScreen,
    ReturnToMultiplayer,
    ReturnToPauseMenu,
    ConnectToWorld(JoinTarget),
    PunchGuest(SocketAddr),
}

/// Polymorphic status feedback eliminating nullable error strings.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PortalNotice {
    Idle,
    StartupTicketCopied,
    TicketCopied,
    ClipboardUnavailable,
    ClipboardEmpty,
    InvalidTarget,
    ConnectionFailed,
    AwaitingHostPunch,
    NatPunchSent,
}

/// Active 12-character portal code, `LC1-...` ticket, and guest FRITZ!Box counter-punch reply code.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PortalInvite {
    code: String,
    ticket: String,
    punch_reply: String,
}

impl PortalInvite {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(code: String, ticket: String, punch_reply: String) -> Self {
        Self { code, ticket, punch_reply }
    }

    pub const fn hosted(code: String, ticket: String) -> Self {
        Self::new(code, ticket, String::new())
    }

    #[cfg(test)]
    pub const fn empty() -> Self {
        Self::new(String::new(), String::new(), String::new())
    }

    pub fn active(&self) -> bool {
        !self.code.is_empty() || !self.ticket.is_empty()
    }

    pub fn record_punch_reply(&mut self, reply: String) {
        self.punch_reply = reply;
    }

    pub fn clear_punch_reply(&mut self) {
        self.punch_reply.clear();
    }

    pub fn punch_reply(&self) -> &str {
        &self.punch_reply
    }

    pub fn verify_guest_punch(&self, input: &str) -> Option<SocketAddr> {
        let host_code = PortalCode::parse(&self.code)?;
        host_code.verify_punch(input)
    }

    pub fn clipboard_payload(&self) -> &str {
        if !self.punch_reply.is_empty() {
            &self.punch_reply
        } else if !self.ticket.is_empty() {
            &self.ticket
        } else {
            &self.code
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
