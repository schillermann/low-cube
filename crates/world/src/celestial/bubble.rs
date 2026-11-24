use glam::{DVec3, Vec3};
use super::belt::AsteroidBelt;
use super::catalog::{BodyId, SolarCatalog};
use super::ephemeris::SolarEphemeris;
use super::proportions::SolarProportions;

pub const DEEP_SPACE_SECTOR_SPAN: f64 = 4_096.0;
const DEEP_SPACE_HYSTERESIS: f64 = 2_560.0;
const EARTH_BUBBLE_MIN_EXIT: f64 = 42_000.0;

/// Polymorphic local reference frame anchoring `Vec3` physics to a moving Hill bubble or `65 km` deep-space sector.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CosmicFrame {
    Body(BodyId),
    BeltAsteroid { id: BodyId, cell: (i32, i32, i32) },
    DeepSpace([i64; 3]),
}

impl CosmicFrame {
    pub const fn earth() -> Self {
        Self::Body(BodyId::EARTH)
    }

    pub const fn earth_centered(self) -> bool {
        matches!(self, Self::Body(id) if id.raw_code() == BodyId::EARTH.raw_code())
    }

    pub const fn anchored_body(self) -> Option<BodyId> {
        match self {
            Self::Body(id) | Self::BeltAsteroid { id, .. } => Some(id),
            Self::DeepSpace(_) => None,
        }
    }

    pub fn inscribe(&self, out: &mut Vec<u8>) {
        match *self {
            Self::Body(id) => {
                out.push(0);
                out.extend_from_slice(&id.raw_code().to_le_bytes());
            }
            Self::BeltAsteroid { id, cell } => {
                out.push(1);
                out.extend_from_slice(&id.raw_code().to_le_bytes());
                for c in [cell.0, cell.1, cell.2] {
                    out.extend_from_slice(&c.to_le_bytes());
                }
            }
            Self::DeepSpace(sec) => {
                out.push(2);
                for s in sec {
                    out.extend_from_slice(&s.to_le_bytes());
                }
            }
        }
    }

    pub fn decode(bytes: &[u8]) -> Option<(Self, usize)> {
        match *bytes.first()? {
            0 if bytes.len() >= 3 => {
                let code = u16::from_le_bytes([bytes[1], bytes[2]]);
                Some((Self::Body(BodyId::from_raw(code)?), 3))
            }
            1 if bytes.len() >= 15 => {
                let code = u16::from_le_bytes([bytes[1], bytes[2]]);
                let id = BodyId::from_raw(code)?;
                let c0 = i32::from_le_bytes(bytes[3..7].try_into().ok()?);
                let c1 = i32::from_le_bytes(bytes[7..11].try_into().ok()?);
                let c2 = i32::from_le_bytes(bytes[11..15].try_into().ok()?);
                Some((Self::BeltAsteroid { id, cell: (c0, c1, c2) }, 15))
            }
            2 if bytes.len() >= 25 => {
                let s0 = i64::from_le_bytes(bytes[1..9].try_into().ok()?);
                let s1 = i64::from_le_bytes(bytes[9..17].try_into().ok()?);
                let s2 = i64::from_le_bytes(bytes[17..25].try_into().ok()?);
                Some((Self::DeepSpace([s0, s1, s2]), 25))
            }
            _ => None,
        }
    }
}

/// Exact `f64`-differenced coordinate and velocity transition across two `CosmicFrame`s (`<= 4` fields).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FrameHandoff {
    next_frame: CosmicFrame,
    next_pos: Vec3,
    next_vel: Vec3,
    pos_shift: Vec3,
}

impl FrameHandoff {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(next_frame: CosmicFrame, next_pos: Vec3, next_vel: Vec3, pos_shift: Vec3) -> Self {
        Self { next_frame, next_pos, next_vel, pos_shift }
    }

    pub const fn next_frame(self) -> CosmicFrame { self.next_frame }
    pub const fn next_pos(self) -> Vec3 { self.next_pos }
    pub const fn next_vel(self) -> Vec3 { self.next_vel }
    pub const fn pos_shift(self) -> Vec3 { self.pos_shift }
}

/// Hill-sphere gravity bubble selector resolving `f64` heliocentric frames and hysteresis-guarded handoffs.
#[derive(Clone, Copy, Debug)]
pub struct GravityBubble {
    proportions: SolarProportions,
    epoch_secs: f64,
}

