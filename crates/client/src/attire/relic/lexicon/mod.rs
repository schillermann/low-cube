mod bridge;

use world::{BlockMedium, SurfaceFind};
use super::HeldRelic;
use self::bridge::RelicWorldBridge;

/// Domain lexicon and property catalog for Space-Age `HeldRelic` items (`1` field).
pub struct RelicLexicon {
    relic: HeldRelic,
}

impl RelicLexicon {
    pub const TRAY_ORDER: [HeldRelic; HeldRelic::COUNT - 1] = [
        HeldRelic::RigWrench,
        HeldRelic::SuitRadio,
        HeldRelic::TitaniumBillet,
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
        HeldRelic::FieldCamera,
        HeldRelic::VehicleBlueprint,
        HeldRelic::SpawnStone,
        HeldRelic::CantileverCrane,
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
                | HeldRelic::RigWrench
                | HeldRelic::SuitRadio
                | HeldRelic::FieldCamera
                | HeldRelic::VehicleBlueprint
                | HeldRelic::CantileverCrane
        )
    }

    pub const fn title(&self) -> &'static str {
        match self.relic {
            HeldRelic::EmptyPalm => "EMPTY SATCHEL",
            HeldRelic::QuarriedStone => "STONE BLOCK",
            HeldRelic::RigWrench => "RIG WRENCH",
            HeldRelic::SuitRadio => "SUIT RADIO",
            HeldRelic::ExcavatedEarth => "EARTH BLOCK",
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
            HeldRelic::VehicleBlueprint => "BLUEPRINT BLOCK",
            HeldRelic::SpawnStone => "SPAWN STONE",
            HeldRelic::TitaniumHull => "TITANIUM HULL",
            HeldRelic::CarbonSilicate => "CARBON LACE",
            HeldRelic::VitrifiedBasalt => "VITRIFIED BASALT",
            HeldRelic::BlackBoxPod => "BLACKBOX POD",
            HeldRelic::LaserBeacon => "LASER BEACON",
            HeldRelic::TitaniumBillet => "SPACE TITANIUM",
        }
    }

    pub const fn block_medium(&self) -> BlockMedium {
        RelicWorldBridge::new(self.relic).block_medium()
    }

    pub const fn from_medium(medium: BlockMedium) -> HeldRelic {
        RelicWorldBridge::from_medium(medium)
    }

    pub const fn surface_find(&self) -> SurfaceFind {
        RelicWorldBridge::new(self.relic).surface_find()
    }

    pub const fn from_surface_find(find: SurfaceFind) -> HeldRelic {
        RelicWorldBridge::from_surface_find(find)
    }
}
