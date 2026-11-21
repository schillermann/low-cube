use glam::{Mat4, Vec3};
use glow::HasContext;
use winit::keyboard::KeyCode;
use engine::{CpuMeshBatch, Mesh, ShaderProgram};
use world::MeshTarget;
use crate::avatar::Avatar;
use crate::controls::{InputTrigger, KeyboardControls};
use crate::menu::black_box_panel::{BlackBoxDirective, BlackBoxPanel};
use crate::menu::clipboard::TextClipboard;
use crate::menu::pause_panel::{PausePanel, PauseSelection};
use crate::menu::portal::{BeaconHud, PortalInvite, PortalStudio, ReticleHud};
use crate::menu::settings::{ControlPanel, SavedControls, SettingsDirective};
use crate::menu::settings_panel::{SettingsPanel, SettingsSelection};
use crate::menu::studio::{SavedWardrobe, StudioDirective, WardrobePanel};
use crate::menu::view::{MenuView, VersionedView};
use crate::menu::widget::ScreenPoint;
use crate::menu::ModalDirective;
use crate::surface::PlatformSurface;
use super::phase::ModalPhase;

/// Strictly encapsulated collaborator panels owned by `EscapeMenu` (no public fields, pure messaging).
pub(super) struct StudioPanels {
    studio: Box<dyn WardrobePanel>,
    settings: Box<dyn ControlPanel>,
    portal: PortalStudio,
    black_box: BlackBoxPanel,
}

