use glam::Vec3;
use crate::WorldSeed;
use super::bubble::CosmicFrame;
use super::catalog::SolarCatalog;

/// Orbital distance multiplier and procedural asteroid belt occupancy density.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BeltAndReach {
    distance_scale: f32,
    belt_density: f32,
}

impl BeltAndReach {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(distance_scale: f32, belt_density: f32) -> Self {
        Self { distance_scale, belt_density }
    }

    pub const fn true_scale() -> Self {
        Self::new(1.0, 1.0)
    }

    pub const fn distance_factor(self) -> f32 {
        self.distance_scale
    }

    pub const fn density_factor(self) -> f32 {
        self.belt_density
    }
}

/// Scalable solar-system proportions coupled directly to `earth_blocks` (`size = 1..=32`) and `< 1h` Epstein outer-belt reach.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SolarProportions {
    earth_blocks: u32,
    reach: BeltAndReach,
    seed: WorldSeed,
}

impl SolarProportions {
    pub const MAX_WORLD_SCALE_FACTOR: u32 = 32;
    pub const EPSTEIN_HOUR_AU_CAP: f64 = 48_087_040.0;

    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn with_world_seed(earth_blocks: u32, reach: BeltAndReach, seed: WorldSeed) -> Self {
        Self { earth_blocks, reach, seed }
    }

    /// Secondary constructor pairing `earth_blocks` and `reach` with the canonical origin seed.
    pub const fn new(earth_blocks: u32, reach: BeltAndReach) -> Self {
        Self::with_world_seed(earth_blocks, reach, WorldSeed::origin())
    }

    /// True 1:1 astronomical proportions anchored to `earth_blocks` (`1 AU = 11_740 × Earth`, `Sun = 109.2 × Earth`).
    pub const fn true_scale(earth_blocks: u32) -> Self {
        Self::new(earth_blocks, BeltAndReach::true_scale())
    }

    /// Compact proportional solar system scaling orbital spans by `distance_scale` for rapid testing.
    pub const fn compressed(earth_blocks: u32, distance_scale: f32) -> Self {
        Self::new(earth_blocks, BeltAndReach::new(distance_scale, 1.0))
    }

    pub const fn with_seed(self, seed: WorldSeed) -> Self {
        Self::with_world_seed(self.earth_blocks, self.reach, seed)
    }

    pub const fn seed(self) -> WorldSeed {
        self.seed
    }

    pub const fn with_distance_scale(self, factor: f32) -> Self {
        Self::with_world_seed(
            self.earth_blocks,
            BeltAndReach::new(factor, self.reach.density_factor()),
            self.seed,
        )
    }

    pub const fn with_belt_density(self, density: f32) -> Self {
        Self::with_world_seed(
            self.earth_blocks,
            BeltAndReach::new(self.reach.distance_factor(), density),
            self.seed,
        )
    }

    pub const fn earth_blocks(self) -> u32 {
        self.earth_blocks
    }

    pub fn earth_half(self) -> f32 {
        (self.earth_blocks as f32) * 0.5
    }

    /// Astronomical Unit (`1 AU`) in world meters (`11,740 × Earth_Diameter`, capped to `< 1h` Epstein outer-belt reach).
    pub fn astronomical_unit(self) -> f64 {
        let edge = (self.earth_blocks.max(8) as f64) * 11_740.0;
        let d_factor = f64::from(self.reach.distance_factor().max(0.0005));
        (edge * d_factor)
            .min(Self::EPSTEIN_HOUR_AU_CAP)
            .max(f64::from(self.earth_blocks) * 6.0)
    }

    /// Half-size of the cubic Sun (`54.6 × Earth_Diameter`, preserving `0.53°` angular span inside Mercury's inner orbit).
    pub fn sun_half(self) -> f64 {
        let angular_lock = f64::from(self.reach.distance_factor().clamp(0.0005, 1.0));
        let raw = (self.earth_blocks.max(8) as f64) * 54.6 * angular_lock;
        let mercury_clearance = self.astronomical_unit() * 0.387 * 0.45;
        raw.min(mercury_clearance).max(4.0)
    }

