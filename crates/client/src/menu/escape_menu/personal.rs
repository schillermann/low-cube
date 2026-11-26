use glam::Mat4;
use winit::keyboard::KeyCode;
use world::MeshTarget;
use crate::avatar::Avatar;
use crate::controls::{InputTrigger, KeyboardControls};
use crate::menu::black_box_panel::{BlackBoxDirective, BlackBoxPanel};
use crate::menu::clipboard::TextClipboard;
use crate::menu::settings::{ControlPanel, SavedControls, SettingsDirective};
use crate::menu::settings_panel::{SettingsPanel, SettingsSelection};
use crate::menu::studio::{SavedWardrobe, StudioDirective, WardrobePanel};
use crate::menu::view::{MenuView, VersionedView};
use crate::menu::widget::ScreenPoint;
use crate::surface::PlatformSurface;
use super::phase::ModalPhase;

/// Encapsulated Character Wardrobe, Keybinding Settings, and Flight Blackbox panels (`3` fields).
pub(super) struct PersonalPanels {
    studio: Box<dyn WardrobePanel>,
    settings: Box<dyn ControlPanel>,
    black_box: BlackBoxPanel,
}

impl PersonalPanels {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub(super) fn new(
        studio: Box<dyn WardrobePanel>,
        settings: Box<dyn ControlPanel>,
        black_box: BlackBoxPanel,
    ) -> Self {
        Self { studio, settings, black_box }
    }

    pub(super) fn live(report_clipboard: Box<dyn TextClipboard>) -> Self {
        Self::new(
            Box::new(SavedWardrobe::live()),
            Box::new(SavedControls::live()),
            BlackBoxPanel::with_clipboard(report_clipboard),
        )
    }

    pub(super) fn cancel_rebind(&mut self) {
        self.settings.cancel_focus();
    }

    pub(super) fn release_grip(&mut self) {
        self.studio.release_grip();
    }

    pub(super) fn move_pointer(&mut self, phase: ModalPhase, point: ScreenPoint) {
        match phase {
            ModalPhase::CharacterWardrobe => self.studio.move_pointer(point),
            ModalPhase::ControlSettings => self.settings.move_pointer(point),
            _ => {}
        }
    }

    pub(super) fn focus_wardrobe(&mut self, point: ScreenPoint) {
        self.studio.move_pointer(point);
    }

    pub(super) fn press_panel(
        &mut self,
        phase: ModalPhase,
        pointer: ScreenPoint,
        surface: &dyn PlatformSurface,
        mut avatar: &mut dyn Avatar,
    ) -> ModalPhase {
        match phase {
            ModalPhase::CharacterWardrobe => {
                if matches!(self.studio.press_pointer(&mut avatar), StudioDirective::ReturnToPauseMenu) {
                    ModalPhase::EscapePause
                } else {
                    phase
                }
            }
            ModalPhase::SettingsHub => match SettingsPanel::new().select(pointer) {
                SettingsSelection::OpenKeybindings => {
                    self.settings.move_pointer(pointer);
                    ModalPhase::ControlSettings
                }
                SettingsSelection::OpenBlackBox => ModalPhase::FlightBlackBox,
                SettingsSelection::ReturnToPauseMenu => ModalPhase::EscapePause,
                SettingsSelection::Unmatched => phase,
            },
            ModalPhase::ControlSettings => {
                if matches!(self.settings.press_pointer(&mut avatar), SettingsDirective::ReturnToSettings) {
                    ModalPhase::SettingsHub
                } else {
                    phase
                }
            }
            ModalPhase::FlightBlackBox => match self.black_box.press_pointer(pointer, avatar) {
                BlackBoxDirective::StayInBlackBox => phase,
                BlackBoxDirective::ReturnToSettings => ModalPhase::SettingsHub,
                BlackBoxDirective::ResumeWithPolaroid => {
                    surface.capture_cursor();
                    ModalPhase::Gameplay
                }
            },
            _ => phase,
        }
    }

    pub(super) fn handle_key(&mut self, phase: ModalPhase, code: KeyCode, mut avatar: &mut dyn Avatar) {
        match phase {
            ModalPhase::CharacterWardrobe => {
                let _ = self.studio.handle_key(code, &mut avatar);
            }
            ModalPhase::ControlSettings => {
                let _ = self.settings.handle_trigger(InputTrigger::Key(code), &mut avatar);
            }
            _ => {}
        }
    }

    pub(super) fn handle_trigger(&mut self, trigger: InputTrigger, mut avatar: &mut dyn Avatar) -> bool {
        self.settings.handle_trigger(trigger, &mut avatar)
    }

    pub(super) fn sync_controls(&self, mut controls: &mut dyn KeyboardControls) {
        self.settings.sync_controls(&mut controls);
    }

    pub(super) fn step_wardrobe(&mut self, dt: f32) {
        self.studio.step(dt);
    }

    pub(super) fn emit_stage(&self, target: &mut dyn MeshTarget) {
        self.studio.emit_stage(target);
    }

    pub(super) fn stage_projection(&self, aspect: f32) -> Mat4 {
        self.studio.stage_projection(aspect)
    }

    pub(super) fn emit_ui(&self, phase: ModalPhase, pointer: ScreenPoint, avatar: &dyn Avatar, target: &mut dyn MeshTarget) {
        match phase {
            ModalPhase::CharacterWardrobe => VersionedView::current(&*self.studio).emit(target),
            ModalPhase::SettingsHub => {
                VersionedView::current(SettingsPanel::new().view(pointer, avatar.black_box_active())).emit(target);
            }
            ModalPhase::ControlSettings => VersionedView::current(&*self.settings).emit(target),
            ModalPhase::FlightBlackBox => VersionedView::current(self.black_box.view(pointer, avatar)).emit(target),
            _ => {}
        }
    }
}
