use glam::Vec3;
use world::MeshTarget;
use crate::menu::glyph::PixelStamp;
use crate::menu::view::{MenuView, TitledCard};
use crate::menu::widget::{ScreenBounds, ScreenPoint};
use super::beacon::PortalStatus;
use super::invite::{PortalDirective, PortalNotice};

/// Stateless button geometry for the dedicated Step 2/2 NAT Counter-Punch Waiting screen.
pub struct WaitingBounds;

impl WaitingBounds {
    /// Primary constructor initializing the waiting screen button bounds.
    pub const fn new() -> Self {
        Self
    }

    pub fn hero_code_box(&self) -> ScreenBounds {
        ScreenBounds::new(-0.58, 0.08, 0.58, 0.23)
    }

    pub fn copy_reply_button(&self) -> ScreenBounds {
        ScreenBounds::new(-0.58, -0.22, 0.58, -0.10)
    }

    pub fn play_waiting_button(&self) -> ScreenBounds {
        ScreenBounds::new(-0.58, -0.39, 0.58, -0.27)
    }

    pub fn cancel_dial_button(&self) -> ScreenBounds {
        ScreenBounds::new(-0.42, -0.58, 0.42, -0.46)
    }

    pub fn select(&self, pointer: ScreenPoint) -> Option<WaitingChoice> {
        if self.copy_reply_button().contains(pointer) {
            Some(WaitingChoice::CopyReply)
        } else if self.play_waiting_button().contains(pointer) {
            Some(WaitingChoice::PlayWhileWaiting)
        } else if self.cancel_dial_button().contains(pointer) {
            Some(WaitingChoice::CancelConnection)
        } else {
            None
        }
    }
}

/// User selection on the dedicated Step 2/2 Waiting card.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WaitingChoice {
    CopyReply,
    PlayWhileWaiting,
    CancelConnection,
}

impl WaitingChoice {
    pub const fn into_directive(self) -> PortalDirective {
        match self {
            Self::CopyReply => PortalDirective::StayInPortal,
            Self::PlayWhileWaiting => PortalDirective::ResumeWaitingInWorld,
            Self::CancelConnection => PortalDirective::CancelPunchDial,
        }
    }
}

/// Dedicated Step 2/2 Co-op Waiting card replacing the text-input form while awaiting host NAT punch.
#[derive(Clone, Copy)]
pub struct PunchWaitBody<'a> {
    status: &'a PortalStatus,
    pointer: ScreenPoint,
}

impl<'a> PunchWaitBody<'a> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(status: &'a PortalStatus, pointer: ScreenPoint) -> Self {
        Self { status, pointer }
    }

    pub const fn card(self) -> TitledCard<'static, Self> {
        TitledCard::new(
            ScreenBounds::new(-0.72, -0.68, 0.72, 0.50),
            "STEP 2/2: SEND REPLY TO HOST",
            0.39,
            self,
        )
    }
}

impl MenuView for PunchWaitBody<'_> {
    fn emit(&self, target: &mut dyn MeshTarget) {
        let bounds = WaitingBounds::new();
        let (headline, col) = match self.status.notice() {
            PortalNotice::ReplyCopied => (
                "1. AUTO-COPIED! PASTE IN CHAT TO YOUR HOST",
                Vec3::new(0.56, 0.90, 0.68),
            ),
            _ => (
                "1. SEND THIS 12-CHAR REPLY CODE TO YOUR HOST",
                Vec3::new(0.96, 0.78, 0.34),
            ),
        };
        PixelStamp::caption(col).emit_centered(headline, 0.0, 0.29, target);
        self.status.beacon().emit_waiting_hero(bounds.hero_code_box(), target);
        bounds.copy_reply_button().emit_button(
            "COPY REPLY CODE (CTRL+C)",
            self.pointer,
            Vec3::new(0.20, 0.38, 0.56),
            matches!(self.status.notice(), PortalNotice::ReplyCopied),
            target,
        );
        bounds.play_waiting_button().emit_button(
            "PLAY WHILE WAITING (ESC)",
            self.pointer,
            Vec3::new(0.18, 0.48, 0.32),
            true,
            target,
        );
        bounds.cancel_dial_button().emit_button(
            "CANCEL CONNECTION",
            self.pointer,
            Vec3::new(0.46, 0.22, 0.24),
            false,
            target,
        );
    }
}
