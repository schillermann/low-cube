mod panels;

use winit::keyboard::KeyCode;
use engine::{CpuMeshBatch, ShaderProgram};
use world::MeshTarget;
use crate::avatar::Avatar;
use crate::controls::{InputTrigger, KeyboardControls};
use crate::surface::PlatformSurface;
#[cfg(test)]
use super::black_box_panel::BlackBoxPanel;
use super::clipboard::TextClipboard;
#[cfg(test)]
use super::clipboard::{MemoryClipboard, SystemClipboard};
use super::portal::PortalInvite;
#[cfg(test)]
use super::portal::PortalStudio;
#[cfg(test)]
use super::settings::ControlStudio;
#[cfg(test)]
use super::studio::WardrobeStudio;
use super::widget::ScreenPoint;
use super::{ModalDirective, SessionModal};
use panels::{ModalPhase, StudioPanels};

/// Encapsulated Escape menu, 3D Character Studio, Settings submenu, Co-op Portal, and Flight Blackbox studio.
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
            StudioPanels::new(
                Box::new(WardrobeStudio::seeded()),
                Box::new(ControlStudio::seeded()),
                portal,
                BlackBoxPanel::with_clipboard(Box::new(MemoryClipboard::seeded(""))),
            ),
        )
    }

    #[cfg(test)]
    pub fn with_custom_black_box(black_box: BlackBoxPanel) -> Self {
        Self::new(
            ModalPhase::Gameplay,
            ScreenPoint::center(),
            StudioPanels::new(
                Box::new(WardrobeStudio::seeded()),
                Box::new(ControlStudio::seeded()),
                PortalStudio::with_invite(PortalInvite::empty(), Box::new(MemoryClipboard::seeded(""))),
                black_box,
            ),
        )
    }

    #[cfg(test)]
    pub fn with_clipboard(invite: PortalInvite, clipboard: Box<dyn TextClipboard>) -> Self {
        Self::new(
            ModalPhase::Gameplay,
            ScreenPoint::center(),
            StudioPanels::new(
                Box::new(WardrobeStudio::seeded()),
                Box::new(ControlStudio::seeded()),
                PortalStudio::with_invite(invite, clipboard),
                BlackBoxPanel::with_clipboard(SystemClipboard::new().detached()),
            ),
        )
    }

    pub fn with_clipboards(
        invite: PortalInvite,
        portal_clipboard: Box<dyn TextClipboard>,
        report_clipboard: Box<dyn TextClipboard>,
    ) -> Self {
        Self::new(
            ModalPhase::Gameplay,
            ScreenPoint::center(),
            StudioPanels::assembled(invite, portal_clipboard, report_clipboard),
        )
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
                self.panels.open_pause();
                surface.release_cursor();
            }
            ModalPhase::EscapePause => {
                self.phase = ModalPhase::Gameplay;
                surface.capture_cursor();
            }
            ModalPhase::CharacterWardrobe | ModalPhase::SettingsHub | ModalPhase::MultiplayerHub => {
                self.phase = ModalPhase::EscapePause;
                surface.release_cursor();
            }
            ModalPhase::PortalJoin => {
                self.panels.cancel_join_dial();
                self.phase = ModalPhase::MultiplayerHub;
                surface.release_cursor();
            }
            ModalPhase::ControlSettings => {
                self.panels.cancel_rebind_focus();
                self.phase = ModalPhase::SettingsHub;
                surface.release_cursor();
            }
            ModalPhase::FlightBlackBox => {
                self.phase = ModalPhase::SettingsHub;
                surface.release_cursor();
            }
        }
    }

    fn move_cursor(&mut self, point: ScreenPoint) {
        self.pointer = point;
        self.panels.move_pointer(self.phase, point);
    }

    fn press_mouse(&mut self, surface: &dyn PlatformSurface, avatar: &mut dyn Avatar) -> ModalDirective {
        match self.phase {
            ModalPhase::Gameplay => ModalDirective::ContinueSession,
            ModalPhase::EscapePause => {
                let (next_phase, directive) = self.panels.press_pause(self.pointer, surface);
                self.phase = next_phase;
                directive
            }
            other => {
                let (next_phase, directive) = self.panels.press_subpanel(other, self.pointer, surface, avatar);
                self.phase = next_phase;
                directive
            }
        }
    }

    fn release_mouse(&mut self) {
        self.panels.release_grip();
    }

    fn handle_key(&mut self, code: KeyCode, ctrl_held: bool, avatar: &mut dyn Avatar) -> ModalDirective {
        let (next_phase, directive) = self.panels.handle_key(self.phase, code, ctrl_held, avatar);
        self.phase = next_phase;
        directive
    }

    fn handle_text(&mut self, text: &str) {
        if matches!(self.phase, ModalPhase::PortalJoin) {
            self.panels.append_join_text(text);
        }
    }

    fn handle_trigger(&mut self, trigger: InputTrigger, avatar: &mut dyn Avatar) -> bool {
        matches!(self.phase, ModalPhase::ControlSettings) && self.panels.handle_trigger(trigger, avatar)
    }

    fn sync_controls(&self, controls: &mut dyn KeyboardControls) {
        self.panels.sync_controls(controls);
    }

    fn announce_portal(&mut self, invite: PortalInvite) {
        self.panels.announce_portal(invite);
    }

    fn announce_punch_code(&mut self, punch_code: String) {
        self.phase = ModalPhase::PortalJoin;
        self.panels.announce_punch_code(punch_code);
    }

    fn complete_join(&mut self, surface: &dyn PlatformSurface) {
        self.panels.cancel_join_dial();
        self.phase = ModalPhase::Gameplay;
        surface.capture_cursor();
    }

    fn report_join_failure(&mut self) {
        self.phase = ModalPhase::PortalJoin;
        self.panels.report_join_failure();
    }

    fn step(&mut self, dt: f32) {
        if matches!(self.phase, ModalPhase::CharacterWardrobe) {
            self.panels.step_studio(dt);
        }
    }

    fn emit_ui(&self, avatar: &dyn Avatar, target: &mut dyn MeshTarget) {
        self.panels.emit_ui(self.phase, self.pointer, avatar, target);
    }

    fn emit_studio(&self, target: &mut dyn MeshTarget) {
        if matches!(self.phase, ModalPhase::CharacterWardrobe) {
            self.panels.emit_studio(target);
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
