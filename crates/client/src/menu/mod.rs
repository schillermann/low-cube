use std::net::SocketAddr;
use sanctuary::JoinTarget;
use winit::keyboard::KeyCode;
use engine::ShaderProgram;
use world::MeshTarget;
use crate::avatar::Avatar;
use crate::controls::{InputTrigger, KeyboardControls};
use crate::surface::PlatformSurface;
use crate::world_vault::{WorldSlug, WorldSpec};

mod archetype;
mod black_box_panel;
mod chat_modal;
mod clipboard;
mod controls_slate;
mod escape_menu;
mod glyph;
mod inspection_modal;
mod lobby_modal;
mod orrery_hud;
mod pause_panel;
mod portal;
mod settings;
mod settings_panel;
mod solar_chart;
mod stage;
mod studio;
#[cfg(test)]
mod tests;
mod version_stamp;
mod view;
mod viewfinder;
mod widget;
mod worlds;

#[cfg(test)]
pub use archetype::AvatarArchetype;
#[cfg(test)]
pub use black_box_panel::{BlackBoxLayout, BlackBoxPanel};
pub use chat_modal::ProximityChatModal;
#[cfg(test)]
pub use clipboard::{MemoryClipboard, ScratchClipboard};
pub use clipboard::{SystemClipboard, TextClipboard};
pub use controls_slate::{ControlsSlate, ControlsSlateHud, SlateContext};
pub use escape_menu::EscapeMenu;
pub use glyph::PixelStamp;
pub use inspection_modal::InspectionModal;
pub use lobby_modal::LobbyGateModal;
pub use orrery_hud::{ChannelStripOverlay, OpticOverlay, ShipOpticHud};
#[cfg(test)]
pub use orrery_hud::{
    BoresightTape, GlassPlateScale, PhosphorTone, RangeScale, ReticleStandoffScale, ScaleDomain,
    SilentOverlay, TestApproachZoneScale as ApproachZoneScale,
    TestChevronReadoutScale as ChevronReadoutScale, TestGraduatedSpine as GraduatedSpine,
    TestOrreryFrame as OrreryFrame, TestRangeTapeOverlay as RangeTapeOverlay,
    TestTargetLockOverlay as TargetLockOverlay, WireframeShipOverlay,
};
#[cfg(test)]
pub use pause_panel::PausePanel;
pub use portal::{AutoPunchModal, PortalInvite};
#[cfg(test)]
pub use portal::{PortalBounds, PortalNotice, PortalStudio, WaitingBounds};
#[cfg(test)]
pub use settings::{ControlLayout, ControlPanel, ControlStudio, SavedControls};
#[cfg(test)]
pub use settings_panel::SettingsPanel;
#[cfg(test)]
pub use solar_chart::{ChartSelection, ChartViewport};
pub use solar_chart::SolarChartModal;
#[cfg(test)]
pub use stage::{StudioAnimation, TurntableStage};
#[cfg(test)]
pub use studio::{SavedWardrobe, WardrobePanel, WardrobeStudio};
#[cfg(test)]
pub use version_stamp::VersionStamp;
#[cfg(test)]
pub use view::{MenuView, VersionedView};
pub use viewfinder::{ActiveChannel, CameraFeed, GuidanceReadout, PreviewStripLayout};
pub use widget::ScreenPoint;
#[cfg(test)]
pub use worlds::{SavedWorldStudio, WorldLayout, WorldStudio};

/// Directive returned when interacting with the Escape menu, Character studio, Settings, Worlds, or Portal panel.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ModalDirective {
    ContinueSession,
    ActivateWorld(WorldSpec),
    EvictWorld(WorldSlug),
    OpenPortal,
    JoinPortal(JoinTarget),
    CancelJoin,
    PunchGuest(SocketAddr),
    ExitGame,
}

/// Interface for session modal overlay interaction and rendering.
pub trait SessionModal {
    fn open(&self) -> bool;
    #[cfg(test)]
    fn dialing_active(&self) -> bool { false }
    fn toggle_gameplay_trigger(
        &mut self,
        _trigger: InputTrigger,
        _surface: &dyn PlatformSurface,
        _avatar: &mut dyn Avatar,
        _world: &dyn world::PlanetaryWorld,
    ) -> bool {
        false
    }
    fn sync_world_chart(&mut self, _avatar: &dyn Avatar, _world: &dyn world::PlanetaryWorld) {}
    fn sync_portal_chat(&mut self, _world: &mut dyn world::PlanetaryWorld) {}
    fn toggle_escape(&mut self, surface: &dyn PlatformSurface) -> ModalDirective;
    fn focus_regained(&mut self) -> ModalDirective { ModalDirective::ContinueSession }
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
    fn announce_auto_punch(&mut self) {}
    fn announce_punch_code(&mut self, _punch_code: String) {}
    fn complete_join(&mut self, _surface: &dyn PlatformSurface) {}
    fn report_join_failure(&mut self) {}
    fn step(&mut self, dt: f32);
    #[cfg(test)]
    fn emit_ui(&self, avatar: &dyn Avatar, target: &mut dyn MeshTarget);
    fn emit_studio(&self, target: &mut dyn MeshTarget);
    fn draw_layer(&self, gl: &glow::Context, shader: &dyn ShaderProgram, avatar: &dyn Avatar, aspect: f32);
}
