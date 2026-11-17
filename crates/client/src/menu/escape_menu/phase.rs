use crate::menu::portal::PortalDirective;
use crate::menu::ModalDirective;

/// Polymorphic modal state machine for gameplay, Escape pause menu, Character studio, Settings, Multiplayer, and Blackbox.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ModalPhase {
    Gameplay,
    EscapePause,
    CharacterWardrobe,
    SettingsHub,
    ControlSettings,
    MultiplayerHub,
    PortalJoin,
    FlightBlackBox,
}

impl ModalPhase {
    pub(super) fn apply_portal(self, directive: PortalDirective) -> (Self, ModalDirective) {
        match directive {
            PortalDirective::StayInPortal => (self, ModalDirective::ContinueSession),
            PortalDirective::OpenLivePortal => (self, ModalDirective::OpenPortal),
            PortalDirective::OpenJoinScreen => (Self::PortalJoin, ModalDirective::ContinueSession),
            PortalDirective::ReturnToMultiplayer => (Self::MultiplayerHub, ModalDirective::ContinueSession),
            PortalDirective::ReturnToPauseMenu => (Self::EscapePause, ModalDirective::ContinueSession),
            PortalDirective::ResumeWaitingInWorld => (Self::Gameplay, ModalDirective::ContinueSession),
            PortalDirective::CancelPunchDial => (Self::MultiplayerHub, ModalDirective::CancelJoin),
            PortalDirective::ConnectToWorld(target) => (self, ModalDirective::JoinPortal(target)),
            PortalDirective::PunchGuest(peer) => (self, ModalDirective::PunchGuest(peer)),
        }
    }
}
