use glam::Vec3;
use super::magnetosphere::{Magnetosphere, MagnetosphereEnvelope};
use super::medium::{CubicSlab, RaySlab};
use super::strata::{BandCurve, PlateauZenith, SmoothBand, ZenithField};

/// Axis-aligned 3D polar ionosphere slab (`[b_min, b_max]`, `<= 4` fields).
#[derive(Clone, Copy, Debug)]
struct PolarBoxSlab {
    min_corner: Vec3,
    max_corner: Vec3,
}

impl PolarBoxSlab {
    const fn new(min_corner: Vec3, max_corner: Vec3) -> Self {
        Self { min_corner, max_corner }
    }

    fn span(self, origin: Vec3, dir: Vec3) -> Option<(f32, f32)> {
        let (mut t0, mut t1) = (-1.0e9_f32, 1.0e9_f32);
        for axis in 0..3 {
            let (o, d) = (origin[axis], dir[axis]);
            let (lo, hi) = (self.min_corner[axis], self.max_corner[axis]);
            if d.abs() < 1e-6 {
                if o < lo || o > hi {
                    return None;
                }
            } else {
                let inv = 1.0 / d;
                let mut a = (lo - o) * inv;
                let mut b = (hi - o) * inv;
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

/// Focused contract integrating 3D polar auroral curtains along a sight ray.
pub trait IonosphereMedium: Copy {
    fn radiance_along(&self, cam_pos: Vec3, ray_dir: Vec3, sun_dir: Vec3) -> Vec3;
}

/// Penetrable 3D `Arctic (-X)` ionosphere volume with cubic `L^4` field-line curtains (`<= 4` fields).
#[derive(Clone, Copy, Debug)]
pub struct PolarIonosphere {
    body_half: f32,
    canopy_scales: (f32, f32),
    magnetosphere: Magnetosphere,
    activity_and_time: (f32, f32),
}

impl PolarIonosphere {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(
        body_half: f32,
        canopy_scales: (f32, f32),
        magnetosphere: Magnetosphere,
        activity_and_time: (f32, f32),
    ) -> Self {
        Self { body_half, canopy_scales, magnetosphere, activity_and_time }
    }

    /// Secondary constructor calibrated to a planetary `body_half` with active nocturnal storm factor.
    pub fn planetary(body_half: f32, magnetosphere: Magnetosphere, activity: f32, time: f32) -> Self {
        let radius = (body_half * 0.66).max(4.0);
        let ceiling = (body_half * 0.44).max(8.0);
        Self::new(body_half, (radius, ceiling), magnetosphere, (activity, time))
    }
}

impl IonosphereMedium for PolarIonosphere {
    fn radiance_along(&self, cam_pos: Vec3, ray_dir: Vec3, sun_dir: Vec3) -> Vec3 {
        let (lower, upper) = self.magnetosphere.transfer_bands(|l, u| (l, u));
        let strength = lower[3];
        let activity = self.activity_and_time.0;
        if strength <= 1e-4 || activity <= 0.01 || self.body_half <= 0.5 {
            return Vec3::ZERO;
        }
        let dir = ray_dir.normalize_or_zero();
        let sun = sun_dir.normalize_or_zero();
        let flare = upper[3];
        let oval_half = self.canopy_scales.0.max(self.body_half * 0.66);
        let iono_base = self.canopy_scales.1.max(self.body_half * 0.44);
        let iono_thick = (iono_base * 1.10).max(self.body_half * 0.52);
        let iono_top = iono_base + iono_thick;

        let lat_bound = self.body_half * (1.0 + (iono_top / self.body_half) * flare);
        let slab = PolarBoxSlab::new(
            Vec3::new(-(self.body_half + iono_top), -lat_bound, -lat_bound),
            Vec3::new(-(self.body_half + iono_base * 0.85), lat_bound, lat_bound),
        );
        let Some((raw_in, mut t_out)) = slab.span(cam_pos, dir) else {
            return Vec3::ZERO;
        };
        let mut t_in = raw_in.max(0.0);
        let core_h = (self.body_half - 1.5).max(self.body_half * 0.92);
        if let Some((c_in, c_out)) = CubicSlab::new(core_h).span(cam_pos, dir) {
            if c_in > 0.0 {
                t_out = t_out.min(c_in);
            } else if c_out > 0.0 {
                t_in = t_in.max(c_out);
            }
        }
        let seg_len = t_out - t_in;
        if seg_len <= 0.01 {
            return Vec3::ZERO;
        }

        let cam_cheb = cam_pos.abs().max_element();
        let cam_up = PlateauZenith::new().zenith_at(cam_pos);
        let ground_prox = 1.0
            - SmoothBand::new(self.body_half + 2.0, self.body_half + iono_base * 0.6).sample(cam_cheb);
        let horizon_gate = 1.0 - ground_prox
            + ground_prox * SmoothBand::new(-0.04, 0.08).sample(dir.dot(cam_up));
        if horizon_gate <= 0.001 {
            return Vec3::ZERO;
        }

        let midnight_tail = sun.x.clamp(0.0, 1.0);
        let drift = self.activity_and_time.1 * (0.18 + 0.06 * midnight_tail);
        let dt = seg_len * 0.20;
        let d_weight = (dt / iono_thick).min(0.35);
        let lower_rgb = Vec3::new(lower[0], lower[1], lower[2]);
        let upper_rgb = Vec3::new(upper[0], upper[1], upper[2]);
        let mut accum = Vec3::ZERO;

        for step in 0..5 {
            let p = cam_pos + dir * (t_in + dt * (step as f32 + 0.5));
            let alt = -p.x - self.body_half;
            let h_norm = ((alt - iono_base) / iono_thick).clamp(0.0, 1.0);
            let vert_env =
                SmoothBand::new(0.0, 0.16).sample(h_norm) * SmoothBand::new(1.0, 0.55).sample(h_norm);
            let mu_sun = PlateauZenith::new().zenith_at(p).dot(sun);
            let local_dark = 1.0 - SmoothBand::new(-0.22, 0.08).sample(mu_sun);
            if vert_env * local_dark <= 0.002 {
                continue;
            }
            let flare_scale = 1.0 + (alt.max(0.0) / self.body_half) * flare;
            let foot_y = p.y / flare_scale;
            let foot_z = p.z / flare_scale;
            let face_cheb = foot_y.abs().max(foot_z.abs()) / self.body_half;
            let face_gate = 1.0 - SmoothBand::new(0.80, 0.98).sample(face_cheb);
            if face_gate <= 0.001 {
                continue;
            }
            let qx = foot_y / oval_half;
            let qy = foot_z / oval_half;
            let dist_l4 = (qx.powi(4) + qy.powi(4)).powf(0.25);
            let oval_band = SmoothBand::new(0.20, 0.46).sample(dist_l4)
                * (1.0 - SmoothBand::new(1.02, 1.44).sample(dist_l4));
            if oval_band <= 0.002 {
                continue;
            }
            let angle = qy.atan2(qx);
            let wave1 = (angle * 4.0 + drift).sin() * 0.09 + (angle * 8.0 - drift * 0.65).cos() * 0.04;
            let wave2 = (angle * 4.0 - drift * 0.80 + 1.1).sin() * 0.10;
            let pillars = ((angle * 24.0 + dist_l4 * 14.0 + drift * 2.2).sin() * 0.5 + 0.5)
                .powf(2.5)
                * 0.78
                + 0.22;
            let c1 = (-(dist_l4 - 0.72 + wave1).abs() * 8.5).exp() * pillars;
            let c2 = (-(dist_l4 - 1.12 + wave2).abs() * 6.8).exp() * (0.58 * pillars + 0.20);
            let alt_mix = SmoothBand::new(0.18, 0.78).sample(h_norm + (dist_l4 - 0.75) * 0.22);
            let spectral = lower_rgb.lerp(upper_rgb, alt_mix);
            accum += spectral * ((c1 + c2) * oval_band * face_gate * vert_env * local_dark * d_weight);
        }

        let tail_surge = 1.0 + 0.28 * midnight_tail;
        accum * (activity * strength * tail_surge * horizon_gate * 2.6)
    }
}
