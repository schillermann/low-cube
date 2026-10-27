use std::collections::HashMap;
use std::sync::OnceLock;
use glam::Vec3;
use crate::MeshTarget;
use crate::block::{BlockMedium, CuboidShape, ElevatedShape, GridCell, VoxelShape};
use crate::frame::SurfaceFrame;
use super::elevation::ReliefElevation;
use super::crust_vault::CrustVault;
const BASE_REGROWTH_EFFORT: f32 = 320.0;
const JITTER_REGROWTH_EFFORT: f32 = 440.0;

/// Decorator recording excavated pits, elevated molded blocks, and grass regrowth atop planetary relief.
pub struct TerracedRelief {
    origin: Box<dyn ReliefElevation>, deltas: HashMap<(i32, i32), i32>, ceilings: HashMap<(i32, i32), i32>, molds: HashMap<(i32, i32), Vec<ElevatedShape>>,
    bare_dirt: HashMap<(i32, i32), f32>, natural_cache: OnceLock<Vec<i16>>, half_span: i32, dig_stage: ((i32, i32), f32, u8),
}

impl TerracedRelief {
    /// Primary constructor doing nothing except assigning arguments to fields.
    #[allow(clippy::too_many_arguments)]
    pub fn new(origin: Box<dyn ReliefElevation>, deltas: HashMap<(i32, i32), i32>, ceilings: HashMap<(i32, i32), i32>, molds: HashMap<(i32, i32), Vec<ElevatedShape>>, bare_dirt: HashMap<(i32, i32), f32>, natural_cache: OnceLock<Vec<i16>>, half_span: i32, dig_stage: ((i32, i32), f32, u8)) -> Self {
        Self { origin, deltas, ceilings, molds, bare_dirt, natural_cache, half_span, dig_stage }
    }
    pub fn in_span(origin: Box<dyn ReliefElevation>, half_blocks: i32) -> Self { Self::new(origin, HashMap::new(), HashMap::new(), HashMap::new(), HashMap::new(), OnceLock::new(), half_blocks + 2, ((0, 0), 0.0, 0)) }
    pub fn pristine(origin: Box<dyn ReliefElevation>) -> Self { Self::in_span(origin, 128) }
    fn delta_at(&self, u: i32, v: i32) -> i32 { if self.deltas.is_empty() { 0 } else { self.deltas.get(&(u, v)).copied().unwrap_or(0) } }
    fn upward_at(&self, u: i32, v: i32) -> i32 { if self.ceilings.is_empty() { 0 } else { self.ceilings.get(&(u, v)).copied().unwrap_or(0) } }
    fn adjacent_upward_at(&self, u: i32, v: i32) -> i32 {
        if self.ceilings.is_empty() { return i32::MIN; }
        [(-1, 0), (1, 0), (0, -1), (0, 1), (-1, -1), (1, -1), (-1, 1), (1, 1)].iter().filter(|&&(du, dv)| self.upward_at(u + du, v + dv) > 0).map(|&(du, dv)| self.crust_floor_at(u + du, v + dv)).max().unwrap_or(i32::MIN)
    }
    fn vault(&self) -> CrustVault { CrustVault::beneath(self.half_span - 2) }
    fn deepened_steps(&self, u: i32, v: i32) -> i32 {
        let (elev, floor) = (self.elevation_at(u, v), self.crust_floor_at(u, v));
        if elev < floor { elev } else if elev > floor + 1 { elev - 1 } else if self.vault().cell_opens(u, v) { self.vault().core().summit_steps().min(floor - 2) } else { floor }
    }
    fn pair_elevation(&self, u: i32, v: i32, du: i32, dv: i32) -> i32 {
        let max_e = self.elevation_at(u, v).max(self.elevation_at(u + du, v + dv));
        let max_d = self.delta_at(u, v).max(self.delta_at(u + du, v + dv));
        max_e.min(self.natural_elevation_at(u, v) + max_d)
    }
    fn body_elevation_at(&self, u: i32, v: i32) -> i32 {
        if !self.reworked_at(u, v) || self.upward_at(u, v) > 0 { self.elevation_at(u, v) } else { self.pair_elevation(u, v, 1, 0).min(self.pair_elevation(u, v, -1, 0)).min(self.pair_elevation(u, v, 0, 1)).min(self.pair_elevation(u, v, 0, -1)) }
    }
    fn in_unbreached_shaft(&self, u: i32, v: i32, elev: f32) -> bool { self.reworked_at(u, v) && { let b = self.body_elevation_at(u, v); b >= self.vault().floor_steps_at(u, v) && elev >= (b as f32) * 0.5 - 0.1 } }
    fn in_upward_shaft(&self, u: i32, v: i32, elev: f32) -> bool { self.upward_at(u, v) > 0 && !self.breached_at(u, v) && elev < (self.crust_floor_at(u, v) as f32) * 0.5 }
    fn cell_ceiling_at(&self, cu: i32, cv: i32) -> f32 {
        if self.breached_at(cu, cv) { f32::INFINITY } else if self.upward_at(cu, cv) > 0 { (self.crust_floor_at(cu, cv) as f32) * 0.5 - 0.30 } else { self.vault().ceiling_height_at((cu as f32) * 0.5 + 0.25, (cv as f32) * 0.5 + 0.25) - 0.30 }
    }
    fn top_steps_at(&self, u: i32, v: i32) -> i32 {
        let (base_steps, ground_top) = (self.elevation_at(u, v), (self.elevation_at(u, v) as f32) * 0.5);
        self.molds.get(&(u, v)).map_or(base_steps, |list| (list.iter().map(|b| b.ceiling_above(ground_top)).fold(ground_top, f32::max) * 2.0).round() as i32)
    }
    fn top_layer_at(&self, u: i32, v: i32) -> f32 {
        let ground_top = (self.elevation_at(u, v) as f32) * 0.5;
        self.molds.get(&(u, v)).and_then(|l| l.last()).map_or(ground_top - 0.25, |b| b.ceiling_above(ground_top) - 0.25)
    }
    fn neighbor_grass_count(&self, u: i32, v: i32) -> u32 {
        let (h, mut count) = (self.top_steps_at(u, v), 0);
        for du in -1..=1 { for dv in -1..=1 { if (du != 0 || dv != 0) && !self.bare_dirt_at(u + du, v + dv) && (h - self.top_steps_at(u + du, v + dv)).abs() <= 1 { count += 1; } } }
        count
    }
    fn regrowth_threshold(u: i32, v: i32) -> f32 { BASE_REGROWTH_EFFORT + (((u.wrapping_mul(73856093) ^ v.wrapping_mul(19349663)).unsigned_abs() % 100) as f32 * 0.01) * JITTER_REGROWTH_EFFORT }
}

