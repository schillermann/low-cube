use glam::Vec3;
use engine::GraphicPipeline;
use world::{BodyId, CelestialAtlas};

/// Analytical rounded-cubic Rayleigh atmospheric shell surrounding the Earth World-Cube.
#[cfg(test)]
#[derive(Clone, Copy, Debug)]
pub struct CubicRayleighShell {
    earth_half: f32,
    shell_thickness: f32,
}

#[cfg(test)]
impl CubicRayleighShell {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(earth_half: f32, shell_thickness: f32) -> Self {
        Self { earth_half, shell_thickness }
    }

    /// Calibrates the visible orbital atmosphere height from Earth's half-size.
    pub fn around_earth(earth_half: f32) -> Self {
        Self::new(earth_half, (earth_half * 0.30).max(10.0))
    }

    fn box_span(origin: Vec3, dir: Vec3, half: f32) -> Option<(f32, f32)> {
        let (mut t0, mut t1) = (-1.0e9_f32, 1.0e9_f32);
        for axis in 0..3 {
            let (o, d) = (origin[axis], dir[axis]);
            if d.abs() < 1e-6 {
                if o.abs() > half {
                    return None;
                }
            } else {
                let inv = 1.0 / d;
                let mut a = (-half - o) * inv;
                let mut b = (half - o) * inv;
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
        (t1 > 0.0).then_some((t0.max(0.0), t1))
    }

    fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
        let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
        t * t * (3.0 - 2.0 * t)
    }

    /// Evaluates `(scattered_rgb, optical_opacity)` along a cosmic or orbital sight ray `(cam_pos, ray_dir)`.
    pub fn sample_scatter(self, cam_pos: Vec3, ray_dir: Vec3, sun_dir: Vec3) -> (Vec3, f32) {
        let dir = ray_dir.normalize_or_zero();
        let outer = self.earth_half + self.shell_thickness;
        let Some((t_in, mut t_out)) = Self::box_span(cam_pos, dir, outer) else {
            return (Vec3::ZERO, 0.0);
        };
        if let Some((e_in, e_out)) = Self::box_span(cam_pos, dir, self.earth_half) {
            if e_in > 0.0 {
                t_out = t_out.min(e_in);
            } else if e_out > 0.0 {
                return (Vec3::ZERO, 0.0);
            }
        }
        let air_len = (t_out - t_in).max(0.0);
        if air_len <= 1e-3 {
            return (Vec3::ZERO, 0.0);
        }
        let mid = cam_pos + dir * (0.5 * (t_in + t_out));
        let excess = (mid.abs() - Vec3::splat(self.earth_half)).max(Vec3::ZERO);
        let alt = excess.length();
        if alt >= self.shell_thickness {
            return (Vec3::ZERO, 0.0);
        }
        let scale_h = (self.shell_thickness * 0.34).max(1.0);
        let rho = (-alt / scale_h).exp() * Self::smoothstep(self.shell_thickness, 0.0, alt);
        let tau = (rho * (air_len / self.shell_thickness.max(1.0)) * 0.90).clamp(0.0, 1.0);
        let n_mid = mid.normalize_or_zero();
        let mu = n_mid.dot(sun_dir.normalize_or_zero());
        let day = Self::smoothstep(-0.12, 0.35, mu);
        let twilight = (-((mu - 0.02) / 0.18).powi(2)).exp() * Self::smoothstep(-0.32, -0.02, mu);
        let azure = Vec3::new(0.18, 0.56, 1.00) * day;
        let sunset = Vec3::new(1.00, 0.38, 0.10) * (twilight * 0.88);
        let glow = (azure + sunset) * (tau * 1.35);
        (glow, (tau * (day + twilight * 0.6)).clamp(0.0, 1.0))
    }
}

/// Analytical 3D cubic Sun ray intersection and square solar-eclipse occultation.
#[cfg(test)]
#[derive(Clone, Copy, Debug)]
pub struct CubicSolarBody {
    sun_dir: Vec3,
    pole: Vec3,
    angular_half: f32,
}

#[cfg(test)]
impl CubicSolarBody {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(sun_dir: Vec3, pole: Vec3, angular_half: f32) -> Self {
        Self { sun_dir, pole, angular_half }
    }

