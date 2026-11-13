use crate::BlockMedium;

/// Discrete varieties of forestry yields and ecological harvests.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TimberYield {
    OakTimber,
    AlpineHardwood,
    TropicalHardwood,
    SucculentFiber,
    SteppeDrywood,
    VolcanicBasaltwood,
    Sapling,
    PineCone,
    JungleFruit,
    SucculentFluid,
    Twig,
    Resin,
}

/// Gathered or cast surface item resting loose upon the planetary floor.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SurfaceFind {
    EmptyHanded,
    DeadwoodBough,
    FlintStone,
    TimberTorch,
    WoodenMallet,
    WoodmanAxe,
    KnightlySword,
    IronSpade,
    LooseBlock(BlockMedium),
}

impl SurfaceFind {
    /// Indicates whether a tangible ground resource was gathered or cast.
    pub const fn gathered(self) -> bool {
        !matches!(self, Self::EmptyHanded)
    }

    /// Compact 1-byte wire identifier for world chronicles.
    pub const fn code(self) -> u8 {
        match self {
            Self::EmptyHanded => 0,
            Self::DeadwoodBough => 1,
            Self::FlintStone => 2,
            Self::TimberTorch => 3,
            Self::WoodenMallet => 4,
            Self::WoodmanAxe => 5,
            Self::KnightlySword => 6,
            Self::IronSpade => 7,
            Self::LooseBlock(medium) => 16 + medium.code(),
        }
    }

    /// Reconstructs a `SurfaceFind` from its compact 1-byte chronicle identifier.
    pub const fn from_code(code: u8) -> Self {
        match code {
            1 => Self::DeadwoodBough,
            2 => Self::FlintStone,
            3 => Self::TimberTorch,
            4 => Self::WoodenMallet,
            5 => Self::WoodmanAxe,
            6 => Self::KnightlySword,
            7 => Self::IronSpade,
            16..=33 => Self::LooseBlock(BlockMedium::from_code(code - 16)),
            _ => Self::EmptyHanded,
        }
    }
}

/// Outcome of swinging a felling axe or mallet at standing woodland timber.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CleaveOutcome {
    Missed,
    Notched,
    Felled,
}

impl CleaveOutcome {
    pub const fn struck(self) -> bool {
        !matches!(self, Self::Missed)
    }

    pub const fn felled(self) -> bool {
        matches!(self, Self::Felled)
    }
}
