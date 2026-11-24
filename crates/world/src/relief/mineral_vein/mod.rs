mod entropy;
mod living;

use std::collections::HashMap;
use crate::block::BlockMedium;
use super::natural_strata::NaturalStrata;

pub use living::LivingVeins;

/// Newly air-exposed subterranean cell coordinate and its revealed material medium.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ExposedStratum {
    coords: (i32, i32, i32),
    medium: BlockMedium,
}

impl ExposedStratum {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(coords: (i32, i32, i32), medium: BlockMedium) -> Self {
        Self { coords, medium }
    }

    /// Transfers encapsulated coordinates and medium to a consumer closure ("Tell, Don't Ask").
    pub fn transfer<T>(self, f: impl FnOnce(i32, i32, i32, BlockMedium) -> T) -> T {
        f(self.coords.0, self.coords.1, self.coords.2, self.medium)
    }
}

/// Polymorphic subterranean mineral vein authority decoupling server geology from zero-knowledge clients.
pub trait SubterraneanVeins: Send + Sync {
    fn disclosed_at(&self, u: i32, v: i32, step: i32) -> Option<BlockMedium>;

    fn excavate_and_expose(
        &mut self,
        strata: &NaturalStrata,
        deltas: &HashMap<(i32, i32), i32>,
        dug: (i32, i32, i32),
        fallback: BlockMedium,
    ) -> (BlockMedium, Vec<ExposedStratum>);

    fn accept_disclosure(&mut self, stratum: ExposedStratum);

    fn collect_disclosed_in(
        &self,
        u_range: (i32, i32),
        v_range: (i32, i32),
        out: &mut Vec<(i32, i32, i32, BlockMedium)>,
    );

    fn replace_disclosed_in(
        &mut self,
        u_range: (i32, i32),
        v_range: (i32, i32),
        items: &[(i32, i32, i32, BlockMedium)],
    );

    fn collect_disclosed_sectors(&self, out: &mut Vec<(i32, i32)>);
}

/// Zero-knowledge subterranean vein store holding exclusively air-exposed cells disclosed by the sanctuary.
pub struct HollowVeins {
    disclosed: HashMap<(i32, i32, i32), BlockMedium>,
}

impl HollowVeins {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(disclosed: HashMap<(i32, i32, i32), BlockMedium>) -> Self {
        Self { disclosed }
    }

    /// Secondary constructor for a pristine zero-knowledge client face.
    pub fn untouched() -> Self {
        Self::new(HashMap::new())
    }
}

impl SubterraneanVeins for HollowVeins {
    fn disclosed_at(&self, u: i32, v: i32, step: i32) -> Option<BlockMedium> {
        self.disclosed.get(&(u, v, step)).copied()
    }

    fn excavate_and_expose(
        &mut self,
        _strata: &NaturalStrata,
        _deltas: &HashMap<(i32, i32), i32>,
        dug: (i32, i32, i32),
        fallback: BlockMedium,
    ) -> (BlockMedium, Vec<ExposedStratum>) {
        let harvested = self.disclosed.remove(&dug).unwrap_or(fallback);
        (harvested, Vec::new())
    }

    fn accept_disclosure(&mut self, stratum: ExposedStratum) {
        stratum.transfer(|u, v, step, medium| {
            if medium.mineral_ore() {
                self.disclosed.insert((u, v, step), medium);
            } else {
                self.disclosed.remove(&(u, v, step));
            }
        });
    }

    fn collect_disclosed_in(
        &self,
        (u0, u1): (i32, i32),
        (v0, v1): (i32, i32),
        out: &mut Vec<(i32, i32, i32, BlockMedium)>,
    ) {
        for (&(u, v, step), &med) in &self.disclosed {
            if u >= u0 && u < u1 && v >= v0 && v < v1 && med.mineral_ore() {
                out.push((u, v, step, med));
            }
        }
        out.sort_unstable_by_key(|&(u, v, s, _)| (u, v, s));
    }

    fn replace_disclosed_in(
        &mut self,
        (u0, u1): (i32, i32),
        (v0, v1): (i32, i32),
        items: &[(i32, i32, i32, BlockMedium)],
    ) {
        self.disclosed.retain(|&(u, v, _), _| u < u0 || u >= u1 || v < v0 || v >= v1);
        for &(u, v, step, med) in items {
            if med.mineral_ore() {
                self.disclosed.insert((u, v, step), med);
            }
        }
    }

    fn collect_disclosed_sectors(&self, out: &mut Vec<(i32, i32)>) {
        for (&(u, v, _), &med) in &self.disclosed {
            if med.mineral_ore() {
                out.push((u.div_euclid(8), v.div_euclid(8)));
            }
        }
    }
}
