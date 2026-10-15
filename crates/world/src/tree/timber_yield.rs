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
