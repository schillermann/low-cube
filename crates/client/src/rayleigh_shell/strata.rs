use glam::Vec3;

/// Focused contract for smooth Hermite interpolation across an atmospheric transition band.
pub trait BandCurve: Copy {
    fn sample(&self, x: f32) -> f32;
}

/// Immutable Hermite smoother across `[edge0, edge1]` (`<= 4` fields).
#[derive(Clone, Copy, Debug)]
pub struct SmoothBand {
    edge0: f32,
    edge1: f32,
}

impl SmoothBand {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(edge0: f32, edge1: f32) -> Self {
        Self { edge0, edge1 }
    }
}

impl BandCurve for SmoothBand {
    fn sample(&self, x: f32) -> f32 {
        let t = ((x - self.edge0) / (self.edge1 - self.edge0)).clamp(0.0, 1.0);
        t * t * (3.0 - 2.0 * t)
    }
}

/// Focused contract mapping any 3D atmospheric position to its continuous planetary zenith normal.
pub trait ZenithField: Copy {
    fn zenith_at(&self, point: Vec3) -> Vec3;
}

/// Superellipse (`L^5`) plateau zenith keeping cube continents flat before curving smoothly across seams.
#[derive(Clone, Copy, Debug)]
pub struct PlateauZenith;

impl PlateauZenith {
    /// Primary constructor doing nothing except creating the zenith field instance.
    pub const fn new() -> Self {
        Self
    }
}

impl ZenithField for PlateauZenith {
    fn zenith_at(&self, point: Vec3) -> Vec3 {
        if point.length_squared() <= 1e-6 {
            return Vec3::Y;
        }
        let n = point.normalize();
        let q2 = n * n;
        (n * q2 * q2).normalize_or_zero()
    }
}

/// Focused contract querying geometric angles at a 3D atmospheric quadrature sample.
pub trait StratumPoint: Copy {
    fn altitude(&self) -> f32;
    fn zenith_cos(&self, field: &impl ZenithField) -> f32;
    fn sunward_cos(&self) -> f32;
}

/// Encapsulated 3D atmospheric quadrature point (`<= 4` strictly private fields).
#[derive(Clone, Copy, Debug)]
pub struct AirSample {
    point: Vec3,
    altitude_ratio: f32,
    ray_dir: Vec3,
    sun_dir: Vec3,
}

impl AirSample {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(point: Vec3, altitude_ratio: f32, ray_dir: Vec3, sun_dir: Vec3) -> Self {
        Self { point, altitude_ratio, ray_dir, sun_dir }
    }
}

impl StratumPoint for AirSample {
    fn altitude(&self) -> f32 {
        self.altitude_ratio.clamp(0.0, 1.0)
    }

    fn zenith_cos(&self, field: &impl ZenithField) -> f32 {
        field.zenith_at(self.point).dot(self.sun_dir)
    }

    fn sunward_cos(&self) -> f32 {
        self.ray_dir.dot(self.sun_dir)
    }
}

/// Focused contract evaluating `(local_density, scattered_tint, lit_fraction)` at an `AirSample`.
pub trait AirStratum: Copy {
    fn sample_point(&self, sample: &AirSample) -> (f32, Vec3, f32);
}

/// Innermost barometric Rayleigh stratum (`exp(-h * 1.35)`) over a `PlateauZenith` continent.
#[derive(Clone, Copy, Debug)]
pub struct BarometricStratum {
    day_tint: Vec3,
}

impl BarometricStratum {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(day_tint: Vec3) -> Self {
        Self { day_tint }
    }
}

impl AirStratum for BarometricStratum {
    fn sample_point(&self, sample: &AirSample) -> (f32, Vec3, f32) {
        let rho = (-sample.altitude() * 1.35).exp();
        let mu = sample.zenith_cos(&PlateauZenith::new());
        let day = SmoothBand::new(-0.14, 0.30).sample(mu);
        (rho, self.day_tint * day, day)
    }
}

