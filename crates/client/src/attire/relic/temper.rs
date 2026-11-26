use glam::Vec3;
use world::{BlockMedium, BlockMold, CardinalTurn, MiningEdge, MoldArchetype};
use super::{HeldRelic, MartialStyle};

/// Visible physical wear state of a handheld tool sculpted directly onto its 3D blade, head, and haft.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ToolCondition {
    Pristine,
    Scuffed,
    Chipped,
    Splintered,
}

impl ToolCondition {
    pub const fn stage(self) -> u8 {
        match self {
            Self::Pristine => 0,
            Self::Scuffed => 1,
            Self::Chipped => 2,
            Self::Splintered => 3,
        }
    }

    pub const fn from_stage(stage: u8) -> Self {
        match stage % 4 {
            0 => Self::Pristine,
            1 => Self::Scuffed,
            2 => Self::Chipped,
            _ => Self::Splintered,
        }
    }

    pub const fn from_wear(wear: u16, endurance: u16) -> Self {
        if endurance == 0 || wear == 0 {
            return Self::Pristine;
        }
        let rem = endurance.saturating_sub(wear) as u32;
        let span = endurance as u32;
        if rem * 4 > span * 3 {
            Self::Pristine
        } else if rem * 20 > span * 9 {
            Self::Scuffed
        } else if rem * 5 > span {
            Self::Chipped
        } else {
            Self::Splintered
        }
    }

    pub const fn into_mold(self) -> BlockMold {
        BlockMold::new(MoldArchetype::FullCube, CardinalTurn::from_code(self.stage()))
    }

    pub const fn from_mold(mold: BlockMold) -> Self {
        Self::from_stage(mold.chronicle_bytes()[1])
    }

    pub const fn legend_prefix(self) -> &'static str {
        match self {
            Self::Pristine => "",
            Self::Scuffed => "SCUFFED ",
            Self::Chipped => "CHIPPED ",
            Self::Splintered => "SPLINTERED ",
        }
    }

    pub fn weather_metal(self, fresh: Vec3) -> Vec3 {
        let grime = Vec3::new(0.24, 0.21, 0.19);
        match self {
            Self::Pristine => fresh,
            Self::Scuffed => fresh.lerp(grime, 0.22),
            Self::Chipped => fresh.lerp(grime, 0.42),
            Self::Splintered => fresh.lerp(grime, 0.62),
        }
    }
}

/// Material hardness, excavation speed, combat reach, and endurance catalog for a `HeldRelic` (`1` field).
#[derive(Clone, Copy, Debug)]
pub(super) struct ToolTemper {
    relic: HeldRelic,
}

impl ToolTemper {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub(super) const fn new(relic: HeldRelic) -> Self {
        Self { relic }
    }

    pub(super) const fn endurance_span(self) -> u16 {
        match self.relic {
            HeldRelic::WoodenSpade | HeldRelic::FlintPickaxe => 24,
            HeldRelic::WoodenMallet => 36,
            HeldRelic::CopperSpade | HeldRelic::CopperPickaxe => 64,
            HeldRelic::IronSpade
            | HeldRelic::IronPickaxe
            | HeldRelic::WoodmanAxe
            | HeldRelic::KnightlySword => 160,
            HeldRelic::TitaniumSpade | HeldRelic::TitaniumPickaxe => 420,
            _ => 0,
        }
    }

    pub(super) const fn wearable(self) -> bool {
        self.endurance_span() > 0
    }

    pub(super) const fn spade_blade(self) -> bool {
        matches!(
            self.relic,
            HeldRelic::WoodenSpade
                | HeldRelic::CopperSpade
                | HeldRelic::IronSpade
                | HeldRelic::TitaniumSpade
        )
    }

    pub(super) const fn pickaxe_head(self) -> bool {
        matches!(
            self.relic,
            HeldRelic::FlintPickaxe
                | HeldRelic::CopperPickaxe
                | HeldRelic::IronPickaxe
                | HeldRelic::TitaniumPickaxe
        )
    }

