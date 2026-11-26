use crate::WorldSeed;
use super::PlantingGround;

/// `PlantingGround` decorator scrambling arboreal cell receptivity by a `WorldSeed` (`2` fields).
pub struct SeededGround<G> {
    origin: G,
    seed: WorldSeed,
}

impl<G> SeededGround<G> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(origin: G, seed: WorldSeed) -> Self {
        Self { origin, seed }
    }
}

impl<G: PlantingGround> PlantingGround for SeededGround<G> {
    fn receptive(&self, cell_u: i32, cell_v: i32, hash: i32) -> bool {
        self.origin.receptive(cell_u, cell_v, self.seed.mix_grove(hash))
    }

    fn ground_elevation(&self, cell_u: i32, cell_v: i32) -> f32 {
        self.origin.ground_elevation(cell_u, cell_v)
    }
}
