mod lexicon;
mod mesh;
mod plinth;

use world::{BlockMedium, BlockMold, SurfaceFind};
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
    RocketEngine,
    CabinSeat,
    CarWheel,
    FieldCamera,
    PedalCluster,
    CrossAxle,
    CopperOre,
    IronOre,
    GoldOre,
}

impl HeldRelic {
    #[cfg(test)]
    pub fn next(self) -> Self {
        Self::from_code((self.code() + 1) % 25)
    }

    pub const fn from_code(code: u8) -> Self {
        match code % 25 {
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
            16 => Self::RocketEngine,
            17 => Self::CabinSeat,
            18 => Self::CarWheel,
            19 => Self::FieldCamera,
            20 => Self::PedalCluster,
            21 => Self::CrossAxle,
            22 => Self::CopperOre,
            23 => Self::IronOre,
            _ => Self::GoldOre,
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
            Self::RocketEngine => 16,
            Self::CabinSeat => 17,
            Self::CarWheel => 18,
            Self::FieldCamera => 19,
            Self::PedalCluster => 20,
            Self::CrossAxle => 21,
            Self::CopperOre => 22,
            Self::IronOre => 23,
            Self::GoldOre => 24,
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

    pub const fn rotatable_in_palm(self) -> bool {
        self.placeable_block() || self.field_camera()
    }

    pub const fn turnable_in_palm(self) -> bool {
        self.placeable_block() && self.block_medium().mechanical_part()
    }

    pub const fn blocks_perch(self) -> bool {
        self.placeable_block() && !self.field_camera()
    }

    pub const fn spade_blade(self) -> bool {
        matches!(self, Self::IronSpade)
    }

    pub const fn axe_blade(self) -> bool {
        matches!(self, Self::WoodmanAxe | Self::WoodenMallet)
    }

    pub const fn placeable_block(self) -> bool {
        matches!(
            self,
            Self::ExcavatedEarth
                | Self::QuarriedStone
                | Self::TimberBlock
                | Self::PlankBlock
                | Self::CrustBlock
                | Self::CantileverCrane
                | Self::SteeringWheel
                | Self::AircraftEngine
                | Self::RocketEngine
                | Self::CabinSeat
                | Self::CarWheel
                | Self::FieldCamera
                | Self::PedalCluster
                | Self::CrossAxle
                | Self::CopperOre
                | Self::IronOre
                | Self::GoldOre
        )
    }

    pub const fn block_medium(self) -> BlockMedium {
        match self {
            Self::CrustBlock => BlockMedium::CrustRock,
            Self::QuarriedStone => BlockMedium::QuarriedStone,
            Self::TimberBlock => BlockMedium::RawTimber,
            Self::PlankBlock => BlockMedium::PlanedPlanks,
            Self::CantileverCrane => BlockMedium::CantileverRig,
            Self::SteeringWheel => BlockMedium::SteeringWheel,
            Self::AircraftEngine => BlockMedium::AircraftEngine,
            Self::RocketEngine => BlockMedium::RocketEngine,
            Self::CabinSeat => BlockMedium::CabinSeat,
            Self::CarWheel => BlockMedium::CarWheel,
            Self::FieldCamera => BlockMedium::FieldCamera,
            Self::PedalCluster => BlockMedium::PedalCluster,
            Self::CrossAxle => BlockMedium::CrossAxle,
            Self::CopperOre => BlockMedium::CopperOre,
            Self::IronOre => BlockMedium::IronOre,
            Self::GoldOre => BlockMedium::GoldOre,
            _ => BlockMedium::TurfEarth,
        }
    }

    pub const fn from_medium(medium: BlockMedium) -> Self {
        match medium {
            BlockMedium::TurfEarth | BlockMedium::Sand => Self::ExcavatedEarth,
            BlockMedium::CrustRock => Self::CrustBlock,
            BlockMedium::QuarriedStone => Self::QuarriedStone,
            BlockMedium::RawTimber => Self::TimberBlock,
            BlockMedium::PlanedPlanks => Self::PlankBlock,
            BlockMedium::CantileverRig => Self::CantileverCrane,
            BlockMedium::SteeringWheel => Self::SteeringWheel,
            BlockMedium::AircraftEngine => Self::AircraftEngine,
            BlockMedium::RocketEngine => Self::RocketEngine,
            BlockMedium::CabinSeat => Self::CabinSeat,
            BlockMedium::CarWheel => Self::CarWheel,
            BlockMedium::FieldCamera => Self::FieldCamera,
            BlockMedium::PedalCluster => Self::PedalCluster,
            BlockMedium::CrossAxle => Self::CrossAxle,
            BlockMedium::CopperOre => Self::CopperOre,
            BlockMedium::IronOre => Self::IronOre,
            BlockMedium::GoldOre => Self::GoldOre,
        }
    }

    pub const fn martial_style(self) -> MartialStyle {
        match self {
            Self::EmptyPalm => MartialStyle::Unarmed,
            Self::ExcavatedEarth
            | Self::CrustBlock
            | Self::QuarriedStone
            | Self::TimberBlock
            | Self::PlankBlock
            | Self::CantileverCrane
            | Self::SteeringWheel
            | Self::AircraftEngine
            | Self::RocketEngine
            | Self::CabinSeat
            | Self::CarWheel
            | Self::DeadwoodBough
            | Self::FlintStone
            | Self::FieldCamera
            | Self::PedalCluster
            | Self::CrossAxle
            | Self::CopperOre
            | Self::IronOre
            | Self::GoldOre => MartialStyle::PalmCradle,
            Self::IronSpade => MartialStyle::SpadeDig,
            Self::WoodenMallet | Self::WoodmanAxe => MartialStyle::LumberjackCleave,
            Self::TimberTorch | Self::KnightlySword => MartialStyle::SwordplaySlash,
        }
    }

    pub const fn figure(self, mold: BlockMold) -> RelicFigure {
        RelicFigure::new(self, mold)
    }

    pub const fn surface_find(self) -> SurfaceFind {
        match self {
            Self::EmptyPalm => SurfaceFind::EmptyHanded,
            Self::DeadwoodBough => SurfaceFind::DeadwoodBough,
            Self::FlintStone => SurfaceFind::FlintStone,
            Self::TimberTorch => SurfaceFind::TimberTorch,
            Self::WoodenMallet => SurfaceFind::WoodenMallet,
            Self::WoodmanAxe => SurfaceFind::WoodmanAxe,
            Self::KnightlySword => SurfaceFind::KnightlySword,
            Self::IronSpade => SurfaceFind::IronSpade,
            _ => SurfaceFind::LooseBlock(self.block_medium()),
        }
    }

    pub const fn from_surface_find(find: SurfaceFind) -> Self {
        match find {
            SurfaceFind::EmptyHanded => Self::EmptyPalm,
            SurfaceFind::DeadwoodBough => Self::DeadwoodBough,
            SurfaceFind::FlintStone => Self::FlintStone,
            SurfaceFind::TimberTorch => Self::TimberTorch,
            SurfaceFind::WoodenMallet => Self::WoodenMallet,
            SurfaceFind::WoodmanAxe => Self::WoodmanAxe,
            SurfaceFind::KnightlySword => Self::KnightlySword,
            SurfaceFind::IronSpade => Self::IronSpade,
            SurfaceFind::LooseBlock(medium) => Self::from_medium(medium),
        }
    }
}
