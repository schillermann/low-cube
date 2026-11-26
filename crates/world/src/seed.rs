/// Immutable 64-bit SplitMix scrambler instance eliminating static procedural hash helpers (`1` field).
#[derive(Clone, Copy, Debug)]
struct SplitMix64 {
    state: u64,
}

impl SplitMix64 {
    /// Primary constructor doing nothing except assigning arguments to fields.
    const fn new(state: u64) -> Self {
        Self { state }
    }

    const fn mixed(self) -> u64 {
        let z1 = (self.state ^ (self.state >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        let z2 = (z1 ^ (z1 >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z2 ^ (z2 >> 31)
    }
}

/// Immutable procedural world-generation seed driving relief strata, mineral veins, and woodland groves (`1` field).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct WorldSeed {
    bits: u64,
}

impl WorldSeed {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(bits: u64) -> Self {
        Self { bits }
    }

    /// Canonical unshifted origin seed (`0`) preserving historic procedural terrain, veins, and groves.
    pub const fn origin() -> Self {
        Self::new(0)
    }

    /// Deterministic seed parsed from decimal digits or hashed from an arbitrary text phrase.
    pub fn from_phrase(phrase: &str) -> Self {
        let trimmed = phrase.trim();
        if trimmed.is_empty() {
            return Self::origin();
        }
        if let Ok(parsed) = trimmed.parse::<u64>() {
            return Self::new(parsed);
        }
        let mut h = 0xCBF2_9CE4_8422_2325_u64;
        for byte in trimmed.bytes() {
            h ^= u64::from(byte.to_ascii_uppercase());
            h = h.wrapping_mul(0x0100_0000_01B3);
        }
        let mixed = SplitMix64::new(h).mixed();
        Self::new(if mixed == 0 { 1 } else { mixed })
    }

    /// Generates a human-legible 8-digit non-zero seed from an entropy counter.
    pub const fn randomized(entropy: u64) -> Self {
        let mixed = SplitMix64::new(entropy ^ 0x9E37_79B9_7F4A_7C15).mixed();
        Self::new(10_000_000 + (mixed % 90_000_000))
    }

    pub const fn bits(self) -> u64 {
        self.bits
    }

    /// Deterministic 3D spatial domain shift `(dx, dy, dz)` for `SeededFrequency`, returning `(0, 0, 0)` for `origin()`.
    pub fn spatial_shift(self) -> glam::Vec3 {
        if self.bits == 0 {
            return glam::Vec3::ZERO;
        }
        let a = SplitMix64::new(self.bits ^ 0x517C_C1B7_2722_0A95).mixed();
        let b = SplitMix64::new(a ^ 0x94D0_49BB_1331_11EB).mixed();
        let c = SplitMix64::new(b ^ 0xBF58_476D_1CE4_E5B9).mixed();
        let map = |v: u64| (((v & 0xFFFF) as f32) / 65535.0 - 0.5) * 4_096.0;
        glam::Vec3::new(map(a), map(b), map(c))
    }

    /// Mixes this seed with a subterranean mineral-vein salt, leaving `origin()` (`0`) untouched.
    pub const fn mix_vein(self, zone_salt: u64) -> u64 {
        if self.bits == 0 {
            zone_salt
        } else {
            SplitMix64::new(self.bits ^ zone_salt.wrapping_mul(0x94D0_49BB_1331_11EB)).mixed()
        }
    }

    /// Mixes this seed with a discrete woodland cell hash, leaving `origin()` (`0`) untouched.
    pub const fn mix_grove(self, cell_hash: i32) -> i32 {
        if self.bits == 0 {
            cell_hash
        } else {
            let salt = SplitMix64::new(self.bits).mixed() as i32;
            (cell_hash ^ salt).wrapping_mul(0x27D4_EB2D)
        }
    }
}
