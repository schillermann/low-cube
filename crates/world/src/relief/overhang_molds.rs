use std::collections::HashMap;
use glam::Vec3;
use crate::MeshTarget;
use crate::block::{BlockMedium, CuboidShape, ElevatedShape, GridCell, MediumShape, VoxelShape};
use crate::frame::SurfaceFrame;
use super::natural_strata::NaturalStrata;
use super::shaft_excavation::ShaftExcavation;
use super::turf_scars::TurfScars;

const BLOCK_STEP: f32 = GridCell::SPAN;
const HALF_BLOCK_STEP: f32 = 0.25;
const NICHE_CONTACT_TOLERANCE: f32 = 0.05;
const NICHE_MIN_GAP: f32 = 0.45;
const NICHE_WALL_MARGIN: f32 = 0.10;
const NICHE_ROOF_THICKNESS: f32 = 0.85;
const NICHE_HEAD_CLEARANCE: f32 = 0.15;
const SUB_METER_NICHE_SPAN: f32 = 0.75;
const MAX_STACK_ELEVATION: f32 = 16.0;

/// Overhanging and stacked voxel blocks composed with surface turf scars.
pub struct OverhangMolds {
    blocks: HashMap<(i32, i32), Vec<ElevatedShape>>,
    scars: TurfScars,
}

impl OverhangMolds {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(blocks: HashMap<(i32, i32), Vec<ElevatedShape>>, scars: TurfScars) -> Self {
        Self { blocks, scars }
    }

    /// Secondary constructor for pristine terrain without placed blocks or bare soil.
    pub fn untouched() -> Self {
        Self::new(HashMap::new(), TurfScars::untouched())
    }

    pub fn empty(&self) -> bool {
        self.blocks.is_empty()
    }

    pub fn present_at(&self, u: i32, v: i32) -> bool {
        self.blocks.contains_key(&(u, v))
    }

    pub fn bare_dirt_at(&self, u: i32, v: i32) -> bool {
        self.scars.bare_at(u, v)
    }

    pub fn expose_dirt(&mut self, u: i32, v: i32) {
        self.scars.expose_dirt(u, v);
    }

    pub fn layer_dig_stage_at(&self, u: i32, v: i32, layer_elev: f32) -> u8 {
        let occupied = self.scars.probes_occupied(|focus_elev| self.layer_occupied_at(u, v, focus_elev));
        self.scars.layer_stage(u, v, layer_elev, occupied)
    }

    pub fn clear_stage(&mut self, u: i32, v: i32) -> (i32, i32) {
        self.scars.clear_stage(u, v)
    }

    pub fn clear_if_at(&mut self, u: i32, v: i32) {
        self.scars.clear_if_at(u, v);
    }

    pub fn reset_stage(&mut self) -> ((i32, i32), bool) {
        self.scars.reset_stage()
    }

    pub fn record_stage(&mut self, u: i32, v: i32, layer_elev: f32, stage: u8) -> ((i32, i32), bool) {
        self.scars.record_stage(u, v, layer_elev, stage)
    }

    pub fn top_steps_at(&self, u: i32, v: i32, base_steps: i32) -> i32 {
        let ground_top = (base_steps as f32) * BLOCK_STEP;
        self.blocks.get(&(u, v)).map_or(base_steps, |list| {
            let max_h = list.iter().map(ElevatedShape::ceiling).fold(ground_top, f32::max);
            (max_h / BLOCK_STEP).round() as i32
        })
    }

    fn niche_ceiling_at(&self, u: i32, v: i32, ground_h: f32) -> Option<f32> {
        let list = self.blocks.get(&(u, v))?;
        let mut floor = ground_h;
        for block in list {
            let base = block.floor();
            if base <= floor + NICHE_CONTACT_TOLERANCE {
                floor = floor.max(block.ceiling());
            } else if base >= floor + NICHE_MIN_GAP {
                return Some(base);
            }
        }
        None
    }

    pub fn solid_wall_at(&self, u: i32, v: i32, ground_h: f32, ceiling_h: f32, elev: f32) -> bool {
        if let Some(niche_ceil) = self.niche_ceiling_at(u, v, ground_h) {
            return elev < ground_h - NICHE_WALL_MARGIN || elev > niche_ceil + NICHE_ROOF_THICKNESS;
        }
        (elev > ceiling_h + NICHE_WALL_MARGIN && elev < ground_h - NICHE_WALL_MARGIN) || self.layer_occupied_at(u, v, elev)
    }

    pub fn confine_niche_elevation(&self, u: i32, v: i32, ground_h: f32, elev: f32) -> Option<f32> {
        self.niche_ceiling_at(u, v, ground_h)
            .filter(|&niche_ceil| elev <= niche_ceil + SUB_METER_NICHE_SPAN)
            .map(|niche_ceil| {
                let min_z = ground_h + NICHE_HEAD_CLEARANCE;
                elev.clamp(min_z, (niche_ceil - NICHE_HEAD_CLEARANCE).max(min_z))
            })
    }

