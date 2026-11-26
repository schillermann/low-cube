use world::{SolarProportions, WorldSeed};
use crate::cosmos_scale::{
    BeltDensity, CosmosScale, EpsteinReachCosmos, FileCosmos, MeasuredCosmos, SeededCosmos,
};
use crate::server_scale::ScaledCube;

const SCALE_TIERS: [u32; 6] = [1, 2, 4, 8, 16, 32];

/// Immutable planetary scale tier (`1X = 128m` up to `32X = 4096m`) composing `CosmosScale` decorators (`1` field).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ScaleTier {
    multiple: u32,
}

impl ScaleTier {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(multiple: u32) -> Self {
        Self { multiple }
    }

    pub const fn moonlet() -> Self {
        Self::new(1)
    }

    pub fn clamped(multiple: u32) -> Self {
        Self::new(multiple.clamp(1, SolarProportions::MAX_WORLD_SCALE_FACTOR))
    }

    pub fn next(self) -> Self {
        let pos = SCALE_TIERS.iter().position(|&m| m == self.multiple).unwrap_or(0);
        Self::new(SCALE_TIERS[(pos + 1) % SCALE_TIERS.len()])
    }

    pub fn prev(self) -> Self {
        let pos = SCALE_TIERS.iter().position(|&m| m == self.multiple).unwrap_or(0);
        let prev_idx = if pos == 0 { SCALE_TIERS.len() - 1 } else { pos - 1 };
        Self::new(SCALE_TIERS[prev_idx])
    }

    pub fn blocks(self) -> u32 {
        128 * self.multiple.clamp(1, SolarProportions::MAX_WORLD_SCALE_FACTOR)
    }

    pub fn compact_label(self) -> String {
        format!("{}M ({}X)", self.blocks(), self.multiple)
    }

    pub fn banner(self) -> String {
        let clamped = self.multiple.clamp(1, SolarProportions::MAX_WORLD_SCALE_FACTOR);
        let blocks = 128 * clamped;
        let name = match clamped {
            1 => "MOONLET",
            2 => "POCKET",
            3 | 4 => "VALLEY",
            5..=8 => "HIGHLAND",
            9..=16 => "EXPANSE",
            _ => "CONTINENTAL",
        };
        format!("{blocks}M {name} ({clamped}X)")
    }

    pub fn inscribe_toml(self, out: &mut String) {
        out.push_str(&format!("size = {}\n", self.multiple));
    }

    /// Assembles the `SeededCosmos<EpsteinReachCosmos<FileCosmos<MeasuredCosmos>>>` decorator chain without formula duplication.
    pub fn cosmos_scale(self, belt_density: f32, seed: WorldSeed) -> Box<dyn CosmosScale> {
        let measured = Box::new(MeasuredCosmos::new(Box::new(ScaledCube::new(self.multiple))));
        let belted = FileCosmos::new(measured, BeltDensity::Specified(belt_density));
        Box::new(SeededCosmos::new(EpsteinReachCosmos::new(belted), seed))
    }
}

/// Immutable seed phrase pairing resolved `WorldSeed` bits with the human-readable input label (`2` fields).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SeedPhrase {
    seed: WorldSeed,
    label: String,
}

impl SeedPhrase {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(seed: WorldSeed, label: String) -> Self {
        Self { seed, label }
    }

    pub fn from_input(raw: &str) -> Self {
        let mut clean = String::with_capacity(16);
        for ch in raw.chars() {
            if clean.len() >= 16 {
                break;
            }
            let up = ch.to_ascii_uppercase();
            if up.is_ascii_alphanumeric() || matches!(up, '-' | '_') {
                clean.push(up);
            }
        }
        Self::new(WorldSeed::from_phrase(&clean), clean)
    }

    pub fn rehydrated(seed: WorldSeed, raw_label: &str) -> Self {
        let candidate = Self::from_input(raw_label).label;
        let label = if candidate.is_empty() {
            seed.bits().to_string()
        } else {
            candidate
        };
        Self::new(seed, label)
    }

    pub fn rolled(entropy: u64) -> Self {
        let rolled = WorldSeed::randomized(entropy);
        Self::new(rolled, rolled.bits().to_string())
    }

    pub fn resolved(&self, fallback_entropy: u64) -> Self {
        if self.label.is_empty() {
            Self::rolled(fallback_entropy)
        } else {
            Self::new(WorldSeed::from_phrase(&self.label), self.label.clone())
        }
    }

    pub fn appended(&self, raw: &str) -> Self {
        Self::from_input(&format!("{}{raw}", self.label))
    }

    pub fn popped(&self) -> Self {
        let mut next = self.label.clone();
        next.pop();
        Self::from_input(&next)
    }

    pub fn field_caption(&self, focused: bool) -> String {
        if focused {
            format!("{}_", self.label)
        } else if self.label.is_empty() {
            "RANDOM SEED".to_string()
        } else {
            self.label.clone()
        }
    }

    pub const fn seed(&self) -> WorldSeed {
        self.seed
    }

    pub fn short_label(&self) -> String {
        self.label.chars().take(12).collect()
    }

    pub fn inscribe_toml(&self, out: &mut String) {
        out.push_str(&format!("seed = {}\nseed_label = \"{}\"\n", self.seed.bits(), self.label));
    }

    #[cfg(test)]
    pub fn matches_label(&self, expected: &str) -> bool {
        self.label == expected
    }
}
