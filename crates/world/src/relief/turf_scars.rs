use std::collections::HashMap;
use crate::block::{ElevatedShape, GridCell, TurfSod};
use super::molded_structure::WorldBlocks;
use super::natural_strata::NaturalStrata;
use super::shaft_excavation::ShaftExcavation;
use super::turf_habitat::{CanopyHabitat, NaturalHabitat, TurfHabitat};

const BLOCK_STEP: f32 = GridCell::SPAN;
const HALF_BLOCK_STEP: f32 = 0.25;
const CRACK_LEASE_SECS: f32 = 1.2;

/// Active excavation crack on a single discrete elevation step with self-healing idle lease.
#[derive(Clone, Copy, Debug)]
pub struct LayerCrack {
    layer_step: i32,
    stage: u8,
    lease: f32,
}

impl LayerCrack {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(layer_step: i32, stage: u8, lease: f32) -> Self {
        Self { layer_step, stage, lease }
    }

    fn quantize_step(layer_elev: f32) -> i32 {
        ((layer_elev - HALF_BLOCK_STEP) / BLOCK_STEP).round() as i32
    }

    fn fresh(layer_elev: f32, stage: u8) -> Self {
        Self::new(Self::quantize_step(layer_elev), stage.min(3), CRACK_LEASE_SECS)
    }

    fn matches_step(self, layer_elev: f32) -> bool {
        Self::quantize_step(layer_elev) == self.layer_step
    }

    const fn elevation(self) -> f32 {
        (self.layer_step as f32) * BLOCK_STEP + HALF_BLOCK_STEP
    }

    fn aged(self, dt: f32) -> Option<Self> {
        let remaining = self.lease - dt;
        (remaining > 0.0).then_some(Self::new(self.layer_step, self.stage, remaining))
    }
}

/// Multi-cell excavation crack map allowing independent simultaneous digging across companions.
pub struct ExcavationCracks {
    cells: HashMap<(i32, i32), LayerCrack>,
}

impl ExcavationCracks {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(cells: HashMap<(i32, i32), LayerCrack>) -> Self {
        Self { cells }
    }

    /// Secondary constructor for uncracked terrain.
    pub fn untouched() -> Self {
        Self::new(HashMap::new())
    }

    pub fn dormant(&self) -> bool {
        self.cells.is_empty()
    }

    pub fn clear_in(&mut self, (u0, u1): (i32, i32), (v0, v1): (i32, i32)) {
        self.cells.retain(|&(u, v), _| u < u0 || u >= u1 || v < v0 || v >= v1);
    }

    pub fn layer_stage(&self, u: i32, v: i32, layer_elev: f32, occupied_at: impl FnOnce(f32) -> bool) -> u8 {
        let Some(&crack) = self.cells.get(&(u, v)) else { return 0 };
        if crack.matches_step(layer_elev) && !occupied_at(crack.elevation()) {
            crack.stage
        } else {
            0
        }
    }

    pub fn block_stage(&self, u: i32, v: i32, block: &ElevatedShape) -> u8 {
        let Some(&crack) = self.cells.get(&(u, v)) else { return 0 };
        if block.occupies_layer(crack.elevation()) { crack.stage } else { 0 }
    }

    pub fn clear_at(&mut self, u: i32, v: i32) -> bool {
        self.cells.remove(&(u, v)).is_some()
    }

    pub fn record(&mut self, u: i32, v: i32, layer_elev: f32, stage: u8) -> bool {
        if stage == 0 {
            return self.clear_at(u, v);
        }
        self.cells.insert((u, v), LayerCrack::fresh(layer_elev, stage));
        true
    }

    pub fn expire_step(&mut self, dt: f32) -> Vec<(i32, i32)> {
        if self.cells.is_empty() {
            return Vec::new();
        }
        let mut expired = Vec::new();
        self.cells.retain(|&cell, crack| match crack.aged(dt) {
            Some(next) => {
                *crack = next;
                true
            }
            None => {
                expired.push(cell);
                false
            }
        });
        expired
    }
}

