use glam::{DVec3, Vec3};
use crate::CosmicPos;
use super::bubble::{CosmicFrame, GravityBubble};
use super::catalog::{BodyId, OrbitalSpec, SolarCatalog, MAJOR_BODY_COUNT};
use super::kepler::OrbitalLocus;
use super::proportions::SolarProportions;
use super::spin::{ApexSpin, PlanetarySpin};

/// Instantaneous `CosmicPos` (`0.1 mm` steps), velocity, and gravitational parameter of an orbiting body (`4` fields).
#[derive(Clone, Copy, Debug)]
pub struct BodyKinematics {
    locus: CosmicPos,
    velocity: DVec3,
    orbit_radius: f64,
    gm: f64,
}

impl BodyKinematics {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(locus: CosmicPos, velocity: DVec3, orbit_radius: f64, gm: f64) -> Self {
        Self { locus, velocity, orbit_radius, gm }
    }

    pub const fn cosmic_locus(self) -> CosmicPos { self.locus }
    pub fn center(self) -> DVec3 { self.locus.to_dvec3() }
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
    pub const fn nature(self) -> super::catalog::SurfaceNature { self.spec.physique().nature() }
    pub const fn palette(self) -> (Vec3, Vec3) { self.spec.physique().palette() }
    pub const fn surface_g(self) -> f32 { self.spec.physique().surface_g() }
    pub const fn half_size(self) -> f64 { self.half_size }
    pub const fn cosmic_locus(self) -> CosmicPos { self.kinematics.cosmic_locus() }
    pub fn center(self) -> DVec3 { self.kinematics.center() }
    pub fn camera_relative(self, camera: CosmicPos) -> Vec3 { self.kinematics.cosmic_locus().relative_f32(camera) }
    pub const fn velocity(self) -> DVec3 { self.kinematics.velocity() }
    pub const fn orbit_radius(self) -> f64 { self.kinematics.orbit_radius() }
    pub const fn gm(self) -> f64 { self.kinematics.gm() }
}

/// Cross-OS deterministic Keplerian ephemeris (`i64` `0.1 mm` `CosmicPos`) and `1/r²` N-body field (`<= 4` fields).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SolarEphemeris {
    proportions: SolarProportions,
    epoch_secs: f64,
    frame: CosmicFrame,
}

impl SolarEphemeris {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn with_frame(proportions: SolarProportions, epoch_secs: f64, frame: CosmicFrame) -> Self {
        Self { proportions, epoch_secs, frame }
    }

    /// Secondary constructor anchored to `CosmicFrame::earth()`.
    pub const fn new(proportions: SolarProportions, epoch_secs: f64) -> Self {
        Self::with_frame(proportions, epoch_secs, CosmicFrame::earth())
    }

    pub const fn proportions(self) -> SolarProportions { self.proportions }
    pub const fn epoch_secs(self) -> f64 { self.epoch_secs }
    pub const fn frame(self) -> CosmicFrame { self.frame }

    pub const fn advanced(self, dt: f64) -> Self {
        Self::with_frame(self.proportions, self.epoch_secs + dt, self.frame)
    }

    pub const fn at_epoch(self, epoch_secs: f64) -> Self {
        Self::with_frame(self.proportions, epoch_secs, self.frame)
    }

