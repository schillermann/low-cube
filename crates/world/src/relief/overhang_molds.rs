use std::collections::HashMap;
use glam::Vec3;
use crate::MeshTarget;
use crate::block::{BlockMedium, CuboidShape, ElevatedShape, GridCell, MediumShape, NicheColumn, VoxelShape};
use crate::frame::SurfaceFrame;
use super::cantilever_rig::RigRegistry;
use super::natural_strata::NaturalStrata;
use super::shaft_excavation::ShaftExcavation;
use super::sight_ray::SurfaceSightRay;
use super::turf_scars::TurfScars;
use super::vehicle_assembly::VehicleAssembly;

const BLOCK_STEP: f32 = GridCell::SPAN;
const HALF_BLOCK_STEP: f32 = 0.25;
const NICHE_CONTACT_TOLERANCE: f32 = 0.05;
const NICHE_WALL_MARGIN: f32 = 0.10;
const MAX_STACK_ELEVATION: f32 = 16.0;

/// Overhanging and stacked voxel blocks composed with polyhedral cantilever rigs and surface turf scars.
pub struct OverhangMolds {
    blocks: HashMap<(i32, i32), Vec<ElevatedShape>>,
    rigs: RigRegistry,
    scars: TurfScars,
}

impl OverhangMolds {
    pub fn new(blocks: HashMap<(i32, i32), Vec<ElevatedShape>>, rigs: RigRegistry, scars: TurfScars) -> Self {
        Self { blocks, rigs, scars }
    }

    pub fn untouched() -> Self {
        Self::new(HashMap::new(), RigRegistry::untouched(), TurfScars::untouched())
    }

    pub fn empty(&self) -> bool { self.blocks.is_empty() && self.rigs.empty() }
    pub fn present_at(&self, u: i32, v: i32) -> bool { self.blocks.contains_key(&(u, v)) || self.rigs.present_at(u, v) }
    pub fn bare_dirt_at(&self, u: i32, v: i32) -> bool { self.scars.bare_at(u, v) }
    pub fn expose_dirt(&mut self, u: i32, v: i32) { self.scars.expose_dirt(u, v); }

    pub fn layer_dig_stage_at(&self, u: i32, v: i32, layer_elev: f32) -> u8 {
        self.scars.layer_stage(u, v, layer_elev, self.scars.probes_occupied(|e| self.layer_occupied_at(u, v, e)))
    }

    pub fn clear_stage(&mut self, u: i32, v: i32) -> (i32, i32) { self.scars.clear_stage(u, v) }
    pub fn clear_if_at(&mut self, u: i32, v: i32) { self.scars.clear_if_at(u, v); }
    pub fn reset_stage(&mut self) -> ((i32, i32), bool) { self.scars.reset_stage() }
    pub fn record_stage(&mut self, u: i32, v: i32, layer_elev: f32, stage: u8) -> ((i32, i32), bool) {
        self.scars.record_stage(u, v, layer_elev, stage)
    }

    pub fn top_steps_at(&self, u: i32, v: i32, base_steps: i32) -> i32 {
        let ground_top = (base_steps as f32) * BLOCK_STEP;
        self.blocks.get(&(u, v)).map_or(base_steps, |l| (l.iter().map(ElevatedShape::ceiling).fold(ground_top, f32::max) / BLOCK_STEP).round() as i32)
    }

    pub fn solid_wall_at(&self, u: i32, v: i32, ground_h: f32, ceiling_h: f32, elev: f32) -> bool {
        (elev > ceiling_h + NICHE_WALL_MARGIN && elev < ground_h - NICHE_WALL_MARGIN)
            || self.layer_occupied_at(u, v, elev)
    }

    pub fn confine_niche_elevation(&self, u: i32, v: i32, ground_h: f32, elev: f32) -> Option<f32> {
        NicheColumn::new(self.blocks.get(&(u, v))?, ground_h).confine(elev)
    }

    pub fn constrain_support(&self, cell: GridCell, planar: (f32, f32), mut h: f32, span: (f32, f32)) -> f32 {
        let (cell_u, cell_v) = cell.feed_coordinates(|cu, cv| (cu, cv));
        let ((u, v), (foot_elev, body_span)) = (planar, span);
        h = self.rigs.constrain_support(cell_u, cell_v, h, foot_elev);
        if let Some(list) = self.blocks.get(&(cell_u, cell_v)) {
            let (fu, fv) = cell.local_fraction(u, v);
            for block in list {
                h = block.constrain_support(fu, fv, h, foot_elev, body_span);
            }
        }
        h
    }

    pub fn layer_occupied_at(&self, u: i32, v: i32, layer_elev: f32) -> bool {
        self.rigs.occupies_layer(u, v, layer_elev)
            || self.blocks.get(&(u, v)).is_some_and(|list| list.iter().any(|b| b.occupies_layer(layer_elev)))
    }

