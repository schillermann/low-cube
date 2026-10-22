use std::collections::HashMap;
use std::sync::OnceLock;
use glam::Vec3;
use crate::{MeshTarget, Quad};
use crate::block::{GridCell, VoxelShape};
use crate::frame::SurfaceFrame;
use super::elevation::ReliefElevation;

const BASE_REGROWTH_EFFORT: f32 = 320.0;
const JITTER_REGROWTH_EFFORT: f32 = 440.0;

/// Decorator recording excavated pits, molded earth blocks, and grass regrowth atop planetary relief.
pub struct TerracedRelief {
    origin: Box<dyn ReliefElevation>,
    deltas: HashMap<(i32, i32), i32>,
    molds: HashMap<(i32, i32), (f32, Box<dyn VoxelShape>)>,
    bare_dirt: HashMap<(i32, i32), f32>,
    natural_cache: OnceLock<Vec<i16>>,
    half_span: i32,
    dig_stage: ((i32, i32), u8),
}

impl TerracedRelief {
    /// Primary constructor doing nothing except assigning arguments to fields.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        origin: Box<dyn ReliefElevation>, deltas: HashMap<(i32, i32), i32>, molds: HashMap<(i32, i32), (f32, Box<dyn VoxelShape>)>,
        bare_dirt: HashMap<(i32, i32), f32>, natural_cache: OnceLock<Vec<i16>>, half_span: i32, dig_stage: ((i32, i32), u8),
    ) -> Self {
        Self { origin, deltas, molds, bare_dirt, natural_cache, half_span, dig_stage }
    }

    /// Secondary constructor initializing an untouched terraced relief wrapper for a face span.
    pub fn in_span(origin: Box<dyn ReliefElevation>, half_blocks: i32) -> Self {
        Self::new(origin, HashMap::new(), HashMap::new(), HashMap::new(), OnceLock::new(), half_blocks + 2, ((0, 0), 0))
    }

    /// Secondary constructor initializing an untouched terraced relief wrapper.
    pub fn pristine(origin: Box<dyn ReliefElevation>) -> Self { Self::in_span(origin, 128) }

    fn delta_at(&self, u: i32, v: i32) -> i32 {
        if self.deltas.is_empty() { 0 } else { self.deltas.get(&(u, v)).copied().unwrap_or(0) }
    }

    fn neighbor_grass_count(&self, u: i32, v: i32) -> u32 {
        let (h, mut count) = (self.elevation_at(u, v), 0);
        for du in -1..=1 {
            for dv in -1..=1 {
                if (du != 0 || dv != 0) && !self.bare_dirt_at(u + du, v + dv) && (h - self.elevation_at(u + du, v + dv)).abs() <= 1 { count += 1; }
            }
        }
        count
    }

    fn regrowth_threshold(u: i32, v: i32) -> f32 {
        let hash = ((u.wrapping_mul(73856093) ^ v.wrapping_mul(19349663)).unsigned_abs() % 100) as f32 * 0.01;
        BASE_REGROWTH_EFFORT + hash * JITTER_REGROWTH_EFFORT
    }
}

impl ReliefElevation for TerracedRelief {
    fn natural_elevation_at(&self, u: i32, v: i32) -> i32 {
        if u.abs() <= self.half_span && v.abs() <= self.half_span {
            let side = (self.half_span * 2 + 1) as usize;
            let grid = self.natural_cache.get_or_init(|| {
                let mut table = Vec::with_capacity(side * side);
                for gu in -self.half_span..=self.half_span {
                    for gv in -self.half_span..=self.half_span { table.push(self.origin.elevation_at(gu, gv) as i16); }
                }
                table
            });
            grid[((u + self.half_span) as usize) * side + ((v + self.half_span) as usize)] as i32
        } else {
            self.origin.elevation_at(u, v)
        }
    }

    fn elevation_at(&self, u: i32, v: i32) -> i32 { self.natural_elevation_at(u, v) + self.delta_at(u, v) }

    fn surface_height_at(&self, u: f32, v: f32) -> f32 {
        if self.deltas.is_empty() && self.molds.is_empty() { return self.origin.surface_height_at(u, v); }
        let (cell_u, cell_v) = ((u * 2.0).floor() as i32, (v * 2.0).floor() as i32);
        let ground_h = if self.reworked_at(cell_u, cell_v) { (self.elevation_at(cell_u, cell_v) as f32) * 0.5 } else { self.origin.surface_height_at(u, v) };
        if let Some((base, shape)) = self.molds.get(&(cell_u, cell_v)) {
            let dh = shape.elevation_at((u * 2.0 - cell_u as f32).clamp(0.0, 1.0), (v * 2.0 - cell_v as f32).clamp(0.0, 1.0));
            return if dh > 0.0 { *base + dh } else { (*base).min(ground_h) };
        }
        ground_h
    }

    fn reworked_at(&self, u: i32, v: i32) -> bool { self.delta_at(u, v) != 0 }
    fn bare_dirt_at(&self, u: i32, v: i32) -> bool { self.bare_dirt.contains_key(&(u, v)) }
    fn flat_reworked_at(&self, u: i32, v: i32) -> bool {
        self.reworked_at(u, v) && self.molds.get(&(u, v)).is_none_or(|(b, _)| *b >= (self.elevation_at(u, v) as f32) * 0.5 - 0.1)
    }

