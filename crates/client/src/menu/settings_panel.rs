use glam::Vec3;
use world::MeshTarget;
use super::view::{MenuView, TitledCard};
use super::widget::{ScreenBounds, ScreenPoint};

/// Selection resolved from a mouse click on the Settings submenu.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SettingsSelection {
    Unmatched,
    OpenKeybindings,
    OpenBlackBox,
    ReturnToPauseMenu,
}

/// Button content of the Settings submenu decorated by `TitledCard`.
pub struct SettingsBody {
    pointer: ScreenPoint,
    log_active: bool,
}

impl SettingsBody {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(pointer: ScreenPoint, log_active: bool) -> Self {
        Self { pointer, log_active }
    }
}

impl MenuView for SettingsBody {
    fn emit(&self, target: &mut dyn MeshTarget) {
        let layout = SettingsPanel::new();
        let log_col = if self.log_active {
            Vec3::new(0.50, 0.34, 0.14)
        } else {
            Vec3::new(0.26, 0.28, 0.34)
        };
        layout
            .keybindings_button()
            .emit_button("KEYBINDINGS", self.pointer, Vec3::new(0.28, 0.26, 0.48), false, target);
        layout
            .log_button()
            .emit_button("LOG", self.pointer, log_col, self.log_active, target);
        layout
            .back_button()
            .emit_button("BACK", self.pointer, Vec3::new(0.24, 0.28, 0.38), false, target);
    }
}

/// Encapsulated Settings submenu layout, hit-testing, and decorated `MenuView` assembly.
pub struct SettingsPanel;

impl SettingsPanel {
    /// Primary constructor initializing the settings submenu layout.
    pub const fn new() -> Self {
        Self
    }

    pub fn keybindings_button(&self) -> ScreenBounds {
        ScreenBounds::new(-0.28, 0.08, 0.28, 0.20)
    }

    pub fn log_button(&self) -> ScreenBounds {
        ScreenBounds::new(-0.28, -0.10, 0.28, 0.02)
    }

    pub fn back_button(&self) -> ScreenBounds {
        ScreenBounds::new(-0.28, -0.34, 0.28, -0.22)
    }

    pub fn select(&self, pointer: ScreenPoint) -> SettingsSelection {
        if self.keybindings_button().contains(pointer) {
            SettingsSelection::OpenKeybindings
        } else if self.log_button().contains(pointer) {
            SettingsSelection::OpenBlackBox
        } else if self.back_button().contains(pointer) {
            SettingsSelection::ReturnToPauseMenu
        } else {
            SettingsSelection::Unmatched
        }
    }

    pub const fn view(&self, pointer: ScreenPoint, log_active: bool) -> TitledCard<'static, SettingsBody> {
        TitledCard::new(
            ScreenBounds::new(-0.36, -0.42, 0.36, 0.38),
            "SETTINGS",
            0.29,
            SettingsBody::new(pointer, log_active),
        )
    }
}
