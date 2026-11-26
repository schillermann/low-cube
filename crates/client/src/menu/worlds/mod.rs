mod contract;
mod create_view;
mod draft;
mod layout;
mod roster_view;
mod saved;

use winit::keyboard::KeyCode;
use world::MeshTarget;
use crate::menu::clipboard::TextClipboard;
use crate::menu::view::{MenuView, VersionedView};
use crate::menu::widget::ScreenPoint;
use crate::world_vault::{WallEpoch, WorldSlug, WorldSpec};
pub use self::contract::{WorldDirectiveSink, WorldPanel, WorldPanelDirective};
use self::create_view::CreateWorldBody;
use self::draft::WorldDraft;
pub use self::layout::WorldLayout;
use self::roster_view::WorldRosterBody;
pub use self::saved::SavedWorldStudio;

/// Pure in-memory `MY WORLDS` roster and `CREATE WORLD` configurator studio (`4` fields, zero I/O).
pub struct WorldStudio {
    entries: Vec<WorldSpec>,
    active_slug: WorldSlug,
    selection: (usize, usize),
    draft: WorldDraft,
}

impl WorldStudio {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(
        entries: Vec<WorldSpec>,
        active_slug: WorldSlug,
        selection: (usize, usize),
        draft: WorldDraft,
    ) -> Self {
        Self { entries, active_slug, selection, draft }
    }

    pub fn seeded(entries: Vec<WorldSpec>) -> Self {
        let next_ordinal = entries.len() + 1;
        Self::new(
            entries,
            WorldSlug::absent(),
            (0, 0),
            WorldDraft::fresh(next_ordinal, WallEpoch::new().nanos()),
        )
    }

    #[cfg(test)]
    pub fn empty() -> Self {
        Self::seeded(Vec::new())
    }

    fn clamp_selection(&mut self) {
        if self.entries.is_empty() {
            self.selection = (0, 0);
        } else {
            let idx = self.selection.0.min(self.entries.len() - 1);
            let max_page = self.entries.len().div_ceil(WorldLayout::PAGE_SIZE).saturating_sub(1);
            self.selection = (idx, self.selection.1.min(max_page));
        }
    }

    fn commit_draft(&mut self, sink: &mut dyn WorldDirectiveSink) -> WorldPanelDirective {
        let minted = self.draft.mint_spec(self.entries.len() + 1, WallEpoch::new().nanos());
        let saved = sink.create_and_enter(minted);
        self.active_slug = saved.slug().clone();
        self.entries.retain(|w| !w.same_slug(saved.slug()));
        self.entries.insert(0, saved);
        self.selection = (0, 0);
        WorldPanelDirective::CloseIntoWorld
    }
}

impl WorldPanel for WorldStudio {
    fn empty_roster(&self) -> bool {
        self.entries.is_empty()
    }

    fn replace_roster(&mut self, entries: Vec<WorldSpec>) {
        self.entries = entries;
        self.clamp_selection();
    }

    fn prepare_new_draft(&mut self) {
        self.draft = WorldDraft::fresh(self.entries.len() + 1, WallEpoch::new().nanos());
    }

    fn clear_active_world(&mut self) {
        self.active_slug = WorldSlug::absent();
    }