    pub(super) const fn metallic_ring(self) -> bool {
        matches!(
            self.relic,
            HeldRelic::CopperSpade
                | HeldRelic::IronSpade
                | HeldRelic::TitaniumSpade
                | HeldRelic::FlintPickaxe
                | HeldRelic::CopperPickaxe
                | HeldRelic::IronPickaxe
                | HeldRelic::TitaniumPickaxe
        )
    }

    pub(super) const fn mining_edge(self) -> MiningEdge {
        match self.relic {
            HeldRelic::TitaniumPickaxe => MiningEdge::TitaniumPick,
            HeldRelic::IronPickaxe => MiningEdge::IronPick,
            HeldRelic::CopperPickaxe => MiningEdge::CopperPick,
            HeldRelic::FlintPickaxe => MiningEdge::FlintPick,
            _ => MiningEdge::BarePalm,
        }
    }

    pub(super) const fn excavation_effort(self) -> u32 {
        match self.relic {
            HeldRelic::TitaniumSpade | HeldRelic::TitaniumPickaxe => 8,
            HeldRelic::IronSpade | HeldRelic::IronPickaxe => 4,
            HeldRelic::CopperSpade | HeldRelic::CopperPickaxe | HeldRelic::FlintPickaxe => 3,
            HeldRelic::WoodenSpade => 2,
            _ => 1,
        }
    }

    pub(super) const fn hunting_power(self) -> u8 {
        match self.relic {
            HeldRelic::KnightlySword | HeldRelic::TitaniumPickaxe => 4,
            HeldRelic::WoodmanAxe | HeldRelic::IronPickaxe | HeldRelic::TitaniumSpade => 3,
            HeldRelic::WoodenSpade
            | HeldRelic::CopperSpade
            | HeldRelic::IronSpade
            | HeldRelic::WoodenMallet
            | HeldRelic::FlintPickaxe
            | HeldRelic::CopperPickaxe => 2,
            _ => 1,
        }
    }

    pub(super) const fn hunting_reach(self) -> f32 {
        match self.relic {
            HeldRelic::KnightlySword => 2.55,
            HeldRelic::WoodmanAxe
            | HeldRelic::WoodenSpade
            | HeldRelic::CopperSpade
            | HeldRelic::IronSpade
            | HeldRelic::TitaniumSpade
            | HeldRelic::WoodenMallet
            | HeldRelic::FlintPickaxe
            | HeldRelic::CopperPickaxe
            | HeldRelic::IronPickaxe
            | HeldRelic::TitaniumPickaxe => 2.25,
            HeldRelic::TimberTorch | HeldRelic::DeadwoodBough => 2.00,
            _ => 1.75,
        }
    }

    pub(super) const fn harvest_strain(medium: BlockMedium) -> u16 {
        match medium {
            BlockMedium::CrustRock
            | BlockMedium::GoldOre
            | BlockMedium::VitrifiedBasalt
            | BlockMedium::TitaniumHull => 3,
            BlockMedium::QuarriedStone
            | BlockMedium::CopperOre
            | BlockMedium::IronOre
            | BlockMedium::CarbonSilicate => 2,
            _ => 1,
        }
    }

    pub(super) const fn martial_style(self) -> MartialStyle {
        match self.relic {
            HeldRelic::EmptyPalm => MartialStyle::Unarmed,
            HeldRelic::WoodenSpade
            | HeldRelic::CopperSpade
            | HeldRelic::IronSpade
            | HeldRelic::TitaniumSpade => MartialStyle::SpadeDig,
            HeldRelic::WoodenMallet
            | HeldRelic::WoodmanAxe
            | HeldRelic::FlintPickaxe
            | HeldRelic::CopperPickaxe
            | HeldRelic::IronPickaxe
            | HeldRelic::TitaniumPickaxe => MartialStyle::LumberjackCleave,
            HeldRelic::TimberTorch | HeldRelic::KnightlySword => MartialStyle::SwordplaySlash,
            _ => MartialStyle::PalmCradle,
        }
    }
}
