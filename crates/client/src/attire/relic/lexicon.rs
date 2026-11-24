use world::{BlockMedium, SurfaceFind};
use super::{HeldRelic, MartialStyle};

/// Domain lexicon and property catalog for `HeldRelic` items.
pub struct RelicLexicon {
    relic: HeldRelic,
}

impl RelicLexicon {
    pub const TRAY_ORDER: [HeldRelic; HeldRelic::COUNT - 1] = [
        HeldRelic::GoldOre,
        HeldRelic::IronOre,
        HeldRelic::CopperOre,
        HeldRelic::CabinSeat,
        HeldRelic::PedalCluster,
        HeldRelic::CarWheel,
        HeldRelic::SteeringWheel,
        HeldRelic::FlightConsole,
        HeldRelic::FusionTorch,
        HeldRelic::RocketNozzle,
        HeldRelic::RocketCamera,
        HeldRelic::AircraftEngine,
        HeldRelic::ExcavatedEarth,
        HeldRelic::CrustBlock,
        HeldRelic::QuarriedStone,
        HeldRelic::TimberBlock,
        HeldRelic::PlankBlock,
        HeldRelic::BoarMeat,
        HeldRelic::DeadwoodBough,
        HeldRelic::FlintStone,
        HeldRelic::FieldCamera,
        HeldRelic::VehicleBlueprint,
        HeldRelic::SpawnStone,
        HeldRelic::CantileverCrane,
        HeldRelic::IronPickaxe,
        HeldRelic::FlintPickaxe,
        HeldRelic::IronSpade,
        HeldRelic::WoodmanAxe,
        HeldRelic::KnightlySword,
        HeldRelic::WoodenMallet,
        HeldRelic::TimberTorch,
    ];

    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(relic: HeldRelic) -> Self {
        Self { relic }
    }

    pub const fn stackable_in_tray(&self) -> bool {
        !matches!(
            self.relic,
            HeldRelic::EmptyPalm
                | HeldRelic::FieldCamera
                | HeldRelic::VehicleBlueprint
                | HeldRelic::CantileverCrane
                | HeldRelic::FlintPickaxe
                | HeldRelic::IronPickaxe
                | HeldRelic::IronSpade
                | HeldRelic::WoodmanAxe
                | HeldRelic::KnightlySword
                | HeldRelic::WoodenMallet
                | HeldRelic::TimberTorch
        )
    }