impl StudioPanels {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub(super) fn new(
        studio: Box<dyn WardrobePanel>,
        settings: Box<dyn ControlPanel>,
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
            Box::new(SavedWardrobe::live()),
            Box::new(SavedControls::live()),
            PortalStudio::with_invite(invite, portal_clipboard),
            BlackBoxPanel::with_clipboard(report_clipboard),
        )
    }

    pub(super) fn dialing_active(&self) -> bool {
        self.portal.awaiting_host_punch()
    }

    pub(super) fn open_pause(&mut self) {
        self.portal.refresh_startup_copy();
    }

    pub(super) fn cancel_join_dial(&mut self) {
        self.portal.clear_dial_state();
    }

    pub(super) fn cancel_rebind_focus(&mut self) {
        self.settings.cancel_focus();
    }

    pub(super) fn move_pointer(&mut self, phase: ModalPhase, point: ScreenPoint) {
        match phase {
            ModalPhase::CharacterWardrobe => self.studio.move_pointer(point),
            ModalPhase::ControlSettings => self.settings.move_pointer(point),
            _ => {}
        }
    }

    pub(super) fn release_grip(&mut self) {
        self.studio.release_grip();
    }

    pub(super) fn press_pause(&mut self, pointer: ScreenPoint, surface: &dyn PlatformSurface) -> (ModalPhase, ModalDirective) {
        match PausePanel::new().select(pointer) {
            PauseSelection::Resume => {
                surface.capture_cursor();
                (ModalPhase::Gameplay, ModalDirective::ContinueSession)
            }
            PauseSelection::OpenWardrobe => {
                self.studio.move_pointer(pointer);
                (ModalPhase::CharacterWardrobe, ModalDirective::ContinueSession)
            }
            PauseSelection::OpenSettings => (ModalPhase::SettingsHub, ModalDirective::ContinueSession),
            PauseSelection::OpenMultiplayer => (ModalPhase::MultiplayerHub, ModalDirective::ContinueSession),
            PauseSelection::Quit => (ModalPhase::EscapePause, ModalDirective::ExitGame),
            PauseSelection::Unmatched => (ModalPhase::EscapePause, ModalDirective::ContinueSession),
        }
    }

    pub(super) fn press_subpanel(
        &mut self,
        phase: ModalPhase,
        pointer: ScreenPoint,
        surface: &dyn PlatformSurface,
        mut avatar: &mut dyn Avatar,
    ) -> (ModalPhase, ModalDirective) {
        match phase {
            ModalPhase::CharacterWardrobe => {
                let directive = self.studio.press_pointer(&mut avatar);
                let next = if matches!(directive, StudioDirective::ReturnToPauseMenu) {
                    ModalPhase::EscapePause
                } else {
                    phase
                };
                (next, ModalDirective::ContinueSession)
            }
            ModalPhase::SettingsHub => {
                let next = match SettingsPanel::new().select(pointer) {
                    SettingsSelection::OpenKeybindings => {
                        self.settings.move_pointer(pointer);
                        ModalPhase::ControlSettings
                    }
                    SettingsSelection::OpenBlackBox => ModalPhase::FlightBlackBox,
                    SettingsSelection::ReturnToPauseMenu => ModalPhase::EscapePause,
                    SettingsSelection::Unmatched => phase,
                };
                (next, ModalDirective::ContinueSession)
            }
            ModalPhase::ControlSettings => {
                let directive = self.settings.press_pointer(&mut avatar);
                let next = if matches!(directive, SettingsDirective::ReturnToSettings) {
                    ModalPhase::SettingsHub
                } else {
                    phase
                };
                (next, ModalDirective::ContinueSession)
            }
            ModalPhase::MultiplayerHub => phase.apply_portal(self.portal.press_hub_pointer(pointer)),
            ModalPhase::PortalJoin => {
                let (next, directive) = phase.apply_portal(self.portal.press_pointer(pointer));
                if matches!(next, ModalPhase::Gameplay) {
                    surface.capture_cursor();
                }
                (next, directive)
            }
            ModalPhase::FlightBlackBox => {
                let next = match self.black_box.press_pointer(pointer, avatar) {
                    BlackBoxDirective::StayInBlackBox => phase,
                    BlackBoxDirective::ReturnToSettings => ModalPhase::SettingsHub,
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

    pub(super) fn handle_key(
        &mut self,
        phase: ModalPhase,
        code: KeyCode,
        ctrl_held: bool,
        mut avatar: &mut dyn Avatar,
    ) -> (ModalPhase, ModalDirective) {
        match phase {
            ModalPhase::CharacterWardrobe => {
                let _ = self.studio.handle_key(code, &mut avatar);
                (phase, ModalDirective::ContinueSession)
            }
            ModalPhase::ControlSettings => {
                let _ = self.settings.handle_trigger(InputTrigger::Key(code), &mut avatar);
                (phase, ModalDirective::ContinueSession)
            }
            ModalPhase::PortalJoin => phase.apply_portal(self.portal.handle_key(code, ctrl_held)),
            ModalPhase::EscapePause | ModalPhase::MultiplayerHub if ctrl_held && code == KeyCode::KeyC => {
                self.portal.copy_hosted_ticket();
                (phase, ModalDirective::ContinueSession)
            }
            ModalPhase::EscapePause | ModalPhase::MultiplayerHub if ctrl_held && code == KeyCode::KeyV => {
                phase.apply_portal(self.portal.paste_guest_reply())
            }
            _ => (phase, ModalDirective::ContinueSession),
        }
    }

    pub(super) fn append_join_text(&mut self, text: &str) {
        self.portal.append_text(text);
    }

    pub(super) fn handle_trigger(&mut self, trigger: InputTrigger, mut avatar: &mut dyn Avatar) -> bool {
        self.settings.handle_trigger(trigger, &mut avatar)
    }

    pub(super) fn sync_controls(&self, mut controls: &mut dyn KeyboardControls) {
        self.settings.sync_controls(&mut controls);
    }

    pub(super) fn announce_portal(&mut self, invite: PortalInvite) {
        self.portal.announce_hosted(invite);
    }

    pub(super) fn announce_auto_punch(&mut self) {
        self.portal.announce_auto_punch();
    }

    pub(super) fn announce_punch_code(&mut self, punch_code: String) {
        self.portal.announce_punch_reply(punch_code);
    }

    pub(super) fn report_join_failure(&mut self) {
        self.portal.report_connection_failure();
    }

    pub(super) fn step_overlays(&mut self, phase: ModalPhase, dt: f32) {
        self.portal.step(dt);
        if matches!(phase, ModalPhase::CharacterWardrobe) {
            self.studio.step(dt);
        }
    }

    pub(super) fn emit_studio(&self, target: &mut dyn MeshTarget) {
        self.studio.emit_stage(target);
    }

    pub(super) fn emit_ui(
        &self,
        phase: ModalPhase,
        pointer: ScreenPoint,
        avatar: &dyn Avatar,
        aspect: f32,
        target: &mut dyn MeshTarget,
    ) {
        match phase {
            ModalPhase::Gameplay => {
                BeaconHud::new(self.portal.beacon(), ReticleHud::new(avatar, aspect)).emit(target);
            }
            ModalPhase::EscapePause => {
                let pause = PausePanel::new().view(pointer, self.portal.hosted_active());
                VersionedView::current(BeaconHud::new(self.portal.beacon(), pause)).emit(target);
            }
            ModalPhase::CharacterWardrobe => VersionedView::current(&*self.studio).emit(target),
            ModalPhase::SettingsHub => {
                VersionedView::current(SettingsPanel::new().view(pointer, avatar.black_box_active())).emit(target);
            }
            ModalPhase::ControlSettings => VersionedView::current(&*self.settings).emit(target),
            ModalPhase::MultiplayerHub => VersionedView::current(self.portal.hub_view(pointer)).emit(target),
            ModalPhase::PortalJoin => self.portal.emit_join_screen(pointer, target),
            ModalPhase::FlightBlackBox => VersionedView::current(self.black_box.view(pointer, avatar)).emit(target),
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
