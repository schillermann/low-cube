use super::super::cell::GridCell;

const REGH_EFFORT_FLOOR: f32 = 320.0;
const REGH_EFFORT_SPAN: f32 = 440.0;
const CELL_HASH_U: i32 = 73_856_093;
const CELL_HASH_V: i32 = 19_349_663;

#[derive(Clone, Copy, Debug, PartialEq)]
enum SodPhase {
    Bare(f32),
    Verdant,
}

/// Immutable biological grass-sod value object encapsulated inside an earth block or surface scar.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TurfSod {
    phase: SodPhase,
}

impl TurfSod {
    const fn new(phase: SodPhase) -> Self {
        Self { phase }
    }

    /// Secondary constructor for freshly harvested or placed bare soil without grass.
    pub const fn bare() -> Self {
        Self::new(SodPhase::Bare(0.0))
    }

    /// Secondary constructor for established living turf with grass blades.
    pub const fn verdant() -> Self {
        Self::new(SodPhase::Verdant)
    }

    /// Indicates whether this soil surface currently carries living grass.
    pub const fn verdant_coat(self) -> bool {
        matches!(self.phase, SodPhase::Verdant)
    }

    /// Pure state transition advancing biological grass colonization from adjacent verdant surfaces.
    pub fn colonized(self, cell: GridCell, dt: f32, grass_neighbors: u32) -> (Self, bool) {
        if grass_neighbors == 0 {
            return (self, false);
        }
        match self.phase {
            SodPhase::Bare(effort) => {
                let next_effort = effort + dt * (grass_neighbors as f32);
                let (u, v) = cell.feed_coordinates(|gu, gv| (gu, gv));
                let hash = (u.wrapping_mul(CELL_HASH_U) ^ v.wrapping_mul(CELL_HASH_V)).unsigned_abs() % 100;
                let threshold = REGH_EFFORT_FLOOR + (hash as f32 * 0.01) * REGH_EFFORT_SPAN;
                if next_effort >= threshold {
                    (Self::verdant(), true)
                } else {
                    (Self::new(SodPhase::Bare(next_effort)), false)
                }
            }
            SodPhase::Verdant => (self, false),
        }
    }
}
