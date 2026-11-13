use std::collections::HashMap;
use crate::block::BlockMedium;
use super::super::natural_strata::NaturalStrata;
use super::entropy::EntropyWell;
use super::{ExposedStratum, SubterraneanVeins};

#[derive(Clone, Copy, Debug)]
struct VeinBud {
    medium: BlockMedium,
    budget: u8,
}

impl VeinBud {
    const fn new(medium: BlockMedium, budget: u8) -> Self {
        Self { medium, budget }
    }
}

/// Authoritative growing mineral-vein geology with height stratigraphy, finite bud budgets, and sector saturation.
pub struct LivingVeins {
    entropy: EntropyWell,
    disclosed: HashMap<(i32, i32, i32), BlockMedium>,
    buds: HashMap<(i32, i32, i32), VeinBud>,
    saturation: HashMap<(i32, i32, u8), u8>,
}

impl LivingVeins {
    /// Primary constructor doing nothing except assigning arguments to fields.
    fn new(
        entropy: EntropyWell,
        disclosed: HashMap<(i32, i32, i32), BlockMedium>,
        buds: HashMap<(i32, i32, i32), VeinBud>,
        saturation: HashMap<(i32, i32, u8), u8>,
    ) -> Self {
        Self { entropy, disclosed, buds, saturation }
    }

    /// Secondary constructor seeded with an explicit 128-bit secret key.
    pub fn with_secret(k0: u64, k1: u64) -> Self {
        Self::new(EntropyWell::new(k0, k1, 1), HashMap::new(), HashMap::new(), HashMap::new())
    }

    /// Secondary constructor drawing fresh ambient entropy for standalone worlds.
    pub fn ambient(zone_tag: u64) -> Self {
        Self::new(EntropyWell::ambient(zone_tag), HashMap::new(), HashMap::new(), HashMap::new())
    }

    fn coord_salt(u: i32, v: i32, step: i32) -> u64 {
        ((u as u32 as u64) << 42) ^ ((v as u32 as u64) << 21) ^ (step as u32 as u64)
    }

    fn slope_bonus(strata: &NaturalStrata, u: i32, v: i32) -> u32 {
        let center = strata.elevation_at(u, v);
        let steep = [(1, 0), (-1, 0), (0, 1), (0, -1)]
            .into_iter()
            .map(|(du, dv)| (strata.elevation_at(u + du, v + dv) - center).abs())
            .max()
            .unwrap_or(0);
        if steep >= 2 { 22 } else { 10 }
    }

    fn stratified_candidate(strata: &NaturalStrata, u: i32, v: i32, step: i32) -> Option<(BlockMedium, u32, u8, u8)> {
        let nat = strata.elevation_at(u, v);
        let crust_top = strata.vault().crust_top_step_at(u, v);
        let (depth, above_crust) = (nat - step, step - crust_top);
        if above_crust < 0 || depth < 2 {
            return None;
        }
        if above_crust <= 3 && depth >= 4 {
            Some((BlockMedium::GoldOre, 54, 3, 4))
        } else if depth >= 5 {
            Some((BlockMedium::IronOre, 52, 5, 4))
        } else {
            Some((BlockMedium::CopperOre, 58, 5, 5))
        }
    }

    fn propagate_bud(
        &mut self,
        strata: &NaturalStrata,
        deltas: &HashMap<(i32, i32), i32>,
        origin: (i32, i32, i32),
        medium: BlockMedium,
        remaining: u8,
    ) {
        if remaining == 0 {
            return;
        }
        let (u, v, s) = origin;
        let vault = strata.vault();
        let mut candidates = [(0, 0, 0); 6];
        let mut count = 0usize;
        for (nu, nv, ns) in [(u + 1, v, s), (u - 1, v, s), (u, v + 1, s), (u, v - 1, s), (u, v, s - 1), (u, v, s + 1)] {
            let cur_top = strata.elevation_at(nu, nv) + deltas.get(&(nu, nv)).copied().unwrap_or(0);
            let same_band = Self::stratified_candidate(strata, nu, nv, ns).is_some_and(|(m, _, _, _)| m == medium);
            if same_band && ns < cur_top && ns >= vault.crust_top_step_at(nu, nv) && !self.disclosed.contains_key(&(nu, nv, ns)) {
                candidates[count] = (nu, nv, ns);
                count += 1;
            }
        }
        if count == 0 {
            return;
        }
        let roll = self.entropy.next_word(Self::coord_salt(u, v, s));
        let primary = candidates[(roll as usize) % count];
        if remaining >= 3 && count >= 2 && (roll >> 16) % 3 == 0 {
            let secondary = candidates[((roll as usize) + 1 + ((roll >> 8) as usize % (count - 1))) % count];
            let half = remaining / 2;
            self.buds.entry(primary).or_insert(VeinBud::new(medium, remaining - half));
            self.buds.entry(secondary).or_insert(VeinBud::new(medium, half));
        } else {
            self.buds.entry(primary).or_insert(VeinBud::new(medium, remaining));
        }
    }

