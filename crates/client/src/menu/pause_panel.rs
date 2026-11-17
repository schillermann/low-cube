use glam::Vec3;
use world::MeshTarget;
use super::view::{MenuView, TitledCard};
use super::widget::{ScreenBounds, ScreenPoint};

/// Selection resolved from a mouse click on the Main Pause Menu.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PauseSelection {
    Unmatched,
    Resume,
    OpenWardrobe,
    OpenSettings,
    OpenMultiplayer,
    Quit,
}

/// Button content of the Main Pause Menu decorated by `TitledCard`.
pub struct PauseBody {
    pointer: ScreenPoint,
    hosted: bool,
}

impl PauseBody {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(pointer: ScreenPoint, hosted: bool) -> Self {
        Self { pointer, hosted }
    }
}

impl MenuView for PauseBody {
    fn emit(&self, target: &mut dyn MeshTarget) {
        let layout = PausePanel::new();
        let mp_col = if self.hosted {
            Vec3::new(0.22, 0.52, 0.32)
        } else {
            Vec3::new(0.18, 0.44, 0.46)
        };
        layout.resume_button().emit_button("RESUME", self.pointer, Vec3::new(0.18, 0.42, 0.24), false, target);
        layout.character_button().emit_button("CHARACTER", self.pointer, Vec3::new(0.20, 0.32, 0.52), false, target);
        layout.settings_button().emit_button("SETTINGS", self.pointer, Vec3::new(0.28, 0.26, 0.48), false, target);
        layout.multiplayer_button().emit_button("MULTIPLAYER", self.pointer, mp_col, self.hosted, target);
        layout.quit_button().emit_button("QUIT", self.pointer, Vec3::new(0.54, 0.18, 0.18), false, target);
    }
}

/// Encapsulated Main Pause Menu layout, hit-testing, and decorated `MenuView` assembly.
pub struct PausePanel;

impl PausePanel {
    /// Primary constructor initializing the pause menu layout.
    pub const fn new() -> Self {
        Self
    }

    pub fn resume_button(&self) -> ScreenBounds {
        ScreenBounds::new(-0.28, 0.14, 0.28, 0.26)
    }

    pub fn character_button(&self) -> ScreenBounds {
        ScreenBounds::new(-0.28, -0.04, 0.28, 0.08)
    }

    pub fn settings_button(&self) -> ScreenBounds {
        ScreenBounds::new(-0.28, -0.22, 0.28, -0.10)
    }

    pub fn multiplayer_button(&self) -> ScreenBounds {
        ScreenBounds::new(-0.28, -0.40, 0.28, -0.28)
    }

    pub fn quit_button(&self) -> ScreenBounds {
        ScreenBounds::new(-0.28, -0.58, 0.28, -0.46)
    }

    pub fn select(&self, pointer: ScreenPoint) -> PauseSelection {
        if self.resume_button().contains(pointer) {
            PauseSelection::Resume
        } else if self.character_button().contains(pointer) {
            PauseSelection::OpenWardrobe
        } else if self.settings_button().contains(pointer) {
            PauseSelection::OpenSettings
        } else if self.multiplayer_button().contains(pointer) {
            PauseSelection::OpenMultiplayer
        } else if self.quit_button().contains(pointer) {
            PauseSelection::Quit
        } else {
            PauseSelection::Unmatched
        }
    }

    pub const fn view(&self, pointer: ScreenPoint, hosted: bool) -> TitledCard<'static, PauseBody> {
        TitledCard::new(
            ScreenBounds::new(-0.36, -0.66, 0.36, 0.44),
            "MAIN MENU",
            0.35,
            PauseBody::new(pointer, hosted),
        )
    }
}
