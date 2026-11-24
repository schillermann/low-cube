use std::collections::HashMap;
use std::sync::RwLock;
use glam::Vec3;
use crate::block::GridCell;
use crate::frame::SurfaceFrame;
use crate::{MeshTarget, Quad};
use super::super::elevation::ReliefElevation;
use super::super::palette::ReliefPalette;
use super::super::strata::ReliefStrata;
use super::orbital_shell::OrbitalShell;
use super::patch_grid::{DirtySectors, SECTOR_SPAN};

const MAX_CACHED_CELLS: usize = 65_536;

struct CellQuadTarget<'a>(&'a mut Vec<Quad>);

impl MeshTarget for CellQuadTarget<'_> {
    fn push_quad(&mut self, quad: Quad) {
        self.0.push(quad);
    }
}

struct LiftedTarget<'a> {
    inner: &'a mut dyn MeshTarget,
    lift: Vec3,
}

impl MeshTarget for LiftedTarget<'_> {
    fn push_quad(&mut self, quad: Quad) {
        self.inner.push_quad(quad.shifted(self.lift));
    }
}

/// Cached geometry of a single relief cell partitioning molded structures from ground strata.
#[derive(Clone)]
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
    cells: RwLock<HashMap<(i32, i32), CachedCell>>,
    tufts: RwLock<HashMap<(i32, i32), Vec<Quad>>>,
}

impl CellVault {
    fn empty() -> Self {
        Self { cells: RwLock::new(HashMap::new()), tufts: RwLock::new(HashMap::new()) }
    }
}

