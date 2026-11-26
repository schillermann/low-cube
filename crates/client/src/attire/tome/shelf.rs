use super::super::pouch::BeltSatchel;
use super::super::YouthAttire;
use super::domain::{CraftVenue, FolioTab, StockScope};
use super::entry::{CraftingEntry, RECIPE_CATALOG, RECIPE_COUNT};

/// Focused domain contract for collecting visible `(canonical_index, CraftingEntry)` pairs onto a folio page.
pub trait RecipeShelf {
    fn collect_recipes(&self, out: &mut Vec<(usize, CraftingEntry)>);
}

/// Complete canonical shelf preserving multiplayer `RelicCatalog` index order.
#[derive(Clone, Copy, Debug)]
pub struct CanonicalShelf {
    entries: [CraftingEntry; RECIPE_COUNT],
}

impl CanonicalShelf {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(entries: [CraftingEntry; RECIPE_COUNT]) -> Self {
        Self { entries }
    }

    pub const fn catalog() -> Self {
        Self::new(RECIPE_CATALOG)
    }
}

impl RecipeShelf for CanonicalShelf {
    fn collect_recipes(&self, out: &mut Vec<(usize, CraftingEntry)>) {
        for (idx, entry) in self.entries.iter().copied().enumerate() {
            out.push((idx, entry));
        }
    }
}

/// Decorator over any `RecipeShelf` retaining only recipes belonging to a single `FolioTab`.
#[derive(Clone, Copy, Debug)]
pub struct TabbedShelf<S> {
    origin: S,
    tab: FolioTab,
}

impl<S> TabbedShelf<S> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(origin: S, tab: FolioTab) -> Self {
        Self { origin, tab }
    }
}

impl<S: RecipeShelf> RecipeShelf for TabbedShelf<S> {
    fn collect_recipes(&self, out: &mut Vec<(usize, CraftingEntry)>) {
        let mut raw = Vec::with_capacity(RECIPE_COUNT);
        self.origin.collect_recipes(&mut raw);
        for (idx, entry) in raw {
            if entry.in_tab(self.tab) {
                out.push((idx, entry));
            }
        }
    }
}

/// Decorator over any `RecipeShelf` retaining immediately craftable entries when `StockScope::ReadyStock` is active (`4` fields).
#[derive(Clone, Copy)]
pub struct ReadyStockShelf<'a, S> {
    origin: S,
    filter: (StockScope, CraftVenue),
    satchel: &'a BeltSatchel,
    attire: YouthAttire,
}

impl<'a, S> ReadyStockShelf<'a, S> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn at_venue(
        origin: S,
        filter: (StockScope, CraftVenue),
        satchel: &'a BeltSatchel,
        attire: YouthAttire,
    ) -> Self {
        Self {
            origin,
            filter,
            satchel,
            attire,
        }
    }

    pub const fn new(
        origin: S,
        scope: StockScope,
        satchel: &'a BeltSatchel,
        attire: YouthAttire,
    ) -> Self {
        Self::at_venue(origin, (scope, CraftVenue::FieldHands), satchel, attire)
    }
}

impl<'a, S: RecipeShelf> RecipeShelf for ReadyStockShelf<'a, S> {
    fn collect_recipes(&self, out: &mut Vec<(usize, CraftingEntry)>) {
        let (scope, venue) = self.filter;
        let mut scoped = Vec::with_capacity(12);
        self.origin.collect_recipes(&mut scoped);
        for (idx, entry) in scoped {
            if !scope.ready_only() || entry.craftable_at(self.satchel, self.attire, venue) {
                out.push((idx, entry));
            }
        }
    }
}
