mod lexicon;
mod mesh;
mod plinth;
mod temper;
mod tokens;
mod tools;

use world::{BlockMedium, BlockMold, MiningEdge, SurfaceFind};
pub use super::rhythm::MartialStyle;
pub use mesh::RelicFigure;
pub use temper::ToolCondition;
use lexicon::RelicLexicon;
use temper::ToolTemper;
use tokens::RelicTokenLexicon;

/// Space-Age EVA tool, transceiver, billet, multi-block machine, or scalable module (`0..=40`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HeldRelic {
    EmptyPalm,
    QuarriedStone,
    RigWrench,
    SuitRadio,
    ExcavatedEarth,
    TimberBlock,
    PlankBlock,
    CrustBlock,
    CantileverCrane,
    SteeringWheel,
    AircraftEngine,
    FlightConsole,
    CabinSeat,
    CarWheel,
    FieldCamera,
    PedalCluster,
    CopperOre,
    IronOre,
    GoldOre,
    RocketNozzle,
    RocketCamera,
    VehicleBlueprint,
    FusionTorch,
    SpawnStone,
    TitaniumHull,
    CarbonSilicate,
    VitrifiedBasalt,
    BlackBoxPod,
    LaserBeacon,
    TitaniumBillet,
    ArcFurnace,
    AssemblyBench,
    ElectrolysisPlant,
    AirlockChamber,
    PilotSeat,
    PhosphorConsole,
    DockingRing,
    CrewBerth,
    PropulsionBell,
    PropellantTank,
    ThermalFin,
}

impl HeldRelic {
    pub const COUNT: usize = 41;
    pub const TRAY_ORDER: [Self; Self::COUNT - 1] = RelicLexicon::TRAY_ORDER;

    #[cfg(test)]
    pub fn next(self) -> Self {
        Self::from_code((self.code() + 1) % (Self::COUNT as u8))
    }

    pub const fn from_code(code: u8) -> Self {
        match code % (Self::COUNT as u8) {
            0 => Self::EmptyPalm,
            1 => Self::QuarriedStone,
            2 => Self::RigWrench,
            3 => Self::SuitRadio,
            4 => Self::ExcavatedEarth,
            5 => Self::TimberBlock,
            6 => Self::PlankBlock,
            7 => Self::CrustBlock,
            8 => Self::CantileverCrane,
            9 => Self::SteeringWheel,
            10 => Self::AircraftEngine,
            11 => Self::FlightConsole,
            12 => Self::CabinSeat,
            13 => Self::CarWheel,
            14 => Self::FieldCamera,
            15 => Self::PedalCluster,
            16 => Self::CopperOre,
            17 => Self::IronOre,
            18 => Self::GoldOre,
            19 => Self::RocketNozzle,
            20 => Self::RocketCamera,
            21 => Self::VehicleBlueprint,
            22 => Self::FusionTorch,
            23 => Self::SpawnStone,
            24 => Self::TitaniumHull,
            25 => Self::CarbonSilicate,
            26 => Self::VitrifiedBasalt,
            27 => Self::BlackBoxPod,
            28 => Self::LaserBeacon,
            29 => Self::TitaniumBillet,
            30 => Self::ArcFurnace,
            31 => Self::AssemblyBench,
            32 => Self::ElectrolysisPlant,
            33 => Self::AirlockChamber,
            34 => Self::PilotSeat,
            35 => Self::PhosphorConsole,
            36 => Self::DockingRing,
            37 => Self::CrewBerth,
            38 => Self::PropulsionBell,
            39 => Self::PropellantTank,
            _ => Self::ThermalFin,
        }
    }

