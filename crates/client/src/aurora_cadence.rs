/// Abstraction for aurora occurrence schedule and nocturnal activity factor.
pub trait AuroraCadence: Send + Sync {
    /// Computes the aurora activity factor (0.0 = inactive, 1.0 = peak activity).
    fn activity(&self, progress: f32, elapsed_seconds: f64, night_factor: f32) -> f32;
    fn clone_cadence(&self) -> Box<dyn AuroraCadence>;
}

/// Inactive aurora cadence keeping the aurora permanently disabled.
pub struct DormantAuroraCadence;

impl DormantAuroraCadence {
    /// Primary constructor doing nothing except creating instance.
    pub fn new() -> Self {
        Self
    }
}

impl AuroraCadence for DormantAuroraCadence {
    fn activity(&self, _progress: f32, _elapsed_seconds: f64, _night_factor: f32) -> f32 {
        0.0
    }

    fn clone_cadence(&self) -> Box<dyn AuroraCadence> {
        Box::new(Self::new())
    }
}

/// Nightly aurora cadence active every single night throughout nocturnal hours.
pub struct NightlyAuroraCadence;

impl NightlyAuroraCadence {
    /// Primary constructor doing nothing except creating instance.
    pub fn new() -> Self {
        Self
    }
}

impl AuroraCadence for NightlyAuroraCadence {
    fn activity(&self, _progress: f32, _elapsed_seconds: f64, night_factor: f32) -> f32 {
        night_factor
    }

    fn clone_cadence(&self) -> Box<dyn AuroraCadence> {
        Box::new(Self::new())
    }
}

struct StochasticHash {
    seed: u64,
}

impl StochasticHash {
    fn new(seed: u64) -> Self {
        Self { seed }
    }

    fn sample(&self, salt: u64) -> f32 {
        let mut x = self.seed.wrapping_add(salt);
        x = (x ^ (x >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
        x = (x ^ (x >> 27)).wrapping_mul(0x94d049bb133111eb);
        let h = x ^ (x >> 31);
        ((h >> 40) as f32) / 16777216.0
    }
}

/// Stochastic aurora cadence decorator triggering randomly during nocturnal hours.
pub struct StochasticAuroraCadence {
    chance: f32,
    day_duration: f32,
    origin: Box<dyn AuroraCadence>,
}

impl StochasticAuroraCadence {
    pub const NOCTURNAL_START: f32 = 0.55;
    pub const NOCTURNAL_END: f32 = 0.95;

    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(chance: f32, day_duration: f32, origin: Box<dyn AuroraCadence>) -> Self {
        Self {
            chance,
            day_duration,
            origin,
        }
    }

    /// Secondary constructor for rare aurora occurrence (~20% chance).
    pub fn rare(day_duration: f32, origin: Box<dyn AuroraCadence>) -> Self {
        Self::new(0.20, day_duration, origin)
    }

    /// Secondary constructor for normal aurora occurrence (~50% chance).
    pub fn normal(day_duration: f32, origin: Box<dyn AuroraCadence>) -> Self {
        Self::new(0.50, day_duration, origin)
    }

    /// Secondary constructor for frequent aurora occurrence (~80% chance).
    pub fn frequent(day_duration: f32, origin: Box<dyn AuroraCadence>) -> Self {
        Self::new(0.80, day_duration, origin)
    }
}

impl AuroraCadence for StochasticAuroraCadence {
    fn activity(&self, progress: f32, elapsed_seconds: f64, night_factor: f32) -> f32 {
        let base = self.origin.activity(progress, elapsed_seconds, night_factor);
        if base <= 0.001 || self.chance <= 0.0 {
            return 0.0;
        }

        let dur = (self.day_duration as f64).max(1.0);
        let night_index = (elapsed_seconds / dur).floor() as u64;

        let hash = StochasticHash::new(night_index);
        let roll = hash.sample(0x1a2b3c4d);
        if roll >= self.chance {
            return 0.0;
        }

        let from_progress = Self::NOCTURNAL_START;
        let to_progress = Self::NOCTURNAL_END;
        let span = to_progress - from_progress;
        let offset = hash.sample(0x5e6f7a8b) * (span * 0.40);
        let start_t = from_progress + offset;
        let remaining = (to_progress - start_t).max(0.04);
        let active_span = 0.04 + hash.sample(0x9c0d1e2f) * (remaining - 0.04);
        let end_t = (start_t + active_span).min(to_progress);

        if progress < start_t || progress > end_t {
            return 0.0;
        }

        let fade_in = ((progress - start_t) / 0.02).clamp(0.0, 1.0);
        let fade_out = ((end_t - progress) / 0.02).clamp(0.0, 1.0);
        fade_in * fade_out * base
    }

    fn clone_cadence(&self) -> Box<dyn AuroraCadence> {
        Box::new(Self::new(self.chance, self.day_duration, self.origin.clone_cadence()))
    }
}
