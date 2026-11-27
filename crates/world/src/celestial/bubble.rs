use glam::{DVec3, Vec3};
use crate::CosmicPos;
use super::belt::AsteroidBelt;
use super::catalog::{BodyId, SolarCatalog};
use super::ephemeris::SolarEphemeris;
use super::proportions::SolarProportions;
pub use super::cosmic_frame::{CosmicFrame, FrameHandoff};

pub const DEEP_SPACE_SECTOR_SPAN: f64 = 4_096.0;
const DEEP_SPACE_HYSTERESIS_TICKS: i64 = 2_560 * CosmicPos::TICKS_PER_METER;
const EARTH_BUBBLE_MIN_EXIT: f64 = 42_000.0;

/// Deterministic `CosmicPos` (`0.1 mm`) Hill-sphere gravity bubble selector (`<= 4` fields).
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

    /// Resolves `(heliocentric_cosmic_pos, heliocentric_vel, frame_orbital_accel)` (`SOL = 0`).
    pub fn heliocentric_cosmic_state(self, frame: CosmicFrame) -> (CosmicPos, DVec3, DVec3) {
        let eph = SolarEphemeris::new(self.proportions, self.epoch_secs);
        match frame {
            CosmicFrame::DeepSpace([sx, sy, sz]) => (
                CosmicPos::new([
                    sx.saturating_mul(CosmicPos::SECTOR_SPAN_TICKS),
                    sy.saturating_mul(CosmicPos::SECTOR_SPAN_TICKS),
                    sz.saturating_mul(CosmicPos::SECTOR_SPAN_TICKS),
                ]),
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
                    (rock.cosmic_locus(), vel, accel)
                } else {
                    (CosmicPos::origin(), DVec3::ZERO, DVec3::ZERO)
                }
            }
            CosmicFrame::Body(id) => eph.body_heliocentric_cosmic(id),
        }
    }

    pub fn heliocentric_cosmic_pos(self, frame: CosmicFrame) -> CosmicPos {
        self.heliocentric_cosmic_state(frame).0
    }

    pub fn compose_cosmic_pos(self, frame: CosmicFrame, local_pos: Vec3) -> CosmicPos {
        self.heliocentric_cosmic_pos(frame).translated(CosmicPos::from_vec3(local_pos))
    }

    /// Resolves `(heliocentric_pos, heliocentric_vel, frame_orbital_accel)` in `f64` (`SOL = 0`).
    pub fn heliocentric_state(self, frame: CosmicFrame) -> (DVec3, DVec3, DVec3) {
        let (locus, vel, accel) = self.heliocentric_cosmic_state(frame);
        (locus.to_dvec3(), vel, accel)
    }

    /// Maps `local_pos` from `from_frame` into `to_frame` via `i64` (`0.1 mm`) subtraction without cancellation.
    pub fn map_position(self, from_frame: CosmicFrame, local_pos: Vec3, to_frame: CosmicFrame) -> DVec3 {
        if from_frame == to_frame {
            return local_pos.as_dvec3();
        }
        let world_cosmic = self.compose_cosmic_pos(from_frame, local_pos);
        let to_origin = self.heliocentric_cosmic_pos(to_frame);
        world_cosmic.relative_dvec3(to_origin)
    }

    /// Camera-relative `f32` offset subtracting `(obj - cam)` in exact `i64` `0.1 mm` ticks before `f32` conversion.
    pub fn camera_relative_f32(
        self,
        obj_frame: CosmicFrame,
        obj_local: Vec3,
        cam_frame: CosmicFrame,
        cam_local: Vec3,
    ) -> Vec3 {
        let obj = self.compose_cosmic_pos(obj_frame, obj_local);
        let cam = self.compose_cosmic_pos(cam_frame, cam_local);
        obj.relative_f32(cam)
    }

    /// Coarse `4,096 m` heliocentric sector key via exact `i64` integer division.
    pub fn macro_cell(self, frame: CosmicFrame, local_pos: Vec3) -> (i64, i64, i64) {
        let [sx, sy, sz] = self.compose_cosmic_pos(frame, local_pos).macro_sector();
        (sx, sy, sz)
    }

    /// Radius of the spherical crust anchored at `frame` (`0.0` in `DeepSpace`).
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

    /// Validates whether `local_pos` lies within physical spherical bounds of `frame`.
    pub fn valid_local_point(self, frame: CosmicFrame, local_pos: Vec3) -> bool {
        if !local_pos.is_finite() {
            return false;
        }
        let max_coord = local_pos.abs().max_element();
        let radial = local_pos.length();
        match frame {
            CosmicFrame::DeepSpace(_) => max_coord <= (DEEP_SPACE_SECTOR_SPAN as f32) + 8_192.0,
            CosmicFrame::BeltAsteroid { .. } => {
                let half = self.frame_body_half(frame);
                radial >= (half * 0.25).min(1.5) && radial <= 2_048.0
            }
            CosmicFrame::Body(id) => {
                let half = self.frame_body_half(frame);
                let min_core = if id == BodyId::SOL { 0.0 } else { (half - 24.0).max(0.0) };
                radial >= min_core && radial <= (half * 40.0).max(72_000.0)
            }
        }
    }

    /// Evaluates whether `(local_pos, local_vel)` in `ephemeris.frame()` should hand off to a better `CosmicFrame`.
    pub fn evaluate_handoff(self, ephemeris: SolarEphemeris, local_pos: Vec3, local_vel: Vec3) -> Option<FrameHandoff> {
        let cur_frame = ephemeris.frame();
        let (cur_locus, cur_v, _) = self.heliocentric_cosmic_state(cur_frame);
        let helio_locus = cur_locus.translated(CosmicPos::from_vec3(local_pos));
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
            let max_r_ticks = ((b.half_size() + threshold) * (CosmicPos::TICKS_PER_METER as f64)).round() as i128;
            let dist_sq = helio_locus.distance_sq_i128(b.cosmic_locus());
            if dist_sq <= max_r_ticks * max_r_ticks && chosen.map_or(true, |(best_half, _)| b.half_size() < best_half) {
                chosen = Some((b.half_size(), CosmicFrame::Body(b.id())));
            }
        }

        let next_frame = if let Some((_, body_frame)) = chosen {
            body_frame
        } else {
            let sun_gm = bodies[0].gm();
            let belt = AsteroidBelt::new(self.proportions, self.epoch_secs);
            let belt_margin = if matches!(cur_frame, CosmicFrame::BeltAsteroid { .. }) { 130.0 } else { 96.0 };
            if let Some((rock, cell)) = belt.nearest_rock_with_cell(helio_locus.to_dvec3(), DVec3::ZERO, sun_gm, belt_margin) {
                let cap = f64::from((rock.half_size() * 12.0).max(96.0))
                    * if matches!(cur_frame, CosmicFrame::BeltAsteroid { id, .. } if id == rock.id()) { 1.35 } else { 1.0 };
                let max_r_ticks = ((f64::from(rock.half_size()) + cap) * (CosmicPos::TICKS_PER_METER as f64)).round() as i128;
                if helio_locus.distance_sq_i128(rock.cosmic_locus()) <= max_r_ticks * max_r_ticks {
                    CosmicFrame::BeltAsteroid { id: rock.id(), cell }
                } else {
                    Self::select_deep_space(cur_frame, helio_locus)
                }
            } else {
                Self::select_deep_space(cur_frame, helio_locus)
            }
        };

        if next_frame == cur_frame {
            return None;
        }
        let (next_locus, next_v, _) = self.heliocentric_cosmic_state(next_frame);
        Some(FrameHandoff::new(
            next_frame,
            helio_locus.relative_f32(next_locus),
            (helio_v - next_v).as_vec3(),
            cur_locus.relative_f32(next_locus),
        ))
    }

    fn select_deep_space(cur_frame: CosmicFrame, helio_locus: CosmicPos) -> CosmicFrame {
        if let CosmicFrame::DeepSpace(cur_sec) = cur_frame {
            let cur_origin = CosmicPos::new([
                cur_sec[0].saturating_mul(CosmicPos::SECTOR_SPAN_TICKS),
                cur_sec[1].saturating_mul(CosmicPos::SECTOR_SPAN_TICKS),
                cur_sec[2].saturating_mul(CosmicPos::SECTOR_SPAN_TICKS),
            ]);
            let delta = helio_locus.delta_from(cur_origin).ticks();
            let max_delta = delta[0].abs().max(delta[1].abs()).max(delta[2].abs());
            if max_delta <= DEEP_SPACE_HYSTERESIS_TICKS {
                return cur_frame;
            }
        }
        CosmicFrame::DeepSpace(helio_locus.nearest_sector())
    }
}
