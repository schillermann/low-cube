mod mesh;

use world::{BlockMedium, BlockMold};
pub use super::rhythm::MartialStyle;
pub use mesh::RelicFigure;

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
    CarEngine,
    AircraftEngine,
    RocketEngine,
    CabinSeat,
    CarWheel,
}

impl HeldRelic {
    #[cfg(test)]
    pub fn next(self) -> Self {
        Self::from_code((self.code() + 1) % 19)
    }

    pub const fn from_code(code: u8) -> Self {
        match code % 19 {
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
            14 => Self::CarEngine,
            15 => Self::AircraftEngine,
            16 => Self::RocketEngine,
            17 => Self::CabinSeat,
            _ => Self::CarWheel,
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
            Self::CarEngine => 14,
            Self::AircraftEngine => 15,
            Self::RocketEngine => 16,
            Self::CabinSeat => 17,
            Self::CarWheel => 18,
        }
    }

    pub const fn title(self) -> &'static str {
        match self {
            Self::EmptyPalm => "EMPTY SATCHEL",
            Self::QuarriedStone => "STONE BLOCK",
            Self::TimberTorch => "TIMBER TORCH",
            Self::WoodenMallet => "WOODEN MALLET",
            Self::WoodmanAxe => "WOODMAN AXE",
            Self::KnightlySword => "KNIGHT SWORD",
            Self::IronSpade => "IRON SPADE",
            Self::ExcavatedEarth => "EARTH BLOCK",
            Self::DeadwoodBough => "DEADWOOD",
            Self::FlintStone => "FLINT STONE",
            Self::TimberBlock => "WOOD BLOCK",
            Self::PlankBlock => "WOOD PLANKS",
            Self::CrustBlock => "CRUST BLOCK",
            Self::CantileverCrane => "CRANE LIFT",
            Self::CarEngine => "CAR ENGINE",
            Self::AircraftEngine => "PLANE ENGINE",
            Self::RocketEngine => "ROCKET ENGINE",
            Self::CabinSeat => "CAR SEAT",
            Self::CarWheel => "CAR WHEEL",
        }
    }

    pub const fn gripped(self) -> bool {
        !matches!(self, Self::EmptyPalm)
    }

    pub const fn free_palm(self) -> bool {
        matches!(self, Self::EmptyPalm)
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
                | Self::CarEngine
                | Self::AircraftEngine
                | Self::RocketEngine
                | Self::CabinSeat
                | Self::CarWheel
        )
    }

    pub const fn block_medium(self) -> BlockMedium {
        match self {
            Self::CrustBlock => BlockMedium::CrustRock,
            Self::QuarriedStone => BlockMedium::QuarriedStone,
            Self::TimberBlock => BlockMedium::RawTimber,
            Self::PlankBlock => BlockMedium::PlanedPlanks,
            Self::CantileverCrane => BlockMedium::CantileverRig,
            Self::CarEngine => BlockMedium::CarEngine,
            Self::AircraftEngine => BlockMedium::AircraftEngine,
            Self::RocketEngine => BlockMedium::RocketEngine,
            Self::CabinSeat => BlockMedium::CabinSeat,
            Self::CarWheel => BlockMedium::CarWheel,
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
            BlockMedium::CarEngine => Self::CarEngine,
            BlockMedium::AircraftEngine => Self::AircraftEngine,
            BlockMedium::RocketEngine => Self::RocketEngine,
            BlockMedium::CabinSeat => Self::CabinSeat,
            BlockMedium::CarWheel => Self::CarWheel,
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
            | Self::CarEngine
            | Self::AircraftEngine
            | Self::RocketEngine
            | Self::CabinSeat
            | Self::CarWheel
            | Self::DeadwoodBough
            | Self::FlintStone => MartialStyle::PalmCradle,
            Self::IronSpade => MartialStyle::SpadeDig,
            Self::WoodenMallet | Self::WoodmanAxe => MartialStyle::LumberjackCleave,
            Self::TimberTorch | Self::KnightlySword => MartialStyle::SwordplaySlash,
        }
    }

    pub const fn figure(self, mold: BlockMold) -> RelicFigure {
        RelicFigure::new(self, mold)
    }
}
