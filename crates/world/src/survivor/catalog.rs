use crate::{BlockMedium, SurfaceFind};

/// Total number of held/stowed item codes (`0..=36`, matching `HeldRelic::COUNT` and 6-bit palm wire span).
pub const RELIC_SLOT_COUNT: usize = 37;

/// Domain catalog mapping `BlockMedium`, `SurfaceFind`, config tokens, and 25 crafting recipes to item codes (`<= 4` fields).
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
            SurfaceFind::LooseBlock(m) => Self::from_medium(m),
        }
    }

    pub fn parse_token(token: &str) -> Option<u8> {
        let key: String = token
            .trim()
            .chars()
            .filter(|c| !matches!(c, '_' | '-' | ' '))
            .map(|c| c.to_ascii_lowercase())
            .collect();
        match key.as_str() {
            "camera" | "fieldcamera" => Some(19),
            "stoneblock" | "quarriedstone" | "stone" => Some(1),
            "timbertorch" | "torch" => Some(2),
            "woodenmallet" | "mallet" => Some(3),
            "woodmanaxe" | "axe" => Some(4),
            "knightlysword" | "knightsword" | "sword" => Some(5),
            "ironspade" | "spade" => Some(6),
            "earthblock" | "excavatedearth" | "earth" | "dirt" => Some(7),
            "deadwood" | "deadwoodbough" => Some(8),
            "flintstone" | "flint" => Some(9),
            "woodblock" | "timberblock" | "timber" => Some(10),
            "woodplanks" | "plankblock" | "planks" => Some(11),
            "crustblock" | "crust" => Some(12),
            "cranelift" | "cantilevercrane" | "crane" => Some(13),
            "steeringwheel" | "steering" => Some(14),
            "planeengine" | "aircraftengine" => Some(15),
            "flightconsole" | "console" | "steuerkonsole" => Some(16),
            "fusiontorch" | "fusiondrive" | "epstein" | "fusionsantrieb" => Some(28),
            "rocketnozzle" | "nozzle" | "thruster" | "triebwerk" | "rocket" => Some(25),
            "rocketcamera" | "hullcamera" | "spacecamera" | "raketenkamera" | "nosecamera" => Some(26),
            "seatblock" | "cabinsat" | "cabinseat" | "seat" => Some(17),
            "carwheel" | "wheel" => Some(18),
            "pedalblock" | "pedalcluster" | "pedals" | "pedal" => Some(20),
            "copperore" | "copper" | "kupfer" => Some(21),
            "ironore" | "iron" | "eisen" => Some(22),
            "goldore" | "gold" | "golderz" => Some(23),
            "boarmeat" | "meat" | "venison" | "fleisch" | "wildschwein" => Some(24),
            "blueprint" | "blueprintblock" | "vehicleblueprint" | "bauplan" => Some(27),
            "spawnstone" | "spawnblock" | "spawnpoint" | "arrivalstone" => Some(29),
            "flintpickaxe" | "flintpick" | "stonepickaxe" | "pickaxe" | "hacke" | "spitzhacke" | "feuersteinhacke" => {
                Some(30)
            }
            "ironpickaxe" | "ironpick" | "eisenhacke" => Some(31),
            "titaniumhull" | "titanium" | "titaniumalloy" | "titan" | "titanhuelle" => Some(32),
            "carbonsilicate" | "carbonlace" | "silicatelace" | "cermet" | "keramik" => Some(33),
            "vitrifiedbasalt" | "fusedbasalt" | "scorchbasalt" | "schmelzbasalt" => Some(34),
            "blackboxpod" | "blackbox" | "flightblackbox" | "flugschreiber" => Some(35),
            "laserbeacon" | "opticalbeacon" | "leuchtfeuer" | "beacon" => Some(36),
            _ => None,
        }
    }

    /// Authoritative recipe table `(product_code, batch_yield, [(ing0, qty0), (ing1, qty1)])` for `0..25`.
    pub const fn recipe_spec(recipe_idx: u8) -> Option<(u8, u32, [(u8, u32); 2])> {
        match recipe_idx {
            0 => Some((4, 1, [(8, 1), (9, 1)])),
            1 => Some((11, 4, [(10, 1), (0, 0)])),
            2 => Some((13, 1, [(11, 2), (1, 1)])),
            3 => Some((27, 1, [(11, 1), (9, 1)])),
            4 => Some((3, 1, [(8, 2), (9, 1)])),
            5 => Some((6, 1, [(8, 1), (9, 2)])),
            6 => Some((5, 1, [(8, 1), (9, 3)])),
            7 => Some((2, 1, [(8, 1), (9, 1)])),
            8 => Some((1, 1, [(9, 2), (0, 0)])),
            9 => Some((17, 1, [(11, 1), (0, 0)])),
            10 => Some((20, 1, [(11, 1), (9, 1)])),
            11 => Some((18, 4, [(11, 1), (9, 1)])),
            12 => Some((14, 1, [(21, 1), (11, 1)])),
            13 => Some((19, 1, [(21, 2), (23, 1)])),
            14 => Some((15, 1, [(22, 2), (21, 2)])),
            15 => Some((16, 1, [(23, 2), (21, 2)])),
            16 => Some((25, 2, [(22, 3), (23, 1)])),
            17 => Some((28, 1, [(23, 2), (22, 3)])),
            18 => Some((26, 2, [(21, 2), (23, 1)])),
            19 => Some((30, 1, [(8, 1), (9, 3)])),
            20 => Some((31, 1, [(11, 1), (22, 2)])),
            21 => Some((32, 4, [(22, 2), (12, 1)])),
            22 => Some((33, 4, [(23, 1), (9, 2)])),
            23 => Some((35, 1, [(22, 2), (21, 1)])),
            24 => Some((36, 1, [(23, 2), (21, 1)])),
            _ => None,
        }
    }
}
