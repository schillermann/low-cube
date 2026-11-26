mod bookmark;
mod domain;
mod entry;
mod pages;
mod polaroid;
mod shelf;
mod spread;

use winit::keyboard::KeyCode;
use world::{MeshTarget, PlanetaryWorld};
use crate::controls::{ActionBindings, ControlAction, InputTrigger};
use super::pose::BodyPose;
use super::pouch::BeltSatchel;
use super::YouthAttire;

use self::bookmark::TomeBookmark;
use self::domain::{FolioSection, FolioSelection, FolioTab};
#[cfg(test)]
pub use self::domain::CraftVenue;
pub use self::entry::CraftingEntry;
#[cfg(test)]
use self::entry::{RECIPE_CATALOG, RECIPE_COUNT};
use self::pages::{LeftGridFolio, RightPolaroidFolio};
use self::spread::{FolioSpread, IndexTabbedFolio, RingSpineFolio, SlateFolioCovers};

#[cfg(test)]
pub use self::domain::{
    FolioSelection as TestFolioSelection, FolioTab as TestFolioTab, StockScope as TestStockScope,
};
#[cfg(test)]
pub use self::polaroid::{
    IngredientFormula, PinnedPolaroid, PolaroidCard, ReadinessPipCompartment, VacantScopePlaque,
};
#[cfg(test)]
pub use self::shelf::{CanonicalShelf, ReadyStockShelf, RecipeShelf, TabbedShelf};
#[cfg(test)]
pub use self::spread::{
    FolioSpread as TestFolioSpread, IndexTabbedFolio as TestIndexTabbedFolio,
    RingSpineFolio as TestRingSpineFolio, SlateFolioCovers as TestSlateFolioCovers,
};

/// Immutable fixed-capacity ring of pending crafted recipe indices awaiting authoritative world dispatch (`2` fields).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct CraftJournal {
    notes: [u8; 8],
    len: u8,
}

impl CraftJournal {
    /// Primary constructor doing nothing except assigning arguments to fields.
    const fn new(notes: [u8; 8], len: u8) -> Self {
        Self { notes, len }
    }

    const fn empty() -> Self {
        Self::new([0; 8], 0)
    }

    const fn noted(mut self, recipe: u8) -> Self {
        if (self.len as usize) < self.notes.len() {
            self.notes[self.len as usize] = recipe;
            self.len += 1;
        }
        self
    }

    fn drain_into(mut self, world: &mut dyn PlanetaryWorld) -> Self {
        for &recipe in &self.notes[..usize::from(self.len)] {
            world.note_crafted_recipe(recipe);
        }
        self.len = 0;
        self
    }
}

/// Tactile two-page ringed Crafting Folio with side index tabs, readiness scope, and Polaroid showcase (`2` fields).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ArtisanTome {
    bookmark: TomeBookmark,
    journal: CraftJournal,
}

impl ArtisanTome {
    /// Primary constructor doing nothing except assigning arguments to fields.
    const fn from_parts(bookmark: TomeBookmark, journal: CraftJournal) -> Self {
        Self { bookmark, journal }
    }

    pub const fn new(cursor: usize, open: bool) -> Self {
        Self::from_parts(TomeBookmark::at_recipe(cursor, open), CraftJournal::empty())
    }

    #[cfg(test)]
    pub const fn at_anvil(cursor: usize, open: bool) -> Self {
        Self::from_parts(TomeBookmark::at_anvil_recipe(cursor, open), CraftJournal::empty())
    }

    pub const fn closed() -> Self {
        Self::new(0, false)
    }

    pub const fn open(&self) -> bool {
        self.bookmark.open()
    }

    #[cfg(test)]
    pub const fn venue(&self) -> CraftVenue {
        self.bookmark.venue()
    }

    #[cfg(test)]
    pub const fn entries() -> [CraftingEntry; RECIPE_COUNT] {
        RECIPE_CATALOG
    }

    #[cfg(test)]
    pub const fn selected_entry(&self) -> CraftingEntry {
        self.bookmark.raw_entry()
    }

    pub const fn toggle(self) -> Self {
        Self::from_parts(self.bookmark.toggled_open(), self.journal)
    }

    pub fn open_at_anvil(self, satchel: &BeltSatchel, attire: YouthAttire) -> Self {
        let opened = self.bookmark.opened_at_anvil().selected_tab(FolioTab::Tools, satchel, attire);
        Self::from_parts(opened, self.journal)
    }

    pub const fn noted_recipe(self, recipe_idx: u8) -> Self {
        Self::from_parts(self.bookmark, self.journal.noted(recipe_idx))
    }

