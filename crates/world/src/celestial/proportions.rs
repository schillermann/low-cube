/// Planet, moon, and solar cube size multipliers relative to astronomical 1:1 proportions.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BodyRatios {
    body_scale: f32,
    sun_scale: f32,
}

impl BodyRatios {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(body_scale: f32, sun_scale: f32) -> Self {
        Self { body_scale, sun_scale }
    }

    pub const fn true_scale() -> Self {
        Self::new(1.0, 1.0)
    }

    pub const fn body_factor(self) -> f32 {
        self.body_scale
    }

    pub const fn sun_factor(self) -> f32 {
        self.sun_scale
    }
}

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

/// Scalable solar-system proportions governing Earth edge blocks, body sizes, solar span, AU distance, and belt density.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SolarProportions {
    earth_blocks: u32,
    ratios: BodyRatios,
    reach: BeltAndReach,
}

impl SolarProportions {
    pub const MAX_WORLD_SCALE_FACTOR: u32 = 32;
    pub const EPSTEIN_HOUR_AU_CAP: f64 = 48_087_040.0;

    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(earth_blocks: u32, ratios: BodyRatios, reach: BeltAndReach) -> Self {
        Self { earth_blocks, ratios, reach }
    }

    /// True 1:1 astronomical proportions anchored to `earth_blocks` (`1 AU = 11_740 × Earth`, `Sun = 109.2 × Earth`).
    pub const fn true_scale(earth_blocks: u32) -> Self {
        Self::new(earth_blocks, BodyRatios::true_scale(), BeltAndReach::true_scale())
    }

    /// Compact proportional solar system scaling orbital spans by `distance_scale` for rapid testing or compact servers.
    pub const fn compressed(earth_blocks: u32, distance_scale: f32) -> Self {
        Self::new(earth_blocks, BodyRatios::true_scale(), BeltAndReach::new(distance_scale, 1.0))
    }

    /// Uniformly scales orbital spans, planet/moon sizes, and solar size by `factor`.
    pub const fn with_system_scale(self, factor: f32) -> Self {
        Self::new(
            self.earth_blocks,
            BodyRatios::new(factor, factor),
            BeltAndReach::new(factor, self.reach.density_factor()),
        )
    }

    pub const fn with_body_scale(self, factor: f32) -> Self {
        Self::new(
            self.earth_blocks,
            BodyRatios::new(factor, self.ratios.sun_factor()),
            self.reach,
        )
    }

    pub const fn with_sun_scale(self, factor: f32) -> Self {
        Self::new(
            self.earth_blocks,
            BodyRatios::new(self.ratios.body_factor(), factor),
            self.reach,
        )
    }

    pub const fn with_distance_scale(self, factor: f32) -> Self {
        Self::new(
            self.earth_blocks,
            self.ratios,
            BeltAndReach::new(factor, self.reach.density_factor()),
        )
    }

    pub const fn with_belt_density(self, density: f32) -> Self {
        Self::new(
            self.earth_blocks,
            self.ratios,
            BeltAndReach::new(self.reach.distance_factor(), density),
        )
    }

    pub const fn earth_blocks(self) -> u32 {
        self.earth_blocks
    }

    pub fn earth_half(self) -> f32 {
        (self.earth_blocks as f32) * 0.5
    }

    /// Astronomical Unit (`1 AU`) in world meters (`11,740 × Earth_Diameter × distance_scale`, capped to `< 1h` Epstein outer-belt reach).
    pub fn astronomical_unit(self) -> f64 {
        let edge = (self.earth_blocks.max(8) as f64) * 11_740.0;
        let d_factor = f64::from(self.reach.distance_factor().max(0.0005));
        (edge * d_factor)
            .min(Self::EPSTEIN_HOUR_AU_CAP)
            .max(f64::from(self.earth_blocks) * 6.0)
    }

