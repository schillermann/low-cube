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

/// Gathered or cast Space-Age item resting loose upon the planetary floor.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SurfaceFind {
    EmptyHanded,
    RigWrench,
    SuitRadio,
    TitaniumBillet,
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
            Self::RigWrench => 1,
            Self::SuitRadio => 2,
            Self::TitaniumBillet => 3,
            Self::LooseBlock(medium) => 16 + medium.code(),
        }
    }

    /// Reconstructs a `SurfaceFind` from its compact 1-byte chronicle identifier.
    pub const fn from_code(code: u8) -> Self {
        match code {
            1 => Self::RigWrench,
            2 => Self::SuitRadio,
            3 => Self::TitaniumBillet,
            16..=43 => Self::LooseBlock(BlockMedium::from_code(code - 16)),
            _ => Self::EmptyHanded,
        }
    }
}

/// Outcome of cleaving standing woodland timber with an EVA rig tool.
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
