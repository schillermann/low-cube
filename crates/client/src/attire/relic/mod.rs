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
}

impl HeldRelic {
    #[cfg(test)]
    pub fn next(self) -> Self {
        Self::from_code((self.code() + 1) % 13)
    }

    pub const fn from_code(code: u8) -> Self {
        match code % 13 {
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
            _ => Self::CrustBlock,
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
            Self::ExcavatedEarth | Self::QuarriedStone | Self::TimberBlock | Self::PlankBlock | Self::CrustBlock
        )
    }

    pub const fn block_medium(self) -> BlockMedium {
        match self {
            Self::CrustBlock => BlockMedium::CrustRock,
            Self::QuarriedStone => BlockMedium::QuarriedStone,
            Self::TimberBlock => BlockMedium::RawTimber,
            Self::PlankBlock => BlockMedium::PlanedPlanks,
            _ => BlockMedium::TurfEarth,
        }
    }

    pub const fn from_medium(medium: BlockMedium) -> Self {
        match medium {
            BlockMedium::TurfEarth => Self::ExcavatedEarth,
            BlockMedium::CrustRock => Self::CrustBlock,
            BlockMedium::QuarriedStone => Self::QuarriedStone,
            BlockMedium::RawTimber => Self::TimberBlock,
            BlockMedium::PlanedPlanks => Self::PlankBlock,
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
