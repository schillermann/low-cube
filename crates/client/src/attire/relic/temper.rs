use glam::Vec3;
use world::{BlockMedium, BlockMold, CardinalTurn, MiningEdge, MoldArchetype};
use super::{HeldRelic, MartialStyle};

/// Physical surface finish state of a handheld EVA tool sculpted onto its 3D chassis.
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

/// Material grade, crust-piercing torque, and excavation speed catalog for `HeldRelic` (`1` field).
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
        let _ = self;
        0
    }

    pub(super) const fn wearable(self) -> bool {
        self.endurance_span() > 0
    }

    pub(super) const fn rig_tool(self) -> bool {
        matches!(self.relic, HeldRelic::RigWrench)
    }

    pub(super) const fn metallic_ring(self) -> bool {
        matches!(self.relic, HeldRelic::RigWrench)
    }

    pub(super) const fn mining_edge(self) -> MiningEdge {
        match self.relic {
            HeldRelic::RigWrench => MiningEdge::TitaniumPick,
            _ => MiningEdge::BarePalm,
        }
    }

    pub(super) const fn excavation_effort(self) -> u32 {
        match self.relic {
            HeldRelic::RigWrench => 4,
            _ => 1,
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
            HeldRelic::RigWrench => MartialStyle::RigTorque,
            HeldRelic::SuitRadio => MartialStyle::RadioSignal,
            _ => MartialStyle::PalmCradle,
        }
    }
}
