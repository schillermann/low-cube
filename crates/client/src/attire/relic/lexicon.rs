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
        HeldRelic::TitaniumHull,
        HeldRelic::CarbonSilicate,
        HeldRelic::LaserBeacon,
        HeldRelic::BlackBoxPod,
        HeldRelic::AircraftEngine,
        HeldRelic::ExcavatedEarth,
        HeldRelic::CrustBlock,
        HeldRelic::VitrifiedBasalt,
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

    pub const fn placeable_block(&self) -> bool {
        matches!(
            self.relic,
            HeldRelic::ExcavatedEarth
                | HeldRelic::QuarriedStone
                | HeldRelic::TimberBlock
                | HeldRelic::PlankBlock
                | HeldRelic::CrustBlock
                | HeldRelic::CantileverCrane
                | HeldRelic::SteeringWheel
                | HeldRelic::AircraftEngine
                | HeldRelic::FlightConsole
                | HeldRelic::RocketNozzle
                | HeldRelic::RocketCamera
                | HeldRelic::CabinSeat
                | HeldRelic::CarWheel
                | HeldRelic::FieldCamera
                | HeldRelic::PedalCluster
                | HeldRelic::CopperOre
                | HeldRelic::IronOre
                | HeldRelic::GoldOre
                | HeldRelic::VehicleBlueprint
                | HeldRelic::FusionTorch
                | HeldRelic::SpawnStone
                | HeldRelic::TitaniumHull
                | HeldRelic::CarbonSilicate
                | HeldRelic::VitrifiedBasalt
                | HeldRelic::BlackBoxPod
                | HeldRelic::LaserBeacon
        )
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
            HeldRelic::TitaniumHull => "TITANIUM HULL",
            HeldRelic::CarbonSilicate => "CARBON LACE",
            HeldRelic::VitrifiedBasalt => "VITRIFIED BASALT",
            HeldRelic::BlackBoxPod => "BLACKBOX POD",
            HeldRelic::LaserBeacon => "LASER BEACON",
        }
    }

    pub const fn short_title(&self) -> &'static str {
        match self.relic {
            HeldRelic::TimberBlock => "TIMBER",
            HeldRelic::PlankBlock => "PLANKS",
            HeldRelic::DeadwoodBough => "BOUGH",
            HeldRelic::FlintStone => "FLINT",
            HeldRelic::QuarriedStone => "STONE",
            HeldRelic::CrustBlock => "CRUST",
            HeldRelic::CopperOre => "COPPER",
            HeldRelic::IronOre => "IRON",
            HeldRelic::GoldOre => "GOLD",
            HeldRelic::ExcavatedEarth => "EARTH",
            _ => self.title(),
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
            HeldRelic::TitaniumHull => BlockMedium::TitaniumHull,
            HeldRelic::CarbonSilicate => BlockMedium::CarbonSilicate,
            HeldRelic::VitrifiedBasalt => BlockMedium::VitrifiedBasalt,
            HeldRelic::BlackBoxPod => BlockMedium::BlackBoxPod,
            HeldRelic::LaserBeacon => BlockMedium::LaserBeacon,
            _ => BlockMedium::TurfEarth,
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
            BlockMedium::TitaniumHull => HeldRelic::TitaniumHull,
            BlockMedium::CarbonSilicate => HeldRelic::CarbonSilicate,
            BlockMedium::VitrifiedBasalt => HeldRelic::VitrifiedBasalt,
            BlockMedium::BlackBoxPod => HeldRelic::BlackBoxPod,
            BlockMedium::LaserBeacon => HeldRelic::LaserBeacon,
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
