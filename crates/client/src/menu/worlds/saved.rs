use winit::keyboard::KeyCode;
use world::MeshTarget;
use crate::menu::clipboard::TextClipboard;
use crate::menu::widget::ScreenPoint;
use crate::world_vault::{DiskWorldVault, WorldArchive, WorldSlug, WorldSpec};
use super::{WorldDirectiveSink, WorldPanel, WorldPanelDirective, WorldStudio};

/// Decorator over `WorldDirectiveSink` persisting world creation and inactive deletion to a `WorldArchive` (`2` fields).
struct PersistedWorldSink<'a> {
    inner: &'a mut dyn WorldDirectiveSink,
    archive: &'a mut dyn WorldArchive,
}

impl<'a> PersistedWorldSink<'a> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    fn new(inner: &'a mut dyn WorldDirectiveSink, archive: &'a mut dyn WorldArchive) -> Self {
        Self { inner, archive }
    }
}

impl WorldDirectiveSink for PersistedWorldSink<'_> {
    fn create_and_enter(&mut self, draft: WorldSpec) -> WorldSpec {
        let saved = self.archive.persist_world(&draft);
        self.inner.create_and_enter(saved)
    }

    fn enter_existing(&mut self, spec: WorldSpec) {
        self.inner.enter_existing(spec);
    }

    fn discard_world(&mut self, slug: WorldSlug, active_evicted: bool) {
        if !active_evicted {
            let _ = self.archive.erase_world(&slug);
        }
        self.inner.discard_world(slug, active_evicted);
    }
}

/// Decorator over `WorldPanel` coupling pure in-memory `WorldStudio` UI to a `WorldArchive` (`2` fields).
pub struct SavedWorldStudio {
    inner: Box<dyn WorldPanel>,
    archive: Box<dyn WorldArchive>,
}

impl SavedWorldStudio {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(inner: Box<dyn WorldPanel>, archive: Box<dyn WorldArchive>) -> Self {
        Self { inner, archive }
    }

    /// Secondary constructor loading existing `worlds/` entries into a pure `WorldStudio`.
    pub fn live(vault: DiskWorldVault) -> Self {
        let initial = vault.load_roster();
        Self::new(Box::new(WorldStudio::seeded(initial)), Box::new(vault))
    }
}

impl WorldPanel for SavedWorldStudio {
    fn empty_roster(&self) -> bool {
        self.inner.empty_roster()
    }

    fn sync_roster(&mut self) {
        self.inner.replace_roster(self.archive.load_roster());
    }

    fn replace_roster(&mut self, entries: Vec<WorldSpec>) {
        self.inner.replace_roster(entries);
    }

    fn prepare_new_draft(&mut self) {
        self.sync_roster();
        self.inner.prepare_new_draft();
    }

    fn clear_active_world(&mut self) {
        self.inner.clear_active_world();
    }

    fn press_roster(&mut self, pointer: ScreenPoint, sink: &mut dyn WorldDirectiveSink) -> WorldPanelDirective {
        let mut persisted = PersistedWorldSink::new(sink, self.archive.as_mut());
        self.inner.press_roster(pointer, &mut persisted)
    }

    fn press_creation(&mut self, pointer: ScreenPoint, sink: &mut dyn WorldDirectiveSink) -> WorldPanelDirective {
        let mut persisted = PersistedWorldSink::new(sink, self.archive.as_mut());
        self.inner.press_creation(pointer, &mut persisted)
    }

    fn handle_creation_key(
        &mut self,
        code: KeyCode,
        ctrl_held: bool,
        clipboard: &dyn TextClipboard,
        sink: &mut dyn WorldDirectiveSink,
    ) -> WorldPanelDirective {
        let mut persisted = PersistedWorldSink::new(sink, self.archive.as_mut());
        self.inner.handle_creation_key(code, ctrl_held, clipboard, &mut persisted)
    }

    fn append_creation_text(&mut self, text: &str) {
        self.inner.append_creation_text(text);
    }

    fn emit_roster(&self, pointer: ScreenPoint, target: &mut dyn MeshTarget) {
        self.inner.emit_roster(pointer, target);
    }

    fn emit_creation(&self, pointer: ScreenPoint, target: &mut dyn MeshTarget) {
        self.inner.emit_creation(pointer, target);
    }
}
