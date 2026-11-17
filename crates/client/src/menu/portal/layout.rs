use glam::Vec3;
use sanctuary::JoinPreview;
use world::MeshTarget;
use crate::menu::glyph::PixelStamp;
use crate::menu::view::{MenuView, TitledCard};
use crate::menu::widget::{ScreenBounds, ScreenPoint};
use super::beacon::PortalStatus;
use super::draft::PortalDraft;
use super::invite::{PortalInvite, PortalNotice};

/// Stateless screen bounds layout for the Multiplayer Hub and Step 1/2 Co-op Join panels.
pub struct PortalBounds;

impl PortalBounds {
    /// Primary constructor initializing the portal button layout.
    pub const fn new() -> Self {
        Self
    }

    pub fn portal_button(&self) -> ScreenBounds {
        ScreenBounds::new(-0.38, 0.16, 0.38, 0.27)
    }

    pub fn join_portal_button(&self) -> ScreenBounds {
        ScreenBounds::new(-0.38, 0.01, 0.38, 0.12)
    }

    pub fn hub_paste_button(&self) -> ScreenBounds {
        ScreenBounds::new(-0.38, -0.35, 0.38, -0.24)
    }

    pub fn hub_back_button(&self) -> ScreenBounds {
        ScreenBounds::new(-0.38, -0.52, 0.38, -0.41)
    }

    pub fn paste_button(&self) -> ScreenBounds {
        ScreenBounds::new(-0.64, -0.14, -0.03, -0.01)
    }

    pub fn clear_button(&self) -> ScreenBounds {
        ScreenBounds::new(0.03, -0.14, 0.64, -0.01)
    }

    pub fn join_button(&self) -> ScreenBounds {
        ScreenBounds::new(-0.64, -0.34, 0.64, -0.20)
    }

    pub fn back_button(&self) -> ScreenBounds {
        ScreenBounds::new(-0.34, -0.60, 0.34, -0.48)
    }
}

/// Inner content of the Multiplayer Hub panel decorated by `TitledCard`.
pub struct MultiplayerHubBody<'a> {
    invite: &'a PortalInvite,
    status: &'a PortalStatus,
    pointer: ScreenPoint,
}

impl<'a> MultiplayerHubBody<'a> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(
        invite: &'a PortalInvite,
        status: &'a PortalStatus,
        pointer: ScreenPoint,
    ) -> Self {
        Self { invite, status, pointer }
    }
}

impl MenuView for MultiplayerHubBody<'_> {
    fn emit(&self, target: &mut dyn MeshTarget) {
        let bounds = PortalBounds::new();
        let gold = Vec3::new(0.96, 0.82, 0.34);
        let mint = Vec3::new(0.56, 0.86, 0.68);
        let silver = Vec3::new(0.76, 0.82, 0.90);
        let active = self.invite.active();
        let waiting = self.status.listening();
        let host_label = if active { "COPY INVITE TICKET" } else { "OPEN CO-OP PORTAL" };
        let join_label = if waiting {
            "WAITING FOR HOST (OPEN)"
        } else if active {
            "TYPE GUEST CODE / JOIN"
        } else {
            "JOIN CO-OP / SERVER"
        };
        let host_col = if active { Vec3::new(0.22, 0.52, 0.32) } else { Vec3::new(0.18, 0.44, 0.46) };
        let join_col = if waiting { Vec3::new(0.22, 0.48, 0.36) } else { Vec3::new(0.20, 0.36, 0.56) };
        bounds.portal_button().emit_button(host_label, self.pointer, host_col, active, target);
        bounds.join_portal_button().emit_button(join_label, self.pointer, join_col, waiting, target);
        if active {
            PixelStamp::title(gold).emit_centered(&self.invite.banner_headline(), 0.0, -0.09, target);
            let (sub, sub_col) = match self.status.notice() {
                PortalNotice::AutoPunchSent | PortalNotice::NatPunchSent => (
                    "GUEST REPLY ACCEPTED - NAT PUNCH SENT!",
                    Vec3::new(0.38, 0.90, 0.54),
                ),
                PortalNotice::InvalidTarget => (
                    "CLIPBOARD HAS NO VALID GUEST REPLY CODE",
                    Vec3::new(0.96, 0.72, 0.28),
                ),
                PortalNotice::ClipboardEmpty => ("CLIPBOARD EMPTY", Vec3::new(0.96, 0.72, 0.28)),
                PortalNotice::TicketCopied | PortalNotice::StartupTicketCopied => (
                    "SHARE 12-CHAR CODE WITH FRIEND",
                    mint,
                ),
                _ => ("SHARE 12-CHAR CODE WITH FRIEND", mint),
            };
            PixelStamp::caption(sub_col).emit_centered(sub, 0.0, -0.18, target);
            bounds.hub_paste_button().emit_button(
                "PASTE GUEST REPLY (CTRL+V)",
                self.pointer,
                Vec3::new(0.20, 0.46, 0.38),
                matches!(self.status.notice(), PortalNotice::AutoPunchSent | PortalNotice::NatPunchSent),
                target,
            );
        } else if !self.status.beacon().emit_hub_summary(target) {
            PixelStamp::caption(silver).emit_centered("HOST A 12-CHAR PORTAL OR JOIN A WORLD", 0.0, -0.20, target);
        }
        bounds.hub_back_button().emit_button(
            "BACK TO MAIN MENU",
            self.pointer,
            Vec3::new(0.28, 0.32, 0.40),
            false,
            target,
        );
    }
}

