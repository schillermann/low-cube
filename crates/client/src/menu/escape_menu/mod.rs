mod panels;
mod personal;
mod phase;

use winit::keyboard::KeyCode;
use engine::{CpuMeshBatch, ShaderProgram};
use world::MeshTarget;
use crate::avatar::Avatar;
use crate::controls::{InputTrigger, KeyboardControls};
use crate::surface::PlatformSurface;
use crate::world_vault::DiskWorldVault;
#[cfg(test)]
use super::black_box_panel::BlackBoxPanel;
use super::clipboard::TextClipboard;
#[cfg(test)]
use super::clipboard::{MemoryClipboard, SystemClipboard};
use super::portal::PortalInvite;
#[cfg(test)]
use super::portal::PortalStudio;
use super::widget::ScreenPoint;
#[cfg(test)]
use super::worlds::{WorldPanel, WorldStudio};
use super::{ModalDirective, SessionModal};
use panels::StudioPanels;
use phase::ModalPhase;

/// Encapsulated Escape menu, World Studio, 3D Character Studio, Settings, Co-op Portal, and Flight Blackbox (`3` fields).
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
        let bb = BlackBoxPanel::with_clipboard(Box::new(MemoryClipboard::seeded("")));
        Self::new(ModalPhase::Gameplay, ScreenPoint::center(), StudioPanels::seeded(Box::new(WorldStudio::empty()), portal, bb))
    }

    #[cfg(test)]
    pub fn with_custom_black_box(black_box: BlackBoxPanel) -> Self {
        let portal = PortalStudio::with_invite(PortalInvite::empty(), Box::new(MemoryClipboard::seeded("")));
        Self::new(ModalPhase::Gameplay, ScreenPoint::center(), StudioPanels::seeded(Box::new(WorldStudio::empty()), portal, black_box))
    }

    #[cfg(test)]
    pub fn with_clipboard(invite: PortalInvite, clipboard: Box<dyn TextClipboard>) -> Self {
        let portal = PortalStudio::with_invite(invite, clipboard);
        let bb = BlackBoxPanel::with_clipboard(SystemClipboard::new().detached());
        Self::new(ModalPhase::Gameplay, ScreenPoint::center(), StudioPanels::seeded(Box::new(WorldStudio::empty()), portal, bb))
    }

    #[cfg(test)]
    pub fn opened_on_worlds(worlds: Box<dyn WorldPanel>) -> Self {
        let portal = PortalStudio::with_invite(PortalInvite::empty(), Box::new(MemoryClipboard::seeded("")));
        let bb = BlackBoxPanel::with_clipboard(Box::new(MemoryClipboard::seeded("")));
        let panels = StudioPanels::seeded(worlds, portal, bb);
        Self::new(ModalPhase::initial_worlds(panels.empty_worlds()), ScreenPoint::center(), panels)
    }

    pub fn with_vault_and_clipboards(
        vault: DiskWorldVault,
        invite: PortalInvite,
        portal_clipboard: Box<dyn TextClipboard>,
        report_clipboard: Box<dyn TextClipboard>,
    ) -> Self {
        Self::new(
            ModalPhase::Gameplay,
            ScreenPoint::center(),
            StudioPanels::assembled(vault, invite, portal_clipboard, report_clipboard),
        )
    }

    pub fn startup_worlds(
        vault: DiskWorldVault,
        invite: PortalInvite,
        portal_clipboard: Box<dyn TextClipboard>,
        report_clipboard: Box<dyn TextClipboard>,
    ) -> Self {
        let panels = StudioPanels::assembled(vault, invite, portal_clipboard, report_clipboard);
        Self::new(ModalPhase::initial_worlds(panels.empty_worlds()), ScreenPoint::center(), panels)
    }
}

impl SessionModal for EscapeMenu {
    fn open(&self) -> bool {
        !matches!(self.phase, ModalPhase::Gameplay)
    }

    #[cfg(test)]
    fn dialing_active(&self) -> bool {
        self.panels.dialing_active()
    }

    fn toggle_escape(&mut self, surface: &dyn PlatformSurface) -> ModalDirective {
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
            ModalPhase::WorldCreation => {
                self.phase = if self.panels.empty_worlds() {
                    ModalPhase::EscapePause
                } else {
                    ModalPhase::WorldRoster
                };
                surface.release_cursor();
            }
            ModalPhase::WorldRoster
            | ModalPhase::CharacterWardrobe
            | ModalPhase::SettingsHub
            | ModalPhase::MultiplayerHub => {
                self.phase = ModalPhase::EscapePause;
                surface.release_cursor();
            }
            ModalPhase::PortalJoin => {
                if self.panels.dialing_active() {
                    self.phase = ModalPhase::Gameplay;
                    surface.capture_cursor();
                } else {
                    self.phase = ModalPhase::MultiplayerHub;
                    surface.release_cursor();
                }
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
        ModalDirective::ContinueSession
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
        self.panels.append_text(self.phase, text);
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

    fn announce_auto_punch(&mut self) {
        self.panels.announce_auto_punch();
    }

    fn announce_punch_code(&mut self, punch_code: String) {
        self.phase = ModalPhase::PortalJoin;
        self.panels.announce_punch_code(punch_code);
    }

    fn complete_join(&mut self, surface: &dyn PlatformSurface) {
        self.panels.cancel_join_dial();
        self.panels.clear_active_world();
        self.phase = ModalPhase::Gameplay;
        surface.capture_cursor();
    }

    fn report_join_failure(&mut self) {
        self.phase = ModalPhase::PortalJoin;
        self.panels.report_join_failure();
    }

    fn step(&mut self, dt: f32) {
        self.panels.step_overlays(self.phase, dt);
    }

    #[cfg(test)]
    fn emit_ui(&self, avatar: &dyn Avatar, target: &mut dyn MeshTarget) {
        self.panels.emit_ui(self.phase, self.pointer, avatar, 16.0 / 9.0, target);
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
        self.panels.emit_ui(self.phase, self.pointer, avatar, aspect, &mut ui_batch);
        self.panels.draw_overlay(self.phase, gl, shader, stage_batch, ui_batch, aspect);
    }
}
