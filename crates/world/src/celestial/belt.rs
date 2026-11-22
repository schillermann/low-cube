use glam::{DVec3, Quat, Vec3};
use crate::{MeshTarget, Quad};
use super::catalog::BodyId;
use super::proportions::SolarProportions;

/// Resolved physical belt asteroid generated deterministically from a 3D ring-sector hash (`0 Byte` baseline RAM).
#[derive(Clone, Copy, Debug)]
pub struct BeltRock {
    id: BodyId,
    center: DVec3,
    half_size: f32,
    seed: u64,
}

impl BeltRock {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(id: BodyId, center: DVec3, half_size: f32, seed: u64) -> Self {
        Self { id, center, half_size, seed }
    }

    pub const fn id(self) -> BodyId { self.id }
    pub const fn center(self) -> DVec3 { self.center }
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
        let ring_slots = ((std::f64::consts::TAU * r_base / spacing).round() as i32).max(24);
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
        let theta = ((f64::from(wrapped_a) + 0.5 + ja) / f64::from(ring_slots)) * std::f64::consts::TAU
            + self.angular_drift(r, sun_gm);
        let (sin_t, cos_t) = theta.sin_cos();
        let y = f64::from(y_slot) * spacing * 0.65 + jy;
        let center = sun_center + DVec3::new(r * cos_t, y, r * sin_t);
        let tier = ((h >> 40) & 0x3) as u16 % 3;
        let base_blocks = 16 + tier * 8;
        let scaled_blocks = ((f32::from(base_blocks) * self.proportions.body_scale().clamp(0.5, 2.0)).round() as u32)
            .clamp(16, 48)
            & !1;
        let half = (scaled_blocks as f32) * 0.5;
        let hash13 = (((h >> 48) as u16) ^ ((r_slot as u16).wrapping_mul(31))) & 0x1FFF;
        let rock_id = BodyId::belt_rock((hash13 << 2) | tier);
        Some(BeltRock::new(rock_id, center, half, h))
    }

    /// `O(1)` lookup returning the nearest belt asteroid within `margin` of `pos` (zero heap allocations).
    pub fn nearest_rock(self, pos: DVec3, sun_center: DVec3, sun_gm: f64, margin: f64) -> Option<BeltRock> {
        let rel = pos - sun_center;
        let r = rel.x.hypot(rel.z);
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
        let drift = self.angular_drift(r, sun_gm);
        let base_theta = (rel.z.atan2(rel.x) - drift).rem_euclid(std::f64::consts::TAU);
        let mut closest: Option<(f64, BeltRock)> = None;
        for dr in -1..=1 {
            let rc = r_slot + dr;
            let r_base = (f64::from(rc) + 0.5) * spacing;
            let ring_slots = ((std::f64::consts::TAU * r_base / spacing).round() as i32).max(24);
            let ac = ((base_theta / std::f64::consts::TAU) * f64::from(ring_slots)).floor() as i32;
            for da in -1..=1 {
                for dy in -1..=1 {
                    if let Some(rock) = self.rock_in_cell((rc, ac + da, y_slot + dy), sun_center, sun_gm) {
                        let d = (pos - rock.center()).abs().max_element() - f64::from(rock.half_size());
                        if d <= margin && closest.map_or(true, |(best, _)| d < best) {
                            closest = Some((d, rock));
                        }
                    }
                }
            }
        }
        closest.map(|(_, r)| r)
    }

    /// Emits tumbling cubic belt asteroids within `cone` in a single dynamic batch (`0` allocations).
    pub fn emit_local_field(
        self,
        cone: crate::SightCone,
        sun_center: DVec3,
        sun_gm: f64,
        awakened_id: Option<BodyId>,
        target: &mut dyn MeshTarget,
    ) {
        let viewer = cone.observe_from(|eye| eye.as_dvec3());
        let rel = viewer - sun_center;
        let r = rel.x.hypot(rel.z);
        let au = self.proportions.astronomical_unit();
        if r < 2.10 * au || r > 3.30 * au {
            return;
        }
        let spacing = self.belt_spacing();
        let r_slot = (r / spacing).floor() as i32;
        let y_slot = (rel.y / (spacing * 0.65)).round() as i32;
        let drift = self.angular_drift(r, sun_gm);
        let base_theta = (rel.z.atan2(rel.x) - drift).rem_euclid(std::f64::consts::TAU);

        for dr in -4..=4 {
            let rc = r_slot + dr;
            let r_base = (f64::from(rc) + 0.5) * spacing;
            let ring_slots = ((std::f64::consts::TAU * r_base / spacing).round() as i32).max(24);
            let ac = ((base_theta / std::f64::consts::TAU) * f64::from(ring_slots)).floor() as i32;
            for da in -5..=5 {
                for dy in -2..=2 {
                    let Some(rock) = self.rock_in_cell((rc, ac + da, y_slot + dy), sun_center, sun_gm) else {
                        continue;
                    };
                    if awakened_id == Some(rock.id()) {
                        continue;
                    }
                    if cone.reaches(rock.center().as_vec3(), 2_400.0) {
                        Self::emit_tumbling_cube(rock, self.epoch_secs as f32, target);
                    }
                }
            }
        }
    }

    fn emit_tumbling_cube(rock: BeltRock, t: f32, target: &mut dyn MeshTarget) {
        let c = rock.center().as_vec3();
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
        let faces: [(Vec3, Vec3, Vec3); 6] = [
            (Vec3::Y, Vec3::X, Vec3::NEG_Z),
            (Vec3::NEG_Y, Vec3::X, Vec3::Z),
            (Vec3::X, Vec3::NEG_Z, Vec3::NEG_Y),
            (Vec3::NEG_X, Vec3::Z, Vec3::NEG_Y),
            (Vec3::Z, Vec3::X, Vec3::Y),
            (Vec3::NEG_Z, Vec3::NEG_X, Vec3::Y),
        ];
        for (n_loc, u_loc, v_loc) in faces {
            let (n, u, v) = (rot * n_loc, rot * u_loc * h, rot * v_loc * h);
            let fc = c + n * h;
            target.push_quad(Quad::new(fc - u - v, fc + u - v, fc + u + v, fc - u + v, n, col));
        }
    }
}
