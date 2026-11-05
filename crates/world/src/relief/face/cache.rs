use std::sync::RwLock;
use crate::block::GridCell;
use crate::frame::SurfaceFrame;
use crate::{MeshTarget, Quad};
use super::super::elevation::ReliefElevation;
use super::super::palette::ReliefPalette;
use super::super::strata::ReliefStrata;

struct CellQuadTarget<'a>(&'a mut Vec<Quad>);

impl MeshTarget for CellQuadTarget<'_> {
    fn push_quad(&mut self, quad: Quad) {
        self.0.push(quad);
    }
}

/// Encapsulated per-cell quad cache and strata emitter for a planetary relief face.
pub(super) struct ReliefCellCache {
    strata: ReliefStrata,
    divisions: u32,
    cached_cells: RwLock<Vec<Vec<Quad>>>,
}

impl ReliefCellCache {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub(super) fn new(strata: ReliefStrata, divisions: u32, cached_cells: RwLock<Vec<Vec<Quad>>>) -> Self {
        Self { strata, divisions, cached_cells }
    }

    pub(super) const fn half_blocks(&self) -> i32 {
        (self.divisions as i32) / 2
    }

    fn build_cell(&self, u: i32, v: i32, half: i32, frame: &SurfaceFrame, relief: &dyn ReliefElevation, palette: &ReliefPalette) -> Vec<Quad> {
        let mut cell_quads = Vec::with_capacity(8);
        let mut target = CellQuadTarget(&mut cell_quads);
        self.strata.emit_cell(GridCell::new(u, v), frame, relief, palette, half, &mut target);
        cell_quads
    }

    pub(super) fn refresh_neighborhood(&mut self, cu: i32, cv: i32, half: i32, frame: &SurfaceFrame, relief: &dyn ReliefElevation, palette: &ReliefPalette) {
        let side = self.divisions as usize;
        let total = side * side;
        let populated = self.cached_cells.get_mut().expect("Relief cell cache poisoned").len() == total;
        if populated {
            for v in (cv - 4).max(-half)..=(cv + 4).min(half - 1) {
                for u in (cu - 4).max(-half)..=(cu + 4).min(half - 1) {
                    let quads = self.build_cell(u, v, half, frame, relief, palette);
                    let slot = ((v + half) as usize) * side + ((u + half) as usize);
                    self.cached_cells.get_mut().expect("Relief cell cache poisoned")[slot] = quads;
                }
            }
        }
    }

    pub(super) fn emit_surface(&self, frame: &SurfaceFrame, relief: &dyn ReliefElevation, palette: &ReliefPalette, target: &mut dyn MeshTarget) {
        let half = self.half_blocks();
        let side = self.divisions as usize;
        let total = side * side;
        if let Ok(cells) = self.cached_cells.read() {
            if cells.len() == total {
                for cell_quads in cells.iter() {
                    for &quad in cell_quads {
                        target.push_quad(quad);
                    }
                }
                return;
            }
        }
        let mut cells = self.cached_cells.write().expect("Relief cell cache poisoned");
        if cells.len() != total {
            cells.clear();
            cells.reserve(total);
            for v in -half..half {
                for u in -half..half {
                    cells.push(self.build_cell(u, v, half, frame, relief, palette));
                }
            }
        }
        for cell_quads in cells.iter() {
            for &quad in cell_quads {
                target.push_quad(quad);
            }
        }
    }
}
