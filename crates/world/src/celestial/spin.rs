use std::f32::consts::TAU as TAU_F32;
use std::f64::consts::TAU;
use glam::{DQuat, DVec3, Quat, Vec3};
use super::brad64::Brad64;
use super::ephemeris::SolarEphemeris;

const INV_SQRT_3_F64: f64 = 0.577_350_269_189_625_8;
const INV_SQRT_3_F32: f32 = 0.577_350_26;
const DIURNAL_DAY_SECS: f64 = 1_200.0;

/// Focused contract for planetary spin co-rotation and distant orbital `QuadSphere` posing.
pub trait PlanetarySpin {
    /// Unit spin axis pointing out of the cool northern pole (`Arctic -X`, `Temperate +Y`, `Steppe -Z`).
    fn north_apex(&self) -> Vec3;

    /// Apparent 3D solar center seen from `viewer`, co-rotating near the planet and inertial in deep space.
    fn apparent_sun_center(&self, inertial_sun: DVec3, viewer: DVec3) -> DVec3;

    /// Rotates a homeworld-local vector by the live diurnal phase for distant orbital observation of Earth.
    fn pose_distant_earth(&self, local_vec: Vec3) -> Vec3;

    /// Resolves `(celestial_pole, local_solar_progress)` for `surface_up` under `sun_dir`.
    fn diurnal_meridian(&self, sun_dir: Vec3, surface_up: Vec3, fallback_progress: f32) -> (Vec3, f32);

    /// Advances the spin epoch by `turns` diurnal revolutions (`1.0` = one full solar day).
    fn epoch_after_turns(&self, turns: f64) -> f64;
}

/// Diurnal rotation of the `QuadSphere` homeworld around `(-1, +1, -1)` ↔ `(+1, -1, +1)` (`<= 4` fields).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ApexSpin {
    epoch_secs: f64,
    period_secs: f64,
    earth_half: f64,
    earth_center: DVec3,
}

impl ApexSpin {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn with_center(epoch_secs: f64, period_secs: f64, earth_half: f64, earth_center: DVec3) -> Self {
        Self { epoch_secs, period_secs, earth_half, earth_center }
    }

    pub const fn new(epoch_secs: f64, period_secs: f64, earth_half: f64) -> Self {
        Self::with_center(epoch_secs, period_secs, earth_half, DVec3::ZERO)
    }

    /// Secondary constructor representing the terrestrial 20-minute spin reference.
    pub const fn terrestrial() -> Self {
        Self::new(0.0, DIURNAL_DAY_SECS, 64.0)
    }

    /// Secondary constructor calibrating the 20-minute diurnal cycle to `ephemeris` proportions and frame.
    pub fn from_ephemeris(ephemeris: SolarEphemeris) -> Self {
        let props = ephemeris.proportions();
        let au = props.astronomical_unit();
        let sun_half = props.sun_half();
        let sun_gm = (274.0 * sun_half * sun_half).max(1.0);
        let year_secs = TAU * (au * au * au / sun_gm).sqrt();
        let period = DIURNAL_DAY_SECS.min((year_secs * 0.125).max(15.0));
        let earth_c = if ephemeris.frame().earth_centered() {
            DVec3::ZERO
        } else {
            let bubble = super::bubble::GravityBubble::from_ephemeris(ephemeris);
            let ep = bubble.heliocentric_cosmic_pos(super::bubble::CosmicFrame::earth());
            let fp = bubble.heliocentric_cosmic_pos(ephemeris.frame());
            ep.relative_dvec3(fp)
        };
        Self::with_center(ephemeris.epoch_secs(), period, f64::from(props.earth_half()), earth_c)
    }

    fn spin_angle_rad(self) -> f64 {
        let dur = self.period_secs.max(1.0);
        let turns = self.epoch_secs / dur;
        (turns - turns.floor()) * TAU
    }

    fn corotation_weight(self, viewer: DVec3) -> f64 {
        let alt = (viewer - self.earth_center).length() - self.earth_half;
        let inner = (self.earth_half * 3.0).max(96.0);
        let outer = (self.earth_half * 24.0).max(768.0);
        if alt <= inner {
            1.0
        } else if alt >= outer {
            0.0
        } else {
            let u = (alt - inner) / (outer - inner);
            1.0 - u * u * (3.0 - 2.0 * u)
        }
    }

    fn axis_quat_deterministic(axis: DVec3, angle: f64) -> DQuat {
        let (sin_h, cos_h) = Brad64::from_radians(angle * 0.5).sin_cos_f64();
        DQuat::from_xyzw(axis.x * sin_h, axis.y * sin_h, axis.z * sin_h, cos_h)
    }
}

impl PlanetarySpin for ApexSpin {
    fn north_apex(&self) -> Vec3 {
        Vec3::new(-INV_SQRT_3_F32, INV_SQRT_3_F32, -INV_SQRT_3_F32)
    }

    fn apparent_sun_center(&self, inertial_sun: DVec3, viewer: DVec3) -> DVec3 {
        let weight = self.corotation_weight(viewer);
        if weight <= 1e-6 {
            return inertial_sun;
        }
        let axis = DVec3::new(-INV_SQRT_3_F64, INV_SQRT_3_F64, -INV_SQRT_3_F64);
        let angle = -self.spin_angle_rad() * weight;
        self.earth_center + Self::axis_quat_deterministic(axis, angle) * (inertial_sun - self.earth_center)
    }

    fn pose_distant_earth(&self, local_vec: Vec3) -> Vec3 {
        let (sin_h, cos_h) = Brad64::from_radians(self.spin_angle_rad() * 0.5).sin_cos_f64();
        let a = self.north_apex();
        let rot = Quat::from_xyzw(a.x * (sin_h as f32), a.y * (sin_h as f32), a.z * (sin_h as f32), cos_h as f32);
        rot * local_vec
    }

    fn diurnal_meridian(&self, sun_dir: Vec3, surface_up: Vec3, fallback_progress: f32) -> (Vec3, f32) {
        let pole = self.north_apex();
        let meridian = surface_up - pole * surface_up.dot(pole);
        let east = pole.cross(surface_up);
        let local_prog = if east.length_squared() > 1e-4 {
            let rad = Brad64::atan2_radians(f64::from(sun_dir.dot(meridian)), f64::from(sun_dir.dot(east))) as f32;
            (rad / TAU_F32).clamp(0.0, 1.0)
        } else {
            fallback_progress
        };
        (pole, local_prog)
    }

    fn epoch_after_turns(&self, turns: f64) -> f64 {
        self.epoch_secs + self.period_secs * turns
    }
}
