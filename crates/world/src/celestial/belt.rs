use std::f64::consts::TAU;
use glam::{DVec3, Quat, Vec3};
use crate::{CosmicPos, MeshTarget, Quad};
use super::brad64::Brad64;
use super::catalog::BodyId;
use super::proportions::SolarProportions;

/// Resolved physical belt asteroid anchored at `CosmicPos` (`0.1 mm` steps, `0 Byte` baseline RAM, `4` fields).
#[derive(Clone, Copy, Debug)]
pub struct BeltRock {
    id: BodyId,
    locus: CosmicPos,
    half_size: f32,
    seed: u64,
}

impl BeltRock {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(id: BodyId, locus: CosmicPos, half_size: f32, seed: u64) -> Self {
        Self { id, locus, half_size, seed }
    }

    pub const fn id(self) -> BodyId { self.id }
    pub const fn cosmic_locus(self) -> CosmicPos { self.locus }
    pub fn center(self) -> DVec3 { self.locus.to_dvec3() }
    pub const fn half_size(self) -> f32 { self.half_size }
    pub const fn seed(self) -> u64 { self.seed }
}

/// Procedural 3D toroidal Asteroid Belt (`2.20 .. 3.20 AU` between Mars and Jupiter) with `O(1)` spatial lookup.
#[derive(Clone, Copy, Debug)]
pub struct AsteroidBelt {
    proportions: SolarProportions,
    epoch_secs: f64,
}

impl AsteroidBelt {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(proportions: SolarProportions, epoch_secs: f64) -> Self {
        Self { proportions, epoch_secs }
    }

    pub fn inner_outer_au() -> (f64, f64) {
        (2.20, 3.20)
    }

