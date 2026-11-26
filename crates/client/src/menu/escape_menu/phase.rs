use crate::menu::portal::PortalDirective;
use crate::menu::worlds::WorldPanelDirective;
use crate::menu::ModalDirective;

/// Pure modal phase state machine free of startup-lobby flags (lobby guarding lives in `LobbyGateModal`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ModalPhase {
    Gameplay,
    EscapePause,
    WorldRoster,
    WorldCreation,
    CharacterWardrobe,
    SettingsHub,
    ControlSettings,
    MultiplayerHub,
    PortalJoin,
    FlightBlackBox,
}

impl ModalPhase {
    pub(super) const fn initial_worlds(empty_roster: bool) -> Self {
        if empty_roster { Self::WorldCreation } else { Self::WorldRoster }
    }

    pub(super) fn apply_worlds(self, directive: WorldPanelDirective, outcome: ModalDirective) -> (Self, ModalDirective) {
        let next = match directive {
            WorldPanelDirective::Stay => self,
            WorldPanelDirective::OpenRoster => Self::WorldRoster,
            WorldPanelDirective::OpenCreation => Self::WorldCreation,
            WorldPanelDirective::ReturnToMainMenu => Self::EscapePause,
            WorldPanelDirective::CloseIntoWorld => Self::Gameplay,
        };
        (next, outcome)
    }

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
