use std::collections::HashMap;
use crate::block::GridCell;
use super::natural_strata::NaturalStrata;
use super::overhang_molds::OverhangMolds;

const BLOCK_STEP: f32 = GridCell::SPAN;
const HALF_BLOCK_STEP: f32 = 0.25;
const SHAFT_FLOOR_TOLERANCE: f32 = 0.1;
const CEILING_CLEARANCE_INSET: f32 = 0.30;
const NEIGHBOR_ESCAPE_MARGIN: f32 = 0.02;
const SHAFT_WALL_MARGIN: f32 = 0.21;
const UPWARD_CEILING_CATCH_SPAN: f32 = 2.0;
const UPWARD_CEILING_FLOOR_INSET: f32 = 0.15;
const VAULT_CEILING_FLOOR_INSET: f32 = 0.55;
const SURFACE_HIT_TOLERANCE: f32 = 0.02;
const CEILING_HIT_MARGIN: f32 = 0.05;

/// Excavated downward shaft deltas and upward crust-ceiling breach steps across a relief face.
pub struct ShaftExcavation {
    deltas: HashMap<(i32, i32), i32>,
    ceilings: HashMap<(i32, i32), i32>,
}

impl ShaftExcavation {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(deltas: HashMap<(i32, i32), i32>, ceilings: HashMap<(i32, i32), i32>) -> Self {
        Self { deltas, ceilings }
    }

    /// Secondary constructor for unexcavated terrain without pits or ceiling breaches.
    pub fn untouched() -> Self {
        Self::new(HashMap::new(), HashMap::new())
    }

    pub fn pristine(&self) -> bool {
        self.deltas.is_empty()
    }

    pub fn delta_at(&self, u: i32, v: i32) -> i32 {
        if self.deltas.is_empty() { 0 } else { self.deltas.get(&(u, v)).copied().unwrap_or(0) }
    }

    pub fn upward_at(&self, u: i32, v: i32) -> i32 {
        if self.ceilings.is_empty() { 0 } else { self.ceilings.get(&(u, v)).copied().unwrap_or(0) }
    }

    pub fn reworked_at(&self, u: i32, v: i32) -> bool {
        self.delta_at(u, v) != 0
    }

    pub fn record_delta(&mut self, u: i32, v: i32, delta: i32) {
        self.deltas.insert((u, v), delta);
    }

    pub fn record_ceiling(&mut self, u: i32, v: i32, steps: i32) {
        self.ceilings.insert((u, v), steps);
    }

    pub fn elevation_at(&self, strata: &NaturalStrata, u: i32, v: i32) -> i32 {
        strata.elevation_at(u, v) + self.delta_at(u, v)
    }

    pub fn crust_floor_at(&self, strata: &NaturalStrata, u: i32, v: i32) -> i32 {
        strata.vault().floor_steps_at(u, v) + self.upward_at(u, v)
    }

    pub fn breached_at(&self, strata: &NaturalStrata, u: i32, v: i32) -> bool {
        self.reworked_at(u, v) && self.elevation_at(strata, u, v) < strata.vault().floor_steps_at(u, v)
    }

    pub fn adjacent_upward_at(&self, strata: &NaturalStrata, u: i32, v: i32) -> i32 {
        if self.ceilings.is_empty() {
            return i32::MIN;
        }
        [(-1, 0), (1, 0), (0, -1), (0, 1), (-1, -1), (1, -1), (-1, 1), (1, 1)]
            .iter()
            .filter(|&&(du, dv)| self.upward_at(u + du, v + dv) > 0)
            .map(|&(du, dv)| self.crust_floor_at(strata, u + du, v + dv))
            .max()
            .unwrap_or(i32::MIN)
    }

    pub fn deepened_steps(&self, strata: &NaturalStrata, u: i32, v: i32) -> i32 {
        let (elev, floor, vault) = (self.elevation_at(strata, u, v), self.crust_floor_at(strata, u, v), strata.vault());
        if elev < floor {
            elev
        } else if elev > floor + 1 {
            elev - 1
        } else if vault.cell_opens(u, v) {
            vault.core().summit_steps().min(floor - 2)
        } else {
            floor
        }
    }

    fn shared_pair_elevation(&self, strata: &NaturalStrata, u: i32, v: i32, du: i32, dv: i32) -> i32 {
        let neighbor_floor = self.elevation_at(strata, u, v).max(self.elevation_at(strata, u + du, v + dv));
        let shared_excavation = strata.elevation_at(u, v) + self.delta_at(u, v).max(self.delta_at(u + du, v + dv));
        neighbor_floor.min(shared_excavation)
    }

