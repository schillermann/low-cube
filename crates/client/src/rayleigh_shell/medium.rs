use glam::Vec3;
use super::strata::{AirSample, AirStratum};

/// Focused contract intersecting a 3D ray against an axis-aligned centered cubic volume.
pub trait RaySlab: Copy {
    fn span(&self, origin: Vec3, dir: Vec3) -> Option<(f32, f32)>;
}

/// Centered cubic slab `[-half, +half]^3` (`<= 4` fields).
#[derive(Clone, Copy, Debug)]
pub struct CubicSlab {
    half: f32,
}

impl CubicSlab {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(half: f32) -> Self {
        Self { half }
    }
}

impl RaySlab for CubicSlab {
    fn span(&self, origin: Vec3, dir: Vec3) -> Option<(f32, f32)> {
        let (mut t0, mut t1) = (-1.0e9_f32, 1.0e9_f32);
        for axis in 0..3 {
            let (o, d) = (origin[axis], dir[axis]);
            if d.abs() < 1e-6 {
                if o.abs() > self.half {
                    return None;
                }
            } else {
                let inv = 1.0 / d;
                let mut a = (-self.half - o) * inv;
                let mut b = (self.half - o) * inv;
                if a > b {
                    std::mem::swap(&mut a, &mut b);
                }
                t0 = t0.max(a);
                t1 = t1.min(b);
                if t0 > t1 {
                    return None;
                }
            }
        }
        (t1 > 0.0).then_some((t0, t1))
    }
}

/// Focused contract evaluating `(scattered_rgb, optical_opacity)` along a 3D sight ray.
pub trait OpticalMedium: Copy {
    fn scatter_along(&self, cam_pos: Vec3, ray_dir: Vec3, sun_dir: Vec3) -> (Vec3, f32);
}

/// Null-object `OpticalMedium` representing airless space (`Luna`, `Mercury`, asteroids, deep vacuum).
#[derive(Clone, Copy, Debug)]
pub struct VacuumMedium;

impl VacuumMedium {
    /// Primary constructor doing nothing except returning the airless medium instance.
    pub const fn new() -> Self {
        Self
    }
}

impl OpticalMedium for VacuumMedium {
    fn scatter_along(&self, _cam_pos: Vec3, _ray_dir: Vec3, _sun_dir: Vec3) -> (Vec3, f32) {
        (Vec3::ZERO, 0.0)
    }
}

/// Penetrable 3D cubic atmospheric volume (`[-outer, +outer]^3`) integrating an `AirStratum` along sight rays.
#[derive(Clone, Copy, Debug)]
pub struct CubicAirVolume<S> {
    body_half: f32,
    shell_thickness: f32,
    stratum: S,
}

impl<S> CubicAirVolume<S> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(body_half: f32, shell_thickness: f32, stratum: S) -> Self {
        Self { body_half, shell_thickness, stratum }
    }
}

impl<S: AirStratum> OpticalMedium for CubicAirVolume<S> {
    fn scatter_along(&self, cam_pos: Vec3, ray_dir: Vec3, sun_dir: Vec3) -> (Vec3, f32) {
        if self.shell_thickness <= 1e-3 || self.body_half <= 0.5 {
            return (Vec3::ZERO, 0.0);
        }
        let dir = ray_dir.normalize_or_zero();
        let sun = sun_dir.normalize_or_zero();
        let outer = self.body_half + self.shell_thickness;
        let Some((raw_in, mut t_out)) = CubicSlab::new(outer).span(cam_pos, dir) else {
            return (Vec3::ZERO, 0.0);
        };
        let mut t_in = raw_in.max(0.0);
        let core_h = (self.body_half - 2.0).max(self.body_half * 0.85);
        if let Some((e_in, e_out)) = CubicSlab::new(core_h).span(cam_pos, dir) {
            if e_in > 0.0 {
                t_out = t_out.min(e_in);
            } else if e_out > 0.0 {
                t_in = t_in.max(e_out);
            }
        }
        let air_len = (t_out - t_in).max(0.0);
        if air_len <= 1e-3 {
            return (Vec3::ZERO, 0.0);
        }
        let dt = air_len * 0.25;
        let inv_thick = 1.0 / self.shell_thickness;
        let mut col_opt = 0.0_f32;
        let mut weighted_tint = Vec3::ZERO;
        let mut lit_weight = 0.0_f32;
        for step in 0..4 {
            let p = cam_pos + dir * (t_in + dt * (step as f32 + 0.5));
            let h = ((p.abs().max_element() - self.body_half) / self.shell_thickness).clamp(0.0, 1.0);
            let sample = AirSample::new(p, h, dir, sun);
            let (rho, local_col, lit) = self.stratum.sample_point(&sample);
            let d_opt = rho * (dt * inv_thick);
            col_opt += d_opt;
            weighted_tint += local_col * d_opt;
            lit_weight += lit * d_opt;
        }
        if col_opt <= 1e-4 {
            return (Vec3::ZERO, 0.0);
        }
        let tau = 1.0 - (-col_opt * 2.5).exp();
        let mean_col = weighted_tint / col_opt;
        let mean_lit = (lit_weight / col_opt).clamp(0.0, 1.0);
        (mean_col * tau, (tau * mean_lit).clamp(0.0, 1.0))
    }
}

/// `OpticalMedium` decorator brightening optically thick grazing horizon paths with pale Rayleigh haze.
#[derive(Clone, Copy, Debug)]
pub struct HorizonHazeMedium<M> {
    origin: M,
    haze_tint: Vec3,
}

impl<M> HorizonHazeMedium<M> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(origin: M, haze_tint: Vec3) -> Self {
        Self { origin, haze_tint }
    }

    /// Secondary constructor with pale atmospheric horizon lift.
    pub const fn pale(origin: M) -> Self {
        Self::new(origin, Vec3::new(0.10, 0.14, 0.18))
    }
}

impl<M: OpticalMedium> OpticalMedium for HorizonHazeMedium<M> {
    fn scatter_along(&self, cam_pos: Vec3, ray_dir: Vec3, sun_dir: Vec3) -> (Vec3, f32) {
        let (rgb, lit_tau) = self.origin.scatter_along(cam_pos, ray_dir, sun_dir);
        let horizon_lift = (lit_tau - 0.62).clamp(0.0, 0.38) * 0.45;
        (rgb + self.haze_tint * (horizon_lift * lit_tau), lit_tau)
    }
}

/// `OpticalMedium` decorator scaling scattered sky radiance by elliptical solar flux (`1.0` = mean 1 AU).
#[derive(Clone, Copy, Debug)]
pub struct SolarFluxMedium<M> {
    origin: M,
    flux: f32,
}

impl<M> SolarFluxMedium<M> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(origin: M, flux: f32) -> Self {
        Self { origin, flux }
    }
}

impl<M: OpticalMedium> OpticalMedium for SolarFluxMedium<M> {
    fn scatter_along(&self, cam_pos: Vec3, ray_dir: Vec3, sun_dir: Vec3) -> (Vec3, f32) {
        let (rgb, opacity) = self.origin.scatter_along(cam_pos, ray_dir, sun_dir);
        ((rgb * (1.42 * self.flux)).min(Vec3::ONE), opacity)
    }
}
