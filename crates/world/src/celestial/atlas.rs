use glam::{DVec3, Vec3};
use crate::PlanetaryWorld;
use super::catalog::BodyId;
use super::ephemeris::{BodySnapshot, SolarEphemeris};
use super::CelestialSystem;

/// Polymorphic atlas topology avoiding `Option<SolarEphemeris>` fields.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AtlasMode {
    BinaryPocket,
    Heliosphere(SolarEphemeris),
}

/// Celestial body coordinate layout for cosmic sight-ray locking, `M` System Map, and `GlFirmament` raytracing.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CelestialAtlas {
    home_half: f32,
    moon_spec: (Vec3, f32),
    mode: AtlasMode,
}

impl CelestialAtlas {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn with_mode(home_half: f32, moon_spec: (Vec3, f32), mode: AtlasMode) -> Self {
        Self { home_half, moon_spec, mode }
    }

    /// Secondary constructor for two-body Earth-Moon worlds.
    pub const fn new(home_half: f32, moon_center: Vec3, moon_half: f32) -> Self {
        Self::with_mode(home_half, (moon_center, moon_half), AtlasMode::BinaryPocket)
    }

    /// Secondary constructor for solitary single-cube worlds without an active lunar companion.
    pub const fn solitary(home_half: f32) -> Self {
        let blocks = if home_half >= 4.0 { (home_half * 2.0) as u32 } else { 32 };
        Self::new(home_half, CelestialSystem::moon_center(blocks), 0.0)
    }

    /// Secondary constructor wrapping a live moving `SolarEphemeris` across all planets, moons, and Sun.
    pub fn heliosphere(ephemeris: SolarEphemeris) -> Self {
        let snaps = ephemeris.snapshot_all(0.0);
        let luna = snaps[4];
        Self::with_mode(
            ephemeris.proportions().earth_half(),
            (luna.center().as_vec3(), luna.half_size() as f32),
            AtlasMode::Heliosphere(ephemeris),
        )
    }

    /// Transfers `(home_half, moon_center, moon_half)` to a collaborating closure without getters.
    pub fn transfer<R>(self, f: impl FnOnce(f32, Vec3, f32) -> R) -> R {
        f(self.home_half, self.moon_spec.0, self.moon_spec.1)
    }

    pub const fn ephemeris(self) -> Option<SolarEphemeris> {
        match self.mode {
            AtlasMode::BinaryPocket => None,
            AtlasMode::Heliosphere(eph) => Some(eph),
        }
    }

    /// Resolves `(center, surface_point, surface_normal)` for a named `BodyId` at `delta_t` seconds ahead.
    pub fn target_on_body(self, id: BodyId, ship_pos: DVec3, delta_t: f64) -> Option<(BodySnapshot, DVec3, Vec3)> {
        let eph = self.ephemeris()?;
        let snaps = eph.snapshot_all(delta_t);
        let snap = snaps.into_iter().find(|s| s.id() == id)?;
        let approach = ship_pos - snap.center();
        let out_dir = if approach.length_squared() > 1e-4 {
            approach.normalize()
        } else {
            DVec3::Y
        };
        let norm_v3 = crate::ClimateZone::dominant(out_dir.as_vec3()).normal();
        let stand_dist = if snap.nature().landable_crust() {
            snap.half_size()
        } else {
            snap.half_size() * 1.45
        };
        let target_pt = snap.center() + norm_v3.as_dvec3() * stand_dist;
        Some((snap, target_pt, norm_v3))
    }

