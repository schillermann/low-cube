use std::sync::RwLock;
use crate::block::GridCell;
use crate::frame::SurfaceFrame;
use crate::{MeshTarget, Quad};
use super::super::elevation::ReliefElevation;
use super::super::palette::ReliefPalette;
use super::super::strata::ReliefStrata;
use super::patch_grid::{DirtySectors, SECTOR_SPAN};

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

struct CellVault {
    cells: RwLock<Vec<CachedCell>>,
    tufts: RwLock<Vec<Vec<Quad>>>,
}

impl CellVault {
    fn empty() -> Self {
        Self { cells: RwLock::new(Vec::new()), tufts: RwLock::new(Vec::new()) }
    }
}

/// Encapsulated per-cell quad cache, `16×16` sector tracker, and strata emitter for a relief face.
pub(super) struct ReliefCellCache {
    strata: ReliefStrata,
    divisions: u32,
    vault: CellVault,
    sectors: DirtySectors,
}

impl ReliefCellCache {
    /// Primary constructor doing nothing except assigning arguments to fields.
    fn new(strata: ReliefStrata, divisions: u32, vault: CellVault, sectors: DirtySectors) -> Self {
        Self { strata, divisions, vault, sectors }
    }

    pub(super) fn empty(strata: ReliefStrata, divisions: u32) -> Self {
        Self::new(strata, divisions, CellVault::empty(), DirtySectors::sized(divisions))
    }

    pub(super) const fn half_blocks(&self) -> i32 {
        (self.divisions as i32) / 2
    }

    pub(super) fn sector_count(&self) -> usize {
        self.sectors.sector_count()
    }

    pub(super) fn pending_sectors(&self) -> bool {
        self.sectors.pending()
    }

    pub(super) fn sector_revision(&self, gsu: i32, gsv: i32) -> u32 {
        let half_sec = (self.half_blocks()).div_euclid(SECTOR_SPAN as i32);
        self.sectors.revision_at(gsu + half_sec, gsv + half_sec, self.divisions)
    }

    pub(super) fn center_sector(&self, center: glam::Vec3, frame: &SurfaceFrame) -> (i32, i32) {
        let (u_pos, v_pos) = frame.decompose(center);
        let (cu, cv) = GridCell::at_planar(u_pos, v_pos).feed_coordinates(|gu, gv| (gu, gv));
        (cu.div_euclid(SECTOR_SPAN as i32), cv.div_euclid(SECTOR_SPAN as i32))
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

    fn ensure_populated(&self, frame: &SurfaceFrame, relief: &dyn ReliefElevation, palette: &ReliefPalette) {
        let side = self.divisions as usize;
        let total = side * side;
        if self.vault.cells.read().is_ok_and(|c| c.len() == total) {
            return;
        }
        let half = self.half_blocks();
        let mut cells = self.vault.cells.write().expect("Relief cell cache poisoned");
        if cells.len() != total {
            cells.clear();
            cells.reserve(total);
            for v in -half..half {
                for u in -half..half {
                    cells.push(self.build_cell(u, v, half, frame, relief, palette));
                }
            }
        }
    }

    pub(super) fn refresh_neighborhood(&mut self, cu: i32, cv: i32, half: i32, radius: i32, frame: &SurfaceFrame, relief: &dyn ReliefElevation, palette: &ReliefPalette) {
        let side = self.divisions as usize;
        let total = side * side;
        let populated = self.vault.cells.get_mut().expect("Relief cell cache poisoned").len() == total;
        for v in (cv - radius).max(-half)..=(cv + radius).min(half - 1) {
            for u in (cu - radius).max(-half)..=(cu + radius).min(half - 1) {
                self.sectors.mark_cell_dirty(u, v, half, self.divisions);
                if populated {
                    let cell_mesh = self.build_cell(u, v, half, frame, relief, palette);
                    let slot = ((v + half) as usize) * side + ((u + half) as usize);
                    self.vault.cells.get_mut().expect("Relief cell cache poisoned")[slot] = cell_mesh;
                    let tufts = self.vault.tufts.get_mut().expect("Relief tuft cache poisoned");
                    if slot < tufts.len() {
                        tufts[slot].clear();
                    }
                }
            }
        }
    }

    pub(super) fn emit_surface(&self, frame: &SurfaceFrame, relief: &dyn ReliefElevation, palette: &ReliefPalette, target: &mut dyn MeshTarget) {
        self.ensure_populated(frame, relief, palette);
        if let Ok(cells) = self.vault.cells.read() {
            for cell_mesh in cells.iter() {
                cell_mesh.emit_all(target);
            }
        }
    }

    pub(super) fn emit_sector(&self, patch_slot: usize, frame: &SurfaceFrame, relief: &dyn ReliefElevation, palette: &ReliefPalette, target: &mut dyn MeshTarget) {
        self.ensure_populated(frame, relief, palette);
        let side = self.divisions as usize;
        let half = self.half_blocks();
        let sec_side = DirtySectors::sectors_per_side(self.divisions);
        let (su, sv) = (patch_slot % sec_side, patch_slot / sec_side);
        let u0 = -half + ((su * SECTOR_SPAN) as i32);
        let v0 = -half + ((sv * SECTOR_SPAN) as i32);
        let u1 = (u0 + (SECTOR_SPAN as i32)).min(half);
        let v1 = (v0 + (SECTOR_SPAN as i32)).min(half);
        let Ok(cells) = self.vault.cells.read() else { return };
        for v in v0..v1 {
            for u in u0..u1 {
                let slot = ((v + half) as usize) * side + ((u + half) as usize);
                if let Some(cell) = cells.get(slot) {
                    cell.emit_all(target);
                }
            }
        }
    }

    pub(super) fn drain_dirty_sector(&mut self, frame: &SurfaceFrame, relief: &dyn ReliefElevation, palette: &ReliefPalette, target: &mut dyn MeshTarget) -> Option<usize> {
        let slot = self.sectors.pop_next()?;
        self.emit_sector(slot, frame, relief, palette, target);
        Some(slot)
    }

    pub(super) fn emit_meadow_sector(&self, gsu: i32, gsv: i32, lod: u8, frame: &SurfaceFrame, target: &mut dyn MeshTarget) {
        let half = self.half_blocks();
        let side = self.divisions as usize;
        let total = side * side;
        let Ok(cells) = self.vault.cells.read() else { return };
        if cells.len() != total {
            return;
        }
        let Ok(mut tufts) = self.vault.tufts.write() else { return };
        if tufts.len() != total {
            tufts.resize_with(total, Vec::new);
        }
        let up = frame.normal();
        let u0 = gsu * (SECTOR_SPAN as i32);
        let v0 = gsv * (SECTOR_SPAN as i32);
        for gv in v0..(v0 + (SECTOR_SPAN as i32)) {
            for gu in u0..(u0 + (SECTOR_SPAN as i32)) {
                if gu < -half || gu >= half || gv < -half || gv >= half {
                    continue;
                }
                let slot = ((gv + half) as usize) * side + ((gu + half) as usize);
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

    pub(super) fn emit_meadow_around(&self, center: glam::Vec3, _view_dir: glam::Vec3, frame: &SurfaceFrame, target: &mut dyn MeshTarget) {
        let (csu, csv) = self.center_sector(center, frame);
        for dsv in -3i32..=3 {
            for dsu in -3i32..=3 {
                let lod = if dsu.abs().max(dsv.abs()) <= 1 { 2 } else { 0 };
                self.emit_meadow_sector(csu + dsu, csv + dsv, lod, frame, target);
            }
        }
    }
}