impl ReliefElevation for TerracedRelief {
    fn natural_elevation_at(&self, u: i32, v: i32) -> i32 {
        if u.abs() <= self.half_span && v.abs() <= self.half_span {
            let side = (self.half_span * 2 + 1) as usize;
            let grid = self.natural_cache.get_or_init(|| {
                let mut table = Vec::with_capacity(side * side);
                for gu in -self.half_span..=self.half_span { for gv in -self.half_span..=self.half_span { table.push(self.origin.elevation_at(gu, gv) as i16); } }
                table
            });
            grid[((u + self.half_span) as usize) * side + ((v + self.half_span) as usize)] as i32
        } else { self.origin.elevation_at(u, v) }
    }
    fn elevation_at(&self, u: i32, v: i32) -> i32 { self.natural_elevation_at(u, v) + self.delta_at(u, v) }
    fn surface_height_at(&self, u: f32, v: f32) -> f32 { self.support_height_at(u, v, 0.0, 0.0) }
    fn support_height_at(&self, u: f32, v: f32, foot_elev: f32, body_span: f32) -> f32 {
        let (cell_u, cell_v) = ((u * 2.0).floor() as i32, (v * 2.0).floor() as i32);
        let (vault, crest, center_elev) = (self.vault(), self.vault().core().crest_at(u, v), foot_elev + body_span.min(1.0));
        let ceil_limit = if self.upward_at(cell_u, cell_v) > 0 { (self.crust_floor_at(cell_u, cell_v) as f32) * 0.5 } else { vault.ceiling_height_at(u, v) + 0.5 };
        let mut h = if !self.in_unbreached_shaft(cell_u, cell_v, foot_elev) && body_span > 0.0 && !self.lethal_at(u, v, center_elev) && (vault.encloses(u, v, center_elev, 0.0) || self.in_upward_shaft(cell_u, cell_v, center_elev)) && center_elev < ceil_limit {
            crest
        } else if self.deltas.is_empty() && self.molds.is_empty() { return self.origin.surface_height_at(u, v);
        } else if self.reworked_at(cell_u, cell_v) { ((if body_span > 0.0 { self.body_elevation_at(cell_u, cell_v) } else { self.elevation_at(cell_u, cell_v) }) as f32) * 0.5
        } else { self.origin.surface_height_at(u, v) };
        if let Some(list) = self.molds.get(&(cell_u, cell_v)) {
            let (fu, fv) = ((u * 2.0 - cell_u as f32).clamp(0.0, 1.0), (v * 2.0 - cell_v as f32).clamp(0.0, 1.0));
            for block in list { h = block.constrain_support(fu, fv, h, foot_elev, body_span); }
        }
        h
    }
    fn layer_occupied_at(&self, u: i32, v: i32, layer_elev: f32) -> bool { self.molds.get(&(u, v)).is_some_and(|l| l.iter().any(|b| b.occupies_layer(layer_elev))) }
    fn layer_medium_at(&self, u: i32, v: i32, layer_elev: f32) -> BlockMedium { self.molds.get(&(u, v)).and_then(|l| l.iter().rfind(|b| b.occupies_layer(layer_elev))).map_or(BlockMedium::TurfEarth, ElevatedShape::block_medium) }
    fn hit_top_at(&self, u: f32, v: f32, ray_elev: f32) -> Option<f32> {
        let (cu, cv) = ((u * 2.0).floor() as i32, (v * 2.0).floor() as i32);
        let (ground_h, ceil_h) = ((self.elevation_at(cu, cv) as f32) * 0.5, (self.crust_floor_at(cu, cv) as f32) * 0.5);
        let ground_surf = if self.reworked_at(cu, cv) { ground_h } else { self.origin.surface_height_at(u, v) };
        if let Some(list) = self.molds.get(&(cu, cv)) {
            let (fu, fv) = ((u * 2.0 - cu as f32).clamp(0.0, 1.0), (v * 2.0 - cv as f32).clamp(0.0, 1.0));
            for block in list.iter().rev() { if block.intercepts(fu, fv, ray_elev) { return Some(block.ceiling_above(ground_h)); } }
        }
        let ceil_floor = if self.upward_at(cu, cv) > 0 { ceil_h - 0.15 } else { (self.vault().ceiling_height_at(u, v) - 0.55).min(ground_h - 0.55) };
        if !self.breached_at(cu, cv) && ray_elev >= ceil_floor && ray_elev <= ground_surf + 0.02 { Some(if ray_elev < ceil_h - 0.05 { ceil_h - 0.25 } else if ray_elev < ground_h - 0.55 { ((ray_elev * 2.0).floor() * 0.5 + 0.5).min(ground_h) } else { ground_h }) } else { None }
    }
    fn reworked_at(&self, u: i32, v: i32) -> bool { self.delta_at(u, v) != 0 }
    fn breached_at(&self, u: i32, v: i32) -> bool { self.reworked_at(u, v) && self.elevation_at(u, v) < self.vault().floor_steps_at(u, v) }
    fn crust_floor_at(&self, u: i32, v: i32) -> i32 { self.vault().floor_steps_at(u, v) + self.upward_at(u, v) }
    fn ceiling_inset(&self, u: i32, v: i32, margin: f32) -> f32 { if self.breached_at(u, v) || self.upward_at(u, v) > 0 { 0.0 } else { self.vault().ceiling_inset(margin) } }
    fn weightless_at(&self, u: f32, v: f32, elev: f32) -> bool {
        let (cu, cv) = ((u * 2.0).floor() as i32, (v * 2.0).floor() as i32);
        !self.in_unbreached_shaft(cu, cv, elev) && (self.vault().encloses(u, v, elev, 0.10) || (self.in_upward_shaft(cu, cv, elev) && self.upward_at(cu, cv) < 4)) && !self.lethal_at(u, v, elev)
    }
    fn lethal_at(&self, u: f32, v: f32, elev: f32) -> bool { self.vault().core().engulfs(u, v, elev, 1.05) }
    fn confine(&self, u: f32, v: f32, elev: f32) -> (f32, f32, f32) {
        if self.ceilings.is_empty() && elev > -2.0 { return (u, v, elev); }
        let (mut cu, mut cv, mut tu, mut tv) = ((u * 2.0).floor() as i32, (v * 2.0).floor() as i32, u, v);
        if elev > self.cell_ceiling_at(cu, cv) + 0.20 {
            for (du, dv) in [(-1, 0), (1, 0), (0, -1), (0, 1), (-1, -1), (1, -1), (-1, 1), (1, 1)] {
                let (nu, nv) = (cu + du, cv + dv);
                if (self.upward_at(nu, nv) > 0 || self.breached_at(nu, nv)) && elev <= self.cell_ceiling_at(nu, nv) + 0.20 {
                    (tu, tv, cu, cv) = (u.clamp((nu as f32) * 0.5 + 0.02, ((nu + 1) as f32) * 0.5 - 0.02), v.clamp((nv as f32) * 0.5 + 0.02, ((nv + 1) as f32) * 0.5 - 0.02), nu, nv); break;
                }
            }
        }
        let z = if self.breached_at(cu, cv) { elev } else if self.upward_at(cu, cv) > 0 {
            let ceiling = (self.crust_floor_at(cu, cv) as f32) * 0.5 - 0.30;
            if elev < ceiling + 2.0 && elev > ceiling { ceiling } else { elev }
        } else { self.vault().confine(tu, tv, elev) };
        (tu, tv, z)
    }
    fn bare_dirt_at(&self, u: i32, v: i32) -> bool { self.bare_dirt.contains_key(&(u, v)) }
    fn emit_molded_top(&self, cell: GridCell, _query_base: f32, frame: &SurfaceFrame, top_col: Vec3, wall_col: Vec3, target: &mut dyn MeshTarget) -> bool {
        let (u, v) = cell.feed_coordinates(|u, v| (u, v));
        let Some(list) = self.molds.get(&(u, v)) else { return false; };
        let ground_top = (self.elevation_at(u, v) as f32) * 0.5;
        for block in list {
            let stage = if self.dig_stage.0 == (u, v) && block.occupies_layer(self.dig_stage.1) { self.dig_stage.2 } else { 0 };
            let block_col = if block.floating_above(ground_top) || self.bare_dirt_at(u, v) { wall_col } else { top_col };
            block.emit_terraced(cell, frame, block_col, wall_col, stage, target);
        }
        !list.is_empty()
    }
    fn step_grass(&mut self, dt: f32) -> Vec<i32> {
        if self.bare_dirt.is_empty() { return Vec::new(); }
        let active: Vec<((i32, i32), u32)> = self.bare_dirt.keys().copied().map(|(u, v)| ((u, v), self.neighbor_grass_count(u, v))).filter(|&(_, c)| c > 0).collect();
        let mut regrown_rows = Vec::new();
        for ((u, v), neighbors) in active {
            if let Some(effort) = self.bare_dirt.get_mut(&(u, v)) {
                *effort += dt * (neighbors as f32);
                if *effort >= Self::regrowth_threshold(u, v) { self.bare_dirt.remove(&(u, v)); regrown_rows.push(v); }
            }
        }
        regrown_rows
    }
    fn dig_stage_at(&self, u: i32, v: i32) -> u8 { if self.dig_stage.0 == (u, v) && !self.layer_occupied_at(u, v, self.dig_stage.1) { self.dig_stage.2 } else { 0 } }
    fn mark_dig_stage(&mut self, u: i32, v: i32, stage: u8) -> (i32, bool) { self.mark_dig_layer(u, v, self.top_layer_at(u, v), stage) }
    fn mark_dig_layer(&mut self, u: i32, v: i32, layer_elev: f32, stage: u8) -> (i32, bool) {
        let (prev_v, active) = (self.dig_stage.0.1, self.dig_stage.2 > 0);
        if stage == 0 { self.dig_stage = ((0, 0), 0.0, 0); return (prev_v, active); }
        let up_dig = layer_elev < (self.crust_floor_at(u, v).max(self.adjacent_upward_at(u, v)) as f32) * 0.5 + 0.25;
        if (self.breached_at(u, v) || (self.deepened_steps(u, v) == self.elevation_at(u, v) && !up_dig)) && !self.molds.contains_key(&(u, v)) { return (v, false); }
        self.dig_stage = ((u, v), layer_elev, stage.min(3)); (prev_v, true)
    }
    fn excavate_cell(&mut self, u: i32, v: i32) -> (i32, bool) { self.excavate_layer(u, v, self.top_layer_at(u, v)) }
    fn excavate_layer(&mut self, u: i32, v: i32, layer_elev: f32) -> (i32, bool) {
        let prev_v = self.dig_stage.0.1;
        if let Some(list) = self.molds.get_mut(&(u, v)) { if let Some(pos) = list.iter().rposition(|b| b.occupies_layer(layer_elev)) {
            list.remove(pos); if list.is_empty() { self.molds.remove(&(u, v)); }
            self.dig_stage = ((u, v), 0.0, 0); return (prev_v, true);
        } }
        let (crust, adj_up, nat) = (self.crust_floor_at(u, v), self.adjacent_upward_at(u, v), self.natural_elevation_at(u, v));
        if layer_elev < (crust as f32) * 0.5 || (adj_up > crust && layer_elev < (adj_up as f32) * 0.5 + 0.25 && layer_elev < (nat as f32) * 0.5 - 0.55) {
            if self.breached_at(u, v) || (layer_elev < (crust as f32) * 0.5 && adj_up <= crust && !self.vault().cell_opens(u, v)) { return (v, false); }
            let next_crust = (crust + 1).max(((layer_elev + 0.75) * 2.0).round() as i32).min(adj_up.max(crust + 1));
            if next_crust >= self.elevation_at(u, v) {
                self.deltas.insert((u, v), self.vault().core().summit_steps().min(crust - 2) - nat); self.bare_dirt.insert((u, v), 0.0);
            } else { self.ceilings.insert((u, v), next_crust - self.vault().floor_steps_at(u, v)); }
            self.dig_stage = ((u, v), 0.0, 0); return (prev_v, true);
        }
        let (elev, step_at) = (self.elevation_at(u, v), ((layer_elev - 0.25) * 2.0).round() as i32);
        let min_n = [(-1, 0), (1, 0), (0, -1), (0, 1)].iter().filter(|&&(du, dv)| !self.breached_at(u + du, v + dv)).map(|&(du, dv)| self.elevation_at(u + du, v + dv)).min().unwrap_or(elev);
        let next_elev = if step_at < elev - 1 && layer_elev < (nat as f32) * 0.5 - 0.55 { (step_at - 3).max(min_n).max(crust + 1) } else { self.deepened_steps(u, v) };
        if next_elev == elev { return (v, false); }
        self.deltas.insert((u, v), next_elev - nat); self.bare_dirt.insert((u, v), 0.0); self.dig_stage = ((u, v), 0.0, 0); (prev_v, true)
    }
    fn stack_cell(&mut self, u: i32, v: i32) -> bool { self.stack_molded_cell(u, v, (self.elevation_at(u, v) as f32) * 0.5, Box::new(CuboidShape::new(0.5)), true) }
    fn stack_molded_cell(&mut self, u: i32, v: i32, base: f32, shape: Box<dyn VoxelShape>, _full_flat: bool) -> bool {
        let (ground_top, sub) = ((self.elevation_at(u, v) as f32) * 0.5, !self.breached_at(u, v) && base < (self.crust_floor_at(u, v) as f32) * 0.5 + 0.1);
        let mut next_base = if sub { base } else { base.max(ground_top) };
        while self.layer_occupied_at(u, v, next_base + 0.25) { next_base += if sub { -0.5 } else { 0.5 }; }
        if next_base >= 16.0 { return false; }
        let list = self.molds.entry((u, v)).or_default();
        list.push(ElevatedShape::new(shape, next_base));
        list.sort_by(|a, b| a.ceiling_above(ground_top).total_cmp(&b.ceiling_above(ground_top)));
        self.bare_dirt.insert((u, v), 0.0); if self.dig_stage.0 == (u, v) { self.dig_stage = ((u, v), 0.0, 0); } true
    }
}
