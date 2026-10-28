use std::collections::HashMap;
use glam::Vec3;
use crate::MeshTarget;
use crate::block::{BlockMedium, CuboidShape, ElevatedShape, GridCell, VoxelShape};
use crate::frame::SurfaceFrame;
use super::natural_strata::NaturalStrata;
use super::shaft_excavation::ShaftExcavation;

const REGH_EFFORT_FLOOR: f32 = 320.0;
const REGH_EFFORT_SPAN: f32 = 440.0;

/// Overhanging and stacked voxel blocks, bare-soil regrowth progress, and active digging stage.
pub struct OverhangMolds {
    blocks: HashMap<(i32, i32), Vec<ElevatedShape>>,
    bare_dirt: HashMap<(i32, i32), f32>,
    dig_stage: ((i32, i32), f32, u8),
}

impl OverhangMolds {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(
        blocks: HashMap<(i32, i32), Vec<ElevatedShape>>,
        bare_dirt: HashMap<(i32, i32), f32>,
        dig_stage: ((i32, i32), f32, u8),
    ) -> Self {
        Self { blocks, bare_dirt, dig_stage }
    }

    /// Secondary constructor for pristine terrain without placed blocks or bare soil.
    pub fn untouched() -> Self {
        Self::new(HashMap::new(), HashMap::new(), ((0, 0), 0.0, 0))
    }

    pub fn empty(&self) -> bool {
        self.blocks.is_empty()
    }

    pub fn present_at(&self, u: i32, v: i32) -> bool {
        self.blocks.contains_key(&(u, v))
    }

    pub fn bare_dirt_at(&self, u: i32, v: i32) -> bool {
        self.bare_dirt.contains_key(&(u, v))
    }

    pub fn expose_dirt(&mut self, u: i32, v: i32) {
        self.bare_dirt.insert((u, v), 0.0);
    }

    pub fn dig_stage_at(&self, u: i32, v: i32) -> u8 {
        if self.dig_stage.0 == (u, v) && !self.layer_occupied_at(u, v, self.dig_stage.1) {
            self.dig_stage.2
        } else {
            0
        }
    }

    pub fn clear_stage(&mut self, u: i32, v: i32) -> i32 {
        let prev_v = self.dig_stage.0.1;
        self.dig_stage = ((u, v), 0.0, 0);
        prev_v
    }

    pub fn clear_if_at(&mut self, u: i32, v: i32) {
        if self.dig_stage.0 == (u, v) {
            self.dig_stage = ((u, v), 0.0, 0);
        }
    }

    pub fn record_stage(&mut self, u: i32, v: i32, layer_elev: f32, stage: u8) -> (i32, bool) {
        let (prev_v, active) = (self.dig_stage.0.1, self.dig_stage.2 > 0);
        if stage == 0 {
            self.dig_stage = ((0, 0), 0.0, 0);
            return (prev_v, active);
        }
        self.dig_stage = ((u, v), layer_elev, stage.min(3));
        (prev_v, true)
    }

    pub fn top_steps_at(&self, u: i32, v: i32, base_steps: i32) -> i32 {
        let ground_top = (base_steps as f32) * 0.5;
        self.blocks.get(&(u, v)).map_or(base_steps, |list| {
            let max_h = list.iter().map(|b| b.ceiling_above(ground_top)).fold(ground_top, f32::max);
            (max_h * 2.0).round() as i32
        })
    }

    pub fn top_layer_at(&self, u: i32, v: i32, ground_top: f32) -> f32 {
        self.blocks
            .get(&(u, v))
            .and_then(|list| list.last())
            .map_or(ground_top - 0.25, |block| block.ceiling_above(ground_top) - 0.25)
    }

    fn niche_ceiling_at(&self, u: i32, v: i32, ground_h: f32) -> Option<f32> {
        let list = self.blocks.get(&(u, v))?;
        let mut floor = ground_h;
        for block in list {
            let base = block.ceiling_above(ground_h) - 0.5;
            if base <= floor + 0.05 {
                floor = floor.max(block.ceiling_above(ground_h));
            } else if base >= floor + 0.45 {
                return Some(base);
            }
        }
        None
    }

    pub fn solid_wall_at(&self, u: i32, v: i32, ground_h: f32, ceiling_h: f32, elev: f32) -> bool {
        if let Some(niche_ceil) = self.niche_ceiling_at(u, v, ground_h) {
            return elev < ground_h - 0.10 || elev > niche_ceil + 0.85;
        }
        (elev > ceiling_h + 0.10 && elev < ground_h - 0.10) || self.layer_occupied_at(u, v, elev)
    }

    pub fn confine_niche_elevation(&self, u: i32, v: i32, ground_h: f32, elev: f32) -> Option<f32> {
        self.niche_ceiling_at(u, v, ground_h).map(|niche_ceil| elev.clamp(ground_h + 0.18, niche_ceil - 0.18))
    }

    #[allow(clippy::too_many_arguments)]
    pub fn constrain_support(&self, cell_u: i32, cell_v: i32, u: f32, v: f32, mut h: f32, foot_elev: f32, body_span: f32) -> f32 {
        if h < -2.0 && body_span > 1e-4 {
            if let Some(niche_ceil) = self.niche_ceiling_at(cell_u, cell_v, h) {
                if foot_elev <= niche_ceil + 0.10 {
                    return h.min(niche_ceil - 1.98);
                }
            }
        }
        if let Some(list) = self.blocks.get(&(cell_u, cell_v)) {
            let fu = (u * 2.0 - cell_u as f32).clamp(0.0, 1.0);
            let fv = (v * 2.0 - cell_v as f32).clamp(0.0, 1.0);
            for block in list {
                h = block.constrain_support(fu, fv, h, foot_elev, body_span);
            }
        }
        h
    }

