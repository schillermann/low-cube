mod slot;

use glam::Vec3;

pub use self::slot::SlotBurden;

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

    pub(super) const fn added_mass(self, extra: u32) -> Self {
        Self::new(self.mass.saturating_add(extra))
    }

    /// Physical spacecraft ballast tonnage (`a = F / m`) contributed by carried satchel payload.
    pub fn ballast_tonnage(self) -> f32 {
        (self.mass as f32) * 0.16
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
