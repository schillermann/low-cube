use std::sync::OnceLock;
use super::crust_vault::CrustVault;
use super::elevation::ReliefElevation;

/// Memoized natural planetary strata and inner crust vault geometry for a relief face.
pub struct NaturalStrata {
    origin: Box<dyn ReliefElevation>,
    grid: OnceLock<Vec<i16>>,
    half_span: i32,
}

impl NaturalStrata {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(origin: Box<dyn ReliefElevation>, grid: OnceLock<Vec<i16>>, half_span: i32) -> Self {
        Self { origin, grid, half_span }
    }

    /// Secondary constructor sizing the memoized natural grid for a face with `half_blocks` radius.
    pub fn in_span(origin: Box<dyn ReliefElevation>, half_blocks: i32) -> Self {
        Self::new(origin, OnceLock::new(), half_blocks + 2)
    }

    /// Inner planetary crust vault beneath this relief face.
    pub fn vault(&self) -> CrustVault {
        CrustVault::beneath(self.half_span - 2)
    }

    /// Pristine discrete natural block elevation at (u, v) prior to any excavation.
    pub fn elevation_at(&self, u: i32, v: i32) -> i32 {
        if u.abs() <= self.half_span && v.abs() <= self.half_span {
            let side = (self.half_span * 2 + 1) as usize;
            let table = self.grid.get_or_init(|| {
                let mut cells = Vec::with_capacity(side * side);
                for gu in -self.half_span..=self.half_span {
                    for gv in -self.half_span..=self.half_span {
                        cells.push(self.origin.elevation_at(gu, gv) as i16);
                    }
                }
                cells
            });
            table[((u + self.half_span) as usize) * side + ((v + self.half_span) as usize)] as i32
        } else {
            self.origin.elevation_at(u, v)
        }
    }

    /// Continuous natural surface elevation in meters at tangential coordinates (u, v).
    pub fn surface_height_at(&self, u: f32, v: f32) -> f32 {
        self.origin.surface_height_at(u, v)
    }
}