    #[allow(clippy::too_many_arguments)]
    pub fn constrain_support(&self, cell_u: i32, cell_v: i32, u: f32, v: f32, mut h: f32, foot_elev: f32, body_span: f32) -> f32 {
        if body_span > 1e-4 {
            if let Some(niche_ceil) = self.niche_ceiling_at(cell_u, cell_v, h) {
                if niche_ceil - h < SUB_METER_NICHE_SPAN && foot_elev <= niche_ceil + NICHE_WALL_MARGIN {
                    return h;
                }
            }
        }
        if let Some(list) = self.blocks.get(&(cell_u, cell_v)) {
            let (fu, fv) = GridCell::new(cell_u, cell_v).local_fraction(u, v);
            for block in list {
                h = block.constrain_support(fu, fv, h, foot_elev, body_span);
            }
        }
        h
    }

    pub fn layer_occupied_at(&self, u: i32, v: i32, layer_elev: f32) -> bool {
        self.blocks.get(&(u, v)).is_some_and(|list| list.iter().any(|b| b.occupies_layer(layer_elev)))
    }

    pub fn layer_medium_at(
        &self,
        strata: &NaturalStrata,
        shafts: &ShaftExcavation,
        u: i32,
        v: i32,
        layer_elev: f32,
    ) -> BlockMedium {
        self.blocks
            .get(&(u, v))
            .and_then(|list| list.iter().rfind(|b| b.occupies_layer(layer_elev)))
            .map_or_else(
                || shafts.excavated_medium_at(strata, u, v, layer_elev),
                ElevatedShape::block_medium,
            )
    }

    pub fn hit_top_at(&self, cu: i32, cv: i32, u: f32, v: f32, ray_elev: f32) -> Option<f32> {
        let list = self.blocks.get(&(cu, cv))?;
        let (fu, fv) = GridCell::new(cu, cv).local_fraction(u, v);
        for block in list.iter().rev() {
            if block.intercepts(fu, fv, ray_elev) {
                return Some(block.ceiling());
            }
        }
        None
    }

    pub fn emit_molded_top(&self, cell: GridCell, surf_base: f32, frame: &SurfaceFrame, colors: (Vec3, Vec3), target: &mut dyn MeshTarget) -> bool {
        let (u, v) = cell.feed_coordinates(|u, v| (u, v));
        let Some(list) = self.blocks.get(&(u, v)) else { return false };
        let (top_col, wall_col) = colors;
        for block in list {
            let stage = self.scars.block_stage(u, v, block);
            let bare_top = block.floating_above(surf_base) || self.bare_dirt_at(u, v);
            block.emit_terraced(cell, frame, top_col, wall_col, bare_top, stage, target);
        }
        !list.is_empty()
    }

    pub fn remove_layer(&mut self, u: i32, v: i32, layer_elev: f32, ground_top: f32) -> Option<((i32, i32), BlockMedium)> {
        let list = self.blocks.get_mut(&(u, v))?;
        let pos = list.iter().rposition(|b| b.occupies_layer(layer_elev))?;
        let removed = list.remove(pos);
        if list.is_empty() {
            self.blocks.remove(&(u, v));
            if removed.floor() >= ground_top - NICHE_CONTACT_TOLERANCE {
                self.scars.expose_dirt(u, v);
            }
        }
        Some((self.clear_stage(u, v), removed.block_medium()))
    }

    pub fn retain_overhang(&mut self, strata: &NaturalStrata, u: i32, v: i32, next_elev: i32, elev: i32) {
        let (nat, vault) = (strata.elevation_at(u, v), strata.vault());
        let list = self.blocks.entry((u, v)).or_default();
        for step in (next_elev + 1)..elev {
            let shape: Box<dyn VoxelShape> = if vault.crust_layer_at(u, v, step) {
                Box::new(MediumShape::new(Box::new(CuboidShape::new(BLOCK_STEP)), BlockMedium::CrustRock, true, false))
            } else if step == nat - 1 {
                strata.crown_shape_at(u, v)
            } else {
                Box::new(CuboidShape::new(BLOCK_STEP))
            };
            list.push(ElevatedShape::new(shape, (step as f32) * BLOCK_STEP));
        }
        list.sort_by(|a, b| a.ceiling().total_cmp(&b.ceiling()));
    }

    pub fn stack_molded(&mut self, u: i32, v: i32, ground_top: f32, sub: bool, base: f32, shape: Box<dyn VoxelShape>) -> bool {
        let mut next_base = if sub { base } else { base.max(ground_top) };
        while self.layer_occupied_at(u, v, next_base + HALF_BLOCK_STEP) {
            next_base += if sub { -BLOCK_STEP } else { BLOCK_STEP };
        }
        if next_base >= MAX_STACK_ELEVATION {
            return false;
        }
        let list = self.blocks.entry((u, v)).or_default();
        list.push(ElevatedShape::new(shape, next_base));
        list.sort_by(|a, b| a.ceiling().total_cmp(&b.ceiling()));
        if !sub {
            self.scars.expose_dirt(u, v);
        }
        self.clear_if_at(u, v);
        true
    }

    pub fn step_grass(&mut self, strata: &NaturalStrata, shafts: &ShaftExcavation, dt: f32) -> Vec<(i32, i32)> {
        let blocks = &self.blocks;
        self.scars.regrow_step(dt, |u, v| {
            let base_steps = shafts.elevation_at(strata, u, v);
            let ground_top = (base_steps as f32) * BLOCK_STEP;
            blocks.get(&(u, v)).map_or(base_steps, |list| {
                let max_h = list.iter().map(ElevatedShape::ceiling).fold(ground_top, f32::max);
                (max_h / BLOCK_STEP).round() as i32
            })
        })
    }
}
