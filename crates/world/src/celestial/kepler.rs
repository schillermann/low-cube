use std::f64::consts::{PI, TAU};
use glam::DVec3;
use super::catalog::BodyId;

/// Narrow contract resolving 3D Keplerian focal states and orbital ellipse trajectories.
pub trait OrbitalLocus {
    /// Instantaneous `(position, velocity)` relative to the parent focus `(0, 0, 0)` at `epoch_secs`.
    fn sample_state(&self, semi_major: f64, parent_gm: f64, epoch_secs: f64) -> (DVec3, DVec3);

    /// Emits `segments` line-chord pairs around the true focal-displaced 3D Keplerian ellipse.
    fn trace_ellipse(
        &self,
        focus: DVec3,
        semi_major: f64,
        segments: usize,
        sink: &mut dyn FnMut(DVec3, DVec3),
    );
}

/// Transcendental Kepler anomaly state (`M = E - e sin E`) solved via Newton-Raphson iteration (`<= 4` fields).
#[derive(Clone, Copy, Debug)]
pub struct KeplerAnomaly {
    eccentricity: f64,
    mean_anomaly: f64,
}

impl KeplerAnomaly {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(eccentricity: f64, mean_anomaly: f64) -> Self {
        Self { eccentricity, mean_anomaly }
    }

    /// Resolves eccentric anomaly `E` in `(-π, π]` to double-precision convergence.
    pub fn eccentric_rad(self) -> f64 {
        let e = self.eccentricity.clamp(0.0, 0.85);
        let mut m = self.mean_anomaly.rem_euclid(TAU);
        if m > PI {
            m -= TAU;
        }
        let mut ecc = if e > 0.6 {
            m.signum() * (PI * 0.75)
        } else {
            m + e * m.sin()
        };
        for _ in 0..6 {
            let (se, ce) = ecc.sin_cos();
            let residual = ecc - e * se - m;
            let deriv = (1.0 - e * ce).max(1e-6);
            ecc -= residual / deriv;
        }
        ecc
    }
}

/// Semi-major orbital span (in AU or parent diameters) and Keplerian focal eccentricity (`<= 4` fields).
#[derive(Clone, Copy, Debug)]
pub struct EllipticShape {
    semi_major_units: f64,
    eccentricity: f32,
}

impl EllipticShape {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(semi_major_units: f64, eccentricity: f32) -> Self {
        Self { semi_major_units, eccentricity }
    }

    pub fn scaled_semi_major(self, unit_span: f64) -> f64 {
        self.semi_major_units * unit_span
    }

    fn clamped_eccentricity(self) -> f64 {
        f64::from(self.eccentricity).clamp(0.0, 0.85)
    }

    fn perifocal_point(self, semi_major: f64, eccentric_rad: f64) -> (f64, f64) {
        let e = self.clamped_eccentricity();
        let (se, ce) = eccentric_rad.sin_cos();
        let minor_ratio = (1.0 - e * e).max(0.0).sqrt();
        (semi_major * (ce - e), semi_major * minor_ratio * se)
    }

    fn perifocal_kinematics(
        self,
        semi_major: f64,
        mean_motion: f64,
        signed_rate: f64,
        mean_anom: f64,
    ) -> ((f64, f64), (f64, f64)) {
        let e = self.clamped_eccentricity();
        let ecc = KeplerAnomaly::new(e, mean_anom).eccentric_rad();
        let (sin_e, cos_e) = ecc.sin_cos();
        let minor_ratio = (1.0 - e * e).max(0.0).sqrt();
        let xp = semi_major * (cos_e - e);
        let zp = semi_major * minor_ratio * sin_e;
        let ecc_rate = signed_rate * mean_motion / (1.0 - e * cos_e).max(1e-6);
        let vxp = -semi_major * sin_e * ecc_rate;
        let vzp = semi_major * minor_ratio * cos_e * ecc_rate;
        ((xp, zp), (vxp, vzp))
    }
}

/// 3D inclination, argument of periapsis, and epoch mean anomaly of an orbital plane (`<= 4` fields).
#[derive(Clone, Copy, Debug)]
pub struct OrbitalPlane {
    inclination_rad: f32,
    periapsis_rad: f32,
    mean_anomaly_zero: f32,
}

