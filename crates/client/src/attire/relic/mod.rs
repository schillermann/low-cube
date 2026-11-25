mod lexicon;
mod mesh;
mod plinth;
mod tokens;
mod tools;

use world::{BlockMedium, BlockMold, MiningEdge, SurfaceFind};
pub use super::rhythm::MartialStyle;
pub use mesh::RelicFigure;
use lexicon::RelicLexicon;

/// Object held in the avatar's right or left hand visible in first-person and multiplayer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HeldRelic {
    EmptyPalm,
    QuarriedStone,
    TimberTorch,
    WoodenMallet,
    WoodmanAxe,
    KnightlySword,
    IronSpade,
    ExcavatedEarth,
    DeadwoodBough,
    FlintStone,
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
    BoarMeat,
    RocketNozzle,
    RocketCamera,
    VehicleBlueprint,
    FusionTorch,
    SpawnStone,
    FlintPickaxe,
    IronPickaxe,
    TitaniumHull,
    CarbonSilicate,
    VitrifiedBasalt,
    BlackBoxPod,
}

impl HeldRelic {
    pub const COUNT: usize = 36;
    pub const TRAY_ORDER: [Self; Self::COUNT - 1] = RelicLexicon::TRAY_ORDER;

    #[cfg(test)]
    pub fn next(self) -> Self {
        Self::from_code((self.code() + 1) % (Self::COUNT as u8))
    }

    pub const fn from_code(code: u8) -> Self {
        match code % (Self::COUNT as u8) {
            0 => Self::EmptyPalm,
            1 => Self::QuarriedStone,
            2 => Self::TimberTorch,
            3 => Self::WoodenMallet,
            4 => Self::WoodmanAxe,
            5 => Self::KnightlySword,
            6 => Self::IronSpade,
            7 => Self::ExcavatedEarth,
            8 => Self::DeadwoodBough,
            9 => Self::FlintStone,
            10 => Self::TimberBlock,
            11 => Self::PlankBlock,
            12 => Self::CrustBlock,
            13 => Self::CantileverCrane,
            14 => Self::SteeringWheel,
            15 => Self::AircraftEngine,
            16 => Self::FlightConsole,
            17 => Self::CabinSeat,
            18 => Self::CarWheel,
            19 => Self::FieldCamera,
            20 => Self::PedalCluster,
            21 => Self::CopperOre,
            22 => Self::IronOre,
            23 => Self::GoldOre,
            24 => Self::BoarMeat,
            25 => Self::RocketNozzle,
            26 => Self::RocketCamera,
            27 => Self::VehicleBlueprint,
            28 => Self::FusionTorch,
            29 => Self::SpawnStone,
            30 => Self::FlintPickaxe,
            31 => Self::IronPickaxe,
            32 => Self::TitaniumHull,
            33 => Self::CarbonSilicate,
            34 => Self::VitrifiedBasalt,
            _ => Self::BlackBoxPod,
        }
    }

    pub const fn code(self) -> u8 {
        match self {
            Self::EmptyPalm => 0,
            Self::QuarriedStone => 1,
            Self::TimberTorch => 2,
            Self::WoodenMallet => 3,
            Self::WoodmanAxe => 4,
            Self::KnightlySword => 5,
            Self::IronSpade => 6,
            Self::ExcavatedEarth => 7,
            Self::DeadwoodBough => 8,
            Self::FlintStone => 9,
            Self::TimberBlock => 10,
            Self::PlankBlock => 11,
            Self::CrustBlock => 12,
            Self::CantileverCrane => 13,
            Self::SteeringWheel => 14,
            Self::AircraftEngine => 15,
            Self::FlightConsole => 16,
            Self::CabinSeat => 17,
            Self::CarWheel => 18,
            Self::FieldCamera => 19,
            Self::PedalCluster => 20,
            Self::CopperOre => 21,
            Self::IronOre => 22,
            Self::GoldOre => 23,
            Self::BoarMeat => 24,
            Self::RocketNozzle => 25,
            Self::RocketCamera => 26,
            Self::VehicleBlueprint => 27,
            Self::FusionTorch => 28,
            Self::SpawnStone => 29,
            Self::FlintPickaxe => 30,
            Self::IronPickaxe => 31,
            Self::TitaniumHull => 32,
            Self::CarbonSilicate => 33,
            Self::VitrifiedBasalt => 34,
            Self::BlackBoxPod => 35,
        }
    }