    /// Returns true when `ray_dir` strikes the square/cubic solar face (`max(|u|, |v|) <= angular_half`).
    pub fn strikes_cube(self, ray_dir: Vec3) -> bool {
        let dir = ray_dir.normalize_or_zero();
        let fwd = self.sun_dir.normalize_or_zero();
        if dir.dot(fwd) <= 0.85 {
            return false;
        }
        let right = fwd.cross(self.pole).normalize_or_zero();
        let up = right.cross(fwd).normalize_or_zero();
        let u = (dir - fwd).dot(right).abs();
        let v = (dir - fwd).dot(up).abs();
        u.max(v) <= self.angular_half
    }

    /// Evaluates core solar transmittance (`0.0` = full square eclipse umbra, `1.0` = unoccluded solar face).
    pub fn eclipse_transmittance(self, ray_dir: Vec3, occluder: [f32; 4]) -> f32 {
        let occ_half = occluder[3];
        if occ_half <= 0.0005 {
            return 1.0;
        }
        let dir = ray_dir.normalize_or_zero();
        let occ_dir = Vec3::new(occluder[0], occluder[1], occluder[2]).normalize_or_zero();
        if dir.dot(occ_dir) <= 0.92 {
            return 1.0;
        }
        let right = occ_dir.cross(self.pole).normalize_or_zero();
        let up = right.cross(occ_dir).normalize_or_zero();
        let box_d = (dir - occ_dir).dot(right).abs().max((dir - occ_dir).dot(up).abs());
        let t = ((box_d - occ_half * 0.92) / (occ_half * 0.08).max(1e-5)).clamp(0.0, 1.0);
        let smooth = t * t * (3.0 - 2.0 * t);
        1.0 - (1.0 - smooth) * 0.985
    }
}

/// Compact real-time planet directions `[MERCURY, VENUS, MARS, JUPITER, SATURN, URANUS, NEPTUNE]` & eclipse occluder.
#[derive(Clone, Copy, Debug)]
pub struct FirmamentPlanets {
    planet_cubes: [[f32; 4]; 7],
    occluder: [f32; 4],
}

impl FirmamentPlanets {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(planet_cubes: [[f32; 4]; 7], occluder: [f32; 4]) -> Self {
        Self { planet_cubes, occluder }
    }

    pub const fn absent() -> Self {
        Self::new([[0.0; 4]; 7], [0.0; 4])
    }

    /// Resolves exact viewer-centric planet directions and nearest foreground solar-eclipse occluder from `atlas`.
    pub fn from_atlas(atlas: CelestialAtlas, viewer: Vec3) -> Self {
        let Some(eph) = atlas.ephemeris() else {
            return Self::absent();
        };
        let viewer_d = viewer.as_dvec3();
        let snaps = eph.snapshot_all(0.0);
        let to_sun = snaps[0].center() - viewer_d;
        let sun_dist = to_sun.length().max(1.0);
        let sun_dir = (to_sun / sun_dist).as_vec3();

        let order = [
            BodyId::MERCURY,
            BodyId::VENUS,
            BodyId::MARS,
            BodyId::JUPITER,
            BodyId::SATURN,
            BodyId::URANUS,
            BodyId::NEPTUNE,
        ];
        let mut cubes = [[0.0_f32; 4]; 7];
        for (slot, target_id) in order.into_iter().enumerate() {
            if let Some(s) = snaps.iter().find(|b| b.id() == target_id) {
                let rel = s.center() - viewer_d;
                let dist = rel.length().max(1.0);
                let dir = (rel / dist).as_vec3();
                let half_ang = ((s.half_size() / dist) as f32).clamp(0.0020, 0.14);
                cubes[slot] = [dir.x, dir.y, dir.z, half_ang];
            }
        }

        let mut best_occ = (-2.0_f32, [0.0_f32; 4]);
        for s in snaps.iter().skip(1) {
            let rel = s.center() - viewer_d;
            let dist = rel.length();
            if dist <= s.half_size() * 1.05 || dist >= sun_dist * 0.98 {
                continue;
            }
            let dir = (rel / dist).as_vec3();
            let align = dir.dot(sun_dir);
            if align > best_occ.0 {
                let half_ang = ((s.half_size() / dist) as f32).clamp(0.0018, 0.38);
                best_occ = (align, [dir.x, dir.y, dir.z, half_ang]);
            }
        }
        Self::new(cubes, best_occ.1)
    }

    pub fn sync_pipeline(&self, pipeline: &dyn GraphicPipeline) {
        pipeline.sync_ephemeris(&self.planet_cubes, self.occluder);
    }

    #[cfg(test)]
    pub fn transfer<R>(self, f: impl FnOnce(&[[f32; 4]; 7], [f32; 4]) -> R) -> R {
        f(&self.planet_cubes, self.occluder)
    }
}
