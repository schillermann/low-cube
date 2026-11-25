use std::fmt;
use glam::Vec3;
use world::{MeshTarget, Quad};
use super::super::limb::LimbFrame;
use super::super::relic::HeldRelic;
use super::{CarryLoad, SatchelBurden};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum WeightGrade {
    Light,
    Moderate,
    Weighty,
    Heavy,
}

impl WeightGrade {
    const fn tier_rank(self) -> u32 {
        match self {
            Self::Light => 0,
            Self::Moderate => 1,
            Self::Weighty => 2,
            Self::Heavy => 3,
        }
    }

    const fn escalated(self, steps: u32) -> Self {
        match self.tier_rank().saturating_add(steps) {
            0 => Self::Light,
            1 => Self::Moderate,
            2 => Self::Weighty,
            _ => Self::Heavy,
        }
    }
}

impl fmt::Display for WeightGrade {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Light => "LIGHT",
            Self::Moderate => "MODERATE",
            Self::Weighty => "WEIGHTY",
            Self::Heavy => "HEAVY",
        })
    }
}

/// Encapsulated slot stack escalating one weight grade per 10 stacked items (`1..=9`, `10..=19`, `20..=29`, `30+`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SlotBurden {
    relic: HeldRelic,
    count: u32,
}

impl SlotBurden {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(relic: HeldRelic, count: u32) -> Self {
        Self { relic, count }
    }

    const fn unit_grade(&self) -> WeightGrade {
        match self.relic {
            HeldRelic::EmptyPalm
            | HeldRelic::DeadwoodBough
            | HeldRelic::TimberTorch
            | HeldRelic::PlankBlock
            | HeldRelic::SteeringWheel
            | HeldRelic::PedalCluster
            | HeldRelic::FieldCamera
            | HeldRelic::VehicleBlueprint
            | HeldRelic::TitaniumHull
            | HeldRelic::CarbonSilicate => WeightGrade::Light,
            HeldRelic::FlintStone
            | HeldRelic::FlintPickaxe
            | HeldRelic::ExcavatedEarth
            | HeldRelic::TimberBlock
            | HeldRelic::WoodenMallet
            | HeldRelic::CabinSeat
            | HeldRelic::RocketCamera
            | HeldRelic::BlackBoxPod
            | HeldRelic::BoarMeat => WeightGrade::Moderate,
            HeldRelic::QuarriedStone
            | HeldRelic::IronPickaxe
            | HeldRelic::VitrifiedBasalt
            | HeldRelic::IronSpade
            | HeldRelic::WoodmanAxe
            | HeldRelic::KnightlySword
            | HeldRelic::CarWheel
            | HeldRelic::CopperOre
            | HeldRelic::IronOre => WeightGrade::Weighty,
            HeldRelic::CrustBlock
            | HeldRelic::GoldOre
            | HeldRelic::CantileverCrane
            | HeldRelic::AircraftEngine
            | HeldRelic::FlightConsole
            | HeldRelic::FusionTorch
            | HeldRelic::SpawnStone
            | HeldRelic::RocketNozzle => WeightGrade::Heavy,
        }
    }

    const fn unit_mass(&self) -> u32 {
        if self.relic.free_palm() {
            0
        } else {
            match self.unit_grade() {
                WeightGrade::Light => 1,
                WeightGrade::Moderate => 2,
                WeightGrade::Weighty => 4,
                WeightGrade::Heavy => 8,
            }
        }
    }

    const fn total_mass(&self) -> u32 {
        self.unit_mass().saturating_mul(self.count)
    }

    const fn escalated_rank(&self) -> u32 {
        if self.relic.free_palm() || self.count == 0 {
            0
        } else {
            self.unit_grade().tier_rank().saturating_add(self.count / 10)
        }
    }

    const fn summed_grade(&self) -> WeightGrade {
        if self.relic.free_palm() || self.count == 0 {
            WeightGrade::Light
        } else {
            self.unit_grade().escalated(self.count / 10)
        }
    }

    const fn corner_pigment(&self) -> Vec3 {
        match self.escalated_rank() {
            0 => Vec3::new(0.20, 1.42, 0.52),
            1 => Vec3::new(1.56, 1.18, 0.20),
            2 => Vec3::new(1.62, 0.72, 0.16),
            3 => Vec3::new(1.58, 0.28, 0.22),
            _ => Vec3::new(1.26, 0.12, 0.36),
        }
    }

    pub fn emit_corner_triangle(&self, slot_frame: &LimbFrame, target: &mut dyn MeshTarget) {
        if self.relic.free_palm() || self.count == 0 {
            return;
        }
        let (x_left, y_top, span, z_fill) = (-0.043, 0.043, 0.025, 0.0195);
        let normal = (slot_frame.point(0.0, 1.0, -1.0) - slot_frame.point(0.0, 0.0, 0.0)).normalize_or_zero();
        let top_left = slot_frame.point(x_left, y_top, z_fill);
        let top_right = slot_frame.point(x_left + span, y_top, z_fill);
        let bottom_left = slot_frame.point(x_left, y_top - span, z_fill);
        target.push_quad(Quad::new(
            bottom_left,
            top_right,
            top_left,
            top_left,
            normal,
            self.corner_pigment(),
        ));
    }
}

impl CarryLoad for SlotBurden {
    fn fold_burden(&self, acc: SatchelBurden) -> SatchelBurden {
        acc.added_mass(self.total_mass())
    }
}

impl CarryLoad for HeldRelic {
    fn fold_burden(&self, acc: SatchelBurden) -> SatchelBurden {
        SlotBurden::new(*self, u32::from(self.gripped())).fold_burden(acc)
    }
}

impl fmt::Display for SlotBurden {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.relic.free_palm() || self.count == 0 {
            return f.write_str("EMPTY SLOT");
        }
        let (unit, summed) = (self.unit_grade(), self.summed_grade());
        if unit == summed {
            write!(f, "{} X{} - {}", self.relic.title(), self.count, unit)
        } else {
            write!(f, "{} X{} - {} -> {}", self.relic.title(), self.count, unit, summed)
        }
    }
}
