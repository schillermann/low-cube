mod bridge;

use world::{BlockMedium, SurfaceFind};
use super::HeldRelic;
use self::bridge::RelicWorldBridge;

/// Domain lexicon and property catalog for `HeldRelic` items (`1` field).
pub struct RelicLexicon {
    relic: HeldRelic,
}

impl RelicLexicon {
    pub const TRAY_ORDER: [HeldRelic; HeldRelic::COUNT - 1] = [
        HeldRelic::TitaniumBillet,
        HeldRelic::IronBloom,
        HeldRelic::CopperBloom,
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
        HeldRelic::BakingOven,
        HeldRelic::BloomeryForge,
        HeldRelic::SmithyAnvil,
        HeldRelic::ExcavatedEarth,
        HeldRelic::CrustBlock,
        HeldRelic::VitrifiedBasalt,
        HeldRelic::QuarriedStone,
        HeldRelic::TimberBlock,
        HeldRelic::PlankBlock,
        HeldRelic::RoastedBoar,
        HeldRelic::BoarMeat,
        HeldRelic::DeadwoodBough,
        HeldRelic::FlintStone,
        HeldRelic::FieldCamera,
        HeldRelic::VehicleBlueprint,
        HeldRelic::SpawnStone,
        HeldRelic::CantileverCrane,
        HeldRelic::TitaniumPickaxe,
        HeldRelic::IronPickaxe,
        HeldRelic::CopperPickaxe,
        HeldRelic::FlintPickaxe,
        HeldRelic::TitaniumSpade,
        HeldRelic::IronSpade,
        HeldRelic::CopperSpade,
        HeldRelic::WoodenSpade,
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
                | HeldRelic::BakingOven
                | HeldRelic::BloomeryForge
                | HeldRelic::SmithyAnvil
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
                | HeldRelic::CopperPickaxe
                | HeldRelic::IronPickaxe
                | HeldRelic::TitaniumPickaxe
                | HeldRelic::WoodenSpade
                | HeldRelic::CopperSpade
                | HeldRelic::IronSpade
                | HeldRelic::TitaniumSpade
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
            HeldRelic::WoodenSpade => "WOODEN SPADE",
            HeldRelic::CopperSpade => "COPPER SPADE",
            HeldRelic::IronSpade => "IRON SPADE",
            HeldRelic::TitaniumSpade => "TITANIUM SPADE",
            HeldRelic::FlintPickaxe => "FLINT PICKAXE",
            HeldRelic::CopperPickaxe => "COPPER PICKAXE",
            HeldRelic::IronPickaxe => "IRON PICKAXE",
            HeldRelic::TitaniumPickaxe => "TITANIUM PICKAXE",
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
            HeldRelic::RoastedBoar => "ROAST BOAR",
            HeldRelic::VehicleBlueprint => "BLUEPRINT BLOCK",
            HeldRelic::SpawnStone => "SPAWN STONE",
            HeldRelic::TitaniumHull => "TITANIUM HULL",
            HeldRelic::CarbonSilicate => "CARBON LACE",
            HeldRelic::VitrifiedBasalt => "VITRIFIED BASALT",
            HeldRelic::BlackBoxPod => "BLACKBOX POD",
            HeldRelic::LaserBeacon => "LASER BEACON",
            HeldRelic::BakingOven => "BAKING OVEN",
            HeldRelic::BloomeryForge => "SMELTING FORGE",
            HeldRelic::SmithyAnvil => "SMITHY ANVIL",
            HeldRelic::CopperBloom => "COPPER BLOOM",
            HeldRelic::IronBloom => "IRON BLOOM",
            HeldRelic::TitaniumBillet => "SPACE TITANIUM",
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
            HeldRelic::TitaniumHull => "TITANIUM",
            HeldRelic::TimberTorch => "TORCH",
            HeldRelic::BakingOven => "OVEN",
            HeldRelic::BloomeryForge => "FORGE",
            HeldRelic::SmithyAnvil => "ANVIL",
            HeldRelic::RoastedBoar => "ROAST",
            HeldRelic::CopperBloom => "CU BLOOM",
            HeldRelic::IronBloom => "FE BLOOM",
            HeldRelic::TitaniumBillet => "TI BILLET",
            _ => self.title(),
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