/// Step 1/2 Join Form layout and status presenter for the in-game Co-op / NAT Punch studio.
#[derive(Clone, Copy)]
pub struct PortalLayout<'a> {
    invite: &'a PortalInvite,
    draft: &'a PortalDraft,
    status: &'a PortalStatus,
    pointer: ScreenPoint,
}

impl<'a> PortalLayout<'a> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(
        invite: &'a PortalInvite,
        draft: &'a PortalDraft,
        status: &'a PortalStatus,
        pointer: ScreenPoint,
    ) -> Self {
        Self { invite, draft, status, pointer }
    }

    pub const fn hub_view(self) -> TitledCard<'static, MultiplayerHubBody<'a>> {
        TitledCard::new(
            ScreenBounds::new(-0.58, -0.60, 0.58, 0.44),
            "MULTIPLAYER",
            0.35,
            MultiplayerHubBody::new(self.invite, self.status, self.pointer),
        )
    }

    pub fn join_view(self) -> TitledCard<'static, Self> {
        let title = if self.invite.active() {
            "PUNCH GUEST NAT OR JOIN"
        } else {
            "STEP 1/2: JOIN CO-OP OR SERVER"
        };
        TitledCard::new(ScreenBounds::new(-0.72, -0.68, 0.72, 0.50), title, 0.39, self)
    }

    fn status_line(&self, silver: Vec3) -> (String, Vec3) {
        let green = Vec3::new(0.38, 0.88, 0.52);
        let amber = Vec3::new(0.96, 0.72, 0.28);
        let red = Vec3::new(0.94, 0.36, 0.34);
        match self.status.notice() {
            PortalNotice::NatPunchSent | PortalNotice::AutoPunchSent => {
                ("NAT PUNCH SENT - GUEST LINKING...".to_string(), green)
            }
            PortalNotice::ClipboardEmpty => ("CLIPBOARD EMPTY".to_string(), amber),
            PortalNotice::InvalidTarget => ("INVALID 12-CHAR CODE, TICKET, OR IP".to_string(), red),
            PortalNotice::ConnectionFailed => ("CONNECTION FAILED OR TIMED OUT".to_string(), red),
            PortalNotice::Idle
            | PortalNotice::StartupTicketCopied
            | PortalNotice::TicketCopied
            | PortalNotice::ReplyCopied
            | PortalNotice::ClipboardUnavailable => {
                if self.draft.verify_punch_for(self.invite).is_some() {
                    return ("READY: PUNCH GUEST NAT - PRESS ENTER".to_string(), green);
                }
                match self.draft.preview() {
                    JoinPreview::CodeReady(code) => (format!("READY: {} - PRESS ENTER", code.encode()), green),
                    JoinPreview::CodeTypo => ("TYPO IN 12-CHAR CODE - CHECK LETTERS".to_string(), amber),
                    JoinPreview::TicketOrAddressReady => ("READY TO CONNECT - PRESS ENTER".to_string(), green),
                    JoinPreview::Empty | JoinPreview::Malformed => {
                        let hint = if self.invite.active() {
                            "HOSTING: PASTE GUEST REPLY CODE OR SERVER"
                        } else {
                            "ENTER HOST'S 12-CHAR CODE OR PASTE"
                        };
                        (hint.to_string(), silver)
                    }
                }
            }
        }
    }
}

impl MenuView for PortalLayout<'_> {
    fn emit(&self, target: &mut dyn MeshTarget) {
        let bounds = PortalBounds::new();
        let silver = Vec3::new(0.76, 0.82, 0.90);
        PixelStamp::caption(silver).emit_centered(
            "TYPE 12 CODE CHARS (AUTO-GROUPED), TICKET, OR IP",
            0.0,
            0.27,
            target,
        );
        self.draft.emit_box(ScreenBounds::new(-0.64, 0.06, 0.64, 0.20), target);
        let action_label = if self.draft.verify_punch_for(self.invite).is_some() {
            "PUNCH GUEST NAT (ENTER)"
        } else {
            "JOIN WORLD (ENTER)"
        };
        bounds.paste_button().emit_button("PASTE (CTRL+V)", self.pointer, Vec3::new(0.22, 0.38, 0.56), false, target);
        bounds.clear_button().emit_button("CLEAR", self.pointer, Vec3::new(0.36, 0.26, 0.32), false, target);
        bounds.join_button().emit_button(action_label, self.pointer, Vec3::new(0.18, 0.48, 0.28), false, target);
        bounds.back_button().emit_button("BACK TO MULTIPLAYER", self.pointer, Vec3::new(0.28, 0.32, 0.40), false, target);

        let (status_text, status_col) = self.status_line(silver);
        PixelStamp::caption(status_col).emit_centered(&status_text, 0.0, -0.41, target);
    }
}