    pub fn drain_crafted_notes(self, world: &mut dyn PlanetaryWorld) -> Self {
        Self::from_parts(self.bookmark, self.journal.drain_into(world))
    }

    pub fn craft_selected(
        self,
        side: f32,
        satchel: BeltSatchel,
        attire: YouthAttire,
    ) -> (Self, BeltSatchel, YouthAttire, bool) {
        let state = satchel.reconcile_attire(attire);
        let FolioSelection::Highlighted((recipe_idx, venue), entry) =
            self.bookmark.resolve_selection(&state, attire)
        else {
            return (self, state, attire, false);
        };
        if !entry.craftable_at(&state, attire, venue) {
            return (self, state, attire, false);
        }
        let (next_satchel, next_attire) = state.consume_and_equip(
            entry.ingredients(),
            entry.product(),
            entry.batch_yield(),
            side,
            attire,
        );
        let closed = Self::from_parts(
            self.bookmark.with_open(false),
            self.journal.noted(recipe_idx as u8),
        );
        (closed, next_satchel, next_attire, true)
    }

    pub fn dispatch_trigger(
        self,
        trigger: InputTrigger,
        bindings: &ActionBindings,
        satchel: BeltSatchel,
        attire: YouthAttire,
    ) -> (Self, BeltSatchel, YouthAttire, bool) {
        let state = satchel.reconcile_attire(attire);
        if let InputTrigger::Key(code) = trigger {
            if let Some(tab) = FolioTab::from_digit(code) {
                let next_bm = self.bookmark.selected_tab(tab, &state, attire);
                return (Self::from_parts(next_bm, self.journal), state, attire, true);
            }
            if code == KeyCode::KeyF {
                let next_bm = self.bookmark.toggled_scope(&state, attire);
                return (Self::from_parts(next_bm, self.journal), state, attire, true);
            }
        }
        if bindings.matches(ControlAction::Satchel, trigger) {
            let next_bm = self.bookmark.cycled_tab(&state, attire);
            return (Self::from_parts(next_bm, self.journal), state, attire, true);
        }
        if bindings.matches(ControlAction::MoldLeft, trigger) {
            let next_bm = self.bookmark.stepped_linear(-1, &state, attire);
            return (Self::from_parts(next_bm, self.journal), state, attire, true);
        }
        if bindings.matches(ControlAction::MoldRight, trigger) {
            let next_bm = self.bookmark.stepped_linear(1, &state, attire);
            return (Self::from_parts(next_bm, self.journal), state, attire, true);
        }
        let dx = if bindings.matches(ControlAction::MoveLeft, trigger) {
            -1
        } else if bindings.matches(ControlAction::MoveRight, trigger) {
            1
        } else {
            0
        };
        let dy = if bindings.matches(ControlAction::MoveForward, trigger) {
            -1
        } else if bindings.matches(ControlAction::MoveBackward, trigger) {
            1
        } else {
            0
        };
        if dx != 0 || dy != 0 {
            let next_bm = self.bookmark.stepped_grid(dx, dy, &state, attire);
            return (Self::from_parts(next_bm, self.journal), state, attire, true);
        }
        if bindings.matches(ControlAction::SatchelRight, trigger)
            || bindings.matches(ControlAction::SatchelLeft, trigger)
            || bindings.matches(ControlAction::Jump, trigger)
            || bindings.matches(ControlAction::MoldForm, trigger)
            || trigger == InputTrigger::Key(KeyCode::Enter)
        {
            let side = if bindings.matches(ControlAction::SatchelLeft, trigger) { -1.0 } else { 1.0 };
            let (next_tome, next_satchel, next_attire, _) = self.craft_selected(side, state, attire);
            return (next_tome, next_satchel, next_attire, true);
        }
        (self, state, attire, false)
    }

    pub fn emit_book(
        &self,
        satchel: &BeltSatchel,
        attire: YouthAttire,
        pose: &BodyPose,
        target: &mut dyn MeshTarget,
    ) {
        if !self.open() {
            return;
        }
        let frame = pose.view_frame().shifted(-0.022, -0.004, 0.45);
        let tab = self.bookmark.focused_tab();
        let selection = self.bookmark.resolve_selection(satchel, attire);
        let spread = RightPolaroidFolio::new(
            LeftGridFolio::new(
                IndexTabbedFolio::with_section(
                    RingSpineFolio::new(SlateFolioCovers::new(tab.pigment())),
                    FolioSection::new(tab, self.bookmark.venue()),
                    satchel,
                    attire,
                ),
                self.bookmark,
                satchel,
                attire,
            ),
            selection,
            satchel,
            attire,
        );
        spread.emit_spread(&frame, target);
    }
}