/// Bare-soil regrowth progress on the natural ground and per-cell digging cracks.
pub struct TurfScars {
    bare_dirt: HashMap<(i32, i32), TurfSod>,
    cracks: ExcavationCracks,
}

impl TurfScars {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(bare_dirt: HashMap<(i32, i32), TurfSod>, cracks: ExcavationCracks) -> Self {
        Self { bare_dirt, cracks }
    }

    /// Secondary constructor for unscarred turf without bare dirt or digging cracks.
    pub fn untouched() -> Self {
        Self::new(HashMap::new(), ExcavationCracks::untouched())
    }

    pub fn bare_at(&self, u: i32, v: i32) -> bool {
        self.bare_dirt.contains_key(&(u, v))
    }

    pub fn expose_dirt(&mut self, u: i32, v: i32) {
        self.bare_dirt.insert((u, v), TurfSod::bare());
    }

    pub fn layer_stage(&self, u: i32, v: i32, layer_elev: f32, occupied_at: impl FnOnce(f32) -> bool) -> u8 {
        self.cracks.layer_stage(u, v, layer_elev, occupied_at)
    }

    pub fn block_stage(&self, u: i32, v: i32, block: &ElevatedShape) -> u8 {
        self.cracks.block_stage(u, v, block)
    }

    pub fn clear_stage(&mut self, u: i32, v: i32) -> bool {
        self.cracks.clear_at(u, v)
    }

    pub fn record_stage(&mut self, u: i32, v: i32, layer_elev: f32, stage: u8) -> bool {
        self.cracks.record(u, v, layer_elev, stage)
    }

    pub fn regrow_step(
        &mut self,
        strata: &NaturalStrata,
        shafts: &ShaftExcavation,
        blocks: &WorldBlocks,
        dt: f32,
    ) -> (Vec<(i32, i32)>, bool) {
        let mut refreshed = self.cracks.expire_step(dt);
        if self.bare_dirt.is_empty() {
            return (refreshed, !self.cracks.dormant());
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
        let mut still_colonizing = !self.cracks.dormant();
        for ((u, v), neighbors) in active {
            if let Some(sod) = self.bare_dirt.get_mut(&(u, v)) {
                let (next_sod, grown) = sod.colonized(GridCell::new(u, v), dt, neighbors);
                if grown {
                    self.bare_dirt.remove(&(u, v));
                    refreshed.push((u, v));
                } else {
                    *sod = next_sod;
                    still_colonizing = true;
                }
            }
        }
        (refreshed, still_colonizing)
    }

    pub fn capture_bare_mask(&self, gsu: i32, gsv: i32) -> u64 {
        let (u0, v0) = (gsu * 8, gsv * 8);
        let mut mask = 0u64;
        for dv in 0..8 {
            for du in 0..8 {
                if self.bare_dirt.contains_key(&(u0 + du, v0 + dv)) {
                    let bit = (du as u32) | ((dv as u32) << 3);
                    mask |= 1u64 << bit;
                }
            }
        }
        mask
    }

    pub fn apply_bare_mask(&mut self, gsu: i32, gsv: i32, bare_mask: u64) {
        let (u0, v0) = (gsu * 8, gsv * 8);
        self.bare_dirt.retain(|&(u, v), _| u < u0 || u >= u0 + 8 || v < v0 || v >= v0 + 8);
        self.cracks.clear_in((u0, u0 + 8), (v0, v0 + 8));
        for bit in 0..64u32 {
            if (bare_mask >> bit) & 1 != 0 {
                let (u, v) = crate::SectorSnapshot::unpack_uv(bit as u8, gsu, gsv);
                self.bare_dirt.insert((u, v), TurfSod::bare());
            }
        }
    }

    pub fn collect_modified_sectors(&self, out: &mut Vec<(i32, i32)>) {
        for &(u, v) in self.bare_dirt.keys() {
            out.push((u.div_euclid(8), v.div_euclid(8)));
        }
    }
}