impl GravityBubble {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(proportions: SolarProportions, epoch_secs: f64) -> Self {
        Self { proportions, epoch_secs }
    }

    pub const fn from_ephemeris(ephemeris: SolarEphemeris) -> Self {
        Self::new(ephemeris.proportions(), ephemeris.epoch_secs())
    }

    /// Resolves `(heliocentric_pos, heliocentric_vel, frame_orbital_accel)` in `f64` (`SOL = 0`).
    pub fn heliocentric_state(self, frame: CosmicFrame) -> (DVec3, DVec3, DVec3) {
        let eph = SolarEphemeris::new(self.proportions, self.epoch_secs);
        match frame {
            CosmicFrame::DeepSpace([sx, sy, sz]) => (
                DVec3::new(
                    (sx as f64) * DEEP_SPACE_SECTOR_SPAN,
                    (sy as f64) * DEEP_SPACE_SECTOR_SPAN,
                    (sz as f64) * DEEP_SPACE_SECTOR_SPAN,
                ),
                DVec3::ZERO,
                DVec3::ZERO,
            ),
            CosmicFrame::BeltAsteroid { cell, .. } => {
                let sun_gm = eph.solar_gm();
                let belt = AsteroidBelt::new(self.proportions, self.epoch_secs);
                if let Some((rock, vel)) = belt.rock_kinematics_in_cell(cell, sun_gm) {
                    let c = rock.center();
                    let r_sq = c.length_squared().max(1.0);
                    let accel = -c * (sun_gm / (r_sq * r_sq.sqrt()));
                    (c, vel, accel)
                } else {
                    (DVec3::ZERO, DVec3::ZERO, DVec3::ZERO)
                }
            }
            CosmicFrame::Body(id) => eph.body_heliocentric_state(id),
        }
    }

    /// Maps `local_pos` from `from_frame` into `to_frame` in `f64` without catastrophic `f32` cancellation.
    pub fn map_position(self, from_frame: CosmicFrame, local_pos: Vec3, to_frame: CosmicFrame) -> DVec3 {
        if from_frame == to_frame {
            return local_pos.as_dvec3();
        }
        let (from_p, _, _) = self.heliocentric_state(from_frame);
        let (to_p, _, _) = self.heliocentric_state(to_frame);
        (from_p + local_pos.as_dvec3()) - to_p
    }

    /// Coarse `65,536 m` heliocentric sector key for `O(1)` cross-bubble multiplayer proximity pruning.
    pub fn macro_cell(self, frame: CosmicFrame, local_pos: Vec3) -> (i64, i64, i64) {
        let (origin, _, _) = self.heliocentric_state(frame);
        let p = origin + local_pos.as_dvec3();
        (
            (p.x / DEEP_SPACE_SECTOR_SPAN).floor() as i64,
            (p.y / DEEP_SPACE_SECTOR_SPAN).floor() as i64,
            (p.z / DEEP_SPACE_SECTOR_SPAN).floor() as i64,
        )
    }

    /// Half-size of the cubic crust anchored at `frame` (`0.0` in `DeepSpace`).
    pub fn frame_body_half(self, frame: CosmicFrame) -> f32 {
        match frame {
            CosmicFrame::DeepSpace(_) => 0.0,
            CosmicFrame::BeltAsteroid { id, .. } => {
                let tier = u32::from(id.raw_code() & 0x0003);
                (self.proportions.belt_rock_blocks(tier) as f32) * 0.5
            }
            CosmicFrame::Body(id) => SolarCatalog::new()
                .spec_for(id)
                .map_or(self.proportions.earth_half(), |s| {
                    (self.proportions.voxel_blocks(s.physique().diameter_ratio()) as f32) * 0.5
                }),
        }
    }

    /// Validates whether `local_pos` lies within physical bounds of `frame` (preventing core tunneling or NaN spoofing).
    pub fn valid_local_point(self, frame: CosmicFrame, local_pos: Vec3) -> bool {
        if !local_pos.is_finite() {
            return false;
        }
        let max_coord = local_pos.abs().max_element();
        match frame {
            CosmicFrame::DeepSpace(_) => max_coord <= (DEEP_SPACE_SECTOR_SPAN as f32) + 8_192.0,
            CosmicFrame::BeltAsteroid { .. } => {
                let half = self.frame_body_half(frame);
                max_coord >= (half * 0.25).min(1.5) && max_coord <= 2_048.0
            }
            CosmicFrame::Body(id) => {
                let half = self.frame_body_half(frame);
                let min_core = if id == BodyId::SOL { 0.0 } else { (half - 24.0).max(0.0) };
                max_coord >= min_core && max_coord <= (half * 40.0).max(72_000.0)
            }
        }
    }

