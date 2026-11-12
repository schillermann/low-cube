use std::collections::HashMap;
use crate::block::{ElevatedShape, GridCell, TurfSod};
use super::molded_structure::WorldBlocks;
use super::natural_strata::NaturalStrata;
use super::shaft_excavation::ShaftExcavation;
use super::turf_habitat::{CanopyHabitat, NaturalHabitat, TurfHabitat};

const BLOCK_STEP: f32 = GridCell::SPAN;
const HALF_BLOCK_STEP: f32 = 0.25;

/// Active excavation crack focus on a specific grid cell, elevation layer, and stage (0..=3).
#[derive(Clone, Copy, Debug)]
pub struct DigStageFocus {
    cell: (i32, i32),
    layer_elev: f32,
    stage: u8,
}

impl DigStageFocus {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(cell: (i32, i32), layer_elev: f32, stage: u8) -> Self {
        Self {
            cell,
            layer_elev,
            stage,
        }
    }

    /// Secondary constructor for an inactive excavation focus.
    pub const fn idle() -> Self {
        Self::new((0, 0), 0.0, 0)
    }

    pub fn probes_occupied<F: FnOnce(f32) -> bool>(self, predicate: F) -> bool {
        predicate(self.layer_elev)
    }

    pub fn layer_stage(self, u: i32, v: i32, layer_elev: f32, layer_occupied: bool) -> u8 {
        let q = ((layer_elev - HALF_BLOCK_STEP) / BLOCK_STEP).round() as i32;
        let f = ((self.layer_elev - HALF_BLOCK_STEP) / BLOCK_STEP).round() as i32;
        if self.cell == (u, v) && !layer_occupied && q == f {
            self.stage
        } else {
            0
        }
    }

    pub fn block_stage(self, u: i32, v: i32, block: &ElevatedShape) -> u8 {
        if self.cell == (u, v) && block.occupies_layer(self.layer_elev) {
            self.stage
        } else {
            0
        }
    }

    pub fn clear_at(&mut self, u: i32, v: i32) -> (i32, i32) {
        let prev = self.cell;
        *self = Self::new((u, v), 0.0, 0);
        prev
    }

    pub fn clear_if_at(&mut self, u: i32, v: i32) {
        if self.cell == (u, v) {
            *self = Self::new((u, v), 0.0, 0);
        }
    }

    pub fn reset(&mut self) -> ((i32, i32), bool) {
        let (prev, active) = (self.cell, self.stage > 0);
        *self = Self::idle();
        (prev, active)
    }

    pub fn record(&mut self, u: i32, v: i32, layer_elev: f32, stage: u8) -> ((i32, i32), bool) {
        if stage == 0 {
            return self.reset();
        }
        let prev = self.cell;
        *self = Self::new((u, v), layer_elev, stage.min(3));
        (prev, true)
    }
}

/// Bare-soil regrowth progress on the natural ground and active digging crack focus across cells.
pub struct TurfScars {
    bare_dirt: HashMap<(i32, i32), TurfSod>,
    dig_stage: DigStageFocus,
}

impl TurfScars {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(bare_dirt: HashMap<(i32, i32), TurfSod>, dig_stage: DigStageFocus) -> Self {
        Self {
            bare_dirt,
            dig_stage,
        }
    }

    /// Secondary constructor for unscarred turf without bare dirt or digging cracks.
    pub fn untouched() -> Self {
        Self::new(HashMap::new(), DigStageFocus::idle())
    }

    pub fn bare_at(&self, u: i32, v: i32) -> bool {
        self.bare_dirt.contains_key(&(u, v))
    }

    pub fn expose_dirt(&mut self, u: i32, v: i32) {
        self.bare_dirt.insert((u, v), TurfSod::bare());
    }

    pub fn probes_occupied<F: FnOnce(f32) -> bool>(&self, predicate: F) -> bool {
        self.dig_stage.probes_occupied(predicate)
    }

    pub fn layer_stage(&self, u: i32, v: i32, layer_elev: f32, occupied: bool) -> u8 {
        self.dig_stage.layer_stage(u, v, layer_elev, occupied)
    }

    pub fn block_stage(&self, u: i32, v: i32, block: &ElevatedShape) -> u8 {
        self.dig_stage.block_stage(u, v, block)
    }

    pub fn clear_stage(&mut self, u: i32, v: i32) -> (i32, i32) {
        self.dig_stage.clear_at(u, v)
    }

    pub fn clear_if_at(&mut self, u: i32, v: i32) {
        self.dig_stage.clear_if_at(u, v);
    }

    pub fn reset_stage(&mut self) -> ((i32, i32), bool) {
        self.dig_stage.reset()
    }

    pub fn record_stage(&mut self, u: i32, v: i32, layer_elev: f32, stage: u8) -> ((i32, i32), bool) {
        self.dig_stage.record(u, v, layer_elev, stage)
    }

    pub fn regrow_step(
        &mut self,
        strata: &NaturalStrata,
        shafts: &ShaftExcavation,
        blocks: &WorldBlocks,
        dt: f32,
    ) -> Vec<(i32, i32)> {
        if self.bare_dirt.is_empty() {
            return Vec::new();
        }
        let active: Vec<((i32, i32), u32)> = {
            let natural = NaturalHabitat::new(strata, shafts, self);
            let habitat = CanopyHabitat::new(&natural, blocks);
            self.bare_dirt
                .keys()
                .copied()
                .filter_map(|(u, v)| {
                    let g_step = natural.ground_step_at(u, v);
                    let smothered = blocks.covers_ground_at(u, v, (g_step as f32) * BLOCK_STEP);
                    if !natural.fertile_ground(u, v) || smothered {
                        return None;
                    }
                    let count = habitat.verdant_neighbors(u, v, g_step);
                    (count > 0).then_some(((u, v), count))
                })
                .collect()
        };
        let mut regrown_cells = Vec::new();
        for ((u, v), neighbors) in active {
            if let Some(sod) = self.bare_dirt.get_mut(&(u, v)) {
                let (next_sod, grown) = sod.colonized(GridCell::new(u, v), dt, neighbors);
                if grown {
                    self.bare_dirt.remove(&(u, v));
                    regrown_cells.push((u, v));
                } else {
                    *sod = next_sod;
                }
            }
        }
        regrown_cells
    }
}