    pub fn body_elevation_at(&self, strata: &NaturalStrata, u: i32, v: i32) -> i32 {
        if !self.reworked_at(u, v) || self.upward_at(u, v) > 0 || self.adjacent_upward_at(strata, u, v) > i32::MIN {
            self.elevation_at(strata, u, v)
        } else {
            [(1, 0), (-1, 0), (0, 1), (0, -1)]
                .into_iter()
                .map(|(du, dv)| self.shared_pair_elevation(strata, u, v, du, dv))
                .min()
                .unwrap_or_else(|| self.elevation_at(strata, u, v))
        }
    }

    pub fn in_unbreached_shaft(&self, strata: &NaturalStrata, u: i32, v: i32, elev: f32) -> bool {
        if !self.reworked_at(u, v) {
            return false;
        }
        let body = self.body_elevation_at(strata, u, v);
        body >= strata.vault().floor_steps_at(u, v) && elev >= (body as f32) * BLOCK_STEP - SHAFT_FLOOR_TOLERANCE
    }

    pub fn in_upward_shaft(&self, strata: &NaturalStrata, u: i32, v: i32, elev: f32) -> bool {
        self.upward_at(u, v) > 0 && !self.breached_at(strata, u, v) && elev < (self.crust_floor_at(strata, u, v) as f32) * BLOCK_STEP
    }

    fn cell_ceiling_at(&self, strata: &NaturalStrata, cu: i32, cv: i32) -> f32 {
        if self.breached_at(strata, cu, cv) {
            f32::INFINITY
        } else if self.upward_at(cu, cv) > 0 {
            (self.crust_floor_at(strata, cu, cv) as f32) * BLOCK_STEP - CEILING_CLEARANCE_INSET
        } else {
            let (cx, cy) = GridCell::new(cu, cv).center();
            strata.vault().ceiling_height_at(cx, cy) - CEILING_CLEARANCE_INSET
        }
    }

    fn solid_wall_at(&self, strata: &NaturalStrata, molds: &OverhangMolds, cu: i32, cv: i32, elev: f32) -> bool {
        if self.breached_at(strata, cu, cv) {
            return molds.layer_occupied_at(cu, cv, elev);
        }
        let ground = (self.elevation_at(strata, cu, cv) as f32) * BLOCK_STEP;
        let ceiling = self.cell_ceiling_at(strata, cu, cv);
        molds.solid_wall_at(cu, cv, ground, ceiling, elev)
    }

    pub fn confine(&self, strata: &NaturalStrata, molds: &OverhangMolds, u: f32, v: f32, elev: f32) -> (f32, f32, f32) {
        if self.pristine() && self.ceilings.is_empty() && molds.empty() {
            return (u, v, strata.vault().confine(u, v, elev));
        }
        let (mut cu, mut cv) = GridCell::at_planar(u, v).feed_coordinates(|cu, cv| (cu, cv));
        let (mut tu, mut tv) = (u, v);
        let natural_top = (strata.elevation_at(cu, cv) as f32) * BLOCK_STEP;
        if self.ceilings.is_empty() && molds.empty() && elev >= natural_top {
            return (u, v, elev);
        }
        let sub_surface = !self.ceilings.is_empty() || elev < natural_top;
        if sub_surface && self.solid_wall_at(strata, molds, cu, cv, elev) {
            for (du, dv) in [(-1, 0), (1, 0), (0, -1), (0, 1), (-1, -1), (1, -1), (-1, 1), (1, 1)] {
                let (nu, nv) = (cu + du, cv + dv);
                if (self.upward_at(nu, nv) > 0 || self.breached_at(strata, nu, nv) || molds.present_at(nu, nv))
                    && !self.solid_wall_at(strata, molds, nu, nv, elev)
                {
                    (tu, tv) = GridCell::new(nu, nv).clamp_interior(u, v, NEIGHBOR_ESCAPE_MARGIN);
                    (cu, cv) = (nu, nv);
                    break;
                }
            }
        }
        if sub_surface && (self.upward_at(cu, cv) > 0 || self.adjacent_upward_at(strata, cu, cv) > i32::MIN) && !self.solid_wall_at(strata, molds, cu, cv, elev) {
            let walls = (
                self.solid_wall_at(strata, molds, cu - 1, cv, elev),
                self.solid_wall_at(strata, molds, cu + 1, cv, elev),
                self.solid_wall_at(strata, molds, cu, cv - 1, elev),
                self.solid_wall_at(strata, molds, cu, cv + 1, elev),
            );
            (tu, tv) = GridCell::new(cu, cv).clamp_walled(tu, tv, SHAFT_WALL_MARGIN, walls);
        }
        let ground = (self.elevation_at(strata, cu, cv) as f32) * BLOCK_STEP;
        let z = if self.breached_at(strata, cu, cv) {
            elev
        } else if let Some(confined) = molds.confine_niche_elevation(cu, cv, ground, elev) {
            confined
        } else if self.upward_at(cu, cv) > 0 {
            let ceiling = (self.crust_floor_at(strata, cu, cv) as f32) * BLOCK_STEP - CEILING_CLEARANCE_INSET;
            if elev < ceiling + UPWARD_CEILING_CATCH_SPAN && elev > ceiling { ceiling } else { elev }
        } else {
            strata.vault().confine(tu, tv, elev)
        };
        (tu, tv, z)
    }