    fn press_roster(&mut self, pointer: ScreenPoint, sink: &mut dyn WorldDirectiveSink) -> WorldPanelDirective {
        let layout = WorldLayout::new();
        let total = self.entries.len();
        let page_count = total.max(1).div_ceil(WorldLayout::PAGE_SIZE);
        let safe_page = self.selection.1.min(page_count - 1);
        let start = safe_page * WorldLayout::PAGE_SIZE;

        for slot in 0..WorldLayout::PAGE_SIZE {
            let idx = start + slot;
            if idx < total && layout.slot_row(slot).contains(pointer) {
                self.selection = (idx, safe_page);
                return WorldPanelDirective::Stay;
            }
        }
        if page_count > 1 && layout.prev_page_button().contains(pointer) {
            let next_p = if safe_page == 0 { page_count - 1 } else { safe_page - 1 };
            self.selection = ((next_p * WorldLayout::PAGE_SIZE).min(total.saturating_sub(1)), next_p);
            return WorldPanelDirective::Stay;
        }
        if page_count > 1 && layout.next_page_button().contains(pointer) {
            let next_p = (safe_page + 1) % page_count;
            self.selection = ((next_p * WorldLayout::PAGE_SIZE).min(total.saturating_sub(1)), next_p);
            return WorldPanelDirective::Stay;
        }
        if layout.play_button().contains(pointer) {
            if let Some(spec) = self.entries.get(self.selection.0).cloned() {
                self.active_slug = spec.slug().clone();
                sink.enter_existing(spec);
                return WorldPanelDirective::CloseIntoWorld;
            }
            return WorldPanelDirective::Stay;
        }
        if layout.new_world_button().contains(pointer) {
            self.prepare_new_draft();
            return WorldPanelDirective::OpenCreation;
        }
        if layout.delete_button().contains(pointer) {
            if let Some(spec) = self.entries.get(self.selection.0).cloned() {
                let slug = spec.slug().clone();
                let was_active = spec.same_slug(&self.active_slug);
                if was_active {
                    self.active_slug = WorldSlug::absent();
                }
                self.entries.retain(|w| !w.same_slug(&slug));
                self.clamp_selection();
                sink.discard_world(slug, was_active);
                if self.entries.is_empty() {
                    self.prepare_new_draft();
                    return WorldPanelDirective::OpenCreation;
                }
            }
            return WorldPanelDirective::Stay;
        }
        if layout.roster_back_button().contains(pointer) {
            return WorldPanelDirective::ReturnToMainMenu;
        }
        WorldPanelDirective::Stay
    }

    fn press_creation(&mut self, pointer: ScreenPoint, sink: &mut dyn WorldDirectiveSink) -> WorldPanelDirective {
        let layout = WorldLayout::new();
        if layout.name_field().contains(pointer) {
            self.draft.focus_name();
            return WorldPanelDirective::Stay;
        }
        if layout.clear_name_button().contains(pointer) {
            self.draft.clear_name();
            return WorldPanelDirective::Stay;
        }
        if layout.size_prev_button().contains(pointer) {
            self.draft.prev_scale();
            return WorldPanelDirective::Stay;
        }
        if layout.size_next_button().contains(pointer) {
            self.draft.next_scale();
            return WorldPanelDirective::Stay;
        }
        if layout.seed_field().contains(pointer) {
            self.draft.focus_seed();
            return WorldPanelDirective::Stay;
        }
        if layout.random_seed_button().contains(pointer) {
            self.draft.reroll_seed(WallEpoch::new().nanos().wrapping_add(17));
            return WorldPanelDirective::Stay;
        }
        if layout.create_button().contains(pointer) {
            return self.commit_draft(sink);
        }
        if layout.create_back_button().contains(pointer) {
            return if self.entries.is_empty() {
                WorldPanelDirective::ReturnToMainMenu
            } else {
                WorldPanelDirective::OpenRoster
            };
        }
        WorldPanelDirective::Stay
    }

    fn handle_creation_key(
        &mut self,
        code: KeyCode,
        ctrl_held: bool,
        clipboard: &dyn TextClipboard,
        sink: &mut dyn WorldDirectiveSink,
    ) -> WorldPanelDirective {
        if ctrl_held && code == KeyCode::KeyV {
            let pasted = clipboard.read_text();
            if !pasted.is_empty() {
                self.draft.append_text(&pasted);
            }
            return WorldPanelDirective::Stay;
        }
        match code {
            KeyCode::Backspace => {
                self.draft.backspace();
                WorldPanelDirective::Stay
            }
            KeyCode::Tab => {
                self.draft.toggle_focus();
                WorldPanelDirective::Stay
            }
            KeyCode::ArrowLeft => {
                self.draft.prev_scale();
                WorldPanelDirective::Stay
            }
            KeyCode::ArrowRight => {
                self.draft.next_scale();
                WorldPanelDirective::Stay
            }
            KeyCode::Enter | KeyCode::NumpadEnter => self.commit_draft(sink),
            _ => WorldPanelDirective::Stay,
        }
    }

    fn append_creation_text(&mut self, text: &str) {
        self.draft.append_text(text);
    }

    fn emit_roster(&self, pointer: ScreenPoint, target: &mut dyn MeshTarget) {
        VersionedView::current(
            WorldRosterBody::new(&self.entries, self.selection, &self.active_slug, pointer).card(),
        )
        .emit(target);
    }

    fn emit_creation(&self, pointer: ScreenPoint, target: &mut dyn MeshTarget) {
        VersionedView::current(CreateWorldBody::new(&self.draft, pointer).card()).emit(target);
    }
}
