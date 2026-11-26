use super::super::pouch::BeltSatchel;
use super::super::relic::HeldRelic;
use super::super::YouthAttire;
use super::domain::FolioTab;

/// Single craftable recipe entry inside the tactile Field Folio (`4` private fields).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CraftingEntry {
    product: HeldRelic,
    batch_yield: u32,
    tab: FolioTab,
    ingredients: [(HeldRelic, u32); 2],
}

impl CraftingEntry {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(
        product: HeldRelic,
        batch_yield: u32,
        tab: FolioTab,
        ingredients: [(HeldRelic, u32); 2],
    ) -> Self {
        Self {
            product,
            batch_yield,
            tab,
            ingredients,
        }
    }

    pub const fn product(&self) -> HeldRelic {
        self.product
    }

    pub const fn batch_yield(&self) -> u32 {
        self.batch_yield
    }

    pub const fn ingredients(&self) -> [(HeldRelic, u32); 2] {
        self.ingredients
    }

    pub const fn in_tab(&self, candidate: FolioTab) -> bool {
        matches!(
            (self.tab, candidate),
            (FolioTab::Tools, FolioTab::Tools)
                | (FolioTab::Blocks, FolioTab::Blocks)
                | (FolioTab::Vehicles, FolioTab::Vehicles)
                | (FolioTab::Orbital, FolioTab::Orbital)
        )
    }

    pub fn craftable(&self, satchel: &BeltSatchel, attire: YouthAttire) -> bool {
        self.ingredients
            .iter()
            .all(|&(relic, needed)| needed == 0 || satchel.resource_tally(relic, attire) >= needed)
    }
}

pub const RECIPE_CATALOG: [CraftingEntry; 30] = {
    use FolioTab::{Blocks, Orbital, Tools, Vehicles};
    [
        CraftingEntry::new(HeldRelic::WoodmanAxe, 1, Tools, [(HeldRelic::PlankBlock, 1), (HeldRelic::IronOre, 2)]),
        CraftingEntry::new(HeldRelic::PlankBlock, 4, Blocks, [(HeldRelic::TimberBlock, 1), (HeldRelic::EmptyPalm, 0)]),
        CraftingEntry::new(HeldRelic::CantileverCrane, 1, Blocks, [(HeldRelic::PlankBlock, 2), (HeldRelic::QuarriedStone, 1)]),
        CraftingEntry::new(HeldRelic::VehicleBlueprint, 1, Blocks, [(HeldRelic::PlankBlock, 1), (HeldRelic::FlintStone, 1)]),
        CraftingEntry::new(HeldRelic::WoodenMallet, 1, Tools, [(HeldRelic::DeadwoodBough, 2), (HeldRelic::EmptyPalm, 0)]),
        CraftingEntry::new(HeldRelic::IronSpade, 1, Tools, [(HeldRelic::PlankBlock, 1), (HeldRelic::IronOre, 2)]),
        CraftingEntry::new(HeldRelic::KnightlySword, 1, Tools, [(HeldRelic::PlankBlock, 1), (HeldRelic::IronOre, 3)]),
        CraftingEntry::new(HeldRelic::TimberTorch, 1, Tools, [(HeldRelic::DeadwoodBough, 1), (HeldRelic::FlintStone, 1)]),
        CraftingEntry::new(HeldRelic::QuarriedStone, 1, Blocks, [(HeldRelic::FlintStone, 2), (HeldRelic::EmptyPalm, 0)]),
        CraftingEntry::new(HeldRelic::CabinSeat, 1, Vehicles, [(HeldRelic::PlankBlock, 1), (HeldRelic::EmptyPalm, 0)]),
        CraftingEntry::new(HeldRelic::PedalCluster, 1, Vehicles, [(HeldRelic::PlankBlock, 1), (HeldRelic::FlintStone, 1)]),
        CraftingEntry::new(HeldRelic::CarWheel, 4, Vehicles, [(HeldRelic::PlankBlock, 1), (HeldRelic::FlintStone, 1)]),
        CraftingEntry::new(HeldRelic::SteeringWheel, 1, Vehicles, [(HeldRelic::CopperOre, 1), (HeldRelic::PlankBlock, 1)]),
        CraftingEntry::new(HeldRelic::FieldCamera, 1, Orbital, [(HeldRelic::CopperOre, 2), (HeldRelic::GoldOre, 1)]),
        CraftingEntry::new(HeldRelic::AircraftEngine, 1, Vehicles, [(HeldRelic::IronOre, 2), (HeldRelic::CopperOre, 2)]),
        CraftingEntry::new(HeldRelic::FlightConsole, 1, Orbital, [(HeldRelic::GoldOre, 2), (HeldRelic::CopperOre, 2)]),
        CraftingEntry::new(HeldRelic::RocketNozzle, 2, Orbital, [(HeldRelic::IronOre, 3), (HeldRelic::GoldOre, 1)]),
        CraftingEntry::new(HeldRelic::FusionTorch, 1, Orbital, [(HeldRelic::GoldOre, 2), (HeldRelic::IronOre, 3)]),
        CraftingEntry::new(HeldRelic::RocketCamera, 2, Orbital, [(HeldRelic::CopperOre, 2), (HeldRelic::GoldOre, 1)]),
        CraftingEntry::new(HeldRelic::FlintPickaxe, 1, Tools, [(HeldRelic::DeadwoodBough, 1), (HeldRelic::FlintStone, 3)]),
        CraftingEntry::new(HeldRelic::IronPickaxe, 1, Tools, [(HeldRelic::PlankBlock, 1), (HeldRelic::IronOre, 2)]),
        CraftingEntry::new(HeldRelic::TitaniumHull, 4, Blocks, [(HeldRelic::IronOre, 2), (HeldRelic::CrustBlock, 1)]),
        CraftingEntry::new(HeldRelic::CarbonSilicate, 4, Blocks, [(HeldRelic::GoldOre, 1), (HeldRelic::FlintStone, 2)]),
        CraftingEntry::new(HeldRelic::BlackBoxPod, 1, Orbital, [(HeldRelic::IronOre, 2), (HeldRelic::CopperOre, 1)]),
        CraftingEntry::new(HeldRelic::LaserBeacon, 1, Orbital, [(HeldRelic::GoldOre, 2), (HeldRelic::CopperOre, 1)]),
        CraftingEntry::new(HeldRelic::WoodenSpade, 1, Tools, [(HeldRelic::DeadwoodBough, 2), (HeldRelic::EmptyPalm, 0)]),
        CraftingEntry::new(HeldRelic::CopperSpade, 1, Tools, [(HeldRelic::PlankBlock, 1), (HeldRelic::CopperOre, 2)]),
        CraftingEntry::new(HeldRelic::TitaniumSpade, 1, Tools, [(HeldRelic::PlankBlock, 1), (HeldRelic::TitaniumHull, 2)]),
        CraftingEntry::new(HeldRelic::CopperPickaxe, 1, Tools, [(HeldRelic::PlankBlock, 1), (HeldRelic::CopperOre, 2)]),
        CraftingEntry::new(HeldRelic::TitaniumPickaxe, 1, Tools, [(HeldRelic::PlankBlock, 1), (HeldRelic::TitaniumHull, 2)]),
    ]
};
