use winit::keyboard::KeyCode;
use world::MeshTarget;
use crate::menu::clipboard::TextClipboard;
use crate::menu::widget::ScreenPoint;
use crate::world_vault::{WorldSlug, WorldSpec};

/// Narrow Tell-Don't-Ask recipient for enacted world creation, selection, and deletion events.
pub trait WorldDirectiveSink {
    fn create_and_enter(&mut self, draft: WorldSpec) -> WorldSpec {
        draft
    }
    fn enter_existing(&mut self, spec: WorldSpec);
    fn discard_world(&mut self, slug: WorldSlug, active_evicted: bool);
}

/// Focused interface for world roster & creation studios and persistence decorators (mirroring `WardrobePanel`).
pub trait WorldPanel {
    fn empty_roster(&self) -> bool;
    fn sync_roster(&mut self) {}
    fn replace_roster(&mut self, entries: Vec<WorldSpec>);
    fn prepare_new_draft(&mut self);
    fn clear_active_world(&mut self);
    fn press_roster(&mut self, pointer: ScreenPoint, sink: &mut dyn WorldDirectiveSink) -> WorldPanelDirective;
    fn press_creation(&mut self, pointer: ScreenPoint, sink: &mut dyn WorldDirectiveSink) -> WorldPanelDirective;
    fn handle_creation_key(
        &mut self,
        code: KeyCode,
        ctrl_held: bool,
        clipboard: &dyn TextClipboard,
        sink: &mut dyn WorldDirectiveSink,
    ) -> WorldPanelDirective;
    fn append_creation_text(&mut self, text: &str);
    fn emit_roster(&self, pointer: ScreenPoint, target: &mut dyn MeshTarget);
    fn emit_creation(&self, pointer: ScreenPoint, target: &mut dyn MeshTarget);
}

/// Phase transition returned by `WorldPanel` interactions.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WorldPanelDirective {
    Stay,
    OpenRoster,
    OpenCreation,
    ReturnToMainMenu,
    CloseIntoWorld,
}