    pub fn seat_backrest_cell(&self, u: i32, v: i32, layer_elev: f32) -> Option<(i32, i32)> {
        let block = self.blocks.get(&(u, v))?.iter().rfind(|b| b.occupies_layer(layer_elev))?;
        if block.block_medium() != BlockMedium::CabinSeat {
            return None;
        }
        Some(match block.block_heading() {
            crate::block::WedgeHeading::East => (u - 1, v),
            crate::block::WedgeHeading::West => (u + 1, v),
            crate::block::WedgeHeading::North => (u, v - 1),
            crate::block::WedgeHeading::South => (u, v + 1),
        })
    }

    pub fn layer_medium_at(&self, strata: &NaturalStrata, shafts: &ShaftExcavation, u: i32, v: i32, layer_elev: f32) -> BlockMedium {
        if self.rigs.occupies_layer(u, v, layer_elev) {
            return BlockMedium::CantileverRig;
        }
        self.blocks
            .get(&(u, v))
            .and_then(|list| list.iter().rfind(|b| b.occupies_layer(layer_elev)))
            .map_or_else(|| shafts.excavated_medium_at(strata, u, v, layer_elev), ElevatedShape::block_medium)
    }

    pub fn hit_top_at(&self, cu: i32, cv: i32, u: f32, v: f32, ray_elev: f32) -> Option<f32> {
        if let Some(rig_top) = self.rigs.hit_top(cu, cv, ray_elev, &self.blocks) {
            return Some(rig_top);
        }
        let list = self.blocks.get(&(cu, cv))?;
        let (fu, fv) = GridCell::new(cu, cv).local_fraction(u, v);
        list.iter().rev().find(|b| b.intercepts(fu, fv, ray_elev)).map(ElevatedShape::ceiling)
    }

    pub fn emit_molded_top(&self, cell: GridCell, surf_base: f32, frame: &SurfaceFrame, colors: (Vec3, Vec3), target: &mut dyn MeshTarget) -> bool {
        let (u, v) = cell.feed_coordinates(|u, v| (u, v));
        let rig_emitted = self.rigs.emit_at(cell, frame, target);
        let Some(list) = self.blocks.get(&(u, v)) else { return rig_emitted };
        let (top_col, wall_col) = colors;
        for (idx, block) in list.iter().enumerate() {
            let stage = self.scars.block_stage(u, v, block);
            let bare_top = block.floating_above(surf_base) || self.bare_dirt_at(u, v);
            let flank_only = list.len() > 2
                && idx > 0
                && idx + 1 < list.len()
                && block.full_cube_envelope()
                && list[idx - 1].full_cube_envelope()
                && list[idx + 1].full_cube_envelope()
                && (block.floor() - list[idx - 1].ceiling()).abs() < NICHE_CONTACT_TOLERANCE
                && (list[idx + 1].floor() - block.ceiling()).abs() < NICHE_CONTACT_TOLERANCE;
            block.emit_terraced(cell, frame, top_col, wall_col, bare_top, flank_only, stage, target);
        }
        rig_emitted || !list.is_empty()
    }

