use super::cell::GridCell;

const REGH_EFFORT_FLOOR_TENTHS: u32 = 3_200;
const REGH_EFFORT_SPAN_TENTHS: u32 = 4_400;
const CELL_HASH_U: i32 = 73_856_093;
const CELL_HASH_V: i32 = 19_349_663;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum SodPhase {
    Bare(u16),
    Verdant,
}

/// Immutable biological grass-sod value object encapsulated inside an earth atom or surface scar (`1` field, `Copy + Eq`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TurfSod {
    phase: SodPhase,
}

impl TurfSod {
    const fn new(phase: SodPhase) -> Self {
        Self { phase }
    }

    /// Secondary constructor for freshly harvested or placed bare soil without grass.
    pub const fn bare() -> Self {
        Self::new(SodPhase::Bare(0))
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
            SodPhase::Bare(tenths) => {
                let step = ((dt * (grass_neighbors as f32) * 10.0).round().max(1.0)) as u32;
                let next_tenths = u32::from(tenths).saturating_add(step);
                let (u, v) = cell.feed_coordinates(|gu, gv| (gu, gv));
                let hash = (u.wrapping_mul(CELL_HASH_U) ^ v.wrapping_mul(CELL_HASH_V)).unsigned_abs() % 100;
                let threshold = REGH_EFFORT_FLOOR_TENTHS + (hash * REGH_EFFORT_SPAN_TENTHS) / 100;
                if next_tenths >= threshold {
                    (Self::verdant(), true)
                } else {
                    (Self::new(SodPhase::Bare(next_tenths.min(u16::MAX as u32) as u16)), false)
                }
            }
            SodPhase::Verdant => (self, false),
        }
    }
}
