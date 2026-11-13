use glam::Vec3;
use world::MeshTarget;
use super::glyph::PixelStamp;
use super::portal::PortalStudio;
use super::widget::{ScreenBounds, ScreenPoint};

/// Selection resolved from a mouse click on the Main Pause Menu.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PauseSelection {
    Unmatched,
    ResumeGame,
    OpenWardrobe,
    OpenSettings,
    OpenMultiplayer,
    QuitGame,
}

/// Encapsulated Main Pause Menu layout, hit-testing, and 2D quad emission.
pub struct PausePanel;

impl PausePanel {
    /// Primary constructor initializing the pause menu layout.
    pub const fn new() -> Self {
        Self
    }

    fn resume_button(&self) -> ScreenBounds {
        ScreenBounds::new(-0.28, 0.14, 0.28, 0.26)
    }

    fn character_button(&self) -> ScreenBounds {
        ScreenBounds::new(-0.28, -0.04, 0.28, 0.08)
    }

    fn settings_button(&self) -> ScreenBounds {
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
            PauseSelection::ResumeGame
        } else if self.character_button().contains(pointer) {
            PauseSelection::OpenWardrobe
        } else if self.settings_button().contains(pointer) {
            PauseSelection::OpenSettings
        } else if self.multiplayer_button().contains(pointer) {
            PauseSelection::OpenMultiplayer
        } else if self.quit_button().contains(pointer) {
            PauseSelection::QuitGame
        } else {
            PauseSelection::Unmatched
        }
    }

    pub fn emit(&self, pointer: ScreenPoint, portal: &PortalStudio, target: &mut dyn MeshTarget) {
        let bg = Vec3::new(0.08, 0.10, 0.15);
        let rim = Vec3::new(0.32, 0.39, 0.50);
        let gold = Vec3::new(0.96, 0.82, 0.34);
        let hosted = portal.hosted_active();
        let mp_col = if hosted { Vec3::new(0.22, 0.52, 0.32) } else { Vec3::new(0.18, 0.44, 0.46) };
        ScreenBounds::new(-0.36, -0.66, 0.36, 0.44).emit_framed(bg, rim, 0.008, target);
        PixelStamp::title(gold).emit_centered("MAIN MENU", 0.0, 0.35, target);
        self.resume_button().emit_button("RESUME GAME", pointer, Vec3::new(0.18, 0.42, 0.24), false, target);
        self.character_button().emit_button("CHARACTER", pointer, Vec3::new(0.20, 0.32, 0.52), false, target);
        self.settings_button().emit_button("KEYBINDINGS", pointer, Vec3::new(0.28, 0.26, 0.48), false, target);
        self.multiplayer_button().emit_button("MULTIPLAYER", pointer, mp_col, hosted, target);
        self.quit_button().emit_button("QUIT GAME", pointer, Vec3::new(0.54, 0.18, 0.18), false, target);
    }
}
