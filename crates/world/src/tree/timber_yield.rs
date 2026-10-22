/// Discrete varieties of forestry yields and ecological harvests.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TimberYield {
    StandardLumber,
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

/// Gathered surface item discovered when foraging the forest floor.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SurfaceFind {
    EmptyHanded,
    DeadwoodBough,
    FlintStone,
}

impl SurfaceFind {
    /// Indicates whether a tangible ground resource was gathered.
    pub const fn gathered(self) -> bool {
        !matches!(self, Self::EmptyHanded)
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