    pub fn crust_hit_top_at(&self, strata: &NaturalStrata, cu: i32, cv: i32, u: f32, v: f32, ray_elev: f32) -> Option<f32> {
        let (ground_h, ceil_h) = ((self.elevation_at(strata, cu, cv) as f32) * BLOCK_STEP, (self.crust_floor_at(strata, cu, cv) as f32) * BLOCK_STEP);
        let ground_surf = if self.reworked_at(cu, cv) { ground_h } else { strata.surface_height_at(u, v) };
        let ceil_floor = if self.upward_at(cu, cv) > 0 {
            ceil_h - UPWARD_CEILING_FLOOR_INSET
        } else {
            (strata.vault().ceiling_height_at(u, v) - VAULT_CEILING_FLOOR_INSET).min(ground_h - VAULT_CEILING_FLOOR_INSET)
        };
        if !self.breached_at(strata, cu, cv) && ray_elev >= ceil_floor && ray_elev <= ground_surf + SURFACE_HIT_TOLERANCE {
            let hit = if ray_elev < ceil_h - CEILING_HIT_MARGIN {
                ceil_h - HALF_BLOCK_STEP
            } else if ray_elev < ground_h - BLOCK_STEP {
                ((ray_elev / BLOCK_STEP).floor() * BLOCK_STEP + BLOCK_STEP).min(ground_h)
            } else {
                ground_h
            };
            Some(hit)
        } else {
            None
        }
    }

    pub fn diggable_at(&self, strata: &NaturalStrata, u: i32, v: i32, layer_elev: f32) -> bool {
        let up_dig = layer_elev < (self.crust_floor_at(strata, u, v).max(self.adjacent_upward_at(strata, u, v)) as f32) * BLOCK_STEP + HALF_BLOCK_STEP;
        !self.breached_at(strata, u, v) && (self.deepened_steps(strata, u, v) != self.elevation_at(strata, u, v) || up_dig)
    }

    pub fn excavate_layer(&mut self, strata: &NaturalStrata, molds: &mut OverhangMolds, u: i32, v: i32, layer_elev: f32) -> (i32, bool) {
        let (crust, adj_up, nat) = (self.crust_floor_at(strata, u, v), self.adjacent_upward_at(strata, u, v), strata.elevation_at(u, v));
        if layer_elev < (crust as f32) * BLOCK_STEP {
            if self.breached_at(strata, u, v) || (adj_up <= crust && !strata.vault().cell_opens(u, v)) {
                return (v, false);
            }
            let next_crust = crust + 1;
            if next_crust >= self.elevation_at(strata, u, v) {
                self.record_delta(u, v, strata.vault().core().summit_steps().min(crust - 2) - nat);
                molds.expose_dirt(u, v);
            } else {
                self.record_ceiling(u, v, next_crust - strata.vault().floor_steps_at(u, v));
            }
            return (molds.clear_stage(u, v), true);
        }
        let (elev, step_at) = (self.elevation_at(strata, u, v), ((layer_elev - HALF_BLOCK_STEP) / BLOCK_STEP).round() as i32);
        let next_elev = if step_at < elev - 1 { step_at.max(crust) } else { self.deepened_steps(strata, u, v) };
        if next_elev == elev {
            return (v, false);
        }
        if step_at < elev - 1 {
            molds.retain_overhang(u, v, next_elev, elev);
        } else if !molds.present_at(u, v) {
            molds.expose_dirt(u, v);
        }
        self.record_delta(u, v, next_elev - nat);
        (molds.clear_stage(u, v), true)
    }
}