    fn emit_molded_top(&self, cell: GridCell, query_base: f32, frame: &SurfaceFrame, top_col: Vec3, wall_col: Vec3, target: &mut dyn MeshTarget) -> bool {
        struct TurfTarget<'a> { up: Vec3, sum: Vec3, block_col: Vec3, wall_col: Vec3, under_col: Vec3, floating: bool, bottom: bool, out: &'a mut dyn MeshTarget }
        impl MeshTarget for TurfTarget<'_> {
            fn push_quad(&mut self, quad: Quad) {
                let recolored = quad.transfer(|v0, v1, v2, v3, n, _| {
                    if n.dot(self.up) < -0.9 {
                        self.bottom = true;
                        if !self.floating { return Quad::new(self.sum - v0 - v1 - v2, v0, v1, v1, self.up, self.under_col); }
                    }
                    Quad::new(v0, v1, v2, v3, n, if n.dot(self.up) > 0.25 { self.block_col } else { self.wall_col })
                });
                self.out.push_quad(recolored);
            }
        }
        let (u, v) = cell.feed_coordinates(|u, v| (u, v));
        if let Some((base, shape)) = self.molds.get(&(u, v)) {
            let ground_top = (self.elevation_at(u, v) as f32) * 0.5;
            let floating = *base >= ground_top - 0.1;
            if floating != (query_base >= ground_top - 0.1) { return false; }
            let [b0, b1, b2, b3] = cell.corner_points(frame, *base);
            let block_col = if floating || self.bare_dirt_at(u, v) { wall_col } else { top_col };
            let mut turf = TurfTarget { up: frame.normal(), sum: b0 + b1 + b2 + b3, block_col, wall_col, under_col: top_col, floating, bottom: false, out: target };
            shape.emit_shape(cell, *base, frame, wall_col, &mut turf);
            if floating && !turf.bottom { target.push_quad(Quad::new(b0, b3, b2, b1, -frame.normal(), wall_col)); }
            return true;
        }
        false
    }

    fn step_grass(&mut self, dt: f32) -> Vec<i32> {
        if self.bare_dirt.is_empty() { return Vec::new(); }
        let active: Vec<((i32, i32), u32)> = self.bare_dirt.keys().copied()
            .map(|(u, v)| ((u, v), self.neighbor_grass_count(u, v)))
            .filter(|&(_, count)| count > 0)
            .collect();
        let mut regrown_rows = Vec::new();
        for ((u, v), neighbors) in active {
            if let Some(effort) = self.bare_dirt.get_mut(&(u, v)) {
                *effort += dt * (neighbors as f32);
                if *effort >= Self::regrowth_threshold(u, v) {
                    self.bare_dirt.remove(&(u, v));
                    regrown_rows.push(v);
                }
            }
        }
        regrown_rows
    }

    fn dig_stage_at(&self, u: i32, v: i32) -> u8 { if self.dig_stage.0 == (u, v) { self.dig_stage.1 } else { 0 } }

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
        let prev_v = self.dig_stage.0.1;
        if self.molds.get(&(u, v)).is_some_and(|(b, _)| *b >= (self.elevation_at(u, v) as f32) * 0.5 - 0.1) {
            self.molds.remove(&(u, v));
            self.dig_stage = ((u, v), 0);
            return (prev_v, true);
        }
        if self.elevation_at(u, v) <= -16 { return (v, false); }
        self.molds.remove(&(u, v));
        self.deltas.insert((u, v), self.delta_at(u, v) - 1);
        self.bare_dirt.insert((u, v), 0.0);
        self.dig_stage = ((u, v), 0);
        (prev_v, true)
    }

    fn stack_cell(&mut self, u: i32, v: i32) -> bool {
        if self.elevation_at(u, v) >= 32 { return false; }
        self.molds.remove(&(u, v));
        self.deltas.insert((u, v), self.delta_at(u, v) + 1);
        self.bare_dirt.insert((u, v), 0.0);
        if self.dig_stage.0 == (u, v) { self.dig_stage = ((u, v), 0); }
        true
    }

    fn stack_molded_cell(&mut self, u: i32, v: i32, base: f32, shape: Box<dyn VoxelShape>, full_flat: bool) -> bool {
        let ground_top = (self.elevation_at(u, v) as f32) * 0.5;
        if base > ground_top + 0.1 {
            self.molds.insert((u, v), (base, shape));
            if self.dig_stage.0 == (u, v) { self.dig_stage = ((u, v), 0); }
            return true;
        }
        if self.elevation_at(u, v) >= 32 { return false; }
        self.deltas.insert((u, v), self.delta_at(u, v) + 1);
        self.bare_dirt.insert((u, v), 0.0);
        if full_flat { self.molds.remove(&(u, v)); } else { self.molds.insert((u, v), (ground_top, shape)); }
        if self.dig_stage.0 == (u, v) { self.dig_stage = ((u, v), 0); }
        true
    }
}
