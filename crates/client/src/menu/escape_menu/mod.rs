mod panels;

use winit::keyboard::KeyCode;
use engine::{CpuMeshBatch, ShaderProgram};
use world::MeshTarget;
use crate::avatar::Avatar;
use crate::controls::{InputTrigger, KeyboardControls};
use crate::surface::PlatformSurface;
use super::clipboard::TextClipboard;
#[cfg(test)]
use super::clipboard::MemoryClipboard;
use super::pause_panel::{PausePanel, PauseSelection};
use super::portal::{PortalDirective, PortalInvite, PortalStudio};
use super::settings::{ControlStudio, SettingsDirective};
use super::studio::{StudioDirective, WardrobeStudio};
use super::widget::ScreenPoint;
use super::{ModalDirective, SessionModal};
use panels::{ModalPhase, StudioPanels};

/// Encapsulated Escape menu, 3D Character Studio, Keybinding Settings, and Co-op Portal studio.
pub struct EscapeMenu {
    phase: ModalPhase,
    pointer: ScreenPoint,
    panels: StudioPanels,
}

impl EscapeMenu {
    /// Primary constructor doing nothing except assigning arguments to fields.
    fn new(phase: ModalPhase, pointer: ScreenPoint, panels: StudioPanels) -> Self {
        Self { phase, pointer, panels }
    }

    #[cfg(test)]
    pub fn closed() -> Self {
        Self::with_clipboard(PortalInvite::empty(), Box::new(MemoryClipboard::seeded("")))
    }

    #[cfg(test)]
    pub fn with_custom_portal(portal: PortalStudio) -> Self {
        Self::new(
            ModalPhase::Gameplay,
            ScreenPoint::center(),
            StudioPanels::new(WardrobeStudio::from_profile(), ControlStudio::from_profile(), portal),
        )
    }

    pub fn with_clipboard(invite: PortalInvite, clipboard: Box<dyn TextClipboard>) -> Self {
        Self::new(
            ModalPhase::Gameplay,
            ScreenPoint::center(),
            StudioPanels::new(
                WardrobeStudio::from_profile(),
                ControlStudio::from_profile(),
                PortalStudio::with_invite(invite, clipboard),
            ),
        )
    }

    fn map_portal_directive(&mut self, directive: PortalDirective) -> ModalDirective {
        match directive {
            PortalDirective::StayInPortal => ModalDirective::ContinueSession,
            PortalDirective::ReturnToPauseMenu => {
                self.phase = ModalPhase::EscapePause;
                ModalDirective::ContinueSession
            }
            PortalDirective::ConnectToWorld(target) => ModalDirective::JoinPortal(target),
            PortalDirective::PunchGuest(peer) => ModalDirective::PunchGuest(peer),
        }
    }

    fn press_pause_panel(&mut self, surface: &dyn PlatformSurface) -> ModalDirective {
        match PausePanel::new().select(self.pointer) {
            PauseSelection::ResumeGame => {
                self.phase = ModalPhase::Gameplay;
                surface.capture_cursor();
                ModalDirective::ContinueSession
            }
            PauseSelection::OpenWardrobe => {
                self.phase = ModalPhase::CharacterWardrobe;
                self.panels.studio.move_pointer(self.pointer);
                ModalDirective::ContinueSession
            }
            PauseSelection::OpenSettings => {
                self.phase = ModalPhase::ControlSettings;
                self.panels.settings.move_pointer(self.pointer);
                ModalDirective::ContinueSession
            }
            PauseSelection::HostOrCopyPortal => {
                if self.panels.portal.hosted_active() {
                    self.panels.portal.copy_hosted_ticket();
                    ModalDirective::ContinueSession
                } else {
                    ModalDirective::OpenPortal
                }
            }
            PauseSelection::OpenJoinPortal => {
                self.phase = ModalPhase::PortalJoin;
                ModalDirective::ContinueSession
            }
            PauseSelection::QuitGame => ModalDirective::ExitGame,
            PauseSelection::Unmatched => ModalDirective::ContinueSession,
        }
    }
}

impl SessionModal for EscapeMenu {
    fn open(&self) -> bool {
        !matches!(self.phase, ModalPhase::Gameplay)
    }

    fn dialing_active(&self) -> bool {
        matches!(self.phase, ModalPhase::PortalJoin)
    }

    fn toggle_escape(&mut self, surface: &dyn PlatformSurface) {
        match self.phase {
            ModalPhase::Gameplay => {
                self.phase = ModalPhase::EscapePause;
                self.panels.portal.refresh_startup_copy();
                surface.release_cursor();
            }
            ModalPhase::EscapePause => {
                self.phase = ModalPhase::Gameplay;
                surface.capture_cursor();
            }
            ModalPhase::CharacterWardrobe | ModalPhase::PortalJoin => {
                self.panels.portal.clear_dial_state();
                self.phase = ModalPhase::EscapePause;
                surface.release_cursor();
            }
            ModalPhase::ControlSettings => {
                self.panels.settings.cancel_focus();
                self.phase = ModalPhase::EscapePause;
                surface.release_cursor();
            }
        }
    }

