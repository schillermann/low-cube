mod recipes;

use crate::{BlockMedium, SurfaceFind};
use self::recipes::RecipeLedger;

/// Total number of Space-Age held/stowed item codes (`0..=29`, fitting inside the 6-bit `0..=63` palm wire span).
pub const RELIC_SLOT_COUNT: usize = 30;

/// Domain catalog mapping `BlockMedium`, `SurfaceFind`, config tokens, and Space-Age fabrication recipes to item codes (`<= 4` fields).
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
        self.code > 0 && (self.code as usize) < RELIC_SLOT_COUNT && self.code != 23
    }

    /// Indicates whether a survivor may expend this item through wear or consumption.
    pub const fn expendable(self) -> bool {
        false
    }

    pub const fn slug(self) -> &'static str {
        match self.code {
            1 => "quarried_stone",
            2 => "rig_wrench",
            3 => "suit_radio",
            4 => "earth",
            5 => "timber",
            6 => "planks",
            7 => "crust_rock",
            8 => "cantilever_crane",
            9 => "steering_wheel",
            10 => "aircraft_engine",
            11 => "flight_console",
            12 => "cabin_seat",
            13 => "car_wheel",
            14 => "field_camera",
            15 => "pedal_cluster",
            16 => "copper_ore",
            17 => "iron_ore",
            18 => "gold_ore",
            19 => "rocket_nozzle",
            20 => "rocket_camera",
            21 => "vehicle_blueprint",
            22 => "fusion_torch",
            23 => "spawn_stone",
            24 => "titanium_hull",
            25 => "carbon_silicate",
            26 => "vitrified_basalt",
            27 => "black_box_pod",
            28 => "laser_beacon",
            29 => "titanium_billet",
            _ => "empty",
        }
    }

    pub const fn from_medium(medium: BlockMedium) -> u8 {
        match medium {
            BlockMedium::QuarriedStone => 1,
            BlockMedium::TurfEarth | BlockMedium::Sand => 4,
            BlockMedium::RawTimber => 5,
            BlockMedium::PlanedPlanks => 6,
            BlockMedium::CrustRock => 7,
            BlockMedium::CantileverRig => 8,
            BlockMedium::SteeringWheel => 9,
            BlockMedium::AircraftEngine => 10,
            BlockMedium::FlightConsole => 11,
            BlockMedium::CabinSeat => 12,
            BlockMedium::CarWheel => 13,
            BlockMedium::FieldCamera => 14,
            BlockMedium::PedalCluster => 15,
            BlockMedium::CopperOre => 16,
            BlockMedium::IronOre => 17,
            BlockMedium::GoldOre => 18,
            BlockMedium::RocketNozzle => 19,
            BlockMedium::RocketCamera => 20,
            BlockMedium::VehicleBlueprint => 21,
            BlockMedium::FusionTorch => 22,
            BlockMedium::SpawnStone => 23,
            BlockMedium::TitaniumHull => 24,
            BlockMedium::CarbonSilicate => 25,
            BlockMedium::VitrifiedBasalt => 26,
            BlockMedium::BlackBoxPod => 27,
            BlockMedium::LaserBeacon => 28,
            BlockMedium::OriginStone => 0,
        }
    }

    pub const fn from_find(find: SurfaceFind) -> u8 {
        match find {
            SurfaceFind::EmptyHanded => 0,
            SurfaceFind::RigWrench => 2,
            SurfaceFind::SuitRadio => 3,
            SurfaceFind::TitaniumBillet => 29,
            SurfaceFind::LooseBlock(m) => Self::from_medium(m),
        }
    }

    pub fn parse_token(token: &str) -> Option<u8> {
        RecipeLedger::parse_token(token)
    }

    pub const fn recipe_spec(recipe_idx: u8) -> Option<(u8, u32, [(u8, u32); 2])> {
        RecipeLedger::new(recipe_idx).spec()
    }

    pub const fn vacuum_smelt_recipe(recipe_idx: u8) -> bool {
        RecipeLedger::new(recipe_idx).vacuum_smelt()
    }
}
