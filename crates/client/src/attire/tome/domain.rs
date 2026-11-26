use glam::Vec3;
use winit::keyboard::KeyCode;
use super::entry::RECIPE_CATALOG;
use super::CraftingEntry;

pub const GRID_COLS: usize = 3;
pub const GRID_SLOTS: usize = 12;

/// Workstation context governing whether hand-only or anvil-forged recipes can be executed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CraftVenue {
    FieldHands,
    SmithyAnvil,
}

impl CraftVenue {
    pub const fn anvil_station(self) -> bool {
        matches!(self, Self::SmithyAnvil)
    }

    pub const fn permits(self, required: Self) -> bool {
        match (self, required) {
            (_, Self::FieldHands) | (Self::SmithyAnvil, Self::SmithyAnvil) => true,
            (Self::FieldHands, Self::SmithyAnvil) => false,
        }
    }
}

/// Four color-coded side index tabs partitioning all canonical recipes into zero-scroll `3x4` pages.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FolioTab {
    Tools,
    Blocks,
    Vehicles,
    Orbital,
}

impl FolioTab {
    pub const fn all() -> [Self; 4] {
        [Self::Tools, Self::Blocks, Self::Vehicles, Self::Orbital]
    }

    pub const fn next(self) -> Self {
        match self {
            Self::Tools => Self::Blocks,
            Self::Blocks => Self::Vehicles,
            Self::Vehicles => Self::Orbital,
            Self::Orbital => Self::Tools,
        }
    }

    pub const fn from_digit(code: KeyCode) -> Option<Self> {
        match code {
            KeyCode::Digit1 => Some(Self::Tools),
            KeyCode::Digit2 => Some(Self::Blocks),
            KeyCode::Digit3 => Some(Self::Vehicles),
            KeyCode::Digit4 => Some(Self::Orbital),
            _ => None,
        }
    }

    pub const fn containing_recipe(canonical_idx: usize) -> Self {
        let entry = RECIPE_CATALOG[canonical_idx % RECIPE_CATALOG.len()];
        if entry.in_tab(Self::Tools) {
            Self::Tools
        } else if entry.in_tab(Self::Blocks) {
            Self::Blocks
        } else if entry.in_tab(Self::Vehicles) {
            Self::Vehicles
        } else {
            Self::Orbital
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::Tools => "TOOLS",
            Self::Blocks => "BLOCKS",
            Self::Vehicles => "ROVER",
            Self::Orbital => "ORBITAL",
        }
    }

    pub const fn badge_label(self) -> &'static str {
        match self {
            Self::Tools => "1 TOOLS",
            Self::Blocks => "2 BLOCK",
            Self::Vehicles => "3 ROVER",
            Self::Orbital => "4 ORBIT",
        }
    }

    pub const fn pigment(self) -> Vec3 {
        match self {
            Self::Tools => Vec3::new(0.20, 0.72, 0.40),
            Self::Blocks => Vec3::new(0.88, 0.66, 0.20),
            Self::Vehicles => Vec3::new(0.84, 0.44, 0.18),
            Self::Orbital => Vec3::new(0.78, 0.28, 0.24),
        }
    }
}

/// Cohesive pairing of a `FolioTab` and its required `CraftVenue` (`2` fields).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FolioSection {
    tab: FolioTab,
    venue: CraftVenue,
}

impl FolioSection {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(tab: FolioTab, venue: CraftVenue) -> Self {
        Self { tab, venue }
    }

    pub const fn field(tab: FolioTab) -> Self {
        Self::new(tab, CraftVenue::FieldHands)
    }

    pub const fn anvil(tab: FolioTab) -> Self {
        Self::new(tab, CraftVenue::SmithyAnvil)
    }

    pub const fn tab(self) -> FolioTab {
        self.tab
    }

    pub const fn venue(self) -> CraftVenue {
        self.venue
    }

    pub const fn requires_anvil(self) -> bool {
        self.venue.anvil_station()
    }

    pub const fn permitted_at(self, active: CraftVenue) -> bool {
        active.permits(self.venue)
    }
}

/// Polymorphic readiness scope toggling `[X] READY TO CRAFT` on the left folio page.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StockScope {
    FullCatalog,
    ReadyStock,
}

impl StockScope {
    pub const fn toggled(self) -> Self {
        match self {
            Self::FullCatalog => Self::ReadyStock,
            Self::ReadyStock => Self::FullCatalog,
        }
    }

    pub const fn ready_only(self) -> bool {
        matches!(self, Self::ReadyStock)
    }
}

/// Polymorphic open/closed state and workstation venue of `ArtisanTome`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TomeState {
    Closed,
    FieldOpen,
    AnvilOpen,
}

impl TomeState {
    pub const fn from_flag(open: bool) -> Self {
        if open { Self::FieldOpen } else { Self::Closed }
    }

    pub const fn at_anvil(open: bool) -> Self {
        if open { Self::AnvilOpen } else { Self::Closed }
    }

    pub const fn open(self) -> bool {
        !matches!(self, Self::Closed)
    }

    pub const fn venue(self) -> CraftVenue {
        match self {
            Self::AnvilOpen => CraftVenue::SmithyAnvil,
            Self::Closed | Self::FieldOpen => CraftVenue::FieldHands,
        }
    }

    pub const fn toggled(self) -> Self {
        match self {
            Self::Closed => Self::FieldOpen,
            Self::FieldOpen | Self::AnvilOpen => Self::Closed,
        }
    }
}

/// Polymorphic selection outcome avoiding `Option` for empty ready-filtered tabs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FolioSelection {
    Highlighted((usize, CraftVenue), CraftingEntry),
    VacantTab(FolioTab),
}

impl FolioSelection {
    pub const fn highlights_recipe(self, canonical_idx: usize) -> bool {
        matches!(self, Self::Highlighted((idx, _), _) if idx == canonical_idx)
    }
}