    pub const fn code(self) -> u8 {
        match self {
            Self::EmptyPalm => 0,
            Self::QuarriedStone => 1,
            Self::RigWrench => 2,
            Self::SuitRadio => 3,
            Self::ExcavatedEarth => 4,
            Self::TimberBlock => 5,
            Self::PlankBlock => 6,
            Self::CrustBlock => 7,
            Self::CantileverCrane => 8,
            Self::SteeringWheel => 9,
            Self::AircraftEngine => 10,
            Self::FlightConsole => 11,
            Self::CabinSeat => 12,
            Self::CarWheel => 13,
            Self::FieldCamera => 14,
            Self::PedalCluster => 15,
            Self::CopperOre => 16,
            Self::IronOre => 17,
            Self::GoldOre => 18,
            Self::RocketNozzle => 19,
            Self::RocketCamera => 20,
            Self::VehicleBlueprint => 21,
            Self::FusionTorch => 22,
            Self::SpawnStone => 23,
            Self::TitaniumHull => 24,
            Self::CarbonSilicate => 25,
            Self::VitrifiedBasalt => 26,
            Self::BlackBoxPod => 27,
            Self::LaserBeacon => 28,
            Self::TitaniumBillet => 29,
            Self::ArcFurnace => 30,
            Self::AssemblyBench => 31,
            Self::ElectrolysisPlant => 32,
            Self::AirlockChamber => 33,
            Self::PilotSeat => 34,
            Self::PhosphorConsole => 35,
            Self::DockingRing => 36,
            Self::CrewBerth => 37,
            Self::PropulsionBell => 38,
            Self::PropellantTank => 39,
            Self::ThermalFin => 40,
        }
    }

    pub const fn title(self) -> &'static str { RelicLexicon::new(self).title() }
    pub fn parse_token(&self, token: &str) -> Option<Self> { RelicLexicon::new(*self).parse(token) }
    pub const fn gripped(self) -> bool { !matches!(self, Self::EmptyPalm) }
    pub const fn free_palm(self) -> bool { matches!(self, Self::EmptyPalm) }
    pub const fn field_camera(self) -> bool { matches!(self, Self::FieldCamera) }
    pub const fn scalable_module(self) -> bool {
        matches!(self, Self::PropulsionBell | Self::PropellantTank | Self::ThermalFin)
    }
    pub const fn independent_palm_mold(self) -> bool {
        matches!(self, Self::FieldCamera | Self::RocketNozzle | Self::RocketCamera) || self.scalable_module()
    }
    pub const fn wearable(self) -> bool { ToolTemper::new(self).wearable() }
    pub const fn endurance_span(self) -> u16 { ToolTemper::new(self).endurance_span() }
    pub const fn harvest_strain(medium: BlockMedium) -> u16 { ToolTemper::harvest_strain(medium) }
    pub const fn rotatable_in_palm(self) -> bool {
        (self.placeable_block()
            && !matches!(self, Self::VehicleBlueprint | Self::BlackBoxPod | Self::LaserBeacon))
            || self.field_camera()
    }
    pub const fn turnable_in_palm(self) -> bool {
        self.placeable_block() && self.block_medium().mechanical_part()
    }
    pub const fn blocks_perch(self) -> bool { self.placeable_block() && !self.field_camera() }
    pub const fn stowed_block(self) -> bool {
        self.placeable_block()
            && !matches!(
                self,
                Self::CantileverCrane | Self::FieldCamera | Self::VehicleBlueprint | Self::BlackBoxPod
            )
    }
    pub const fn stackable_in_tray(self) -> bool { RelicLexicon::new(self).stackable_in_tray() }
    pub const fn rig_tool(self) -> bool { ToolTemper::new(self).rig_tool() }
    pub const fn metallic_ring(self) -> bool { ToolTemper::new(self).metallic_ring() }
    pub const fn mining_edge(self) -> MiningEdge { ToolTemper::new(self).mining_edge() }
    pub const fn excavation_effort(self) -> u32 { ToolTemper::new(self).excavation_effort() }
    pub const fn timber_cutter(self) -> bool { matches!(self, Self::RigWrench) }
    pub const fn placeable_block(self) -> bool { RelicLexicon::new(self).placeable_block() }
    pub const fn block_medium(self) -> BlockMedium { RelicLexicon::new(self).block_medium() }
    pub const fn from_medium(medium: BlockMedium) -> Self { RelicLexicon::from_medium(medium) }
    pub const fn martial_style(self) -> MartialStyle { ToolTemper::new(self).martial_style() }
    pub const fn figure(self, mold: BlockMold) -> RelicFigure {
        RelicFigure::new(self, mold, ToolCondition::from_mold(mold))
    }
    pub const fn worn_figure(self, mold: BlockMold, condition: ToolCondition) -> RelicFigure {
        RelicFigure::new(self, mold, condition)
    }
    pub const fn surface_find(self) -> SurfaceFind { RelicLexicon::new(self).surface_find() }
    pub const fn from_surface_find(find: SurfaceFind) -> Self { RelicLexicon::from_surface_find(find) }
}
