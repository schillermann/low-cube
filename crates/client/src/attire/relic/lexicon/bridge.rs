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
            BlockMedium::OriginStone => HeldRelic::EmptyPalm,
        }
    }

    pub(super) const fn surface_find(&self) -> SurfaceFind {
        match self.relic {
            HeldRelic::EmptyPalm => SurfaceFind::EmptyHanded,
            HeldRelic::RigWrench => SurfaceFind::RigWrench,
            HeldRelic::SuitRadio => SurfaceFind::SuitRadio,
            HeldRelic::TitaniumBillet => SurfaceFind::TitaniumBillet,
            _ => SurfaceFind::LooseBlock(self.block_medium()),
        }
    }

    pub(super) const fn from_surface_find(find: SurfaceFind) -> HeldRelic {
        match find {
            SurfaceFind::EmptyHanded => HeldRelic::EmptyPalm,
            SurfaceFind::RigWrench => HeldRelic::RigWrench,
            SurfaceFind::SuitRadio => HeldRelic::SuitRadio,
            SurfaceFind::TitaniumBillet => HeldRelic::TitaniumBillet,
            SurfaceFind::LooseBlock(medium) => Self::from_medium(medium),
        }
    }
}
