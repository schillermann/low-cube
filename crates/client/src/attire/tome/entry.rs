use super::super::pouch::BeltSatchel;
use super::super::relic::HeldRelic;
use super::super::YouthAttire;
use super::domain::{CraftVenue, FolioSection, FolioTab};

pub const RECIPE_COUNT: usize = 33;

/// Single craftable recipe entry inside the tactile Field Folio (`4` private fields).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CraftingEntry {
    product: HeldRelic,
    batch_yield: u32,
    section: FolioSection,
    ingredients: [(HeldRelic, u32); 2],
}

impl CraftingEntry {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(
        product: HeldRelic,
        batch_yield: u32,
        section: FolioSection,
        ingredients: [(HeldRelic, u32); 2],
    ) -> Self {
        Self {
            product,
            batch_yield,
            section,
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

    pub const fn requires_anvil(&self) -> bool {
        self.section.requires_anvil()
    }

    pub const fn in_tab(&self, candidate: FolioTab) -> bool {
        matches!(
            (self.section.tab(), candidate),
            (FolioTab::Tools, FolioTab::Tools)
                | (FolioTab::Blocks, FolioTab::Blocks)
                | (FolioTab::Vehicles, FolioTab::Vehicles)
                | (FolioTab::Orbital, FolioTab::Orbital)
        )
    }

    pub fn ingredients_stocked(&self, satchel: &BeltSatchel, attire: YouthAttire) -> bool {
        self.ingredients
            .iter()
            .all(|&(relic, needed)| needed == 0 || satchel.resource_tally(relic, attire) >= needed)
    }

    pub fn craftable_at(&self, satchel: &BeltSatchel, attire: YouthAttire, venue: CraftVenue) -> bool {
        self.section.permitted_at(venue) && self.ingredients_stocked(satchel, attire)
    }
}

pub const RECIPE_CATALOG: [CraftingEntry; RECIPE_COUNT] = {
    use FolioTab::{Blocks, Orbital, Tools, Vehicles};
    let (field_t, anvil_t) = (FolioSection::field(Tools), FolioSection::anvil(Tools));
    let (field_b, anvil_b) = (FolioSection::field(Blocks), FolioSection::anvil(Blocks));
    let (field_v, field_o) = (FolioSection::field(Vehicles), FolioSection::field(Orbital));
    [
        CraftingEntry::new(HeldRelic::WoodmanAxe, 1, anvil_t, [(HeldRelic::PlankBlock, 1), (HeldRelic::IronBloom, 2)]),
        CraftingEntry::new(HeldRelic::PlankBlock, 4, field_b, [(HeldRelic::TimberBlock, 1), (HeldRelic::EmptyPalm, 0)]),
        CraftingEntry::new(HeldRelic::CantileverCrane, 1, field_b, [(HeldRelic::PlankBlock, 2), (HeldRelic::QuarriedStone, 1)]),
        CraftingEntry::new(HeldRelic::VehicleBlueprint, 1, field_b, [(HeldRelic::PlankBlock, 1), (HeldRelic::FlintStone, 1)]),
        CraftingEntry::new(HeldRelic::WoodenMallet, 1, field_t, [(HeldRelic::DeadwoodBough, 2), (HeldRelic::EmptyPalm, 0)]),
        CraftingEntry::new(HeldRelic::IronSpade, 1, anvil_t, [(HeldRelic::PlankBlock, 1), (HeldRelic::IronBloom, 2)]),
        CraftingEntry::new(HeldRelic::KnightlySword, 1, anvil_t, [(HeldRelic::PlankBlock, 1), (HeldRelic::IronBloom, 3)]),
        CraftingEntry::new(HeldRelic::TimberTorch, 1, field_t, [(HeldRelic::DeadwoodBough, 1), (HeldRelic::FlintStone, 1)]),
        CraftingEntry::new(HeldRelic::QuarriedStone, 1, field_b, [(HeldRelic::FlintStone, 2), (HeldRelic::EmptyPalm, 0)]),
        CraftingEntry::new(HeldRelic::CabinSeat, 1, field_v, [(HeldRelic::PlankBlock, 1), (HeldRelic::EmptyPalm, 0)]),
        CraftingEntry::new(HeldRelic::PedalCluster, 1, field_v, [(HeldRelic::PlankBlock, 1), (HeldRelic::FlintStone, 1)]),
        CraftingEntry::new(HeldRelic::CarWheel, 4, field_v, [(HeldRelic::PlankBlock, 1), (HeldRelic::FlintStone, 1)]),
        CraftingEntry::new(HeldRelic::SteeringWheel, 1, field_v, [(HeldRelic::CopperOre, 1), (HeldRelic::PlankBlock, 1)]),
        CraftingEntry::new(HeldRelic::FieldCamera, 1, field_o, [(HeldRelic::CopperOre, 2), (HeldRelic::GoldOre, 1)]),
        CraftingEntry::new(HeldRelic::AircraftEngine, 1, field_v, [(HeldRelic::IronOre, 2), (HeldRelic::CopperOre, 2)]),
        CraftingEntry::new(HeldRelic::FlightConsole, 1, field_o, [(HeldRelic::GoldOre, 2), (HeldRelic::CopperOre, 2)]),
        CraftingEntry::new(HeldRelic::RocketNozzle, 2, field_o, [(HeldRelic::IronOre, 3), (HeldRelic::GoldOre, 1)]),
        CraftingEntry::new(HeldRelic::FusionTorch, 1, field_o, [(HeldRelic::GoldOre, 2), (HeldRelic::IronOre, 3)]),
        CraftingEntry::new(HeldRelic::RocketCamera, 2, field_o, [(HeldRelic::CopperOre, 2), (HeldRelic::GoldOre, 1)]),
        CraftingEntry::new(HeldRelic::FlintPickaxe, 1, field_t, [(HeldRelic::DeadwoodBough, 1), (HeldRelic::FlintStone, 3)]),
        CraftingEntry::new(HeldRelic::IronPickaxe, 1, anvil_t, [(HeldRelic::PlankBlock, 1), (HeldRelic::IronBloom, 2)]),
        CraftingEntry::new(HeldRelic::TitaniumHull, 4, anvil_b, [(HeldRelic::TitaniumBillet, 2), (HeldRelic::IronBloom, 1)]),
        CraftingEntry::new(HeldRelic::CarbonSilicate, 4, field_b, [(HeldRelic::GoldOre, 1), (HeldRelic::FlintStone, 2)]),
        CraftingEntry::new(HeldRelic::BlackBoxPod, 1, field_o, [(HeldRelic::IronOre, 2), (HeldRelic::CopperOre, 1)]),
        CraftingEntry::new(HeldRelic::LaserBeacon, 1, field_o, [(HeldRelic::GoldOre, 2), (HeldRelic::CopperOre, 1)]),
        CraftingEntry::new(HeldRelic::WoodenSpade, 1, field_t, [(HeldRelic::DeadwoodBough, 2), (HeldRelic::EmptyPalm, 0)]),
        CraftingEntry::new(HeldRelic::CopperSpade, 1, anvil_t, [(HeldRelic::PlankBlock, 1), (HeldRelic::CopperBloom, 2)]),
        CraftingEntry::new(HeldRelic::TitaniumSpade, 1, anvil_t, [(HeldRelic::PlankBlock, 1), (HeldRelic::TitaniumBillet, 1)]),
        CraftingEntry::new(HeldRelic::CopperPickaxe, 1, anvil_t, [(HeldRelic::PlankBlock, 1), (HeldRelic::CopperBloom, 2)]),
        CraftingEntry::new(HeldRelic::TitaniumPickaxe, 1, anvil_t, [(HeldRelic::PlankBlock, 1), (HeldRelic::TitaniumBillet, 2)]),
        CraftingEntry::new(HeldRelic::BakingOven, 1, field_b, [(HeldRelic::QuarriedStone, 2), (HeldRelic::DeadwoodBough, 1)]),
        CraftingEntry::new(HeldRelic::BloomeryForge, 1, field_b, [(HeldRelic::QuarriedStone, 3), (HeldRelic::TimberTorch, 1)]),
        CraftingEntry::new(HeldRelic::SmithyAnvil, 1, field_b, [(HeldRelic::QuarriedStone, 2), (HeldRelic::TimberBlock, 1)]),
    ]
};