    pub const fn title(self) -> &'static str {
        RelicLexicon::new(self).title()
    }

    pub fn parse_token(&self, token: &str) -> Option<Self> {
        RelicLexicon::new(*self).parse(token)
    }

    pub const fn gripped(self) -> bool {
        !matches!(self, Self::EmptyPalm)
    }

    pub const fn free_palm(self) -> bool {
        matches!(self, Self::EmptyPalm)
    }

    pub const fn field_camera(self) -> bool {
        matches!(self, Self::FieldCamera)
    }

    pub const fn independent_palm_mold(self) -> bool {
        matches!(self, Self::FieldCamera | Self::RocketNozzle | Self::RocketCamera)
    }

    pub const fn edible_provision(self) -> bool {
        matches!(self, Self::BoarMeat)
    }

    pub const fn hunting_power(self) -> u8 {
        match self {
            Self::KnightlySword => 4,
            Self::WoodmanAxe | Self::IronPickaxe => 3,
            Self::IronSpade | Self::WoodenMallet | Self::FlintPickaxe => 2,
            _ => 1,
        }
    }

    pub const fn hunting_reach(self) -> f32 {
        match self {
            Self::KnightlySword => 2.55,
            Self::WoodmanAxe | Self::IronSpade | Self::WoodenMallet | Self::FlintPickaxe | Self::IronPickaxe => 2.25,
            Self::TimberTorch | Self::DeadwoodBough => 2.00,
            _ => 1.75,
        }
    }

    pub const fn rotatable_in_palm(self) -> bool {
        (self.placeable_block() && !matches!(self, Self::VehicleBlueprint | Self::BlackBoxPod))
            || self.field_camera()
    }

    pub const fn turnable_in_palm(self) -> bool {
        self.placeable_block() && self.block_medium().mechanical_part()
    }

    pub const fn blocks_perch(self) -> bool {
        self.placeable_block() && !self.field_camera()
    }

    pub const fn stowed_block(self) -> bool {
        self.placeable_block()
            && !matches!(
                self,
                Self::CantileverCrane | Self::FieldCamera | Self::VehicleBlueprint | Self::BlackBoxPod
            )
    }

    pub const fn stackable_in_tray(self) -> bool {
        RelicLexicon::new(self).stackable_in_tray()
    }

    pub const fn spade_blade(self) -> bool {
        matches!(self, Self::IronSpade)
    }

    pub const fn pickaxe_head(self) -> bool {
        matches!(self, Self::FlintPickaxe | Self::IronPickaxe)
    }

    pub const fn mining_edge(self) -> MiningEdge {
        match self {
            Self::IronPickaxe => MiningEdge::IronPick,
            Self::FlintPickaxe => MiningEdge::FlintPick,
            _ => MiningEdge::BarePalm,
        }
    }

    pub const fn excavation_effort(self) -> u32 {
        match self {
            Self::IronSpade | Self::IronPickaxe => 4,
            Self::FlintPickaxe => 3,
            _ => 1,
        }
    }

    pub const fn axe_blade(self) -> bool {
        matches!(self, Self::WoodmanAxe | Self::WoodenMallet)
    }

    pub const fn placeable_block(self) -> bool {
        RelicLexicon::new(self).placeable_block()
    }

    pub const fn block_medium(self) -> BlockMedium {
        RelicLexicon::new(self).block_medium()
    }

    pub const fn from_medium(medium: BlockMedium) -> Self {
        RelicLexicon::from_medium(medium)
    }

    pub const fn martial_style(self) -> MartialStyle {
        RelicLexicon::new(self).martial_style()
    }

    pub const fn figure(self, mold: BlockMold) -> RelicFigure {
        RelicFigure::new(self, mold)
    }

    pub const fn surface_find(self) -> SurfaceFind {
        RelicLexicon::new(self).surface_find()
    }

    pub const fn from_surface_find(find: SurfaceFind) -> Self {
        RelicLexicon::from_surface_find(find)
    }
}
