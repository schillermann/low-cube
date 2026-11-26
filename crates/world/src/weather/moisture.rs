use crate::ClimateZone;

/// Smooth Hermite interpolation curve without external dependencies (`<= 4` fields).
#[derive(Clone, Copy, Debug)]
pub(super) struct WeatherCurve {
    edge0: f32,
    edge1: f32,
}

impl WeatherCurve {
    pub(super) const fn new(edge0: f32, edge1: f32) -> Self {
        Self { edge0, edge1 }
    }

    pub(super) fn sample(self, x: f32) -> f32 {
        let t = ((x - self.edge0) / (self.edge1 - self.edge0)).clamp(0.0, 1.0);
        t * t * (3.0 - 2.0 * t)
    }
}

/// Focused contract for deterministic 2D procedural moisture fields.
pub trait PlanarNoise: Copy {
    fn sample(&self, x: f32, y: f32) -> f32;
}

/// Allocation-free 2D Hermite value-noise field (`<= 4` fields).
#[derive(Clone, Copy, Debug)]
pub struct SmoothScalarField {
    seed: u32,
}

impl SmoothScalarField {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(seed: u32) -> Self {
        Self { seed }
    }

    fn lattice(self, ix: i32, iy: i32) -> f32 {
        let mut h = self.seed
            .wrapping_add((ix as u32).wrapping_mul(0x85EB_CA6B))
            .wrapping_add((iy as u32).wrapping_mul(0xC2B2_AE35));
        h ^= h >> 16;
        h = h.wrapping_mul(0x7FEB_352D);
        h ^= h >> 15;
        h = h.wrapping_mul(0x846C_A68B);
        h ^= h >> 16;
        (h >> 8) as f32 / 16_777_215.0
    }
}

impl PlanarNoise for SmoothScalarField {
    fn sample(&self, x: f32, y: f32) -> f32 {
        let ix = x.floor() as i32;
        let iy = y.floor() as i32;
        let fx = x - (ix as f32);
        let fy = y - (iy as f32);
        let ux = fx * fx * (3.0 - 2.0 * fx);
        let uy = fy * fy * (3.0 - 2.0 * fy);
        let v00 = self.lattice(ix, iy);
        let v10 = self.lattice(ix + 1, iy);
        let v01 = self.lattice(ix, iy + 1);
        let v11 = self.lattice(ix + 1, iy + 1);
        let a = v00 + (v10 - v00) * ux;
        let b = v01 + (v11 - v01) * ux;
        a + (b - a) * uy
    }
}

/// Focused contract mapping a `ClimateZone` to hydrological evaporation, wind pace, and freezing state.
pub trait BiomeMoisture: Copy {
    fn profile(&self) -> (f32, f32, bool);
}

/// Biome-calibrated evaporation supply, wind drift multiplier, and polar freezing (`<= 4` fields).
#[derive(Clone, Copy, Debug)]
pub struct ClimateHumidity {
    zone: ClimateZone,
}

impl ClimateHumidity {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(zone: ClimateZone) -> Self {
        Self { zone }
    }
}

impl BiomeMoisture for ClimateHumidity {
    fn profile(&self) -> (f32, f32, bool) {
        match self.zone {
            ClimateZone::Jungle => (0.14, 1.00, false),
            ClimateZone::Temperate => (0.05, 0.95, false),
            ClimateZone::Arctic => (0.04, 0.88, true),
            ClimateZone::Steppe => (-0.03, 1.12, false),
            ClimateZone::Volcanic => (-0.20, 0.92, false),
            ClimateZone::Desert => (-0.38, 1.05, false),
        }
    }
}

/// Focused contract inspecting condensed cloud mass, optical storm darkness, and precipitation rate.
pub trait WeatherSample: Copy {
    fn present(&self) -> bool;
    fn raining(&self) -> bool;
    fn frozen(&self) -> bool;
    fn transfer<R>(&self, sink: impl FnOnce(f32, f32, f32, bool) -> R) -> R;
}

/// Hydrological state of one atmospheric cell (`<= 4` strictly private fields).
#[derive(Clone, Copy, Debug)]
pub struct ConvergenceCell {
    mass: f32,
    darkness: f32,
    rain_rate: f32,
    frozen: bool,
}

impl ConvergenceCell {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(mass: f32, darkness: f32, rain_rate: f32, frozen: bool) -> Self {
        Self { mass, darkness, rain_rate, frozen }
    }

    pub const fn clear() -> Self {
        Self::new(0.0, 0.0, 0.0, false)
    }
}

impl WeatherSample for ConvergenceCell {
    fn present(&self) -> bool {
        self.mass > 0.02
    }

    fn raining(&self) -> bool {
        self.rain_rate > 0.04
    }

