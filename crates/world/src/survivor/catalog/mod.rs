mod recipes;

use crate::{BlockMedium, SurfaceFind};
use self::recipes::RecipeLedger;

/// Total number of held/stowed item codes (`0..=48`, fitting inside the 6-bit `0..=63` palm wire span).
pub const RELIC_SLOT_COUNT: usize = 49;

/// Domain catalog mapping `BlockMedium`, `SurfaceFind`, config tokens, and crafting recipes to item codes (`<= 4` fields).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RelicCatalog {
    code: u8,
}

impl RelicCatalog {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(code: u8) -> Self {
        Self { code }
    }

    pub const fn code(self) -> u8 {
        self.code
    }

    pub const fn guest_permitted(self) -> bool {
        self.code > 0 && (self.code as usize) < RELIC_SLOT_COUNT && self.code != 29
    }

    /// Indicates whether a survivor may expend this item through nourishment or tool fracture.
    pub const fn expendable(self) -> bool {
        matches!(self.code, 3..=6 | 24 | 30 | 31 | 37..=41 | 45)
    }

    pub const fn slug(self) -> &'static str {
        match self.code {
            1 => "quarried_stone",
            2 => "timber_torch",
            3 => "wooden_mallet",
            4 => "woodman_axe",
            5 => "knightly_sword",
            6 => "iron_spade",
            7 => "earth",
            8 => "deadwood",
            9 => "flint",
            10 => "timber",
            11 => "planks",
            12 => "crust_rock",
            13 => "cantilever_crane",
            14 => "steering_wheel",
            15 => "aircraft_engine",
            16 => "flight_console",
            17 => "cabin_seat",
            18 => "car_wheel",
            19 => "field_camera",
            20 => "pedal_cluster",
            21 => "copper_ore",
            22 => "iron_ore",
            23 => "gold_ore",
            24 => "boar_meat",
            25 => "rocket_nozzle",
            26 => "rocket_camera",
            27 => "vehicle_blueprint",
            28 => "fusion_torch",
            29 => "spawn_stone",
            30 => "flint_pickaxe",
            31 => "iron_pickaxe",
            32 => "titanium_hull",
            33 => "carbon_silicate",
            34 => "vitrified_basalt",
            35 => "black_box_pod",
            36 => "laser_beacon",
            37 => "wooden_spade",
            38 => "copper_spade",
            39 => "titanium_spade",
            40 => "copper_pickaxe",
            41 => "titanium_pickaxe",
            42 => "baking_oven",
            43 => "bloomery_forge",
            44 => "smithy_anvil",
            45 => "roasted_boar",
            46 => "copper_bloom",
            47 => "iron_bloom",
            48 => "titanium_billet",
            _ => "empty",
        }
    }

    pub const fn from_medium(medium: BlockMedium) -> u8 {
        match medium {
            BlockMedium::TurfEarth | BlockMedium::Sand => 7,
            BlockMedium::CrustRock => 12,
            BlockMedium::QuarriedStone => 1,
            BlockMedium::RawTimber => 10,
            BlockMedium::PlanedPlanks => 11,
            BlockMedium::CantileverRig => 13,
            BlockMedium::SteeringWheel => 14,
            BlockMedium::AircraftEngine => 15,
            BlockMedium::FlightConsole => 16,
            BlockMedium::CabinSeat => 17,
            BlockMedium::CarWheel => 18,
            BlockMedium::FieldCamera => 19,
            BlockMedium::PedalCluster => 20,
            BlockMedium::CopperOre => 21,
            BlockMedium::IronOre => 22,
            BlockMedium::GoldOre => 23,
            BlockMedium::RocketNozzle => 25,
            BlockMedium::RocketCamera => 26,
            BlockMedium::VehicleBlueprint => 27,
            BlockMedium::FusionTorch => 28,
            BlockMedium::SpawnStone => 29,
            BlockMedium::TitaniumHull => 32,
            BlockMedium::CarbonSilicate => 33,
            BlockMedium::VitrifiedBasalt => 34,
            BlockMedium::BlackBoxPod => 35,
            BlockMedium::LaserBeacon => 36,
            BlockMedium::BakingOven => 42,
            BlockMedium::BloomeryForge => 43,
            BlockMedium::SmithyAnvil => 44,
            BlockMedium::OriginStone => 0,
        }
    }

    pub const fn from_find(find: SurfaceFind) -> u8 {
        match find {
            SurfaceFind::EmptyHanded => 0,
            SurfaceFind::DeadwoodBough => 8,
            SurfaceFind::FlintStone => 9,
            SurfaceFind::TimberTorch => 2,
            SurfaceFind::WoodenMallet => 3,
            SurfaceFind::WoodmanAxe => 4,
            SurfaceFind::KnightlySword => 5,
            SurfaceFind::IronSpade => 6,
            SurfaceFind::BoarMeat => 24,
            SurfaceFind::FlintPickaxe => 30,
            SurfaceFind::IronPickaxe => 31,
            SurfaceFind::WoodenSpade => 37,
            SurfaceFind::CopperSpade => 38,
            SurfaceFind::TitaniumSpade => 39,
            SurfaceFind::CopperPickaxe => 40,
            SurfaceFind::TitaniumPickaxe => 41,
            SurfaceFind::RoastedBoar => 45,
            SurfaceFind::CopperBloom => 46,
            SurfaceFind::IronBloom => 47,
            SurfaceFind::TitaniumBillet => 48,
            SurfaceFind::LooseBlock(m) => Self::from_medium(m),
        }
    }

    pub fn parse_token(token: &str) -> Option<u8> {
        RecipeLedger::parse_token(token)
    }

    pub const fn recipe_spec(recipe_idx: u8) -> Option<(u8, u32, [(u8, u32); 2])> {
        RecipeLedger::new(recipe_idx).spec()
    }

    pub const fn vacuum_hearth_recipe(recipe_idx: u8) -> bool {
        RecipeLedger::new(recipe_idx).vacuum_hearth()
    }

    pub const fn air_hearth_recipe(recipe_idx: u8) -> bool {
        RecipeLedger::new(recipe_idx).air_hearth()
    }
}
