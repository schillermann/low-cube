use glam::{Mat4, Vec3};
use glow::HasContext;
use winit::keyboard::KeyCode;
use engine::{CpuMeshBatch, Mesh, ShaderProgram};
use world::MeshTarget;
use crate::avatar::Avatar;
use crate::controls::{InputTrigger, KeyboardControls};
use crate::surface::PlatformSurface;
use super::pause_panel::{PausePanel, PauseSelection};
use super::portal::{PortalDirective, PortalStudio};
use super::settings::{ControlStudio, SettingsDirective};
use super::studio::{StudioDirective, WardrobeStudio};
use super::widget::ScreenPoint;
use super::{ModalDirective, SessionModal};

/// Polymorphic modal state machine for gameplay, Escape pause menu, Character studio, Settings, and Co-op join.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ModalPhase {
    Gameplay,
    EscapePause,
    CharacterWardrobe,
    ControlSettings,
    PortalJoin,
}

struct StudioPanels {
    studio: WardrobeStudio,
    settings: ControlStudio,
    portal: PortalStudio,
}

impl StudioPanels {
    fn new(studio: WardrobeStudio, settings: ControlStudio, portal: PortalStudio) -> Self {
        Self { studio, settings, portal }
    }
}

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
        Self::with_portal(String::new())
    }

    #[cfg(test)]
    pub fn with_custom_portal(portal: PortalStudio) -> Self {
        Self::new(
            ModalPhase::Gameplay,
            ScreenPoint::center(),
            StudioPanels::new(WardrobeStudio::from_profile(), ControlStudio::from_profile(), portal),
        )
    }

    pub fn with_portal(portal_code: String) -> Self {
        Self::new(
            ModalPhase::Gameplay,
            ScreenPoint::center(),
            StudioPanels::new(
                WardrobeStudio::from_profile(),
                ControlStudio::from_profile(),
                PortalStudio::with_ticket(portal_code),
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

    fn toggle_escape(&mut self, surface: &dyn PlatformSurface) {
        match self.phase {
            ModalPhase::Gameplay => {
                self.phase = ModalPhase::EscapePause;
                surface.release_cursor();
            }
            ModalPhase::EscapePause => {
                self.phase = ModalPhase::Gameplay;
                surface.capture_cursor();
            }
            ModalPhase::CharacterWardrobe | ModalPhase::PortalJoin => {
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

    fn announce_portal(&mut self, ticket: String) {
        self.panels.portal.announce_hosted(ticket);
    }

    fn complete_join(&mut self, surface: &dyn PlatformSurface) {
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
        match self.phase {
            ModalPhase::Gameplay => avatar.emit_reticle(target),
            ModalPhase::EscapePause => PausePanel::new().emit(self.pointer, &self.panels.portal, target),
            ModalPhase::CharacterWardrobe => self.panels.studio.emit_overlay(target),
            ModalPhase::ControlSettings => self.panels.settings.emit_overlay(target),
            ModalPhase::PortalJoin => self.panels.portal.emit_overlay(self.pointer, target),
        }
    }

    fn emit_studio(&self, target: &mut dyn MeshTarget) {
        if matches!(self.phase, ModalPhase::CharacterWardrobe) {
            self.panels.studio.emit_stage(target);
        }
    }

    fn draw_layer(&self, gl: &glow::Context, shader: &dyn ShaderProgram, avatar: &dyn Avatar, aspect: f32) {
        if matches!(self.phase, ModalPhase::CharacterWardrobe) {
            let mut stage_batch = CpuMeshBatch::empty();
            self.emit_studio(&mut stage_batch);
            let vp = self.panels.studio.stage_projection(aspect);
            let light = Vec3::new(0.35, 0.55, 0.75).normalize();
            unsafe {
                gl.clear(glow::DEPTH_BUFFER_BIT);
            }
            shader.bind(gl, &vp, light);
            stage_batch.draw(gl);
        }
        let mut ui_batch = CpuMeshBatch::empty();
        self.emit_ui(avatar, &mut ui_batch);
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