    /// Half-size of a celestial cube with astronomical diameter `earth_ratio × Earth_Diameter`.
    pub fn body_half(self, earth_ratio: f64) -> f64 {
        let scaled = (self.earth_blocks.max(8) as f64) * 0.5 * earth_ratio;
        scaled.max(2.0)
    }

    /// Even voxel block count (`4..=4096`) for on-demand rocky planets and moons scaled 1:1 to `earth_blocks`.
    pub fn voxel_blocks(self, earth_ratio: f32) -> u32 {
        let raw = (self.earth_blocks as f32) * earth_ratio;
        let clamped = (raw.round() as u32).clamp(4, 4_096);
        clamped.div_ceil(2) * 2
    }

    /// Chemical rocket speed ceiling (`m/s`) scaled by `sqrt(earth_blocks / 128)` so blueprints reach orbit on any cube size.
    pub fn chemical_speed_cap(self) -> f32 {
        45.0 * ((self.earth_blocks.max(128) as f32) / 128.0).sqrt()
    }

    /// Epstein fusion torch speed ceiling (`m/s`) keeping `3.20 AU` outer-belt Flip-&-Burn transits under 1 hour.
    pub const fn fusion_speed_cap(self) -> f32 {
        240_000.0
    }

    /// Even voxel block count (`4..=128`) for procedural belt asteroids scaled to `earth_blocks`.
    pub fn belt_rock_blocks(self, tier: u32) -> u32 {
        let base = (16 + (tier % 3) * 8) as f32;
        let raw = base * (self.earth_blocks.max(8) as f32 / 128.0);
        let clamped = (raw.round() as u32).clamp(4, 128);
        clamped.div_ceil(2) * 2
    }

    pub const fn belt_density(self) -> f32 {
        self.reach.density_factor()
    }

    pub const fn distance_scale(self) -> f32 {
        self.reach.distance_factor()
    }

    /// Maximum active coordinate extent across the 30.1 AU Neptune heliosphere for multiplayer ward bounds.
    pub fn heliosphere_span(self) -> f32 {
        let outer = self.astronomical_unit() * 36.0 + 4_096.0;
        outer.min(1.0e15) as f32
    }

    /// Indicates whether `local_pos` inside `frame` lies in oxygen-free vacuum (airless body or above atmospheric envelope).
    pub fn vacuum_locus(self, frame: CosmicFrame, local_pos: Vec3) -> bool {
        if !frame.atmospheric_sound() {
            return true;
        }
        let CosmicFrame::Body(id) = frame else {
            return true;
        };
        let Some(spec) = SolarCatalog::new().spec_for(id) else {
            return true;
        };
        let half = (self.voxel_blocks(spec.physique().diameter_ratio()) as f32) * 0.5;
        let band = (half * 0.45).clamp(36.0, 160.0);
        local_pos.length() > half + band
    }

    /// Appends the 16-byte handshake tail (`seed_lo`, `seed_hi`, `distance`, `belt`) preserving wire alignment at byte `60`.
    pub fn inscribe_tail(self, out: &mut Vec<u8>) {
        out.extend_from_slice(&self.seed.bits().to_le_bytes());
        for val in [self.reach.distance_factor(), self.reach.density_factor()] {
            out.extend_from_slice(&val.to_le_bytes());
        }
    }

    /// Reconstructs `SolarProportions` from `earth_blocks` and a mandatory 16-byte handshake tail.
    pub fn decode_tail(earth_blocks: u32, tail: &[u8]) -> Option<Self> {
        if tail.len() < 16 {
            return None;
        }
        let seed_bits = u64::from_le_bytes(tail[0..8].try_into().ok()?);
        let dist = f32::from_le_bytes(tail[8..12].try_into().ok()?);
        let belt = f32::from_le_bytes(tail[12..16].try_into().ok()?);
        if !dist.is_finite() || dist < 0.0 || !belt.is_finite() || belt < 0.0 {
            return None;
        }
        Some(Self::with_world_seed(
            earth_blocks,
            BeltAndReach::new(dist.max(0.0005), belt.clamp(0.0, 8.0)),
            WorldSeed::new(seed_bits),
        ))
    }
}
