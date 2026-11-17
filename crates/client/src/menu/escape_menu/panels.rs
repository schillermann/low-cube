use glam::{Mat4, Vec3};
use glow::HasContext;
use engine::{CpuMeshBatch, Mesh, ShaderProgram};
use world::MeshTarget;
use crate::avatar::Avatar;
use crate::menu::black_box_panel::{BlackBoxDirective, BlackBoxPanel};
use crate::menu::clipboard::TextClipboard;
use crate::menu::pause_panel::PausePanel;
use crate::menu::portal::{PortalDirective, PortalInvite, PortalStudio};
use crate::menu::settings::{ControlStudio, SettingsDirective};
use crate::menu::studio::{StudioDirective, WardrobeStudio};
use crate::menu::version_stamp::VersionStamp;
use crate::menu::widget::ScreenPoint;
use crate::menu::ModalDirective;
use crate::surface::PlatformSurface;

/// Polymorphic modal state machine for gameplay, Escape pause menu, Character studio, Settings, Multiplayer, and Blackbox.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ModalPhase {
    Gameplay,
    EscapePause,
    CharacterWardrobe,
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
            PortalDirective::ConnectToWorld(target) => (self, ModalDirective::JoinPortal(target)),
            PortalDirective::PunchGuest(peer) => (self, ModalDirective::PunchGuest(peer)),
        }
    }
}

/// Collaborator panels owned by `EscapeMenu` (wardrobe, controls, Co-op portal, and Flight Blackbox).
pub(super) struct StudioPanels {
    pub(super) studio: WardrobeStudio,
    pub(super) settings: ControlStudio,
    pub(super) portal: PortalStudio,
    pub(super) black_box: BlackBoxPanel,
}

impl StudioPanels {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub(super) fn new(
        studio: WardrobeStudio,
        settings: ControlStudio,
        portal: PortalStudio,
        black_box: BlackBoxPanel,
    ) -> Self {
        Self { studio, settings, portal, black_box }
    }

    pub(super) fn assembled(
        invite: PortalInvite,
        portal_clipboard: Box<dyn TextClipboard>,
        report_clipboard: Box<dyn TextClipboard>,
    ) -> Self {
        Self::new(
            WardrobeStudio::from_profile(),
            ControlStudio::from_profile(),
            PortalStudio::with_invite(invite, portal_clipboard),
            BlackBoxPanel::with_clipboard(report_clipboard),
        )
    }

    pub(super) fn press_subpanel(
        &mut self,
        phase: ModalPhase,
        pointer: ScreenPoint,
        surface: &dyn PlatformSurface,
        avatar: &mut dyn Avatar,
    ) -> (ModalPhase, ModalDirective) {
        match phase {
            ModalPhase::CharacterWardrobe => {
                let directive = self.studio.press_pointer();
                avatar.apply_attire(self.studio.current_attire());
                let next = if matches!(directive, StudioDirective::ReturnToPauseMenu) {
                    ModalPhase::EscapePause
                } else {
                    phase
                };
                (next, ModalDirective::ContinueSession)
            }
            ModalPhase::ControlSettings => {
                let (directive, changed) = self.settings.press_pointer();
                if changed {
                    avatar.apply_bindings(self.settings.current_bindings());
                }
                let next = if matches!(directive, SettingsDirective::ReturnToPauseMenu) {
                    ModalPhase::EscapePause
                } else {
                    phase
                };
                (next, ModalDirective::ContinueSession)
            }
            ModalPhase::MultiplayerHub => phase.apply_portal(self.portal.press_hub_pointer(pointer)),
            ModalPhase::PortalJoin => phase.apply_portal(self.portal.press_pointer(pointer)),
            ModalPhase::FlightBlackBox => {
                let next = match self.black_box.press_pointer(pointer, avatar) {
                    BlackBoxDirective::StayInBlackBox => phase,
                    BlackBoxDirective::ReturnToPauseMenu => ModalPhase::EscapePause,
                    BlackBoxDirective::ResumeWithPolaroid => {
                        surface.capture_cursor();
                        ModalPhase::Gameplay
                    }
                };
                (next, ModalDirective::ContinueSession)
            }
            ModalPhase::Gameplay | ModalPhase::EscapePause => (phase, ModalDirective::ContinueSession),
        }
    }

    pub(super) fn emit_ui(
        &self,
        phase: ModalPhase,
        pointer: ScreenPoint,
        avatar: &dyn Avatar,
        target: &mut dyn MeshTarget,
    ) {
        match phase {
            ModalPhase::Gameplay => avatar.emit_reticle(target),
            ModalPhase::EscapePause => {
                PausePanel::new().emit(pointer, &self.portal, avatar.black_box_active(), target);
                VersionStamp::current().emit(target);
            }
            ModalPhase::CharacterWardrobe => {
                self.studio.emit_overlay(target);
                VersionStamp::current().emit(target);
            }
            ModalPhase::ControlSettings => {
                self.settings.emit_overlay(target);
                VersionStamp::current().emit(target);
            }
            ModalPhase::MultiplayerHub => {
                self.portal.emit_multiplayer_hub(pointer, target);
                VersionStamp::current().emit(target);
            }
            ModalPhase::PortalJoin => {
                self.portal.emit_overlay(pointer, target);
                VersionStamp::current().emit(target);
            }
            ModalPhase::FlightBlackBox => {
                self.black_box.emit(pointer, avatar, target);
                VersionStamp::current().emit(target);
            }
        }
    }

    pub(super) fn draw_overlay(
        &self,
        phase: ModalPhase,
        gl: &glow::Context,
        shader: &dyn ShaderProgram,
        stage_batch: CpuMeshBatch,
        ui_batch: CpuMeshBatch,
        aspect: f32,
    ) {
        if matches!(phase, ModalPhase::CharacterWardrobe) {
            let vp = self.studio.stage_projection(aspect);
            let light = Vec3::new(0.35, 0.55, 0.75).normalize();
            unsafe {
                gl.clear(glow::DEPTH_BUFFER_BIT);
            }
            shader.bind(gl, &vp, light);
            stage_batch.draw(gl);
        }
        unsafe {
            gl.disable(glow::DEPTH_TEST);
            gl.enable(glow::BLEND);
            gl.blend_func(glow::SRC_ALPHA, glow::ONE_MINUS_SRC_ALPHA);
        }
        shader.bind(gl, &Mat4::IDENTITY, Vec3::Z);
        ui_batch.draw(gl);
        unsafe {
            gl.disable(glow::BLEND);
            gl.enable(glow::DEPTH_TEST);
        }
    }
}
