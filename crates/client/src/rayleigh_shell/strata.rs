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

/// Focused contract evaluating `(local_density, scattered_tint, lit_fraction)` at a 3D point inside an atmosphere.
pub trait AirStratum: Copy {
    fn sample_point(&self, point: Vec3, altitude_ratio: f32, sun_dir: Vec3) -> (f32, Vec3, f32);
}

/// Innermost barometric Rayleigh stratum (`exp(-h * 1.35)`) scattering sunlit daytime sky tint.
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
    fn sample_point(&self, point: Vec3, altitude_ratio: f32, sun_dir: Vec3) -> (f32, Vec3, f32) {
        let h = altitude_ratio.clamp(0.0, 1.0);
        let rho = (-h * 1.35).exp();
        let mu = point.normalize_or_zero().dot(sun_dir);
        let day = SmoothBand::new(-0.14, 0.32).sample(mu);
        (rho, self.day_tint * day, day)
    }
}

/// `AirStratum` decorator infusing warm crimson-amber twilight scattering along the solar shadow line.
#[derive(Clone, Copy, Debug)]
pub struct CrimsonDusk<S> {
    origin: S,
    ember_tint: Vec3,
}

impl<S> CrimsonDusk<S> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(origin: S, ember_tint: Vec3) -> Self {
        Self { origin, ember_tint }
    }

    /// Secondary constructor calibrated to warm terrestrial sunset wavelengths.
    pub const fn terrestrial(origin: S) -> Self {
        Self::new(origin, Vec3::new(1.00, 0.38, 0.12))
    }
}

impl<S: AirStratum> AirStratum for CrimsonDusk<S> {
    fn sample_point(&self, point: Vec3, altitude_ratio: f32, sun_dir: Vec3) -> (f32, Vec3, f32) {
        let (rho, daylit_tint, daylit_lit) = self.origin.sample_point(point, altitude_ratio, sun_dir);
        let mu = point.normalize_or_zero().dot(sun_dir);
        let bell = (-((mu - 0.02) / 0.18).powi(2)).exp();
        let twilight = bell * SmoothBand::new(-0.34, -0.02).sample(mu);
        let dusk_tint = daylit_tint + self.ember_tint * (twilight * 0.88);
        (rho, dusk_tint, daylit_lit.max(twilight * 0.65))
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
    fn sample_point(&self, point: Vec3, altitude_ratio: f32, sun_dir: Vec3) -> (f32, Vec3, f32) {
        let h = altitude_ratio.clamp(0.0, 1.0);
        let (lower_rho, tint, lit) = self.origin.sample_point(point, h, sun_dir);
        let rim_sheen = 0.22 * (-((h - self.rim_peak) / 0.07).powi(2)).exp();
        let bounded_rho = (lower_rho + rim_sheen) * SmoothBand::new(1.0, 0.92).sample(h);
        (bounded_rho, tint, lit)
    }
}