    fn mix_hash(r_slot: i32, a_slot: i32, y_slot: i32) -> u64 {
        let mut h = 0x9E37_79B9_7F4A_7C15_u64
            ^ ((r_slot as i64 as u64).wrapping_mul(0xBF58_476D_1CE4_E5B9))
            ^ ((a_slot as i64 as u64).wrapping_mul(0x94D0_49BB_1331_11EB))
            ^ ((y_slot as i64 as u64).wrapping_mul(0x517C_C1B7_2722_0A95));
        h = (h ^ (h >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        h = (h ^ (h >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        h ^ (h >> 31)
    }

    fn belt_spacing(self) -> f64 {
        let au = self.proportions.astronomical_unit();
        ((au * 0.008).clamp(96.0, 420.0)).max(64.0)
    }

    fn angular_drift(self, radius: f64, sun_gm: f64) -> f64 {
        if radius <= 1.0 || sun_gm <= 1.0 {
            0.0
        } else {
            (sun_gm / (radius * radius * radius)).sqrt() * self.epoch_secs
        }
    }

    fn planar_radius(x: f64, z: f64) -> f64 {
        (x * x + z * z).sqrt()
    }

    fn base_azimuth_turns(rel: DVec3, drift: f64) -> f64 {
        let raw_turns = (Brad64::atan2_radians(rel.z, rel.x) - drift) / TAU;
        raw_turns - raw_turns.floor()
    }

    fn rock_in_cell(
        self,
        (r_slot, a_slot, y_slot): (i32, i32, i32),
        sun_center: DVec3,
        sun_gm: f64,
    ) -> Option<BeltRock> {
        let density = self.proportions.belt_density();
        if density <= 0.001 || y_slot.abs() > 2 {
            return None;
        }
        let au = self.proportions.astronomical_unit();
        let (r_min, r_max) = (2.20 * au, 3.20 * au);
        let spacing = self.belt_spacing();
        let r_base = (f64::from(r_slot) + 0.5) * spacing;
        if r_base < r_min || r_base > r_max {
            return None;
        }
        let ring_slots = ((TAU * r_base / spacing).round() as i32).max(24);
        let wrapped_a = a_slot.rem_euclid(ring_slots);
        let h = Self::mix_hash(r_slot, wrapped_a, y_slot);
        let occ = ((h & 0xFFFF) as f32) / 65535.0;
        let threshold = (0.34 * density).clamp(0.0, 0.92);
        if occ > threshold {
            return None;
        }
        let jx = ((((h >> 16) & 0xFF) as f64) / 255.0 - 0.5) * 0.55 * spacing;
        let jy = ((((h >> 24) & 0xFF) as f64) / 255.0 - 0.5) * 0.55 * spacing;
        let ja = ((((h >> 32) & 0xFF) as f64) / 255.0 - 0.5) * 0.55;
        let r = r_base + jx;
        let theta = ((f64::from(wrapped_a) + 0.5 + ja) / f64::from(ring_slots)) * TAU
            + self.angular_drift(r, sun_gm);
        let (sin_t, cos_t) = Brad64::from_radians(theta).sin_cos_f64();
        let y = f64::from(y_slot) * spacing * 0.65 + jy;
        let locus = CosmicPos::from_dvec3(sun_center + DVec3::new(r * cos_t, y, r * sin_t));
        let tier = ((h >> 40) & 0x3) as u16 % 3;
        let scaled_blocks = self.proportions.belt_rock_blocks(u32::from(tier));
        let half = (scaled_blocks as f32) * 0.5;
        let hash13 = (((h >> 48) as u16) ^ ((r_slot as u16).wrapping_mul(31))) & 0x1FFF;
        let rock_id = BodyId::belt_rock((hash13 << 2) | tier);
        Some(BeltRock::new(rock_id, locus, half, h))
    }

    /// Resolves `(BeltRock, orbital_velocity)` in heliocentric space (`SOL = 0`) for an explicit belt cell.
    pub fn rock_kinematics_in_cell(
        self,
        cell: (i32, i32, i32),
        sun_gm: f64,
    ) -> Option<(BeltRock, DVec3)> {
        let rock = self.rock_in_cell(cell, DVec3::ZERO, sun_gm)?;
        let c = rock.center();
        let r = Self::planar_radius(c.x, c.z).max(1.0);
        let v_orb = if sun_gm > 1.0 { (sun_gm / r).sqrt() } else { 0.0 };
        let vel = DVec3::new(-v_orb * (c.z / r), 0.0, v_orb * (c.x / r));
        Some((rock, vel))
    }

    /// `O(1)` lookup returning `(nearest_rock, ring_cell)` within `margin` of `pos` (`0` allocations).
    pub fn nearest_rock_with_cell(
        self,
        pos: DVec3,
        sun_center: DVec3,
        sun_gm: f64,
        margin: f64,
    ) -> Option<(BeltRock, (i32, i32, i32))> {
        let rel = pos - sun_center;
        let r = Self::planar_radius(rel.x, rel.z);
        let au = self.proportions.astronomical_unit();
        if r < 2.15 * au - margin || r > 3.25 * au + margin {
            return None;
        }
        let spacing = self.belt_spacing();
        let r_slot = (r / spacing).floor() as i32;
        let y_slot = (rel.y / (spacing * 0.65)).round() as i32;
        if y_slot.abs() > 3 {
            return None;
        }
        let pos_cosmic = CosmicPos::from_dvec3(pos);
        let turns = Self::base_azimuth_turns(rel, self.angular_drift(r, sun_gm));
        let mut closest: Option<(i128, BeltRock, (i32, i32, i32))> = None;
        for dr in -1..=1 {
            let rc = r_slot + dr;
            let r_base = (f64::from(rc) + 0.5) * spacing;
            let ring_slots = ((TAU * r_base / spacing).round() as i32).max(24);
            let ac = (turns * f64::from(ring_slots)).floor() as i32;
            for da in -1..=1 {
                for dy in -1..=1 {
                    let cell = (rc, ac + da, y_slot + dy);
                    if let Some(rock) = self.rock_in_cell(cell, sun_center, sun_gm) {
                        let reach_ticks = ((f64::from(rock.half_size()) + margin).max(0.0)
                            * (CosmicPos::TICKS_PER_METER as f64))
                            .round() as i128;
                        let d2 = pos_cosmic.distance_sq_i128(rock.cosmic_locus());
                        if d2 <= reach_ticks * reach_ticks && closest.map_or(true, |(best, _, _)| d2 < best) {
                            closest = Some((d2, rock, cell));
                        }
                    }
                }
            }
        }
        closest.map(|(_, r, c)| (r, c))
    }

    /// `O(1)` lookup returning the nearest belt asteroid within `margin` of `pos` (zero heap allocations).
    pub fn nearest_rock(self, pos: DVec3, sun_center: DVec3, sun_gm: f64, margin: f64) -> Option<BeltRock> {
        self.nearest_rock_with_cell(pos, sun_center, sun_gm, margin).map(|(r, _)| r)
    }

    /// Emits tumbling spherical belt asteroids within `cone` in a single dynamic batch (`0` allocations).
    pub fn emit_local_field(
        self,
        cone: crate::SightCone,
        sun_center: DVec3,
        sun_gm: f64,
        awakened_ids: [Option<BodyId>; 2],
        target: &mut dyn MeshTarget,
    ) {
        let viewer_f32 = cone.observe_from(|eye| eye);
        let viewer = viewer_f32.as_dvec3();
        let viewer_cosmic = CosmicPos::from_vec3(viewer_f32);
        let rel = viewer - sun_center;
        let r = Self::planar_radius(rel.x, rel.z);
        let au = self.proportions.astronomical_unit();
        if r < 2.10 * au || r > 3.30 * au {
            return;
        }
        let spacing = self.belt_spacing();
        let r_slot = (r / spacing).floor() as i32;
        let y_slot = (rel.y / (spacing * 0.65)).round() as i32;
        let turns = Self::base_azimuth_turns(rel, self.angular_drift(r, sun_gm));

        for dr in -4..=4 {
            let rc = r_slot + dr;
            let r_base = (f64::from(rc) + 0.5) * spacing;
            let ring_slots = ((TAU * r_base / spacing).round() as i32).max(24);
            let ac = (turns * f64::from(ring_slots)).floor() as i32;
            for da in -5..=5 {
                for dy in -2..=2 {
                    let Some(rock) = self.rock_in_cell((rc, ac + da, y_slot + dy), sun_center, sun_gm) else {
                        continue;
                    };
                    if awakened_ids.contains(&Some(rock.id())) {
                        continue;
                    }
                    let c = viewer_f32 + rock.cosmic_locus().relative_f32(viewer_cosmic);
                    if cone.reaches(c, 2_400.0) {
                        Self::emit_tumbling_rock(rock, c, self.epoch_secs as f32, target);
                    }
                }
            }
        }
    }

    fn emit_tumbling_rock(rock: BeltRock, c: Vec3, t: f32, target: &mut dyn MeshTarget) {
        let h = rock.half_size();
        let axis = Vec3::new(
            ((rock.seed() & 0xFF) as f32) - 127.0,
            (((rock.seed() >> 8) & 0xFF) as f32) - 127.0,
            (((rock.seed() >> 16) & 0xFF) as f32) - 127.0,
        )
        .normalize_or_zero();
        let rot = if axis.length_squared() > 0.5 {
            Quat::from_axis_angle(axis, t * 0.08)
        } else {
            Quat::IDENTITY
        };
        let shade = 0.36 + 0.22 * ((((rock.seed() >> 24) & 0xFF) as f32) / 255.0);
        let warm = 0.06 * ((((rock.seed() >> 32) & 0xFF) as f32) / 255.0);
        let col = Vec3::new(shade + warm, shade + warm * 0.5, shade);
        for zone in crate::ClimateZone::all() {
            let frame = zone.surface_frame(h);
            for sv in -1..1 {
                for su in -1..1 {
                    let (u0, u1) = ((su as f32) * h, ((su + 1) as f32) * h);
                    let (v0, v1) = ((sv as f32) * h, ((sv + 1) as f32) * h);
                    let p0 = c + rot * frame.point_at(u0, v0, 0.0);
                    let p1 = c + rot * frame.point_at(u1, v0, 0.0);
                    let p2 = c + rot * frame.point_at(u1, v1, 0.0);
                    let p3 = c + rot * frame.point_at(u0, v1, 0.0);
                    let mid = ((p0 + p1 + p2 + p3) * 0.25 - c).try_normalize().unwrap_or(Vec3::Y);
                    target.push_quad(Quad::new(p0, p3, p2, p1, mid, col));
                }
            }
        }
    }
}
