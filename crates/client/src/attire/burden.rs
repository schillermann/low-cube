use std::fmt;
use glam::Vec3;
use world::{MeshTarget, Quad};
use super::limb::LimbFrame;
use super::relic::HeldRelic;

/// Interface for any held item, slot stack, or stowed collection contributing physical mass to encumbrance.
pub trait CarryLoad {
    fn fold_burden(&self, acc: SatchelBurden) -> SatchelBurden;
}

/// Decorator combining two `CarryLoad` sources into a single composite burden load.
#[derive(Clone, Copy, Debug)]
pub struct JoinedLoad<A, B> {
    first: A,
    second: B,
}

impl<A, B> JoinedLoad<A, B> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(first: A, second: B) -> Self {
        Self { first, second }
    }
}

impl<A: CarryLoad, B: CarryLoad> CarryLoad for JoinedLoad<A, B> {
    fn fold_burden(&self, acc: SatchelBurden) -> SatchelBurden {
        self.second.fold_burden(self.first.fold_burden(acc))
    }
}

/// Three-tier English weight classification for blocks, tools, and summed satchel stacks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WeightGrade {
    Light,
    Moderate,
    Heavy,
}

impl fmt::Display for WeightGrade {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Light => "LIGHT",
            Self::Moderate => "MODERATE",
            Self::Heavy => "HEAVY",
        })
    }
}

/// Encapsulated slot stack combining a held relic type and stack count to resolve summed weight and backplate triangle color.
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

    pub const fn unit_grade(&self) -> WeightGrade {
        match self.relic {
            HeldRelic::EmptyPalm
            | HeldRelic::DeadwoodBough
            | HeldRelic::TimberTorch
            | HeldRelic::PlankBlock
            | HeldRelic::SteeringWheel
            | HeldRelic::PedalCluster
            | HeldRelic::FieldCamera => WeightGrade::Light,
            HeldRelic::FlintStone
            | HeldRelic::ExcavatedEarth
            | HeldRelic::TimberBlock
            | HeldRelic::WoodenMallet
            | HeldRelic::IronSpade
            | HeldRelic::CabinSeat
            | HeldRelic::CrossAxle
            | HeldRelic::CarWheel => WeightGrade::Moderate,
            HeldRelic::QuarriedStone
            | HeldRelic::CrustBlock
            | HeldRelic::WoodmanAxe
            | HeldRelic::KnightlySword
            | HeldRelic::CantileverCrane
            | HeldRelic::AircraftEngine
            | HeldRelic::RocketEngine => WeightGrade::Heavy,
        }
    }

    const fn unit_mass(&self) -> u32 {
        if self.relic.free_palm() {
            0
        } else {
            match self.unit_grade() {
                WeightGrade::Light => 1,
                WeightGrade::Moderate => 3,
                WeightGrade::Heavy => 6,
            }
        }
    }

    const fn total_mass(&self) -> u32 {
        self.unit_mass().saturating_mul(self.count)
    }

    pub const fn summed_grade(&self) -> WeightGrade {
        match self.total_mass() {
            0..=2 => WeightGrade::Light,
            3..=5 => WeightGrade::Moderate,
            _ => WeightGrade::Heavy,
        }
    }

    pub const fn corner_pigment(&self) -> Vec3 {
        match self.total_mass() {
            0 | 1 => Vec3::new(0.20, 1.42, 0.52),
            2 => Vec3::new(0.68, 1.46, 0.34),
            3 => Vec3::new(1.56, 1.18, 0.20),
            4..=5 => Vec3::new(1.60, 0.76, 0.16),
            6..=11 => Vec3::new(1.58, 0.28, 0.22),
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

/// Total carried mass across hands and stowed satchel compartments governing movement pace and jump altitude.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SatchelBurden {
    mass: u32,
}

impl SatchelBurden {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(mass: u32) -> Self {
        Self { mass }
    }

    pub const fn weightless() -> Self {
        Self::new(0)
    }

    const fn added_mass(self, extra: u32) -> Self {
        Self::new(self.mass.saturating_add(extra))
    }

    pub fn scale_pace(self, unencumbered_pace: f32) -> f32 {
        if self.mass == 0 {
            unencumbered_pace
        } else {
            unencumbered_pace * (0.34 + 0.66 / (1.0 + (self.mass as f32) * 0.026))
        }
    }

    pub fn scale_leap(self, unencumbered_leap: f32) -> f32 {
        if self.mass == 0 {
            unencumbered_leap
        } else {
            unencumbered_leap * (0.48 + 0.52 / (1.0 + (self.mass as f32) * 0.024))
        }
    }
}
