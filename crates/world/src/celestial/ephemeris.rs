use glam::{DVec3, Vec3};
use crate::{BeveledGravity, GravityField};
use super::catalog::{BodyId, OrbitalSpec, SolarCatalog, SurfaceNature, MAJOR_BODY_COUNT};
use super::proportions::SolarProportions;
use super::ZoneGravity;

/// Instantaneous position, velocity, and gravitational parameter of an orbiting body relative to Earth `(0, 0, 0)`.
#[derive(Clone, Copy, Debug)]
pub struct BodyKinematics {
    center: DVec3,
    velocity: DVec3,
    orbit_radius: f64,
    gm: f64,
}

impl BodyKinematics {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(center: DVec3, velocity: DVec3, orbit_radius: f64, gm: f64) -> Self {
        Self { center, velocity, orbit_radius, gm }
    }

    pub const fn center(self) -> DVec3 { self.center }
    pub const fn velocity(self) -> DVec3 { self.velocity }
    pub const fn orbit_radius(self) -> f64 { self.orbit_radius }
    pub const fn gm(self) -> f64 { self.gm }
}

/// Complete resolved state of a celestial body at a given cosmic epoch (`<= 4` fields).
#[derive(Clone, Copy, Debug)]
pub struct BodySnapshot {
    spec: OrbitalSpec,
    half_size: f64,
    kinematics: BodyKinematics,
}

impl BodySnapshot {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(spec: OrbitalSpec, half_size: f64, kinematics: BodyKinematics) -> Self {
        Self { spec, half_size, kinematics }
    }

    pub const fn id(self) -> BodyId { self.spec.id() }
    pub const fn name(self) -> &'static str { self.spec.name() }
    pub const fn parent_id(self) -> BodyId { self.spec.orbit().parent() }
    pub const fn nature(self) -> SurfaceNature { self.spec.physique().nature() }
    pub const fn palette(self) -> (Vec3, Vec3) { self.spec.physique().palette() }
    pub const fn surface_g(self) -> f32 { self.spec.physique().surface_g() }
    pub const fn half_size(self) -> f64 { self.half_size }
    pub const fn center(self) -> DVec3 { self.kinematics.center() }
    pub const fn velocity(self) -> DVec3 { self.kinematics.velocity() }
    pub const fn orbit_radius(self) -> f64 { self.kinematics.orbit_radius() }
    pub const fn gm(self) -> f64 { self.kinematics.gm() }
}

/// Moving Keplerian ephemeris and exact `1/r²` N-body gravitational field in Earth-centered inertial coordinates.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SolarEphemeris {
    proportions: SolarProportions,
    epoch_secs: f64,
}

impl SolarEphemeris {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(proportions: SolarProportions, epoch_secs: f64) -> Self {
        Self { proportions, epoch_secs }
    }

    pub const fn proportions(self) -> SolarProportions { self.proportions }
    pub const fn epoch_secs(self) -> f64 { self.epoch_secs }

    pub const fn advanced(self, dt: f64) -> Self {
        Self::new(self.proportions, self.epoch_secs + dt)
    }

    fn body_half_and_gm(self, spec: OrbitalSpec) -> (f64, f64) {
        let half = if spec.id() == BodyId::SOL {
            self.proportions.sun_half()
        } else if spec.id() == BodyId::EARTH {
            f64::from(self.proportions.earth_half())
        } else if spec.physique().nature().landable_crust() {
            f64::from(self.proportions.voxel_blocks(spec.physique().diameter_ratio())) * 0.5
        } else {
            self.proportions.body_half(f64::from(spec.physique().diameter_ratio()))
        };
        let g = f64::from(spec.physique().surface_g());
        (half, g * half * half)
    }

    fn circular_orbit(radius: f64, parent_gm: f64, tilt: f64, phase0: f64, t: f64) -> (DVec3, DVec3) {
        if radius <= 1e-3 || parent_gm <= 1e-6 {
            return (DVec3::ZERO, DVec3::ZERO);
        }
        let speed = (parent_gm / radius).sqrt();
        let omega = speed / radius;
        let retrograde = if tilt < -0.2 { -1.0 } else { 1.0 };
        let angle = phase0 + retrograde * omega * t;
        let (sin_a, cos_a) = angle.sin_cos();
        let (sin_i, cos_i) = tilt.abs().sin_cos();
        let pos = DVec3::new(radius * cos_a, radius * sin_a * sin_i, radius * sin_a * cos_i);
        let vel = DVec3::new(
            -speed * sin_a * retrograde,
            speed * cos_a * sin_i * retrograde,
            speed * cos_a * cos_i * retrograde,
        );
        (pos, vel)
    }