    /// Half-size of the cubic Sun (`54.6 × Earth_Diameter × sun_scale`, bounded inside Mercury's inner orbit).
    pub fn sun_half(self) -> f64 {
        let raw = (self.earth_blocks.max(8) as f64) * 54.6 * f64::from(self.ratios.sun_factor().max(0.01));
        let mercury_clearance = self.astronomical_unit() * 0.387 * 0.45;
        raw.min(mercury_clearance).max(4.0)
    }

    /// Half-size of a celestial cube with diameter `earth_ratio × Earth_Diameter`.
    pub fn body_half(self, earth_ratio: f64) -> f64 {
        let scaled = (self.earth_blocks.max(8) as f64)
            * 0.5
            * earth_ratio
            * f64::from(self.ratios.body_factor().max(0.02));
        scaled.max(2.0)
    }

    /// Even voxel block count (`4..=4096`) for on-demand rocky planets and moons scaled to `earth_blocks` and `body_scale`.
    pub fn voxel_blocks(self, earth_ratio: f32) -> u32 {
        let raw = (self.earth_blocks as f32) * earth_ratio * self.ratios.body_factor().max(0.02);
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

    /// Even voxel block count (`4..=128`) for procedural belt asteroids scaled to `earth_blocks` and `body_scale`.
    pub fn belt_rock_blocks(self, tier: u32) -> u32 {
        let base = (16 + (tier % 3) * 8) as f32;
        let raw = base * (self.earth_blocks.max(8) as f32 / 128.0) * self.ratios.body_factor().max(0.02);
        let clamped = (raw.round() as u32).clamp(4, 128);
        clamped.div_ceil(2) * 2
    }

    pub const fn belt_density(self) -> f32 {
        self.reach.density_factor()
    }

    pub const fn distance_scale(self) -> f32 {
        self.reach.distance_factor()
    }

    pub const fn body_scale(self) -> f32 {
        self.ratios.body_factor()
    }

    pub const fn sun_scale(self) -> f32 {
        self.ratios.sun_factor()
    }

    /// Maximum active coordinate extent across the 30.1 AU Neptune heliosphere for multiplayer ward bounds.
    pub fn heliosphere_span(self) -> f32 {
        let outer = self.astronomical_unit() * 36.0 + 4_096.0;
        outer.min(1.0e15) as f32
    }

    /// Transfers `(earth_blocks, (body_scale, sun_scale), (distance_scale, belt_density))` without getters.
    pub fn transfer<R>(self, f: impl FnOnce(u32, (f32, f32), (f32, f32)) -> R) -> R {
        f(
            self.earth_blocks,
            (self.ratios.body_factor(), self.ratios.sun_factor()),
            (self.reach.distance_factor(), self.reach.density_factor()),
        )
    }

    /// Appends the 16-byte floating-point tail (`body`, `sun`, `distance`, `belt`) to a handshake frame.
    pub fn inscribe_tail(self, out: &mut Vec<u8>) {
        for val in [
            self.ratios.body_factor(),
            self.ratios.sun_factor(),
            self.reach.distance_factor(),
            self.reach.density_factor(),
        ] {
            out.extend_from_slice(&val.to_le_bytes());
        }
    }

    /// Reconstructs `SolarProportions` from `earth_blocks` and an optional 16-byte handshake tail.
    pub fn decode_tail(earth_blocks: u32, tail: &[u8]) -> Self {
        if tail.len() < 16 {
            return Self::true_scale(earth_blocks);
        }
        let f = |range: std::ops::Range<usize>, fallback: f32| -> f32 {
            tail.get(range)
                .and_then(|s| <[u8; 4]>::try_from(s).ok())
                .map(f32::from_le_bytes)
                .filter(|v| v.is_finite() && *v >= 0.0)
                .unwrap_or(fallback)
        };
        let body = f(0..4, 1.0).max(0.02);
        let sun = f(4..8, 1.0).max(0.01);
        let dist = f(8..12, 1.0).max(0.0005);
        let belt = f(12..16, 1.0).clamp(0.0, 8.0);
        Self::new(earth_blocks, BodyRatios::new(body, sun), BeltAndReach::new(dist, belt))
    }
}
