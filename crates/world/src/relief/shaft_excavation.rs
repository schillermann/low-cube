use std::collections::HashMap;
use super::natural_strata::NaturalStrata;

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

    fn pair_elevation(&self, strata: &NaturalStrata, u: i32, v: i32, du: i32, dv: i32) -> i32 {
        let highest = self.elevation_at(strata, u, v).max(self.elevation_at(strata, u + du, v + dv));
        let cap = strata.elevation_at(u, v) + self.delta_at(u, v).max(self.delta_at(u + du, v + dv));
        highest.min(cap)
    }

    pub fn body_elevation_at(&self, strata: &NaturalStrata, u: i32, v: i32) -> i32 {
        if !self.reworked_at(u, v) || self.upward_at(u, v) > 0 {
            self.elevation_at(strata, u, v)
        } else {
            self.pair_elevation(strata, u, v, 1, 0)
                .min(self.pair_elevation(strata, u, v, -1, 0))
                .min(self.pair_elevation(strata, u, v, 0, 1))
                .min(self.pair_elevation(strata, u, v, 0, -1))
        }
    }

    pub fn in_unbreached_shaft(&self, strata: &NaturalStrata, u: i32, v: i32, elev: f32) -> bool {
        if !self.reworked_at(u, v) {
            return false;
        }
        let body = self.body_elevation_at(strata, u, v);
        body >= strata.vault().floor_steps_at(u, v) && elev >= (body as f32) * 0.5 - 0.1
    }

    pub fn in_upward_shaft(&self, strata: &NaturalStrata, u: i32, v: i32, elev: f32) -> bool {
        self.upward_at(u, v) > 0 && !self.breached_at(strata, u, v) && elev < (self.crust_floor_at(strata, u, v) as f32) * 0.5
    }

    fn cell_ceiling_at(&self, strata: &NaturalStrata, cu: i32, cv: i32) -> f32 {
        if self.breached_at(strata, cu, cv) {
            f32::INFINITY
        } else if self.upward_at(cu, cv) > 0 {
            (self.crust_floor_at(strata, cu, cv) as f32) * 0.5 - 0.30
        } else {
            strata.vault().ceiling_height_at((cu as f32) * 0.5 + 0.25, (cv as f32) * 0.5 + 0.25) - 0.30
        }
    }

    pub fn confine(&self, strata: &NaturalStrata, u: f32, v: f32, elev: f32) -> (f32, f32, f32) {
        if self.ceilings.is_empty() && elev > -2.0 {
            return (u, v, elev);
        }
        let (mut cu, mut cv, mut tu, mut tv) = ((u * 2.0).floor() as i32, (v * 2.0).floor() as i32, u, v);
        if elev > self.cell_ceiling_at(strata, cu, cv) + 0.20 {
            for (du, dv) in [(-1, 0), (1, 0), (0, -1), (0, 1), (-1, -1), (1, -1), (-1, 1), (1, 1)] {
                let (nu, nv) = (cu + du, cv + dv);
                if (self.upward_at(nu, nv) > 0 || self.breached_at(strata, nu, nv)) && elev <= self.cell_ceiling_at(strata, nu, nv) + 0.20 {
                    tu = u.clamp((nu as f32) * 0.5 + 0.02, ((nu + 1) as f32) * 0.5 - 0.02);
                    tv = v.clamp((nv as f32) * 0.5 + 0.02, ((nv + 1) as f32) * 0.5 - 0.02);
                    (cu, cv) = (nu, nv);
                    break;
                }
            }
        }
        let z = if self.breached_at(strata, cu, cv) {
            elev
        } else if self.upward_at(cu, cv) > 0 {
            let ceiling = (self.crust_floor_at(strata, cu, cv) as f32) * 0.5 - 0.30;
            if elev < ceiling + 2.0 && elev > ceiling { ceiling } else { elev }
        } else {
            strata.vault().confine(tu, tv, elev)
        };
        (tu, tv, z)
    }
}