    /// Resolves all 21 major bodies at `self.epoch_secs + delta_t` in Earth-centered inertial `DVec3` coordinates.
    pub fn snapshot_all(self, delta_t: f64) -> [BodySnapshot; MAJOR_BODY_COUNT] {
        let catalog = SolarCatalog::new();
        let entries = catalog.entries();
        let t = self.epoch_secs + delta_t;
        let au = self.proportions.astronomical_unit();
        let (_, sun_gm) = self.body_half_and_gm(entries[0]);

        let mut helio_pos = [DVec3::ZERO; MAJOR_BODY_COUNT];
        let mut helio_vel = [DVec3::ZERO; MAJOR_BODY_COUNT];
        let mut radii = [0.0_f64; MAJOR_BODY_COUNT];
        let mut halves = [0.0_f64; MAJOR_BODY_COUNT];
        let mut gms = [0.0_f64; MAJOR_BODY_COUNT];

        for (i, &spec) in entries.iter().enumerate() {
            let (half, gm) = self.body_half_and_gm(spec);
            halves[i] = half;
            gms[i] = gm;
            if spec.id() == BodyId::SOL {
                continue;
            }
            if spec.orbit().parent() == BodyId::SOL {
                let r = spec.orbit().orbit_units() * au;
                radii[i] = r;
                let (p, v) = Self::circular_orbit(
                    r,
                    sun_gm,
                    f64::from(spec.orbit().inclination_rad()),
                    f64::from(spec.orbit().phase_rad()),
                    t,
                );
                helio_pos[i] = p;
                helio_vel[i] = v;
            }
        }

        let d_scale = f64::from(self.proportions.distance_scale().max(0.0005));
        for (i, &spec) in entries.iter().enumerate() {
            let parent = spec.orbit().parent();
            if parent != BodyId::SOL {
                if let Some(p_idx) = entries.iter().position(|s| s.id() == parent) {
                    let min_clear = halves[p_idx] * 1.6 + halves[i] * 1.6 + 8.0;
                    let r = (spec.orbit().orbit_units() * halves[p_idx] * 2.0 * d_scale).max(min_clear);
                    radii[i] = r;
                    let (lp, lv) = Self::circular_orbit(
                        r,
                        gms[p_idx],
                        f64::from(spec.orbit().inclination_rad()),
                        f64::from(spec.orbit().phase_rad()),
                        t,
                    );
                    helio_pos[i] = helio_pos[p_idx] + lp;
                    helio_vel[i] = helio_vel[p_idx] + lv;
                }
            }
        }

        let earth_p = helio_pos[3];
        let earth_v = helio_vel[3];
        std::array::from_fn(|i| {
            BodySnapshot::new(
                entries[i],
                halves[i],
                BodyKinematics::new(helio_pos[i] - earth_p, helio_vel[i] - earth_v, radii[i], gms[i]),
            )
        })
    }

    pub fn sun_direction_from(self, world_pos: DVec3, delta_t: f64) -> Vec3 {
        let bodies = self.snapshot_all(delta_t);
        let to_sun = bodies[0].center() - world_pos;
        if to_sun.length_squared() > 1e-6 {
            to_sun.normalize().as_vec3()
        } else {
            Vec3::Y
        }
    }

    /// Exact N-body gravitational acceleration (`m/s²`) at `pos` (`epoch + delta_t`) with tidal Earth-frame compensation.
    pub fn gravity_accel_at(self, pos: DVec3, delta_t: f64) -> DVec3 {
        let bodies = self.snapshot_all(delta_t);
        let sun_from_earth = bodies[0].center();
        let r_se_sq = sun_from_earth.length_squared().max(1.0);
        let earth_solar_accel = sun_from_earth * (bodies[0].gm() / (r_se_sq * r_se_sq.sqrt()));

        let mut total = -earth_solar_accel;
        for b in &bodies {
            let rel = pos - b.center();
            let cheb = rel.abs().max_element();
            let h = b.half_size();
            let knee = (h * 0.35).clamp(4.0, 24.0);
            if b.nature().landable_crust() && cheb <= h + knee {
                let bevel = BeveledGravity::decorating(ZoneGravity::new(), h as f32, 1.0);
                let dir = bevel.pull_direction_at(rel.as_vec3()).as_dvec3();
                total += dir * f64::from(b.surface_g());
            } else {
                let dist_sq = rel.length_squared().max(h * h);
                let dist = dist_sq.sqrt();
                if dist > 1e-4 {
                    total -= rel * (b.gm() / (dist_sq * dist));
                }
            }
        }
        total
    }

    /// Dominant local surface-up vector (`ZoneGravity` near planetary crusts, radial outward in deep space).
    pub fn dominant_up_at(self, pos: DVec3) -> Vec3 {
        let bodies = self.snapshot_all(0.0);
        let mut strongest = (0.0_f64, Vec3::Y);
        for b in &bodies {
            let rel = pos - b.center();
            let cheb = rel.abs().max_element();
            let h = b.half_size();
            if b.nature().landable_crust() && cheb <= h + 48.0 {
                let bevel = BeveledGravity::decorating(ZoneGravity::new(), h as f32, 1.0);
                return bevel.surface_up_at(rel.as_vec3());
            }
            let r_sq = rel.length_squared().max(h * h);
            let pull = b.gm() / r_sq;
            if pull > strongest.0 && rel.length_squared() > 1e-6 {
                strongest = (pull, rel.normalize().as_vec3());
            }
        }
        strongest.1
    }
}
