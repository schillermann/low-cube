use super::super::pouch::BeltSatchel;
use super::super::YouthAttire;
use super::domain::{FolioTab, StockScope};
use super::entry::{CraftingEntry, RECIPE_CATALOG};

/// Focused domain contract for collecting visible `(canonical_index, CraftingEntry)` pairs onto a folio page.
pub trait RecipeShelf {
    fn collect_recipes(&self, out: &mut Vec<(usize, CraftingEntry)>);
}

/// Complete 25-recipe canonical shelf preserving multiplayer `RelicCatalog` index order.
#[derive(Clone, Copy, Debug)]
pub struct CanonicalShelf {
    entries: [CraftingEntry; 25],
}

impl CanonicalShelf {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(entries: [CraftingEntry; 25]) -> Self {
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
        let mut raw = Vec::with_capacity(25);
        self.origin.collect_recipes(&mut raw);
        for (idx, entry) in raw {
            if entry.in_tab(self.tab) {
                out.push((idx, entry));
            }
        }
    }
}

/// Decorator over any `RecipeShelf` retaining immediately craftable entries when `StockScope::ReadyStock` is active.
#[derive(Clone, Copy)]
pub struct ReadyStockShelf<'a, S> {
    origin: S,
    scope: StockScope,
    satchel: &'a BeltSatchel,
    attire: YouthAttire,
}

impl<'a, S> ReadyStockShelf<'a, S> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(
        origin: S,
        scope: StockScope,
        satchel: &'a BeltSatchel,
        attire: YouthAttire,
    ) -> Self {
        Self {
            origin,
            scope,
            satchel,
            attire,
        }
    }
}

impl<'a, S: RecipeShelf> RecipeShelf for ReadyStockShelf<'a, S> {
    fn collect_recipes(&self, out: &mut Vec<(usize, CraftingEntry)>) {
        let mut scoped = Vec::with_capacity(12);
        self.origin.collect_recipes(&mut scoped);
        for (idx, entry) in scoped {
            if !self.scope.ready_only() || entry.craftable(self.satchel, self.attire) {
                out.push((idx, entry));
            }
        }
    }
}
