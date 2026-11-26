use super::super::pouch::BeltSatchel;
use super::super::YouthAttire;
use super::domain::{CraftVenue, FolioSelection, FolioTab, StockScope, TomeState, GRID_COLS, GRID_SLOTS};
use super::entry::{CraftingEntry, RECIPE_CATALOG};
use super::shelf::{CanonicalShelf, ReadyStockShelf, RecipeShelf, TabbedShelf};

/// Immutable bookmark tracking canonical recipe cursor, active side tab, stock scope, and open state (`4` fields).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TomeBookmark {
    cursor: usize,
    tab: FolioTab,
    scope: StockScope,
    state: TomeState,
}

impl TomeBookmark {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(cursor: usize, tab: FolioTab, scope: StockScope, state: TomeState) -> Self {
        Self {
            cursor,
            tab,
            scope,
            state,
        }
    }

    pub const fn at_recipe(cursor: usize, open: bool) -> Self {
        let bounded = cursor % RECIPE_CATALOG.len();
        Self::new(
            bounded,
            FolioTab::containing_recipe(bounded),
            StockScope::FullCatalog,
            TomeState::from_flag(open),
        )
    }

    pub const fn at_anvil_recipe(cursor: usize, open: bool) -> Self {
        let bounded = cursor % RECIPE_CATALOG.len();
        Self::new(
            bounded,
            FolioTab::containing_recipe(bounded),
            StockScope::FullCatalog,
            TomeState::at_anvil(open),
        )
    }

    pub const fn open(&self) -> bool {
        self.state.open()
    }

    pub const fn venue(&self) -> CraftVenue {
        self.state.venue()
    }

    pub const fn focused_tab(&self) -> FolioTab {
        self.tab
    }

    pub const fn ready_only(&self) -> bool {
        self.scope.ready_only()
    }

    #[cfg(test)]
    pub const fn raw_entry(&self) -> CraftingEntry {
        RECIPE_CATALOG[self.cursor % RECIPE_CATALOG.len()]
    }

    pub const fn with_open(self, open: bool) -> Self {
        Self::new(self.cursor, self.tab, self.scope, TomeState::from_flag(open))
    }

    pub const fn opened_at_anvil(self) -> Self {
        Self::new(self.cursor, self.tab, self.scope, TomeState::AnvilOpen)
    }

    pub const fn toggled_open(self) -> Self {
        Self::new(self.cursor, self.tab, self.scope, self.state.toggled())
    }

    pub fn visible_recipes(&self, satchel: &BeltSatchel, attire: YouthAttire) -> Vec<(usize, CraftingEntry)> {
        let shelf = ReadyStockShelf::at_venue(
            TabbedShelf::new(CanonicalShelf::catalog(), self.tab),
            (self.scope, self.state.venue()),
            satchel,
            attire,
        );
        let mut list = Vec::with_capacity(GRID_SLOTS);
        shelf.collect_recipes(&mut list);
        list
    }

    pub fn resolve_selection(&self, satchel: &BeltSatchel, attire: YouthAttire) -> FolioSelection {
        let visible = self.visible_recipes(satchel, attire);
        if visible.is_empty() {
            return FolioSelection::VacantTab(self.tab);
        }
        let slot = visible
            .iter()
            .position(|&(idx, _)| idx == self.cursor)
            .unwrap_or(0);
        let (canonical, entry) = visible[slot];
        FolioSelection::Highlighted((canonical, self.state.venue()), entry)
    }

    pub fn selected_tab(self, tab: FolioTab, satchel: &BeltSatchel, attire: YouthAttire) -> Self {
        let probe = Self::new(self.cursor, tab, self.scope, self.state);
        let visible = probe.visible_recipes(satchel, attire);
        let next_cursor = visible.first().map_or(self.cursor, |&(idx, _)| idx);
        Self::new(next_cursor, tab, self.scope, self.state)
    }

    pub fn cycled_tab(self, satchel: &BeltSatchel, attire: YouthAttire) -> Self {
        self.selected_tab(self.tab.next(), satchel, attire)
    }

    pub fn toggled_scope(self, satchel: &BeltSatchel, attire: YouthAttire) -> Self {
        let next_scope = self.scope.toggled();
        let local = Self::new(self.cursor, self.tab, next_scope, self.state);
        let visible = local.visible_recipes(satchel, attire);
        if let Some(&(first_idx, _)) = visible.first() {
            let keep = visible.iter().any(|&(idx, _)| idx == self.cursor);
            let chosen = if keep { self.cursor } else { first_idx };
            return Self::new(chosen, self.tab, next_scope, self.state);
        }
        if next_scope.ready_only() {
            let mut candidate = self.tab.next();
            for _ in 0..3 {
                let check = Self::new(self.cursor, candidate, next_scope, self.state);
                if let Some(&(ready_idx, _)) = check.visible_recipes(satchel, attire).first() {
                    return Self::new(ready_idx, candidate, next_scope, self.state);
                }
                candidate = candidate.next();
            }
        }
        local
    }

    pub fn stepped_linear(self, delta: isize, satchel: &BeltSatchel, attire: YouthAttire) -> Self {
        let visible = self.visible_recipes(satchel, attire);
        if visible.is_empty() {
            return self;
        }
        let pos = visible.iter().position(|&(idx, _)| idx == self.cursor).unwrap_or(0);
        let next_pos = (pos as isize + delta).rem_euclid(visible.len() as isize) as usize;
        Self::new(visible[next_pos].0, self.tab, self.scope, self.state)
    }

    pub fn stepped_grid(self, dx: i32, dy: i32, satchel: &BeltSatchel, attire: YouthAttire) -> Self {
        let visible = self.visible_recipes(satchel, attire);
        if visible.is_empty() {
            return self;
        }
        if dx != 0 {
            return self.stepped_linear(dx as isize, satchel, attire);
        }
        let len = visible.len();
        let pos = visible.iter().position(|&(idx, _)| idx == self.cursor).unwrap_or(0);
        let (col, row, rows) = (pos % GRID_COLS, pos / GRID_COLS, len.div_ceil(GRID_COLS));
        let next_row = if dy > 0 {
            (row + 1) % rows
        } else if dy < 0 {
            (row + rows - 1) % rows
        } else {
            row
        };
        let next_pos = (next_row * GRID_COLS + col).min(len - 1);
        Self::new(visible[next_pos].0, self.tab, self.scope, self.state)
    }
}