    pub fn remove_layer(&mut self, u: i32, v: i32, layer_elev: f32, ground_top: f32) -> Option<((i32, i32), BlockMedium)> {
        if self.rigs.pack_up(u, v, ground_top, &mut self.blocks) {
            return Some((self.clear_stage(u, v), BlockMedium::CantileverRig));
        }
        if self.car_locked_at(u, v, layer_elev) {
            return None;
        }
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

    pub fn car_locked_at(&self, u: i32, v: i32, layer_elev: f32) -> bool {
        let Some(list) = self.blocks.get(&(u, v)) else { return false };
        let Some(pos) = list.iter().rposition(|b| b.occupies_layer(layer_elev)) else { return false };
        let cluster = VehicleAssembly::connected_cluster(u, v, pos, &self.blocks);
        cluster.iter().any(|&(cu, cv, idx)| self.blocks[&(cu, cv)][idx].block_medium() == BlockMedium::SteeringWheel)
            && !self.rigs.cluster_hoisted(&cluster, &self.blocks)
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
        if shape.block_medium() == BlockMedium::CantileverRig {
            self.rigs.deploy(u, v, ground_top);
            self.clear_if_at(u, v);
            return true;
        }
        if let Some((eu, ev, hoist_z)) = self.rigs.mount_target(u, v, &self.blocks) {
            if shape.block_medium() != BlockMedium::SteeringWheel {
                return false;
            }
            let list = self.blocks.entry((eu, ev)).or_default();
            list.push(ElevatedShape::new(shape, hoist_z));
            list.sort_by(|a, b| a.ceiling().total_cmp(&b.ceiling()));
            self.clear_if_at(eu, ev);
            return true;
        }
        if self.rigs.present_at(u, v) && self.rigs.attach_elevation(u, v, &self.blocks).is_none() {
            return false;
        }
        let mut next_base = if sub { base } else { base.max(ground_top) };
        if (next_base - ground_top).abs() <= 0.05 {
            next_base = self.rigs.attach_elevation(u, v, &self.blocks).unwrap_or(next_base);
        }
        while self.blocks.get(&(u, v)).is_some_and(|l| l.iter().any(|b| b.occupies_layer(next_base + HALF_BLOCK_STEP))) {
            next_base += if sub { -BLOCK_STEP } else { BLOCK_STEP };
        }
        if next_base >= MAX_STACK_ELEVATION {
            return false;
        }
        for (du, dv) in [(0, 0), (1, 0), (-1, 0), (0, 1), (0, -1)] {
            if let Some(nlist) = self.blocks.get(&(u + du, v + dv)) {
                if nlist.iter().any(|nb| (nb.floor() - next_base).abs() <= BLOCK_STEP + 0.12 && self.car_locked_at(u + du, v + dv, nb.floor() + HALF_BLOCK_STEP)) {
                    return false;
                }
            }
        }
        self.rigs.retract_for_block(u, v, next_base);
        let list = self.blocks.entry((u, v)).or_default();
        list.push(ElevatedShape::new(shape, next_base));
        list.sort_by(|a, b| a.ceiling().total_cmp(&b.ceiling()));
        if !sub {
            self.scars.expose_dirt(u, v);
        }
        self.scars.reset_stage();
        true
    }

    pub fn adjust_rig(&mut self, strata: &NaturalStrata, shafts: &ShaftExcavation, u: i32, v: i32, delta_steps: i32, rotate_quarter: bool) -> (bool, (i32, i32)) {
        if self.rigs.adjust_at(strata, shafts, u, v, delta_steps, rotate_quarter, &mut self.blocks) {
            return (true, (u, v));
        }
        if rotate_quarter || delta_steps == 0 {
            return (false, (u, v));
        }
        if self.blocks.get_mut(&(u, v)).is_some_and(|l| l.iter_mut().rev().any(|b| b.step_camera_turn(delta_steps))) {
            return (true, (u, v));
        }
        self.blocks
            .iter_mut()
            .find_map(|(&cell, list)| list.iter_mut().rev().any(|b| b.step_camera_turn(delta_steps)).then_some((true, cell)))
            .unwrap_or((false, (u, v)))
    }

    pub fn placed_camera_tripod(&self, frame: &SurfaceFrame) -> Option<(Vec3, Vec3, Vec3)> {
        self.blocks
            .iter()
            .find_map(|(&(u, v), list)| list.iter().rev().find_map(|b| b.camera_tripod_hub(GridCell::new(u, v), frame)))
    }

    pub fn lower_rig(&mut self, strata: &NaturalStrata, shafts: &ShaftExcavation, u: i32, v: i32) -> bool {
        self.rigs.lower_mast_at(strata, shafts, u, v, &mut self.blocks)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn board_vehicle(&mut self, strata: &NaturalStrata, shafts: &ShaftExcavation, frame: &SurfaceFrame, u: i32, v: i32, layer_elev: f32, ray: SurfaceSightRay, engage_engine: bool) -> Option<VehicleAssembly> {
        VehicleAssembly::board_from_blocks(&mut self.blocks, &mut self.rigs, strata, shafts, frame, u, v, layer_elev, ray, engage_engine)
    }

    pub fn park_vehicle(&mut self, strata: &NaturalStrata, shafts: &ShaftExcavation, frame: &SurfaceFrame, assembly: VehicleAssembly) -> (i32, i32) {
        assembly.park_into_blocks(&mut self.blocks, strata, shafts, frame)
    }

    pub fn step_grass(&mut self, strata: &NaturalStrata, shafts: &ShaftExcavation, dt: f32) -> Vec<(i32, i32)> {
        let mut settled_cells = Vec::new();
        for (&(u, v), list) in &mut self.blocks {
            let ground_top = (shafts.elevation_at(strata, u, v) as f32) * BLOCK_STEP;
            let mut moved = false;
            for idx in 0..list.len() {
                let floor_limit = if idx == 0 { ground_top } else { list[idx - 1].ceiling() };
                if list[idx].settles_under_gravity() && list[idx].floor() >= floor_limit + BLOCK_STEP - NICHE_CONTACT_TOLERANCE {
                    list[idx].settle_step();
                    moved = true;
                }
            }
            if moved {
                list.sort_by(|a, b| a.ceiling().total_cmp(&b.ceiling()));
                settled_cells.push((u, v));
            }
        }
        let blocks = &self.blocks;
        let mut regrown = self.scars.regrow_step(dt, |u, v| {
            let base = shafts.elevation_at(strata, u, v);
            blocks.get(&(u, v)).map_or(base, |l| (l.iter().map(ElevatedShape::ceiling).fold((base as f32) * BLOCK_STEP, f32::max) / BLOCK_STEP).round() as i32)
        });
        regrown.extend(settled_cells);
        regrown
    }
}
