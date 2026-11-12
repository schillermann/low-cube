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

/// Cached geometry of a single relief cell partitioning molded structures from ground strata.
pub(super) struct CachedCell {
    quads: Vec<Quad>,
    ground_start: usize,
}

impl CachedCell {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub(super) const fn new(quads: Vec<Quad>, ground_start: usize) -> Self {
        Self { quads, ground_start }
    }

    pub(super) fn emit_all(&self, target: &mut dyn MeshTarget) {
        for &quad in &self.quads {
            target.push_quad(quad);
        }
    }

    pub(super) fn emit_ground(&self, target: &mut dyn MeshTarget) {
        for &quad in &self.quads[self.ground_start..] {
            target.push_quad(quad);
        }
    }
}

/// Encapsulated per-cell quad cache and strata emitter for a planetary relief face.
pub(super) struct ReliefCellCache {
    strata: ReliefStrata,
    divisions: u32,
    cached_cells: RwLock<Vec<CachedCell>>,
    cached_tufts: RwLock<Vec<Vec<Quad>>>,
}

impl ReliefCellCache {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub(super) fn new(
        strata: ReliefStrata,
        divisions: u32,
        cached_cells: RwLock<Vec<CachedCell>>,
        cached_tufts: RwLock<Vec<Vec<Quad>>>,
    ) -> Self {
        Self {
            strata,
            divisions,
            cached_cells,
            cached_tufts,
        }
    }

    pub(super) fn empty(strata: ReliefStrata, divisions: u32) -> Self {
        Self::new(strata, divisions, RwLock::new(Vec::new()), RwLock::new(Vec::new()))
    }

    pub(super) const fn half_blocks(&self) -> i32 {
        (self.divisions as i32) / 2
    }

    fn build_cell(&self, u: i32, v: i32, half: i32, frame: &SurfaceFrame, relief: &dyn ReliefElevation, palette: &ReliefPalette) -> CachedCell {
        let mut cell_quads = Vec::with_capacity(8);
        let ground_start = {
            let mut molded_out = CellQuadTarget(&mut cell_quads);
            let mut ground_quads = Vec::with_capacity(4);
            let mut ground_out = CellQuadTarget(&mut ground_quads);
            self.strata.emit_cell_layers(GridCell::new(u, v), frame, relief, palette, half, &mut molded_out, &mut ground_out);
            let split = cell_quads.len();
            cell_quads.extend(ground_quads);
            split
        };
        CachedCell::new(cell_quads, ground_start)
    }

    pub(super) fn refresh_neighborhood(&mut self, cu: i32, cv: i32, half: i32, radius: i32, frame: &SurfaceFrame, relief: &dyn ReliefElevation, palette: &ReliefPalette) {
        let side = self.divisions as usize;
        let total = side * side;
        let populated = self.cached_cells.get_mut().expect("Relief cell cache poisoned").len() == total;
        if populated {
            for v in (cv - radius).max(-half)..=(cv + radius).min(half - 1) {
                for u in (cu - radius).max(-half)..=(cu + radius).min(half - 1) {
                    let cell_mesh = self.build_cell(u, v, half, frame, relief, palette);
                    let slot = ((v + half) as usize) * side + ((u + half) as usize);
                    self.cached_cells.get_mut().expect("Relief cell cache poisoned")[slot] = cell_mesh;
                    let tufts = self.cached_tufts.get_mut().expect("Relief tuft cache poisoned");
                    if slot < tufts.len() {
                        tufts[slot].clear();
                    }
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
                for cell_mesh in cells.iter() {
                    cell_mesh.emit_all(target);
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
        for cell_mesh in cells.iter() {
            cell_mesh.emit_all(target);
        }
    }

    pub(super) fn emit_meadow_around(
        &self,
        center: glam::Vec3,
        _view_dir: glam::Vec3,
        frame: &SurfaceFrame,
        target: &mut dyn MeshTarget,
    ) {
        let half = self.half_blocks();
        let side = self.divisions as usize;
        let total = side * side;
        let Ok(cells) = self.cached_cells.read() else { return };
        if cells.len() != total {
            return;
        }
        let Ok(mut tufts) = self.cached_tufts.write() else { return };
        if tufts.len() != total {
            tufts.resize_with(total, Vec::new);
        }
        let (u_pos, v_pos) = frame.decompose(center);
        let (cu, cv) = GridCell::at_planar(u_pos, v_pos).feed_coordinates(|gu, gv| (gu, gv));
        let up = frame.normal();
        for dv in -56..=56 {
            for du in -56..=56 {
                let dist_sq = du * du + dv * dv;
                if dist_sq > 3136 {
                    continue;
                }
                let (gu, gv) = (cu + du, cv + dv);
                if gu < -half || gu >= half || gv < -half || gv >= half {
                    continue;
                }
                let slot = ((gv + half) as usize) * side + ((gu + half) as usize);
                let lod = if dist_sq <= 36 { 2 } else if dist_sq <= 144 { 1 } else { 0 };
                if lod == 0 {
                    if tufts[slot].is_empty() {
                        let mut cell_target = CellQuadTarget(&mut tufts[slot]);
                        let mut meadow = crate::NaturalMeadowTarget::with_lod(&mut cell_target, GridCell::new(gu, gv), up, 0);
                        cells[slot].emit_ground(&mut meadow);
                    }
                    for &quad in &tufts[slot] {
                        target.push_quad(quad);
                    }
                } else {
                    let mut meadow = crate::NaturalMeadowTarget::with_lod(target, GridCell::new(gu, gv), up, lod);
                    cells[slot].emit_ground(&mut meadow);
                }
            }
        }
    }
}