    pub const fn title(&self) -> &'static str {
        match self.relic {
            HeldRelic::EmptyPalm => "EMPTY SATCHEL",
            HeldRelic::QuarriedStone => "STONE BLOCK",
            HeldRelic::TimberTorch => "TIMBER TORCH",
            HeldRelic::WoodenMallet => "WOODEN MALLET",
            HeldRelic::WoodmanAxe => "WOODMAN AXE",
            HeldRelic::KnightlySword => "KNIGHT SWORD",
            HeldRelic::IronSpade => "IRON SPADE",
            HeldRelic::FlintPickaxe => "FLINT PICKAXE",
            HeldRelic::IronPickaxe => "IRON PICKAXE",
            HeldRelic::ExcavatedEarth => "EARTH BLOCK",
            HeldRelic::DeadwoodBough => "DEADWOOD",
            HeldRelic::FlintStone => "FLINT STONE",
            HeldRelic::TimberBlock => "WOOD BLOCK",
            HeldRelic::PlankBlock => "WOOD PLANKS",
            HeldRelic::CrustBlock => "CRUST BLOCK",
            HeldRelic::CantileverCrane => "CRANE LIFT",
            HeldRelic::SteeringWheel => "STEERING WHEEL",
            HeldRelic::AircraftEngine => "PLANE ENGINE",
            HeldRelic::FlightConsole => "FLIGHT CONSOLE",
            HeldRelic::FusionTorch => "FUSION TORCH",
            HeldRelic::RocketNozzle => "ROCKET NOZZLE",
            HeldRelic::RocketCamera => "ROCKET CAMERA",
            HeldRelic::CabinSeat => "SEAT BLOCK",
            HeldRelic::CarWheel => "CAR WHEEL",
            HeldRelic::FieldCamera => "FIELD CAMERA",
            HeldRelic::PedalCluster => "PEDAL BLOCK",
            HeldRelic::CopperOre => "COPPER ORE",
            HeldRelic::IronOre => "IRON ORE",
            HeldRelic::GoldOre => "GOLD ORE",
            HeldRelic::BoarMeat => "BOAR MEAT",
            HeldRelic::VehicleBlueprint => "BLUEPRINT BLOCK",
            HeldRelic::SpawnStone => "SPAWN STONE",
        }
    }

    pub fn parse(&self, token: &str) -> Option<HeldRelic> {
        let key: String = token
            .trim()
            .chars()
            .filter(|c| !matches!(c, '_' | '-' | ' '))
            .map(|c| c.to_ascii_lowercase())
            .collect();
        match key.as_str() {
            "camera" | "fieldcamera" => Some(HeldRelic::FieldCamera),
            "stoneblock" | "quarriedstone" | "stone" => Some(HeldRelic::QuarriedStone),
            "timbertorch" | "torch" => Some(HeldRelic::TimberTorch),
            "woodenmallet" | "mallet" => Some(HeldRelic::WoodenMallet),
            "woodmanaxe" | "axe" => Some(HeldRelic::WoodmanAxe),
            "knightlysword" | "knightsword" | "sword" => Some(HeldRelic::KnightlySword),
            "ironspade" | "spade" => Some(HeldRelic::IronSpade),
            "flintpickaxe" | "flintpick" | "stonepickaxe" | "pickaxe" | "hacke" | "spitzhacke" | "feuersteinhacke" => {
                Some(HeldRelic::FlintPickaxe)
            }
            "ironpickaxe" | "ironpick" | "eisenhacke" => Some(HeldRelic::IronPickaxe),
            "earthblock" | "excavatedearth" | "earth" | "dirt" => Some(HeldRelic::ExcavatedEarth),
            "deadwood" | "deadwoodbough" => Some(HeldRelic::DeadwoodBough),
            "flintstone" | "flint" => Some(HeldRelic::FlintStone),
            "woodblock" | "timberblock" | "timber" => Some(HeldRelic::TimberBlock),
            "woodplanks" | "plankblock" | "planks" => Some(HeldRelic::PlankBlock),
            "crustblock" | "crust" => Some(HeldRelic::CrustBlock),
            "cranelift" | "cantilevercrane" | "crane" => Some(HeldRelic::CantileverCrane),
            "steeringwheel" | "steering" => Some(HeldRelic::SteeringWheel),
            "planeengine" | "aircraftengine" => Some(HeldRelic::AircraftEngine),
            "flightconsole" | "console" | "steuerkonsole" => Some(HeldRelic::FlightConsole),
            "fusiontorch" | "fusiondrive" | "epstein" | "fusionsantrieb" => Some(HeldRelic::FusionTorch),
            "rocketnozzle" | "nozzle" | "thruster" | "triebwerk" | "rocket" => Some(HeldRelic::RocketNozzle),
            "rocketcamera" | "hullcamera" | "spacecamera" | "raketenkamera" | "nosecamera" => Some(HeldRelic::RocketCamera),
            "seatblock" | "cabinsat" | "cabinseat" | "seat" => Some(HeldRelic::CabinSeat),
            "carwheel" | "wheel" => Some(HeldRelic::CarWheel),
            "pedalblock" | "pedalcluster" | "pedals" | "pedal" => Some(HeldRelic::PedalCluster),
            "copperore" | "copper" | "kupfer" => Some(HeldRelic::CopperOre),
            "ironore" | "iron" | "eisen" => Some(HeldRelic::IronOre),
            "goldore" | "gold" => Some(HeldRelic::GoldOre),
            "boarmeat" | "meat" | "venison" | "fleisch" | "wildschwein" => Some(HeldRelic::BoarMeat),
            "blueprint" | "blueprintblock" | "vehicleblueprint" | "bauplan" => Some(HeldRelic::VehicleBlueprint),
            "spawnstone" | "spawnblock" | "spawnpoint" | "arrivalstone" => Some(HeldRelic::SpawnStone),
            _ => None,
        }
    }

    pub const fn block_medium(&self) -> BlockMedium {
        match self.relic {
            HeldRelic::CrustBlock => BlockMedium::CrustRock,
            HeldRelic::QuarriedStone => BlockMedium::QuarriedStone,
            HeldRelic::TimberBlock => BlockMedium::RawTimber,
            HeldRelic::PlankBlock => BlockMedium::PlanedPlanks,
            HeldRelic::CantileverCrane => BlockMedium::CantileverRig,
            HeldRelic::SteeringWheel => BlockMedium::SteeringWheel,
            HeldRelic::AircraftEngine => BlockMedium::AircraftEngine,
            HeldRelic::FlightConsole => BlockMedium::FlightConsole,
            HeldRelic::FusionTorch => BlockMedium::FusionTorch,
            HeldRelic::RocketNozzle => BlockMedium::RocketNozzle,
            HeldRelic::RocketCamera => BlockMedium::RocketCamera,
            HeldRelic::CabinSeat => BlockMedium::CabinSeat,
            HeldRelic::CarWheel => BlockMedium::CarWheel,
            HeldRelic::FieldCamera => BlockMedium::FieldCamera,
            HeldRelic::PedalCluster => BlockMedium::PedalCluster,
            HeldRelic::CopperOre => BlockMedium::CopperOre,
            HeldRelic::IronOre => BlockMedium::IronOre,
            HeldRelic::GoldOre => BlockMedium::GoldOre,
            HeldRelic::VehicleBlueprint => BlockMedium::VehicleBlueprint,
            HeldRelic::SpawnStone => BlockMedium::SpawnStone,
            _ => BlockMedium::TurfEarth,
        }
    }

    pub const fn martial_style(&self) -> MartialStyle {
        match self.relic {
            HeldRelic::EmptyPalm => MartialStyle::Unarmed,
            HeldRelic::IronSpade => MartialStyle::SpadeDig,
            HeldRelic::WoodenMallet
            | HeldRelic::WoodmanAxe
            | HeldRelic::FlintPickaxe
            | HeldRelic::IronPickaxe => MartialStyle::LumberjackCleave,
            HeldRelic::TimberTorch | HeldRelic::KnightlySword => MartialStyle::SwordplaySlash,
            _ => MartialStyle::PalmCradle,
        }
    }

    pub const fn surface_find(&self) -> SurfaceFind {
        match self.relic {
            HeldRelic::EmptyPalm => SurfaceFind::EmptyHanded,
            HeldRelic::DeadwoodBough => SurfaceFind::DeadwoodBough,
            HeldRelic::FlintStone => SurfaceFind::FlintStone,
            HeldRelic::TimberTorch => SurfaceFind::TimberTorch,
            HeldRelic::WoodenMallet => SurfaceFind::WoodenMallet,
            HeldRelic::WoodmanAxe => SurfaceFind::WoodmanAxe,
            HeldRelic::KnightlySword => SurfaceFind::KnightlySword,
            HeldRelic::IronSpade => SurfaceFind::IronSpade,
            HeldRelic::FlintPickaxe => SurfaceFind::FlintPickaxe,
            HeldRelic::IronPickaxe => SurfaceFind::IronPickaxe,
            HeldRelic::BoarMeat => SurfaceFind::BoarMeat,
            _ => SurfaceFind::LooseBlock(self.block_medium()),
        }
    }

    pub const fn from_medium(medium: BlockMedium) -> HeldRelic {
        match medium {
            BlockMedium::TurfEarth | BlockMedium::Sand => HeldRelic::ExcavatedEarth,
            BlockMedium::CrustRock => HeldRelic::CrustBlock,
            BlockMedium::QuarriedStone => HeldRelic::QuarriedStone,
            BlockMedium::RawTimber => HeldRelic::TimberBlock,
            BlockMedium::PlanedPlanks => HeldRelic::PlankBlock,
            BlockMedium::CantileverRig => HeldRelic::CantileverCrane,
            BlockMedium::SteeringWheel => HeldRelic::SteeringWheel,
            BlockMedium::AircraftEngine => HeldRelic::AircraftEngine,
            BlockMedium::FlightConsole => HeldRelic::FlightConsole,
            BlockMedium::RocketNozzle => HeldRelic::RocketNozzle,
            BlockMedium::RocketCamera => HeldRelic::RocketCamera,
            BlockMedium::CabinSeat => HeldRelic::CabinSeat,
            BlockMedium::CarWheel => HeldRelic::CarWheel,
            BlockMedium::FieldCamera => HeldRelic::FieldCamera,
            BlockMedium::PedalCluster => HeldRelic::PedalCluster,
            BlockMedium::CopperOre => HeldRelic::CopperOre,
            BlockMedium::IronOre => HeldRelic::IronOre,
            BlockMedium::GoldOre => HeldRelic::GoldOre,
            BlockMedium::VehicleBlueprint => HeldRelic::VehicleBlueprint,
            BlockMedium::FusionTorch => HeldRelic::FusionTorch,
            BlockMedium::SpawnStone => HeldRelic::SpawnStone,
        }
    }

    pub const fn from_surface_find(find: SurfaceFind) -> HeldRelic {
        match find {
            SurfaceFind::EmptyHanded => HeldRelic::EmptyPalm,
            SurfaceFind::DeadwoodBough => HeldRelic::DeadwoodBough,
            SurfaceFind::FlintStone => HeldRelic::FlintStone,
            SurfaceFind::TimberTorch => HeldRelic::TimberTorch,
            SurfaceFind::WoodenMallet => HeldRelic::WoodenMallet,
            SurfaceFind::WoodmanAxe => HeldRelic::WoodmanAxe,
            SurfaceFind::KnightlySword => HeldRelic::KnightlySword,
            SurfaceFind::IronSpade => HeldRelic::IronSpade,
            SurfaceFind::FlintPickaxe => HeldRelic::FlintPickaxe,
            SurfaceFind::IronPickaxe => HeldRelic::IronPickaxe,
            SurfaceFind::BoarMeat => HeldRelic::BoarMeat,
            SurfaceFind::LooseBlock(medium) => Self::from_medium(medium),
        }
    }
}