    pub const fn reanchored(self, frame: CosmicFrame) -> Self {
        Self::with_frame(self.proportions, self.epoch_secs, frame)
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

    pub(super) fn solar_gm(self) -> f64 {
        let entries = SolarCatalog::new().entries();
        self.body_half_and_gm(entries[0]).1
    }

    #[allow(clippy::type_complexity)]
    fn heliocentric_tables(
        self,
        delta_t: f64,
    ) -> (
        [OrbitalSpec; MAJOR_BODY_COUNT],
        [CosmicPos; MAJOR_BODY_COUNT],
        [DVec3; MAJOR_BODY_COUNT],
        [f64; MAJOR_BODY_COUNT],
        [f64; MAJOR_BODY_COUNT],
        [f64; MAJOR_BODY_COUNT],
    ) {
        let entries = *SolarCatalog::new().entries();
        let t = self.epoch_secs + delta_t;
        let au = self.proportions.astronomical_unit();
        let (_, sun_gm) = self.body_half_and_gm(entries[0]);

        let mut helio_locus = [CosmicPos::origin(); MAJOR_BODY_COUNT];
        let mut helio_vel = [DVec3::ZERO; MAJOR_BODY_COUNT];
        let mut semi_majors = [0.0_f64; MAJOR_BODY_COUNT];
        let mut halves = [0.0_f64; MAJOR_BODY_COUNT];
        let mut gms = [0.0_f64; MAJOR_BODY_COUNT];

        for (i, &spec) in entries.iter().enumerate() {
            let (half, gm) = self.body_half_and_gm(spec);
            halves[i] = half;
            gms[i] = gm;
            if spec.id() != BodyId::SOL && spec.orbit().heliocentric() {
                let a = spec.orbit().semi_major_meters(au);
                semi_majors[i] = a;
                let (p, v) = spec.orbit().sample_state(a, sun_gm, t);
                helio_locus[i] = CosmicPos::from_dvec3(p);
                helio_vel[i] = v;
            }
        }

        let d_scale = f64::from(self.proportions.distance_scale().max(0.0005));
        for (i, &spec) in entries.iter().enumerate() {
            let orbit = spec.orbit();
            if !orbit.heliocentric() {
                if let Some(p_idx) = entries.iter().position(|s| s.id() == orbit.parent()) {
                    let min_clear = halves[p_idx] * 1.6 + halves[i] * 1.6 + 8.0;
                    let a = orbit.semi_major_meters(halves[p_idx] * 2.0 * d_scale).max(min_clear);
                    semi_majors[i] = a;
                    let (lp, lv) = orbit.sample_state(a, gms[p_idx], t);
                    helio_locus[i] = helio_locus[p_idx].translated(CosmicPos::from_dvec3(lp));
                    helio_vel[i] = helio_vel[p_idx] + lv;
                }
            }
        }
        (entries, helio_locus, helio_vel, semi_majors, halves, gms)
    }

    pub(super) fn body_heliocentric_cosmic(self, id: BodyId) -> (CosmicPos, DVec3, DVec3) {
        if id == BodyId::SOL {
            return (CosmicPos::origin(), DVec3::ZERO, DVec3::ZERO);
        }
        let (entries, helio_locus, helio_vel, _, _, gms) = self.heliocentric_tables(0.0);
        let Some(idx) = entries.iter().position(|s| s.id() == id) else {
            return (CosmicPos::origin(), DVec3::ZERO, DVec3::ZERO);
        };
        let p = helio_locus[idx].to_dvec3();
        let r_sq = p.length_squared().max(1.0);
        let mut accel = -p * (gms[0] / (r_sq * r_sq.sqrt()));
        let parent = entries[idx].orbit().parent();
        if parent != BodyId::SOL {
            if let Some(p_idx) = entries.iter().position(|s| s.id() == parent) {
                let rel = helio_locus[idx].relative_dvec3(helio_locus[p_idx]);
                let rp_sq = rel.length_squared().max(1.0);
                accel -= rel * (gms[p_idx] / (rp_sq * rp_sq.sqrt()));
            }
        }
        (helio_locus[idx], helio_vel[idx], accel)
    }

    /// Resolves all 21 major bodies at `self.epoch_secs + delta_t` in `self.frame` `CosmicPos` (`0.1 mm`) coordinates.
    pub fn snapshot_all(self, delta_t: f64) -> [BodySnapshot; MAJOR_BODY_COUNT] {
        let (entries, helio_locus, helio_vel, semi_majors, halves, gms) = self.heliocentric_tables(delta_t);
        let bubble = GravityBubble::new(self.proportions, self.epoch_secs + delta_t);
        let (frame_locus, frame_v, _) = bubble.heliocentric_cosmic_state(self.frame);
        std::array::from_fn(|i| {
            BodySnapshot::new(
                entries[i],
                halves[i],
                BodyKinematics::new(helio_locus[i].delta_from(frame_locus), helio_vel[i] - frame_v, semi_majors[i], gms[i]),
            )
        })
    }

    /// Cross-OS deterministic 64-bit FNV-1a digest over all 21 `[i64; 3]` `CosmicPos` ticks and quantized velocities.
    pub fn deterministic_digest(self) -> u64 {
        let snaps = self.snapshot_all(0.0);
        let mut hash = 0xCBF2_9CE4_8422_2325_u64;
        for s in snaps {
            for axis in s.cosmic_locus().ticks() {
                for b in axis.to_le_bytes() {
                    hash ^= u64::from(b);
                    hash = hash.wrapping_mul(0x0100_0000_01B3);
                }
            }
            let v = s.velocity();
            for va in [v.x, v.y, v.z] {
                let qv = (va * 10_000.0).round() as i64;
                for b in qv.to_le_bytes() {
                    hash ^= u64::from(b);
                    hash = hash.wrapping_mul(0x0100_0000_01B3);
                }
            }
        }
        hash
    }

    pub fn sun_direction_from(self, world_pos: DVec3, delta_t: f64) -> Vec3 {
        let stepped = self.advanced(delta_t);
        let bodies = stepped.snapshot_all(0.0);
        let apparent_sun = ApexSpin::from_ephemeris(stepped).apparent_sun_center(bodies[0].center(), world_pos);
        let to_sun = apparent_sun - world_pos;
        if to_sun.length_squared() > 1e-6 {
            to_sun.normalize().as_vec3()
        } else {
            Vec3::Y
        }
    }

    /// Exact spherical N-body gravitational acceleration (`m/s²`) at `pos` (`epoch + delta_t`) in `self.frame`.
    pub fn gravity_accel_at(self, pos: DVec3, delta_t: f64) -> DVec3 {
        let bodies = self.snapshot_all(delta_t);
        let (_, _, frame_accel) = GravityBubble::new(self.proportions, self.epoch_secs + delta_t)
            .heliocentric_cosmic_state(self.frame);
        let pos_cosmic = CosmicPos::from_dvec3(pos);

        let mut total = -frame_accel;
        for b in &bodies {
            let rel = pos_cosmic.relative_dvec3(b.cosmic_locus());
            let r_sq = rel.length_squared();
            let h = b.half_size();
            let knee = (h * 0.35).clamp(4.0, 24.0);
            if b.nature().landable_crust() && r_sq <= (h + knee) * (h + knee) {
                let dir = if r_sq > 1e-8 { -rel / r_sq.sqrt() } else { DVec3::NEG_Y };
                total += dir * f64::from(b.surface_g());
            } else {
                let dist_sq = r_sq.max(h * h);
                let dist = dist_sq.sqrt();
                if dist > 1e-4 {
                    total -= rel * (b.gm() / (dist_sq * dist));
                }
            }
        }
        total
    }

    /// Dominant local spherical surface-up vector (`rel.normalize()` radial outward everywhere).
    pub fn dominant_up_at(self, pos: DVec3) -> Vec3 {
        let bodies = self.snapshot_all(0.0);
        let pos_cosmic = CosmicPos::from_dvec3(pos);
        let mut strongest = (0.0_f64, Vec3::Y);
        for b in &bodies {
            let rel = pos_cosmic.relative_dvec3(b.cosmic_locus());
            let r_sq = rel.length_squared();
            let h = b.half_size();
            if b.nature().landable_crust() && r_sq <= (h + 48.0) * (h + 48.0) {
                return if r_sq > 1e-8 { (rel / r_sq.sqrt()).as_vec3() } else { Vec3::Y };
            }
            let clamped_sq = r_sq.max(h * h);
            let pull = b.gm() / clamped_sq;
            if pull > strongest.0 && r_sq > 1e-6 {
                strongest = (pull, (rel / r_sq.sqrt()).as_vec3());
            }
        }
        strongest.1
    }
}