impl OrbitalPlane {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(inclination_rad: f32, periapsis_rad: f32, mean_anomaly_zero: f32) -> Self {
        Self { inclination_rad, periapsis_rad, mean_anomaly_zero }
    }

    fn direction_sign(self) -> f64 {
        if self.inclination_rad < -0.2 { -1.0 } else { 1.0 }
    }

    fn mean_anomaly_at(self, mean_motion: f64, epoch_secs: f64) -> f64 {
        f64::from(self.mean_anomaly_zero) + self.direction_sign() * mean_motion * epoch_secs
    }

    fn map_perifocal(self, (xp, zp): (f64, f64)) -> DVec3 {
        let (sin_w, cos_w) = f64::from(self.periapsis_rad).sin_cos();
        let (sin_i, cos_i) = f64::from(self.inclination_rad.abs()).sin_cos();
        let u = xp * cos_w - zp * sin_w;
        let w = xp * sin_w + zp * cos_w;
        DVec3::new(u, w * sin_i, w * cos_i)
    }
}

/// Complete Keplerian elliptical orbital elements around a parent celestial body (`<= 4` fields).
#[derive(Clone, Copy, Debug)]
pub struct OrbitElements {
    parent: BodyId,
    shape: EllipticShape,
    plane: OrbitalPlane,
}

impl OrbitElements {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(parent: BodyId, shape: EllipticShape, plane: OrbitalPlane) -> Self {
        Self { parent, shape, plane }
    }

    /// Stationary barycentric anchor for the central solar star (`SOL`).
    pub const fn stationary_star() -> Self {
        Self::new(
            BodyId::SOL,
            EllipticShape::new(0.0, 0.0),
            OrbitalPlane::new(0.0, 0.0, 0.0),
        )
    }

    /// Secondary constructor assembling an inclined, focal-eccentric Keplerian orbit.
    pub const fn keplerian(
        parent: BodyId,
        semi_major_units: f64,
        eccentricity: f32,
        (inclination_rad, periapsis_rad, mean_anomaly_zero): (f32, f32, f32),
    ) -> Self {
        Self::new(
            parent,
            EllipticShape::new(semi_major_units, eccentricity),
            OrbitalPlane::new(inclination_rad, periapsis_rad, mean_anomaly_zero),
        )
    }

    pub const fn parent(self) -> BodyId {
        self.parent
    }

    pub fn heliocentric(self) -> bool {
        self.parent == BodyId::SOL
    }

    pub fn semi_major_meters(self, unit_span: f64) -> f64 {
        self.shape.scaled_semi_major(unit_span)
    }
}

impl OrbitalLocus for OrbitElements {
    fn sample_state(&self, semi_major: f64, parent_gm: f64, epoch_secs: f64) -> (DVec3, DVec3) {
        if semi_major <= 1e-3 || parent_gm <= 1e-6 {
            return (DVec3::ZERO, DVec3::ZERO);
        }
        let mean_motion = (parent_gm / semi_major).sqrt() / semi_major;
        let mean_anom = self.plane.mean_anomaly_at(mean_motion, epoch_secs);
        let (pos_pf, vel_pf) = self.shape.perifocal_kinematics(
            semi_major,
            mean_motion,
            self.plane.direction_sign(),
            mean_anom,
        );
        (self.plane.map_perifocal(pos_pf), self.plane.map_perifocal(vel_pf))
    }

    fn trace_ellipse(
        &self,
        focus: DVec3,
        semi_major: f64,
        segments: usize,
        sink: &mut dyn FnMut(DVec3, DVec3),
    ) {
        if semi_major <= 1.0 || segments == 0 {
            return;
        }
        let mut prev = focus + self.plane.map_perifocal(self.shape.perifocal_point(semi_major, 0.0));
        for step in 1..=segments {
            let ecc = (step as f64) * TAU / (segments as f64);
            let cur = focus + self.plane.map_perifocal(self.shape.perifocal_point(semi_major, ecc));
            sink(prev, cur);
            prev = cur;
        }
    }
}