    fn move_cursor(&mut self, point: ScreenPoint) {
        self.pointer = point;
        match self.phase {
            ModalPhase::CharacterWardrobe => self.panels.studio.move_pointer(point),
            ModalPhase::ControlSettings => self.panels.settings.move_pointer(point),
            _ => {}
        }
    }

    fn press_mouse(&mut self, surface: &dyn PlatformSurface, avatar: &mut dyn Avatar) -> ModalDirective {
        match self.phase {
            ModalPhase::Gameplay => ModalDirective::ContinueSession,
            ModalPhase::EscapePause => self.press_pause_panel(surface),
            ModalPhase::CharacterWardrobe => {
                let directive = self.panels.studio.press_pointer();
                avatar.apply_attire(self.panels.studio.current_attire());
                if matches!(directive, StudioDirective::ReturnToPauseMenu) {
                    self.phase = ModalPhase::EscapePause;
                }
                ModalDirective::ContinueSession
            }
            ModalPhase::ControlSettings => {
                let (directive, changed) = self.panels.settings.press_pointer();
                if changed {
                    avatar.apply_bindings(self.panels.settings.current_bindings());
                }
                if matches!(directive, SettingsDirective::ReturnToPauseMenu) {
                    self.phase = ModalPhase::EscapePause;
                }
                ModalDirective::ContinueSession
            }
            ModalPhase::PortalJoin => {
                let directive = self.panels.portal.press_pointer(self.pointer);
                self.map_portal_directive(directive)
            }
        }
    }

    fn release_mouse(&mut self) {
        self.panels.studio.release_pointer();
    }

    fn handle_key(&mut self, code: KeyCode, ctrl_held: bool, avatar: &mut dyn Avatar) -> ModalDirective {
        match self.phase {
            ModalPhase::CharacterWardrobe if self.panels.studio.handle_key(code) => {
                avatar.apply_attire(self.panels.studio.current_attire());
            }
            ModalPhase::ControlSettings => {
                let _ = self.handle_trigger(InputTrigger::Key(code), avatar);
            }
            ModalPhase::PortalJoin => {
                let directive = self.panels.portal.handle_key(code, ctrl_held);
                return self.map_portal_directive(directive);
            }
            ModalPhase::EscapePause if ctrl_held && code == KeyCode::KeyC => {
                self.panels.portal.copy_hosted_ticket();
            }
            _ => {}
        }
        ModalDirective::ContinueSession
    }

    fn handle_text(&mut self, text: &str) {
        if matches!(self.phase, ModalPhase::PortalJoin) {
            self.panels.portal.append_text(text);
        }
    }

    fn handle_trigger(&mut self, trigger: InputTrigger, avatar: &mut dyn Avatar) -> bool {
        if matches!(self.phase, ModalPhase::ControlSettings) && self.panels.settings.handle_trigger(trigger) {
            avatar.apply_bindings(self.panels.settings.current_bindings());
            return true;
        }
        false
    }

    fn sync_controls(&self, controls: &mut dyn KeyboardControls) {
        controls.apply_bindings(self.panels.settings.current_bindings());
    }

    fn announce_portal(&mut self, invite: PortalInvite) {
        self.panels.portal.announce_hosted(invite);
    }

    fn announce_punch_code(&mut self, punch_code: String) {
        self.phase = ModalPhase::PortalJoin;
        self.panels.portal.announce_punch_reply(punch_code);
    }

    fn complete_join(&mut self, surface: &dyn PlatformSurface) {
        self.panels.portal.clear_dial_state();
        self.phase = ModalPhase::Gameplay;
        surface.capture_cursor();
    }

    fn report_join_failure(&mut self) {
        self.phase = ModalPhase::PortalJoin;
        self.panels.portal.report_connection_failure();
    }

    fn step(&mut self, dt: f32) {
        if matches!(self.phase, ModalPhase::CharacterWardrobe) {
            self.panels.studio.step(dt);
        }
    }

    fn emit_ui(&self, avatar: &dyn Avatar, target: &mut dyn MeshTarget) {
        self.panels.emit_ui(self.phase, self.pointer, avatar, target);
    }

    fn emit_studio(&self, target: &mut dyn MeshTarget) {
        if matches!(self.phase, ModalPhase::CharacterWardrobe) {
            self.panels.studio.emit_stage(target);
        }
    }

    fn draw_layer(&self, gl: &glow::Context, shader: &dyn ShaderProgram, avatar: &dyn Avatar, aspect: f32) {
        let mut stage_batch = CpuMeshBatch::empty();
        self.emit_studio(&mut stage_batch);
        let mut ui_batch = CpuMeshBatch::empty();
        self.emit_ui(avatar, &mut ui_batch);
        self.panels.draw_overlay(self.phase, gl, shader, stage_batch, ui_batch, aspect);
    }
}
