use std::collections::HashMap;
use crate::block::{BlockMedium, GridCell};
use super::mineral_vein::{ExposedStratum, HollowVeins, SubterraneanVeins};
use super::natural_strata::NaturalStrata;
use super::overhang_molds::OverhangMolds;

const BLOCK_STEP: f32 = GridCell::SPAN;
const HALF_BLOCK_STEP: f32 = 0.25;

/// Excavated downward shaft deltas and subterranean vein disclosures across a relief face.
pub struct ShaftExcavation {
    deltas: HashMap<(i32, i32), i32>,
    veins: Box<dyn SubterraneanVeins>,
    pending_disclosures: Vec<ExposedStratum>,
}

impl ShaftExcavation {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(
        deltas: HashMap<(i32, i32), i32>,
        veins: Box<dyn SubterraneanVeins>,
        pending_disclosures: Vec<ExposedStratum>,
    ) -> Self {
        Self { deltas, veins, pending_disclosures }
    }

    /// Secondary constructor for unexcavated zero-knowledge terrain without pits.
    pub fn untouched() -> Self {
        Self::with_veins(Box::new(HollowVeins::untouched()))
    }

    /// Secondary constructor equipping a specific subterranean vein authority.
    pub fn with_veins(veins: Box<dyn SubterraneanVeins>) -> Self {
        Self::new(HashMap::new(), veins, Vec::new())
    }

    pub fn pristine(&self) -> bool {
        self.deltas.is_empty()
    }

    pub fn delta_at(&self, u: i32, v: i32) -> i32 {
        if self.deltas.is_empty() { 0 } else { self.deltas.get(&(u, v)).copied().unwrap_or(0) }
    }

    pub fn reworked_at(&self, u: i32, v: i32) -> bool {
        self.delta_at(u, v) != 0
    }

    pub fn record_delta(&mut self, u: i32, v: i32, delta: i32) {
        self.deltas.insert((u, v), delta);
    }

    pub fn elevation_at(&self, strata: &NaturalStrata, u: i32, v: i32) -> i32 {
        strata.elevation_at(u, v) + self.delta_at(u, v)
    }

    pub fn crust_floor_at(&self, strata: &NaturalStrata, u: i32, v: i32) -> i32 {
        strata.vault().floor_steps_at(u, v)
    }

    pub fn exposed_ore_at(&self, u: i32, v: i32, step: i32) -> Option<BlockMedium> {
        self.veins.disclosed_at(u, v, step)
    }

    pub fn accept_disclosure(&mut self, stratum: ExposedStratum) {
        self.veins.accept_disclosure(stratum);
    }

    pub fn drain_disclosures(&mut self, out: &mut Vec<ExposedStratum>) {
        out.append(&mut self.pending_disclosures);
    }

    pub fn deepened_steps(&self, strata: &NaturalStrata, u: i32, v: i32) -> i32 {
        let (elev, floor) = (self.elevation_at(strata, u, v), self.crust_floor_at(strata, u, v));
        if elev > floor { elev - 1 } else { floor }
    }

    fn shared_pair_elevation(&self, strata: &NaturalStrata, u: i32, v: i32, du: i32, dv: i32) -> i32 {
        let neighbor_floor = self.elevation_at(strata, u, v).max(self.elevation_at(strata, u + du, v + dv));
        let shared_excavation = strata.elevation_at(u, v) + self.delta_at(u, v).max(self.delta_at(u + du, v + dv));
        neighbor_floor.min(shared_excavation)
    }

    pub fn body_elevation_at(&self, strata: &NaturalStrata, u: i32, v: i32) -> i32 {
        if !self.reworked_at(u, v) {
            self.elevation_at(strata, u, v)
        } else {
            [(1, 0), (-1, 0), (0, 1), (0, -1)]
                .into_iter()
                .map(|(du, dv)| self.shared_pair_elevation(strata, u, v, du, dv))
                .min()
                .unwrap_or_else(|| self.elevation_at(strata, u, v))
        }
    }

    pub fn diggable_at(&self, strata: &NaturalStrata, u: i32, v: i32, layer_elev: f32) -> bool {
        let step_at = ((layer_elev - HALF_BLOCK_STEP) / BLOCK_STEP).round() as i32;
        let crust = self.crust_floor_at(strata, u, v);
        step_at >= crust && self.deepened_steps(strata, u, v) != self.elevation_at(strata, u, v)
    }

    fn target_dug_step(&self, strata: &NaturalStrata, u: i32, v: i32, layer_elev: f32) -> i32 {
        let (crust, elev) = (self.crust_floor_at(strata, u, v), self.elevation_at(strata, u, v));
        let step_at = ((layer_elev - HALF_BLOCK_STEP) / BLOCK_STEP).round() as i32;
        if step_at <= crust && step_at < elev - 1 {
            crust
        } else if step_at < elev - 1 {
            step_at.max(crust)
        } else {
            elev - 1
        }
    }

    pub fn excavated_medium_at(&self, strata: &NaturalStrata, u: i32, v: i32, layer_elev: f32) -> BlockMedium {
        let dug_step = self.target_dug_step(strata, u, v, layer_elev);
        if let Some(ore) = self.veins.disclosed_at(u, v, dug_step) {
            return ore;
        }
        if strata.vault().crust_layer_at(u, v, dug_step) {
            BlockMedium::CrustRock
        } else {
            BlockMedium::TurfEarth
        }
    }

    pub fn excavate_layer(
        &mut self,
        strata: &NaturalStrata,
        molds: &mut OverhangMolds,
        u: i32,
        v: i32,
        layer_elev: f32,
    ) -> ((i32, i32), Option<BlockMedium>) {
        let (crust, elev, nat) = (self.crust_floor_at(strata, u, v), self.elevation_at(strata, u, v), strata.elevation_at(u, v));
        let step_at = ((layer_elev - HALF_BLOCK_STEP) / BLOCK_STEP).round() as i32;
        if step_at < crust || elev <= crust {
            return ((u, v), None);
        }
        let dug_step = self.target_dug_step(strata, u, v, layer_elev);
        let base_medium = if strata.vault().crust_layer_at(u, v, dug_step) {
            BlockMedium::CrustRock
        } else {
            BlockMedium::TurfEarth
        };
        let next_elev = if step_at < elev - 1 { step_at.max(crust) } else { self.deepened_steps(strata, u, v) };
        if next_elev == elev {
            return ((u, v), None);
        }
        if step_at < elev - 1 {
            molds.retain_overhang(strata, u, v, next_elev, elev);
        }
        molds.expose_dirt(u, v);
        self.record_delta(u, v, next_elev - nat);
        let (medium, mut revealed) = self.veins.excavate_and_expose(strata, &self.deltas, (u, v, dug_step), base_medium);
        self.pending_disclosures.append(&mut revealed);
        (molds.clear_stage(u, v), Some(medium))
    }
}
