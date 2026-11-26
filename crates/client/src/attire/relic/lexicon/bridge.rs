use world::{BlockMedium, SurfaceFind};
use super::super::HeldRelic;

pub(super) struct RelicWorldBridge {
    relic: HeldRelic,
}

impl RelicWorldBridge {
    pub(super) const fn new(relic: HeldRelic) -> Self {
        Self { relic }
    }

    pub(super) const fn block_medium(&self) -> BlockMedium {
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
            HeldRelic::BakingOven => BlockMedium::BakingOven,
            HeldRelic::BloomeryForge => BlockMedium::BloomeryForge,
            HeldRelic::SmithyAnvil => BlockMedium::SmithyAnvil,
            _ => BlockMedium::TurfEarth,
        }
    }

    pub(super) const fn from_medium(medium: BlockMedium) -> HeldRelic {
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
            BlockMedium::BakingOven => HeldRelic::BakingOven,
            BlockMedium::BloomeryForge => HeldRelic::BloomeryForge,
            BlockMedium::SmithyAnvil => HeldRelic::SmithyAnvil,
        }
    }

    pub(super) const fn surface_find(&self) -> SurfaceFind {
        match self.relic {
            HeldRelic::EmptyPalm => SurfaceFind::EmptyHanded,
            HeldRelic::DeadwoodBough => SurfaceFind::DeadwoodBough,
            HeldRelic::FlintStone => SurfaceFind::FlintStone,
            HeldRelic::TimberTorch => SurfaceFind::TimberTorch,
            HeldRelic::WoodenMallet => SurfaceFind::WoodenMallet,
            HeldRelic::WoodmanAxe => SurfaceFind::WoodmanAxe,
            HeldRelic::KnightlySword => SurfaceFind::KnightlySword,
            HeldRelic::WoodenSpade => SurfaceFind::WoodenSpade,
            HeldRelic::CopperSpade => SurfaceFind::CopperSpade,
            HeldRelic::IronSpade => SurfaceFind::IronSpade,
            HeldRelic::TitaniumSpade => SurfaceFind::TitaniumSpade,
            HeldRelic::FlintPickaxe => SurfaceFind::FlintPickaxe,
            HeldRelic::CopperPickaxe => SurfaceFind::CopperPickaxe,
            HeldRelic::IronPickaxe => SurfaceFind::IronPickaxe,
            HeldRelic::TitaniumPickaxe => SurfaceFind::TitaniumPickaxe,
            HeldRelic::BoarMeat => SurfaceFind::BoarMeat,
            HeldRelic::RoastedBoar => SurfaceFind::RoastedBoar,
            HeldRelic::CopperBloom => SurfaceFind::CopperBloom,
            HeldRelic::IronBloom => SurfaceFind::IronBloom,
            HeldRelic::TitaniumBillet => SurfaceFind::TitaniumBillet,
            _ => SurfaceFind::LooseBlock(self.block_medium()),
        }
    }

    pub(super) const fn from_surface_find(find: SurfaceFind) -> HeldRelic {
        match find {
            SurfaceFind::EmptyHanded => HeldRelic::EmptyPalm,
            SurfaceFind::DeadwoodBough => HeldRelic::DeadwoodBough,
            SurfaceFind::FlintStone => HeldRelic::FlintStone,
            SurfaceFind::TimberTorch => HeldRelic::TimberTorch,
            SurfaceFind::WoodenMallet => HeldRelic::WoodenMallet,
            SurfaceFind::WoodmanAxe => HeldRelic::WoodmanAxe,
            SurfaceFind::KnightlySword => HeldRelic::KnightlySword,
            SurfaceFind::WoodenSpade => HeldRelic::WoodenSpade,
            SurfaceFind::CopperSpade => HeldRelic::CopperSpade,
            SurfaceFind::IronSpade => HeldRelic::IronSpade,
            SurfaceFind::TitaniumSpade => HeldRelic::TitaniumSpade,
            SurfaceFind::FlintPickaxe => HeldRelic::FlintPickaxe,
            SurfaceFind::CopperPickaxe => HeldRelic::CopperPickaxe,
            SurfaceFind::IronPickaxe => HeldRelic::IronPickaxe,
            SurfaceFind::TitaniumPickaxe => HeldRelic::TitaniumPickaxe,
            SurfaceFind::BoarMeat => HeldRelic::BoarMeat,
            SurfaceFind::RoastedBoar => HeldRelic::RoastedBoar,
            SurfaceFind::CopperBloom => HeldRelic::CopperBloom,
            SurfaceFind::IronBloom => HeldRelic::IronBloom,
            SurfaceFind::TitaniumBillet => HeldRelic::TitaniumBillet,
            SurfaceFind::LooseBlock(medium) => Self::from_medium(medium),
        }
    }
}