/// On-demand lazy cell/sector quad cache supporting both moonlets and `4096³` continental worlds (`<= 4` fields).
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

    pub(super) const fn sector_count(&self) -> usize {
        self.sectors.sector_count()
    }

    pub(super) fn pending_sectors(&self) -> bool {
        self.sectors.pending()
    }

    pub(super) fn sector_revision(&self, gsu: i32, gsv: i32) -> u32 {
        self.sectors.meadow_revision_at(gsu, gsv, self.half_blocks())
    }

    pub(super) fn record_molded(&mut self, u: i32, v: i32) {
        self.sectors.record_touched(u, v, self.divisions);
    }

    pub(super) fn center_sector(&self, center: Vec3, frame: &SurfaceFrame) -> (i32, i32) {
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

    fn cell_at(&self, u: i32, v: i32, half: i32, frame: &SurfaceFrame, relief: &dyn ReliefElevation, palette: &ReliefPalette) -> CachedCell {
        if let Ok(cells) = self.vault.cells.read() {
            if let Some(existing) = cells.get(&(u, v)) {
                return existing.clone();
            }
        }
        let built = self.build_cell(u, v, half, frame, relief, palette);
        if let Ok(mut cells) = self.vault.cells.write() {
            if cells.len() >= MAX_CACHED_CELLS {
                cells.clear();
            }
            cells.insert((u, v), built.clone());
        }
        built
    }

    pub(super) fn refresh_neighborhood(&mut self, cu: i32, cv: i32, half: i32, radius: i32, _frame: &SurfaceFrame, _relief: &dyn ReliefElevation, _palette: &ReliefPalette) {
        let cells = self.vault.cells.get_mut().expect("Relief cell cache poisoned");
        let tufts = self.vault.tufts.get_mut().expect("Relief tuft cache poisoned");
        for v in (cv - radius).max(-half)..=(cv + radius).min(half - 1) {
            for u in (cu - radius).max(-half)..=(cu + radius).min(half - 1) {
                self.sectors.mark_cell_dirty(u, v, half, self.divisions);
                cells.remove(&(u, v));
                tufts.remove(&(u, v));
            }
        }
    }

    pub(super) fn emit_surface(&self, frame: &SurfaceFrame, relief: &dyn ReliefElevation, palette: &ReliefPalette, target: &mut dyn MeshTarget) {
        for slot in 0..self.sector_count() {
            self.emit_sector(slot, frame, relief, palette, target);
        }
    }

    pub(super) fn emit_sector(&self, patch_slot: usize, frame: &SurfaceFrame, relief: &dyn ReliefElevation, palette: &ReliefPalette, target: &mut dyn MeshTarget) {
        let half = self.half_blocks();
        let sec_side = DirtySectors::sectors_per_side(self.divisions);
        let span = DirtySectors::patch_cell_span(self.divisions) as i32;
        let (su, sv) = (patch_slot % sec_side, patch_slot / sec_side);
        let u0 = -half + (su as i32) * span;
        let v0 = -half + (sv as i32) * span;
        let u1 = (u0 + span).min(half);
        let v1 = (v0 + span).min(half);
        if DirtySectors::continental(self.divisions) {
            let shell = OrbitalShell::new(frame, relief, palette, self.sectors.touched_cells());
            shell.emit_macro_patch((u0, u1, v0, v1, half), target, |cu, cv, out| {
                self.cell_at(cu, cv, half, frame, relief, palette).emit_all(out);
            });
            return;
        }
        for v in v0..v1 {
            for u in u0..u1 {
                self.cell_at(u, v, half, frame, relief, palette).emit_all(target);
            }
        }
    }

    pub(super) fn drain_dirty_sector(&mut self, frame: &SurfaceFrame, relief: &dyn ReliefElevation, palette: &ReliefPalette, target: &mut dyn MeshTarget) -> Option<usize> {
        let slot = self.sectors.pop_next()?;
        self.emit_sector(slot, frame, relief, palette, target);
        Some(slot)
    }

    pub(super) fn emit_meadow_sector(
        &self,
        gsu: i32,
        gsv: i32,
        lod: u8,
        frame: &SurfaceFrame,
        relief: &dyn ReliefElevation,
        palette: &ReliefPalette,
        target: &mut dyn MeshTarget,
    ) {
        let half = self.half_blocks();
        let continental = DirtySectors::continental(self.divisions);
        let up = frame.normal();
        let u0 = gsu * (SECTOR_SPAN as i32);
        let v0 = gsv * (SECTOR_SPAN as i32);
        let Ok(mut tufts) = self.vault.tufts.write() else { return };
        if tufts.len() >= MAX_CACHED_CELLS {
            tufts.clear();
        }
        for gv in v0..(v0 + (SECTOR_SPAN as i32)) {
            for gu in u0..(u0 + (SECTOR_SPAN as i32)) {
                if gu < -half || gu >= half || gv < -half || gv >= half {
                    continue;
                }
                let cell = self.cell_at(gu, gv, half, frame, relief, palette);
                if continental {
                    let mut lifted = LiftedTarget { inner: target, lift: up * 0.003 };
                    cell.emit_all(&mut lifted);
                }
                if lod == 0 {
                    let entry = tufts.entry((gu, gv)).or_default();
                    if entry.is_empty() {
                        let mut cell_target = CellQuadTarget(entry);
                        let mut meadow = crate::NaturalMeadowTarget::with_lod(&mut cell_target, GridCell::new(gu, gv), up, 0);
                        cell.emit_ground(&mut meadow);
                    }
                    for &quad in entry.iter() {
                        target.push_quad(quad);
                    }
                } else {
                    let mut meadow = crate::NaturalMeadowTarget::with_lod(target, GridCell::new(gu, gv), up, lod);
                    cell.emit_ground(&mut meadow);
                }
            }
        }
    }

    pub(super) fn emit_meadow_around(
        &self,
        center: Vec3,
        _view_dir: Vec3,
        frame: &SurfaceFrame,
        relief: &dyn ReliefElevation,
        palette: &ReliefPalette,
        target: &mut dyn MeshTarget,
    ) {
        let (csu, csv) = self.center_sector(center, frame);
        for dsv in -3i32..=3 {
            for dsu in -3i32..=3 {
                let lod = if dsu.abs().max(dsv.abs()) <= 1 { 2 } else { 0 };
                self.emit_meadow_sector(csu + dsu, csv + dsv, lod, frame, relief, palette, target);
            }
        }
    }
}