    fn frozen(&self) -> bool {
        self.frozen
    }

    fn transfer<R>(&self, sink: impl FnOnce(f32, f32, f32, bool) -> R) -> R {
        sink(self.mass, self.darkness, self.rain_rate, self.frozen)
    }
}

/// Focused contract sampling multi-layer moisture convergence across a planetary face.
pub trait TroposphereField: Copy {
    fn sample_layer(&self, layer: u8, u: f32, v: f32) -> ConvergenceCell;
}

/// Deterministic advecting moisture & convergence field synchronized via `epoch_secs` (`<= 4` fields).
#[derive(Clone, Copy, Debug)]
pub struct MoistureField {
    zone: ClimateZone,
    half_size: f32,
    epoch_secs: f32,
}

impl MoistureField {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(zone: ClimateZone, half_size: f32, epoch_secs: f32) -> Self {
        Self { zone, half_size, epoch_secs }
    }

    pub(super) fn cell_span(self) -> f32 {
        (self.half_size * 0.38).clamp(8.5, 28.0)
    }
}

impl TroposphereField for MoistureField {
    fn sample_layer(&self, layer: u8, u: f32, v: f32) -> ConvergenceCell {
        let (supply, wind_k, frozen) = ClimateHumidity::new(self.zone).profile();
        let span = self.cell_span();
        let seed = ((self.zone.index() as u32) + 1)
            .wrapping_mul(0x9E37_79B9)
            .wrapping_add(u32::from(layer).wrapping_mul(0x517C_C1B7));
        let t = self.epoch_secs * wind_k;

        if layer == 0 {
            let front_span = span * 2.1;
            let su1 = u / front_span - t * 0.015;
            let sv1 = v / front_span - t * 0.006;
            let su2 = u / (front_span * 1.28) + t * 0.005;
            let sv2 = v / (front_span * 1.28) - t * 0.012;
            let bank_a = SmoothScalarField::new(seed ^ 0xA511_E9B3).sample(su1, sv1);
            let bank_b = SmoothScalarField::new(seed ^ 0x63D8_3595).sample(su2, sv2);
            let potential =
                (bank_a * 0.48 + bank_b * 0.35 + (bank_a * bank_b) * 0.44 + supply).clamp(0.0, 1.35);
            if potential <= 0.72 {
                return ConvergenceCell::clear();
            }
            let excess = ((potential - 0.95) / 0.24).clamp(0.0, 1.0);
            let phase = SmoothScalarField::new(seed ^ 0x3C6E_F372).sample(su1 * 0.45, sv2 * 0.45);
            let cycle = (phase + t * 0.0075).rem_euclid(1.0);
            let onset = WeatherCurve::new(0.18, 0.36).sample(cycle);
            let taper = 1.0 - WeatherCurve::new(0.76, 0.92).sample(cycle);
            let rain_rate = (excess * onset * taper * 1.25).clamp(0.0, 1.0);
            let drained = WeatherCurve::new(0.42, 0.94).sample(cycle) * 0.22 * excess;
            let condensed = ((potential - drained - 0.72) / 0.48).clamp(0.0, 1.0);
            let upper_shadow = self.sample_layer(1, u, v).mass * 0.16;
            let darkness = (WeatherCurve::new(0.26, 0.58).sample(condensed + upper_shadow * 0.3)
                * 0.66
                + excess * 0.22
                + rain_rate * 0.12)
                .clamp(0.0, 0.90);
            return ConvergenceCell::new(condensed, darkness, rain_rate, frozen);
        }

        if layer == 1 {
            let su = u / (span * 1.45) - t * 0.036;
            let sv = v / (span * 1.45) + t * 0.021;
            let alt_n = SmoothScalarField::new(seed ^ 0xB7E1_5163).sample(su, sv);
            let raw = alt_n * 0.84 + supply * 0.35;
            if raw <= 0.68 {
                return ConvergenceCell::clear();
            }
            let mass = ((raw - 0.68) / 0.26).clamp(0.0, 0.72);
            let darkness = WeatherCurve::new(0.45, 0.95).sample(mass) * 0.14;
            return ConvergenceCell::new(mass, darkness, 0.0, frozen);
        }

        let su = u / (span * 1.95) - t * 0.052;
        let sv = v / (span * 1.30) - t * 0.028;
        let cir_n = SmoothScalarField::new(seed ^ 0x27D4_EB2F).sample(su, sv);
        let raw = cir_n * 0.88 + supply * 0.16;
        if raw <= 0.73 {
            return ConvergenceCell::clear();
        }
        let mass = ((raw - 0.73) / 0.24).clamp(0.0, 0.58);
        ConvergenceCell::new(mass, 0.03 * mass, 0.0, true)
    }
}