    /// Evaluates whether `(local_pos, local_vel)` in `ephemeris.frame()` should hand off to a better `CosmicFrame`.
    pub fn evaluate_handoff(self, ephemeris: SolarEphemeris, local_pos: Vec3, local_vel: Vec3) -> Option<FrameHandoff> {
        let cur_frame = ephemeris.frame();
        let (cur_p, cur_v, _) = self.heliocentric_state(cur_frame);
        let helio_p = cur_p + local_pos.as_dvec3();
        let helio_v = cur_v + local_vel.as_dvec3();

        let helio_eph = SolarEphemeris::with_frame(self.proportions, self.epoch_secs, CosmicFrame::Body(BodyId::SOL));
        let bodies = helio_eph.snapshot_all(0.0);
        let mut chosen: Option<(f64, CosmicFrame)> = None;
        for b in &bodies {
            if !b.nature().landable_crust() {
                continue;
            }
            let base_entry = if b.id() == BodyId::EARTH {
                (b.half_size() * 24.0).max(EARTH_BUBBLE_MIN_EXIT / 1.35)
            } else {
                (b.half_size() * 22.0).clamp(96.0, 1_800.0)
            };
            let threshold = if cur_frame == CosmicFrame::Body(b.id()) {
                base_entry * 1.35
            } else {
                base_entry
            };
            let surf_gap = (helio_p - b.center()).abs().max_element() - b.half_size();
            if surf_gap <= threshold && chosen.map_or(true, |(best_half, _)| b.half_size() < best_half) {
                chosen = Some((b.half_size(), CosmicFrame::Body(b.id())));
            }
        }

        let next_frame = if let Some((_, body_frame)) = chosen {
            body_frame
        } else {
            let sun_gm = bodies[0].gm();
            let belt = AsteroidBelt::new(self.proportions, self.epoch_secs);
            let belt_margin = if matches!(cur_frame, CosmicFrame::BeltAsteroid { .. }) { 130.0 } else { 96.0 };
            if let Some((rock, cell)) = belt.nearest_rock_with_cell(helio_p, DVec3::ZERO, sun_gm, belt_margin) {
                let cap = f64::from((rock.half_size() * 12.0).max(96.0))
                    * if matches!(cur_frame, CosmicFrame::BeltAsteroid { id, .. } if id == rock.id()) { 1.35 } else { 1.0 };
                let gap = (helio_p - rock.center()).abs().max_element() - f64::from(rock.half_size());
                if gap <= cap {
                    CosmicFrame::BeltAsteroid { id: rock.id(), cell }
                } else {
                    Self::select_deep_space(cur_frame, helio_p)
                }
            } else {
                Self::select_deep_space(cur_frame, helio_p)
            }
        };

        if next_frame == cur_frame {
            return None;
        }
        let (next_p, next_v, _) = self.heliocentric_state(next_frame);
        Some(FrameHandoff::new(
            next_frame,
            (helio_p - next_p).as_vec3(),
            (helio_v - next_v).as_vec3(),
            (cur_p - next_p).as_vec3(),
        ))
    }

    fn select_deep_space(cur_frame: CosmicFrame, helio_p: DVec3) -> CosmicFrame {
        if let CosmicFrame::DeepSpace(cur_sec) = cur_frame {
            let cur_origin = DVec3::new(
                (cur_sec[0] as f64) * DEEP_SPACE_SECTOR_SPAN,
                (cur_sec[1] as f64) * DEEP_SPACE_SECTOR_SPAN,
                (cur_sec[2] as f64) * DEEP_SPACE_SECTOR_SPAN,
            );
            if (helio_p - cur_origin).abs().max_element() <= DEEP_SPACE_HYSTERESIS {
                return cur_frame;
            }
        }
        CosmicFrame::DeepSpace([
            (helio_p.x / DEEP_SPACE_SECTOR_SPAN).round() as i64,
            (helio_p.y / DEEP_SPACE_SECTOR_SPAN).round() as i64,
            (helio_p.z / DEEP_SPACE_SECTOR_SPAN).round() as i64,
        ])
    }
}
