use glam::Vec3;
use crate::PlanetaryWorld;
use super::CelestialSystem;

/// Celestial body coordinate layout for cosmic sight-ray locking and holographic orrery projection.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CelestialAtlas {
    home_half: f32,
    moon_center: Vec3,
    moon_half: f32,
}

impl CelestialAtlas {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(home_half: f32, moon_center: Vec3, moon_half: f32) -> Self {
        Self { home_half, moon_center, moon_half }
    }

    /// Secondary constructor for solitary single-cube worlds without an active lunar companion.
    pub const fn solitary(home_half: f32) -> Self {
        let blocks = if home_half >= 4.0 { (home_half * 2.0) as u32 } else { 32 };
        Self::new(home_half, CelestialSystem::moon_center(blocks), 0.0)
    }

    /// Transfers `(home_half, moon_center, moon_half)` to a collaborating closure without getters.
    pub fn transfer<R>(self, f: impl FnOnce(f32, Vec3, f32) -> R) -> R {
        f(self.home_half, self.moon_center, self.moon_half)
    }

    fn slab_span(eye: Vec3, dir: Vec3, center: Vec3, bound: f32) -> Option<(f32, f32)> {
        let min_b = center - Vec3::splat(bound);
        let max_b = center + Vec3::splat(bound);
        let (mut t0, mut t1) = (0.0_f32, 4_096.0_f32);
        for axis in 0..3 {
            let o = eye[axis];
            let d = dir[axis];
            let (lo, hi) = (min_b[axis], max_b[axis]);
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

    /// Resolves the nearest Homeworld or Moon terrain point intersected by a cosmic camera sight ray.
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
        let home_hit = Self::slab_span(eye, dir, Vec3::ZERO, self.home_half + 6.0);
        let moon_hit = (self.moon_half > 0.0)
            .then(|| Self::slab_span(eye, dir, self.moon_center, self.moon_half + 4.0))
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
