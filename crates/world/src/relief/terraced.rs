use std::collections::HashMap;
use std::sync::OnceLock;
use glam::Vec3;
use crate::frame::SurfaceFrame;
use super::elevation::ReliefElevation;

/// Sight ray locating a discrete planetary surface cell for excavation or block placement.
#[derive(Clone, Copy, Debug)]
pub struct SurfaceSightRay {
    eye: Vec3,
    direction: Vec3,
}

impl SurfaceSightRay {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(eye: Vec3, direction: Vec3) -> Self {
        Self { eye, direction }
    }

    /// Resolves the targeted discrete grid cell (u, v) along the sight ray within at most three blocks of planar reach.
    pub fn target_cell(
        &self,
        frame: &SurfaceFrame,
        relief: &dyn ReliefElevation,
        half_blocks: i32,
        half_size: f32,
    ) -> Option<(i32, i32)> {
        let dir = self.direction.normalize_or_zero();
        let (eu, ev) = frame.decompose(self.eye);
        let (ecu, ecv) = ((eu * 2.0).floor() as i32, (ev * 2.0).floor() as i32);
        let min_c = -half_blocks + 1;
        let max_c = half_blocks - 2;

        let mut step = 1;
        while step <= 85 {
            let t = (step as f32) * 0.04;
            let sample = self.eye + dir * t;
            let (u, v) = frame.decompose(sample);
            let cu = (u * 2.0).floor() as i32;
            let cv = (v * 2.0).floor() as i32;
            if (cu - ecu).abs().max((cv - ecv).abs()) > 3 {
                break;
            }
            if cu >= min_c && cu <= max_c && cv >= min_c && cv <= max_c {
                let ray_elev = frame.height_of(sample) - half_size;
                let surf_elev = relief.surface_height_at(u, v);
                if ray_elev <= surf_elev + 0.02 {
                    return Some((cu, cv));
                }
            }
            step += 1;
        }
        None
    }

    /// Resolves a placement cell within three-block reach, stepping one cell forward if the target coincides with the avatar's stance cell.
    pub fn placement_cell(
        &self,
        frame: &SurfaceFrame,
        relief: &dyn ReliefElevation,
        half_blocks: i32,
        half_size: f32,
        avatar_pos: Vec3,
    ) -> Option<(i32, i32)> {
        let (mut cu, mut cv) = self.target_cell(frame, relief, half_blocks, half_size)?;
        let (au, av) = frame.decompose(avatar_pos);
        let (acu, acv) = ((au * 2.0).floor() as i32, (av * 2.0).floor() as i32);
        if cu == acu && cv == acv {
            let min_c = -half_blocks + 1;
            let max_c = half_blocks - 2;
            let (du, dv) = frame.decompose(self.direction);
            if du.abs() > dv.abs() {
                cu = (cu + if du >= 0.0 { 1 } else { -1 }).clamp(min_c, max_c);
            } else {
                cv = (cv + if dv >= 0.0 { 1 } else { -1 }).clamp(min_c, max_c);
            }
        }
        Some((cu, cv))
    }
}

/// Decorator recording excavated pits and stacked earth blocks atop an underlying planetary relief.
pub struct TerracedRelief {
    origin: Box<dyn ReliefElevation>,
    deltas: HashMap<(i32, i32), i32>,
    natural_cache: OnceLock<Vec<i16>>,
    half_span: i32,
    dig_stage: ((i32, i32), u8),
}

impl TerracedRelief {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(
        origin: Box<dyn ReliefElevation>,
        deltas: HashMap<(i32, i32), i32>,
        natural_cache: OnceLock<Vec<i16>>,
        half_span: i32,
        dig_stage: ((i32, i32), u8),
    ) -> Self {
        Self { origin, deltas, natural_cache, half_span, dig_stage }
    }

    /// Secondary constructor initializing an untouched terraced relief wrapper for a face span.
    pub fn in_span(origin: Box<dyn ReliefElevation>, half_blocks: i32) -> Self {
        Self::new(origin, HashMap::new(), OnceLock::new(), half_blocks + 2, ((0, 0), 0))
    }

    /// Secondary constructor initializing an untouched terraced relief wrapper.
    pub fn pristine(origin: Box<dyn ReliefElevation>) -> Self {
        Self::in_span(origin, 128)
    }

    fn delta_at(&self, u: i32, v: i32) -> i32 {
        if self.deltas.is_empty() {
            0
        } else {
            self.deltas.get(&(u, v)).copied().unwrap_or(0)
        }
    }
}

impl ReliefElevation for TerracedRelief {
    fn natural_elevation_at(&self, u: i32, v: i32) -> i32 {
        if u.abs() <= self.half_span && v.abs() <= self.half_span {
            let side = (self.half_span * 2 + 1) as usize;
            let grid = self.natural_cache.get_or_init(|| {
                let mut table = Vec::with_capacity(side * side);
                for gu in -self.half_span..=self.half_span {
                    for gv in -self.half_span..=self.half_span {
                        table.push(self.origin.elevation_at(gu, gv) as i16);
                    }
                }
                table
            });
            let idx = ((u + self.half_span) as usize) * side + ((v + self.half_span) as usize);
            grid[idx] as i32
        } else {
            self.origin.elevation_at(u, v)
        }
    }

    fn elevation_at(&self, u: i32, v: i32) -> i32 {
        self.natural_elevation_at(u, v) + self.delta_at(u, v)
    }

    fn surface_height_at(&self, u: f32, v: f32) -> f32 {
        if self.deltas.is_empty() {
            return self.origin.surface_height_at(u, v);
        }
        let cell_u = (u * 2.0).floor() as i32;
        let cell_v = (v * 2.0).floor() as i32;
        if self.reworked_at(cell_u, cell_v) {
            (self.elevation_at(cell_u, cell_v) as f32) * 0.5
        } else {
            self.origin.surface_height_at(u, v)
        }
    }

    fn reworked_at(&self, u: i32, v: i32) -> bool {
        self.delta_at(u, v) != 0
    }

    fn dig_stage_at(&self, u: i32, v: i32) -> u8 {
        if self.dig_stage.0 == (u, v) { self.dig_stage.1 } else { 0 }
    }

    fn mark_dig_stage(&mut self, u: i32, v: i32, stage: u8) -> (i32, bool) {
        let prev_v = self.dig_stage.0.1;
        if stage == 0 {
            let active = self.dig_stage.1 > 0;
            self.dig_stage = ((0, 0), 0);
            return (prev_v, active);
        }
        if self.elevation_at(u, v) <= -16 { return (v, false); }
        self.dig_stage = ((u, v), stage.min(3));
        (prev_v, true)
    }

    fn excavate_cell(&mut self, u: i32, v: i32) -> (i32, bool) {
        if self.elevation_at(u, v) <= -16 { return (v, false); }
        let prev_v = self.dig_stage.0.1;
        self.deltas.insert((u, v), self.delta_at(u, v) - 1);
        self.dig_stage = ((u, v), 0);
        (prev_v, true)
    }

    fn stack_cell(&mut self, u: i32, v: i32) -> bool {
        if self.elevation_at(u, v) >= 32 { return false; }
        self.deltas.insert((u, v), self.delta_at(u, v) + 1);
        if self.dig_stage.0 == (u, v) { self.dig_stage = ((u, v), 0); }
        true
    }
}
