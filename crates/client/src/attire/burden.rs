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

/// Four-tier English weight classification for blocks, tools, and summed satchel stacks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WeightGrade {
    Light,
    Moderate,
    Weighty,
    Heavy,
}

impl WeightGrade {
    pub const fn tier_rank(self) -> u32 {
        match self {
            Self::Light => 0,
            Self::Moderate => 1,
            Self::Weighty => 2,
            Self::Heavy => 3,
        }
    }

    pub const fn escalated(self, steps: u32) -> Self {
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
            | HeldRelic::CabinSeat => WeightGrade::Moderate,
            HeldRelic::QuarriedStone
            | HeldRelic::IronSpade
            | HeldRelic::WoodmanAxe
            | HeldRelic::KnightlySword
            | HeldRelic::CrossAxle
            | HeldRelic::CarWheel => WeightGrade::Weighty,
            HeldRelic::CrustBlock
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

    pub const fn summed_grade(&self) -> WeightGrade {
        if self.relic.free_palm() || self.count == 0 {
            WeightGrade::Light
        } else {
            self.unit_grade().escalated(self.count / 10)
        }
    }

    pub const fn corner_pigment(&self) -> Vec3 {
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

/// Total carried mass across hands and stowed satchel compartments governing movement pace, jump height, and hanging sack size.
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

    pub fn swell(self, empty_val: f32, full_delta: f32) -> f32 {
        if self.mass == 0 {
            empty_val
        } else {
            let m = self.mass as f32;
            let ratio = (0.08 + 0.92 * (m / (m + 36.0))).clamp(0.0, 1.0);
            empty_val + full_delta * ratio
        }
    }

    pub const fn pigment(self) -> Vec3 {
        match self.mass {
            0 => Vec3::new(0.46, 0.34, 0.20),
            1..=19 => Vec3::new(0.24, 1.32, 0.50),
            20..=39 => Vec3::new(1.48, 1.12, 0.22),
            40..=79 => Vec3::new(1.56, 0.68, 0.16),
            _ => Vec3::new(1.52, 0.28, 0.22),
        }
    }
}