    fn slab_span_dvec(eye: DVec3, dir: DVec3, center: DVec3, bound: f64, max_range: f64) -> Option<(f64, f64)> {
        let min_b = center - DVec3::splat(bound);
        let max_b = center + DVec3::splat(bound);
        let (mut t0, mut t1) = (0.0_f64, max_range);
        for axis in 0..3 {
            let (o, d, lo, hi) = (eye[axis], dir[axis], min_b[axis], max_b[axis]);
            if d.abs() < 1e-9 {
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
        (t1 > 0.35).then_some((t0.max(0.35), t1))
    }

    fn march_surface(eye: Vec3, dir: Vec3, (t_start, t_end): (f32, f32), world: &dyn PlanetaryWorld) -> Option<(Vec3, Vec3)> {
        let step = 0.40_f32;
        let mut prev_t = t_start;
        let mut t = t_start;
        while t <= t_end + step {
            let clamped_t = t.min(t_end);
            let probe = eye + dir * clamped_t;
            let up = world.surface_up_at(probe);
            let ground = world.constrain_to_surface(probe, 0.0);
            if (probe - ground).dot(up) <= 0.04 {
                let (mut lo, mut hi) = (prev_t, clamped_t);
                for _ in 0..6 {
                    let mid = (lo + hi) * 0.5;
                    let mp = eye + dir * mid;
                    let m_up = world.surface_up_at(mp);
                    let m_ground = world.constrain_to_surface(mp, 0.0);
                    if (mp - m_ground).dot(m_up) <= 0.0 {
                        hi = mid;
                    } else {
                        lo = mid;
                    }
                }
                let hit_probe = eye + dir * hi;
                let surface_pt = world.constrain_to_surface(hit_probe, 0.0);
                let normal = world.surface_up_at(surface_pt).normalize_or_zero();
                return Some((surface_pt, normal));
            }
            if clamped_t >= t_end {
                break;
            }
            prev_t = clamped_t;
            t += step;
        }
        None
    }

    /// Resolves the nearest planetary, lunar, or solar cube point intersected by a cosmic sight ray.
    pub fn lock_sight_surface(
        self,
        eye: Vec3,
        direction: Vec3,
        world: &dyn PlanetaryWorld,
    ) -> Option<(Vec3, Vec3)> {
        let dir = direction.normalize_or_zero();
        if dir.length_squared() <= 0.25 {
            return None;
        }
        if let AtlasMode::Heliosphere(eph) = self.mode {
            let (eye_d, dir_d) = (eye.as_dvec3(), dir.as_dvec3());
            let max_r = eph.proportions().astronomical_unit() * 40.0;
            let mut nearest: Option<(f64, BodySnapshot, (f64, f64))> = None;
            for snap in eph.snapshot_all(0.0) {
                if snap.id() == BodyId::SOL {
                    continue;
                }
                let pad = if snap.nature().landable_crust() { 6.0 } else { 0.0 };
                if let Some(span) = Self::slab_span_dvec(eye_d, dir_d, snap.center(), snap.half_size() + pad, max_r) {
                    if nearest.map_or(true, |(best_t, _, _)| span.0 < best_t) {
                        nearest = Some((span.0, snap, span));
                    }
                }
            }
            if let Some((t_hit, snap, span)) = nearest {
                let near_body = (eye_d - snap.center()).abs().max_element() <= snap.half_size() + 384.0;
                if near_body && snap.nature().landable_crust() {
                    if let Some(hit) = Self::march_surface(eye, dir, (span.0 as f32, span.1 as f32), world) {
                        return Some(hit);
                    }
                }
                let exact_t = Self::slab_span_dvec(eye_d, dir_d, snap.center(), snap.half_size(), max_r)
                    .map_or(t_hit, |(t0, _)| t0);
                let hit_pt = eye_d + dir_d * exact_t;
                let rel = (hit_pt - snap.center()).as_vec3();
                let n = crate::ClimateZone::dominant(rel).normal();
                let clamped = (hit_pt - snap.center()).clamp(
                    DVec3::splat(-snap.half_size()),
                    DVec3::splat(snap.half_size()),
                );
                return Some(((snap.center() + clamped).as_vec3(), n));
            }
            return None;
        }

        let (eye_d, dir_d) = (eye.as_dvec3(), dir.as_dvec3());
        let home_hit = Self::slab_span_dvec(eye_d, dir_d, DVec3::ZERO, f64::from(self.home_half + 6.0), 1.0e8)
            .map(|(a, b)| (a as f32, b as f32));
        let moon_hit = (self.moon_spec.1 > 0.0)
            .then(|| {
                Self::slab_span_dvec(
                    eye_d,
                    dir_d,
                    self.moon_spec.0.as_dvec3(),
                    f64::from(self.moon_spec.1 + 4.0),
                    1.0e8,
                )
                .map(|(a, b)| (a as f32, b as f32))
            })
            .flatten();
        match (home_hit, moon_hit) {
            (Some(h), Some(m)) if m.0 < h.0 => {
                Self::march_surface(eye, dir, m, world).or_else(|| Self::march_surface(eye, dir, h, world))
            }
            (Some(h), Some(m)) => {
                Self::march_surface(eye, dir, h, world).or_else(|| Self::march_surface(eye, dir, m, world))
            }
            (Some(h), None) => Self::march_surface(eye, dir, h, world),
            (None, Some(m)) => Self::march_surface(eye, dir, m, world),
            (None, None) => None,
        }
    }
}