    fn expose_single(
        &mut self,
        strata: &NaturalStrata,
        deltas: &HashMap<(i32, i32), i32>,
        cell: (i32, i32, i32),
    ) -> Option<ExposedStratum> {
        let (u, v, step) = cell;
        if self.disclosed.contains_key(&cell) || strata.vault().crust_layer_at(u, v, step) {
            return None;
        }
        if let Some(bud) = self.buds.remove(&cell) {
            self.disclosed.insert(cell, bud.medium);
            self.propagate_bud(strata, deltas, cell, bud.medium, bud.budget.saturating_sub(1));
            return Some(ExposedStratum::new(cell, bud.medium));
        }
        let Some((ore, base_permille, min_len, span_len)) = Self::stratified_candidate(strata, u, v, step) else {
            self.disclosed.insert(cell, BlockMedium::TurfEarth);
            return None;
        };
        let sector = (u.div_euclid(16), v.div_euclid(16), ore.code());
        let sat = self.saturation.get(&sector).copied().unwrap_or(0);
        let bonus = Self::slope_bonus(strata, u, v);
        let threshold = (base_permille * bonus) / (10 * (1 + u32::from(sat) * 4));
        let roll = self.entropy.next_word(Self::coord_salt(u, v, step));
        if ((roll % 1000) as u32) < threshold.max(2) {
            self.saturation.insert(sector, sat.saturating_add(1));
            let total_budget = min_len + ((roll >> 16) as u8 % span_len);
            self.disclosed.insert(cell, ore);
            self.propagate_bud(strata, deltas, cell, ore, total_budget.saturating_sub(1));
            Some(ExposedStratum::new(cell, ore))
        } else {
            self.disclosed.insert(cell, BlockMedium::TurfEarth);
            None
        }
    }
}

impl SubterraneanVeins for LivingVeins {
    fn disclosed_at(&self, u: i32, v: i32, step: i32) -> Option<BlockMedium> {
        self.disclosed.get(&(u, v, step)).copied().filter(|m| m.mineral_ore())
    }

    fn excavate_and_expose(
        &mut self,
        strata: &NaturalStrata,
        deltas: &HashMap<(i32, i32), i32>,
        dug: (i32, i32, i32),
        fallback: BlockMedium,
    ) -> (BlockMedium, Vec<ExposedStratum>) {
        let harvested = self
            .disclosed
            .remove(&dug)
            .filter(|m| m.mineral_ore())
            .unwrap_or(fallback);
        self.buds.remove(&dug);
        let (u, v, _) = dug;
        let new_elev = strata.elevation_at(u, v) + deltas.get(&(u, v)).copied().unwrap_or(0);
        let crust_floor = strata.vault().floor_steps_at(u, v);
        let mut revealed = Vec::new();
        if new_elev > crust_floor {
            if let Some(exp) = self.expose_single(strata, deltas, (u, v, new_elev - 1)) {
                revealed.push(exp);
            }
        }
        for (nu, nv) in [(u + 1, v), (u - 1, v), (u, v + 1), (u, v - 1)] {
            let n_elev = strata.elevation_at(nu, nv) + deltas.get(&(nu, nv)).copied().unwrap_or(0);
            let n_crust = strata.vault().crust_top_step_at(nu, nv);
            for wall_step in new_elev.max(n_crust)..n_elev {
                if let Some(exp) = self.expose_single(strata, deltas, (nu, nv, wall_step)) {
                    revealed.push(exp);
                }
            }
        }
        (harvested, revealed)
    }

    fn accept_disclosure(&mut self, stratum: ExposedStratum) {
        stratum.transfer(|u, v, step, medium| {
            self.disclosed.insert((u, v, step), medium);
        });
    }
}
