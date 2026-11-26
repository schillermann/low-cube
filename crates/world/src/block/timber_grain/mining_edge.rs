/// Cutting hardness grade of an excavation hand or pickaxe head.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MiningEdge {
    BarePalm,
    FlintPick,
    CopperPick,
    IronPick,
    TitaniumPick,
}

impl MiningEdge {
    /// Indicates whether this edge cleaves quarried stone, vitrified basalt, copper ore, and iron ore.
    pub const fn pierces_stone(self) -> bool {
        matches!(
            self,
            Self::FlintPick | Self::CopperPick | Self::IronPick | Self::TitaniumPick
        )
    }

    /// Indicates whether this edge cleaves deep planetary crust rock and gold ore.
    pub const fn pierces_crust(self) -> bool {
        matches!(self, Self::IronPick | Self::TitaniumPick)
    }
}
