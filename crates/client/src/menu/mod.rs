use std::net::SocketAddr;
use sanctuary::JoinTarget;
use winit::keyboard::KeyCode;
use engine::ShaderProgram;
use world::MeshTarget;
use crate::avatar::Avatar;
use crate::controls::{InputTrigger, KeyboardControls};
use crate::surface::PlatformSurface;

mod archetype;
mod clipboard;
mod escape_menu;
mod glyph;
mod pause_panel;
mod portal;
mod settings;
mod stage;
mod studio;
#[cfg(test)]
mod tests;
mod widget;

#[cfg(test)]
pub use archetype::AvatarArchetype;
#[cfg(test)]
pub use clipboard::{MemoryClipboard, ScratchClipboard};
pub use clipboard::{SystemClipboard, TextClipboard};
pub use escape_menu::EscapeMenu;
pub use glyph::PixelStamp;
#[cfg(test)]
pub use pause_panel::PausePanel;
pub use portal::PortalInvite;
#[cfg(test)]
pub use portal::{PortalNotice, PortalStudio};
#[cfg(test)]
pub use settings::ControlStudio;
#[cfg(test)]
pub use stage::{StudioAnimation, TurntableStage};
#[cfg(test)]
pub use studio::WardrobeStudio;
pub use widget::ScreenPoint;

/// Directive returned when interacting with the Escape menu, Character studio, Settings, or Portal panel.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ModalDirective {
    ContinueSession,
    OpenPortal,
    JoinPortal(JoinTarget),
    PunchGuest(SocketAddr),
    ExitGame,
}

/// Interface for session modal overlay interaction and rendering.
pub trait SessionModal {
    fn open(&self) -> bool;
    fn dialing_active(&self) -> bool { false }
    fn toggle_escape(&mut self, surface: &dyn PlatformSurface);
    fn move_cursor(&mut self, point: ScreenPoint);
    fn press_mouse(&mut self, surface: &dyn PlatformSurface, avatar: &mut dyn Avatar) -> ModalDirective;
    fn release_mouse(&mut self);
    fn handle_key(&mut self, code: KeyCode, ctrl_held: bool, avatar: &mut dyn Avatar) -> ModalDirective;
    fn handle_text(&mut self, _text: &str) {}
    fn handle_trigger(&mut self, _trigger: InputTrigger, _avatar: &mut dyn Avatar) -> bool {
        false
    }
    fn sync_controls(&self, _controls: &mut dyn KeyboardControls) {}
    fn announce_portal(&mut self, _invite: PortalInvite) {}
    fn announce_punch_code(&mut self, _punch_code: String) {}
    fn complete_join(&mut self, _surface: &dyn PlatformSurface) {}
    fn report_join_failure(&mut self) {}
    fn step(&mut self, dt: f32);
    fn emit_ui(&self, avatar: &dyn Avatar, target: &mut dyn MeshTarget);
    fn emit_studio(&self, target: &mut dyn MeshTarget);
    fn draw_layer(&self, gl: &glow::Context, shader: &dyn ShaderProgram, avatar: &dyn Avatar, aspect: f32);
}
