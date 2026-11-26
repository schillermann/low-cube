mod demand;
#[cfg(test)]
mod tests;

pub use self::demand::{PrecipitationCue, RainWaveTrace, ShowerDemand};

use std::f32::consts::TAU;
use super::{AcousticWave, MonoSource};

pub(super) const LOOP_PERIOD_SEC: f32 = 3.0;

/// Origin `MonoSource` synthesizing exact-periodic 4-band pink foliage & air wash (`<= 4` fields).
#[derive(Clone, Copy, Debug)]
pub struct CanopyWash {
    seed: u32,
    period_sec: f32,
}

impl CanopyWash {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(seed: u32, period_sec: f32) -> Self {
        Self { seed, period_sec }
    }

    pub const fn velvet(seed: u32) -> Self {
        Self::new(seed, LOOP_PERIOD_SEC)
    }

    fn lattice(self, salt: u32, idx: u32) -> f32 {
        let mut h = self.seed.wrapping_add(salt).wrapping_add(idx.wrapping_mul(0x9E37_79B9));
        h ^= h >> 16;
        h = h.wrapping_mul(0x7FEB_352D);
        h ^= h >> 15;
        h = h.wrapping_mul(0x846C_A68B);
        h ^= h >> 16;
        ((h >> 8) as f32 / 8_388_607.5) - 1.0
    }

    fn band(self, t: f32, cells: u32, salt: u32) -> f32 {
        let phase = (t / self.period_sec).rem_euclid(1.0) * (cells as f32);
        let base = phase.floor();
        let f = phase - base;
        let u = f * f * (3.0 - 2.0 * f);
        let i0 = (base as u32) % cells;
        let i1 = (i0 + 1) % cells;
        let a = self.lattice(salt, i0);
        let b = self.lattice(salt, i1);
        a + (b - a) * u
    }
}

impl MonoSource for CanopyWash {
    fn dry_sample(&self, seconds: f32) -> f32 {
        let low = self.band(seconds, 480, 0x1357_9BDF) * 0.36;
        let mid = self.band(seconds, 1_200, 0x2468_ACE0) * 0.30;
        let leaf = self.band(seconds, 2_880, 0xA511_E9B3) * 0.22;
        let mist = self.band(seconds, 5_760, 0x63D8_3595) * 0.12;
        (low + mid + leaf + mist).clamp(-1.0, 1.0)
    }
}

/// `MonoSource` decorator overlaying discrete Minnaert raindrop impacts on leaves and turf (`<= 4` fields).
#[derive(Clone, Copy, Debug)]
pub struct PatterDrops<S> {
    origin: S,
    drop_slots: u32,
    drop_gain: f32,
    seed: u32,
}

impl<S> PatterDrops<S> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(origin: S, drop_slots: u32, drop_gain: f32, seed: u32) -> Self {
        Self { origin, drop_slots, drop_gain, seed }
    }

    pub const fn foliage(origin: S) -> Self {
        Self::new(origin, 54, 0.34, 0xC2B2_AE35)
    }

    fn slot_hash(&self, slot: u32) -> (f32, f32) {
        let mut h = self.seed.wrapping_add(slot.wrapping_mul(0x85EB_CA6B));
        h ^= h >> 15;
        h = h.wrapping_mul(0x27D4_EB2F);
        let jitter = ((h >> 16) as f32) / 65_535.0;
        let pitch = ((h & 0xFFFF) as f32) / 65_535.0;
        (jitter, pitch)
    }
}

impl<S: MonoSource> MonoSource for PatterDrops<S> {
    fn dry_sample(&self, seconds: f32) -> f32 {
        let base = self.origin.dry_sample(seconds);
        let slots = self.drop_slots.max(1);
        let cell_sec = LOOP_PERIOD_SEC / (slots as f32);
        let wrapped = seconds.rem_euclid(LOOP_PERIOD_SEC);
        let cur_slot = (wrapped / cell_sec).floor() as u32 % slots;
        let prev_slot = (cur_slot + slots - 1) % slots;
        let mut drops = 0.0_f32;
        for slot in [prev_slot, cur_slot] {
            let (jitter, pitch_u) = self.slot_hash(slot);
            let onset = ((slot as f32) + 0.15 + 0.70 * jitter) * cell_sec;
            let dt = (wrapped - onset).rem_euclid(LOOP_PERIOD_SEC);
            if dt < 0.026 {
                let env = (dt * 190.0) * (-dt * 195.0).exp();
                let freq = 680.0 + 1_120.0 * pitch_u + dt * 9_500.0;
                drops += env * (TAU * freq * dt).sin();
            }
        }
        (base + drops * self.drop_gain).tanh()
    }
}

/// `MonoSource` decorator modulating periodic breeze swells across the shower canopy (`<= 4` fields).
#[derive(Clone, Copy, Debug)]
pub struct WindSwell<S> {
    origin: S,
    period_sec: f32,
    depth: f32,
}

impl<S> WindSwell<S> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(origin: S, period_sec: f32, depth: f32) -> Self {
        Self { origin, period_sec, depth }
    }

    pub const fn periodic(origin: S) -> Self {
        Self::new(origin, LOOP_PERIOD_SEC, 0.18)
    }
}

impl<S: MonoSource> MonoSource for WindSwell<S> {
    fn dry_sample(&self, seconds: f32) -> f32 {
        let w = TAU * (seconds / self.period_sec);
        let gust = 1.0 + self.depth * (0.65 * w.sin() + 0.35 * (2.0 * w).cos());
        (self.origin.dry_sample(seconds) * gust).clamp(-1.0, 1.0)
    }
}

/// `MonoSource` & `AcousticWave` decorator applying symmetric FIR canopy/snow/cabin muffling (`<= 4` fields).
#[derive(Clone, Copy, Debug)]
pub struct MuffledCanopy<S> {
    origin: S,
    cutoff_sec: f32,
    gain: f32,
    sample_rate: u32,
}

impl<S> MuffledCanopy<S> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(origin: S, cutoff_sec: f32, gain: f32, sample_rate: u32) -> Self {
        Self { origin, cutoff_sec, gain, sample_rate }
    }

    pub const fn woodland_rain(origin: S) -> Self {
        Self::new(origin, 0.00018, 0.88, 24_000)
    }

    pub const fn polar_snow(origin: S) -> Self {
        Self::new(origin, 0.00085, 0.72, 24_000)
    }
}

impl<S: MonoSource> MonoSource for MuffledCanopy<S> {
    fn dry_sample(&self, seconds: f32) -> f32 {
        let d = self.cutoff_sec;
        let sm = 0.25 * self.origin.dry_sample(seconds - d)
            + 0.50 * self.origin.dry_sample(seconds)
            + 0.25 * self.origin.dry_sample(seconds + d);
        (sm * self.gain).clamp(-1.0, 1.0)
    }
}

impl<S: MonoSource + Send + Sync> AcousticWave for MuffledCanopy<S> {
    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn amplitude_at(&self, seconds: f32) -> f32 {
        self.dry_sample(seconds)
    }

    fn thump(&self) {
        RainWaveTrace::sink().note(PrecipitationCue::WoodlandRain);
        let _ = self.amplitude_at(0.05);
    }
}