    pub fn layer_occupied_at(&self, u: i32, v: i32, layer_elev: f32) -> bool {
        self.blocks.get(&(u, v)).is_some_and(|list| list.iter().any(|b| b.occupies_layer(layer_elev)))
    }

    pub fn layer_medium_at(&self, u: i32, v: i32, layer_elev: f32) -> BlockMedium {
        self.blocks
            .get(&(u, v))
            .and_then(|list| list.iter().rfind(|b| b.occupies_layer(layer_elev)))
            .map_or(BlockMedium::TurfEarth, ElevatedShape::block_medium)
    }

    pub fn hit_top_at(&self, cu: i32, cv: i32, u: f32, v: f32, ray_elev: f32, ground_h: f32) -> Option<f32> {
        let list = self.blocks.get(&(cu, cv))?;
        let (fu, fv) = ((u * 2.0 - cu as f32).clamp(0.0, 1.0), (v * 2.0 - cv as f32).clamp(0.0, 1.0));
        for block in list.iter().rev() {
            if block.intercepts(fu, fv, ray_elev) {
                return Some(block.ceiling_above(ground_h));
            }
        }
        None
    }

    pub fn emit_molded_top(&self, cell: GridCell, surf_base: f32, vault: &super::CrustVault, frame: &SurfaceFrame, top_col: Vec3, wall_col: Vec3, target: &mut dyn MeshTarget) -> bool {
        let (u, v) = cell.feed_coordinates(|u, v| (u, v));
        let Some(list) = self.blocks.get(&(u, v)) else { return false };
        let floor_step = vault.floor_steps_at(u, v);
        for block in list {
            let stage = if self.dig_stage.0 == (u, v) && block.occupies_layer(self.dig_stage.1) { self.dig_stage.2 } else { 0 };
            let layer = (block.ceiling_above(0.0) * 2.0).round() as i32 - 1;
            if layer < floor_step + 4 && block.block_medium() == BlockMedium::TurfEarth {
                vault.emit_crust_block(cell, layer, floor_step, stage, frame, target);
            } else {
                let block_col = if block.floating_above(surf_base) || self.bare_dirt_at(u, v) { wall_col } else { top_col };
                block.emit_terraced(cell, frame, block_col, wall_col, stage, target);
            }
        }
        !list.is_empty()
    }

    pub fn remove_layer(&mut self, u: i32, v: i32, layer_elev: f32) -> Option<i32> {
        let list = self.blocks.get_mut(&(u, v))?;
        let pos = list.iter().rposition(|b| b.occupies_layer(layer_elev))?;
        list.remove(pos);
        if list.is_empty() {
            self.blocks.remove(&(u, v));
            self.bare_dirt.insert((u, v), 0.0);
        }
        Some(self.clear_stage(u, v))
    }

    pub fn retain_overhang(&mut self, u: i32, v: i32, next_elev: i32, elev: i32) {
        let ground_top = (next_elev as f32) * 0.5;
        let list = self.blocks.entry((u, v)).or_default();
        for step in (next_elev + 1)..elev {
            list.push(ElevatedShape::new(Box::new(CuboidShape::new(0.5)), (step as f32) * 0.5));
        }
        list.sort_by(|a, b| a.ceiling_above(ground_top).total_cmp(&b.ceiling_above(ground_top)));
    }

    pub fn stack_molded(&mut self, u: i32, v: i32, ground_top: f32, sub: bool, base: f32, shape: Box<dyn VoxelShape>) -> bool {
        let mut next_base = if sub { base } else { base.max(ground_top) };
        while self.layer_occupied_at(u, v, next_base + 0.25) {
            next_base += if sub { -0.5 } else { 0.5 };
        }
        if next_base >= 16.0 {
            return false;
        }
        let list = self.blocks.entry((u, v)).or_default();
        list.push(ElevatedShape::new(shape, next_base));
        list.sort_by(|a, b| a.ceiling_above(ground_top).total_cmp(&b.ceiling_above(ground_top)));
        self.bare_dirt.insert((u, v), 0.0);
        self.clear_if_at(u, v);
        true
    }

    pub fn step_grass(&mut self, strata: &NaturalStrata, shafts: &ShaftExcavation, dt: f32) -> Vec<i32> {
        if self.bare_dirt.is_empty() {
            return Vec::new();
        }
        let active: Vec<((i32, i32), u32)> = self.bare_dirt.keys().copied().filter_map(|(u, v)| {
            let h = self.top_steps_at(u, v, shafts.elevation_at(strata, u, v));
            let mut count = 0;
            for du in -1..=1 {
                for dv in -1..=1 {
                    let (nu, nv) = (u + du, v + dv);
                    if (du != 0 || dv != 0) && !self.bare_dirt_at(nu, nv) && (h - self.top_steps_at(nu, nv, shafts.elevation_at(strata, nu, nv))).abs() <= 1 {
                        count += 1;
                    }
                }
            }
            (count > 0).then_some(((u, v), count))
        }).collect();
        let mut regrown_rows = Vec::new();
        for ((u, v), neighbors) in active {
            if let Some(effort) = self.bare_dirt.get_mut(&(u, v)) {
                *effort += dt * (neighbors as f32);
                let threshold = REGH_EFFORT_FLOOR + (((u.wrapping_mul(73856093) ^ v.wrapping_mul(19349663)).unsigned_abs() % 100) as f32 * 0.01) * REGH_EFFORT_SPAN;
                if *effort >= threshold {
                    self.bare_dirt.remove(&(u, v));
                    regrown_rows.push(v);
                }
            }
        }
        regrown_rows
    }
}