/// `AirStratum` decorator infusing cool cobalt-indigo Earth-shadow tones along shadow boundaries.
#[derive(Clone, Copy, Debug)]
pub struct CobaltShadowArch<S> {
    origin: S,
    cobalt_tint: Vec3,
}

impl<S> CobaltShadowArch<S> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(origin: S, cobalt_tint: Vec3) -> Self {
        Self { origin, cobalt_tint }
    }

    /// Secondary constructor calibrated to deep twilight sapphire-indigo.
    pub const fn terrestrial(origin: S) -> Self {
        Self::new(origin, Vec3::new(0.05, 0.16, 0.44))
    }
}

impl<S: AirStratum> AirStratum for CobaltShadowArch<S> {
    fn sample_point(&self, sample: &AirSample) -> (f32, Vec3, f32) {
        let (rho, daylit_tint, daylit_lit) = self.origin.sample_point(sample);
        let mu = sample.zenith_cos(&PlateauZenith::new());
        let arch = (-((mu + 0.04) / 0.18).powi(2)).exp() * SmoothBand::new(-0.36, -0.02).sample(mu);
        let shadowed_tint = daylit_tint + self.cobalt_tint * (arch * 0.52);
        (rho, shadowed_tint, daylit_lit.max(arch * 0.35))
    }
}

/// `AirStratum` decorator adding directional crimson-gold sunset only toward low-sun horizons.
#[derive(Clone, Copy, Debug)]
pub struct CrimsonDusk<S> {
    origin: S,
    ember_tint: Vec3,
    low_sun_gate: f32,
}

impl<S> CrimsonDusk<S> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(origin: S, ember_tint: Vec3, low_sun_gate: f32) -> Self {
        Self { origin, ember_tint, low_sun_gate }
    }

    /// Secondary constructor calibrated to directional terrestrial sunset wavelengths.
    pub const fn directional(origin: S, low_sun_gate: f32) -> Self {
        Self::new(origin, Vec3::new(1.00, 0.38, 0.12), low_sun_gate)
    }
}

impl<S: AirStratum> AirStratum for CrimsonDusk<S> {
    fn sample_point(&self, sample: &AirSample) -> (f32, Vec3, f32) {
        let (rho, prior_tint, prior_lit) = self.origin.sample_point(sample);
        if self.low_sun_gate <= 1e-4 {
            return (rho, prior_tint, prior_lit);
        }
        let mu = sample.zenith_cos(&PlateauZenith::new());
        let sunward = SmoothBand::new(-0.42, 0.25).sample(sample.sunward_cos());
        let bell = (-((mu - 0.02) / 0.18).powi(2)).exp();
        let twilight = bell * SmoothBand::new(-0.42, 0.04).sample(mu) * sunward * self.low_sun_gate;
        let dusk_tint = prior_tint + self.ember_tint * (twilight * 1.05);
        (rho, dusk_tint, prior_lit.max(twilight * 0.70))
    }
}

/// `AirStratum` decorator crowning the outer cubic atmosphere boundary with a crisp stratospheric sheen.
#[derive(Clone, Copy, Debug)]
pub struct TropopauseRim<S> {
    origin: S,
    rim_peak: f32,
}

impl<S> TropopauseRim<S> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(origin: S, rim_peak: f32) -> Self {
        Self { origin, rim_peak }
    }

    /// Secondary constructor placing the luminous cubic ceiling band at `88%` shell height.
    pub const fn cubic_ceiling(origin: S) -> Self {
        Self::new(origin, 0.88)
    }
}

impl<S: AirStratum> AirStratum for TropopauseRim<S> {
    fn sample_point(&self, sample: &AirSample) -> (f32, Vec3, f32) {
        let h = sample.altitude();
        let (lower_rho, tint, lit) = self.origin.sample_point(sample);
        let rim_sheen = 0.22 * (-((h - self.rim_peak) / 0.07).powi(2)).exp();
        let bounded_rho = (lower_rho + rim_sheen) * SmoothBand::new(1.0, 0.92).sample(h);
        (bounded_rho, tint, lit)
    }
}
