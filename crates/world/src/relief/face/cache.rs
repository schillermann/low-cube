use std::collections::HashMap;
use std::sync::RwLock;
use glam::Vec3;
use crate::block::GridCell;
use crate::frame::SurfaceFrame;
use crate::{MeshTarget, Quad};
use super::super::clipmap::{ClipmapLod, PlanetHeightClipmap};
use super::super::elevation::ReliefElevation;
use super::super::palette::ReliefPalette;
use super::super::strata::ReliefStrata;
use super::patch_grid::{DirtySectors, SECTOR_SPAN};

const MAX_CACHED_TUFTS: usize = 16_384;

struct NullMoldedTarget;

impl MeshTarget for NullMoldedTarget {
    fn push_quad(&mut self, _quad: Quad) {}
}

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

/// Zero-cell-cache clipmap sector streamer with bounded local meadow tuft ring (`3` private fields).
pub(super) struct ReliefCellCache {
    strata: ReliefStrata,
    divisions: u32,
    sectors: DirtySectors,
    tufts: RwLock<HashMap<(i32, i32), Vec<Quad>>>,
}

impl ReliefCellCache {
    /// Primary constructor doing nothing except assigning arguments to fields.
    fn new(
        strata: ReliefStrata,
        divisions: u32,
        sectors: DirtySectors,
        tufts: RwLock<HashMap<(i32, i32), Vec<Quad>>>,
    ) -> Self {
        Self { strata, divisions, sectors, tufts }
    }

    pub(super) fn empty(strata: ReliefStrata, divisions: u32) -> Self {
        Self::new(strata, divisions, DirtySectors::sized(divisions), RwLock::new(HashMap::new()))
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
        self.sectors.record_touched(u, v);
    }

    pub(super) fn center_sector(&self, center: Vec3, frame: &SurfaceFrame) -> (i32, i32) {
        let (u_pos, v_pos) = frame.decompose(center);
        let (cu, cv) = GridCell::at_planar(u_pos, v_pos).feed_coordinates(|gu, gv| (gu, gv));
        (cu.div_euclid(SECTOR_SPAN as i32), cv.div_euclid(SECTOR_SPAN as i32))
    }

    pub(super) fn refresh_neighborhood(
        &mut self,
        cu: i32,
        cv: i32,
        half: i32,
        radius: i32,
        _frame: &SurfaceFrame,
        _relief: &dyn ReliefElevation,
        _palette: &ReliefPalette,
    ) {
        let tufts = self.tufts.get_mut().expect("Relief tuft cache poisoned");
        for v in (cv - radius).max(-half)..=(cv + radius).min(half - 1) {
            for u in (cu - radius).max(-half)..=(cu + radius).min(half - 1) {
                self.sectors.mark_cell_dirty(u, v, half, self.divisions);
                tufts.remove(&(u, v));
            }
        }
    }

    pub(super) fn emit_surface(
        &self,
        frame: &SurfaceFrame,
        relief: &dyn ReliefElevation,
        palette: &ReliefPalette,
        target: &mut dyn MeshTarget,
    ) {
        for slot in 0..self.sector_count() {
            self.emit_sector(slot, frame, relief, palette, target);
        }
    }

    pub(super) fn emit_sector(
        &self,
        patch_slot: usize,
        frame: &SurfaceFrame,
        relief: &dyn ReliefElevation,
        palette: &ReliefPalette,
        target: &mut dyn MeshTarget,
    ) {
        let half = self.half_blocks();
        let sec_side = DirtySectors::sectors_per_side(self.divisions);
        let span = DirtySectors::patch_cell_span(self.divisions) as i32;
        let (su, sv) = (patch_slot % sec_side, patch_slot / sec_side);
        let u0 = -half + (su as i32) * span;
        let v0 = -half + (sv as i32) * span;
        let u1 = (u0 + span).min(half);
        let v1 = (v0 + span).min(half);
        let lod = if DirtySectors::continental(self.divisions) {
            ClipmapLod::OrbitalMacro
        } else if self.divisions > 192 {
            let mid = (sec_side as i32) / 2;
            let dist = ((su as i32) - mid).abs().max(((sv as i32) - mid).abs());
            ClipmapLod::for_sector_distance(dist, false)
        } else {
            ClipmapLod::SteppedCell
        };
        let clipmap = PlanetHeightClipmap::new(frame, relief, palette, self.sectors.touched_cells());
        clipmap.emit_sector_bounds((u0, u1, v0, v1, half), &self.strata, lod, target);
    }

    pub(super) fn drain_dirty_sector(
        &mut self,
        frame: &SurfaceFrame,
        relief: &dyn ReliefElevation,
        palette: &ReliefPalette,
        target: &mut dyn MeshTarget,
    ) -> Option<usize> {
        let slot = self.sectors.pop_next()?;
        self.emit_sector(slot, frame, relief, palette, target);
        Some(slot)
    }

    #[allow(clippy::too_many_arguments)]
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
        let u0 = gsu * (SECTOR_SPAN as i32);
        let v0 = gsv * (SECTOR_SPAN as i32);
        let Ok(mut tufts) = self.tufts.write() else { return };
        if tufts.len() >= MAX_CACHED_TUFTS {
            let (cu, cv) = (u0 + 4, v0 + 4);
            tufts.retain(|&(ku, kv), _| (ku - cu).abs().max((kv - cv).abs()) <= 128);
            if tufts.len() >= MAX_CACHED_TUFTS {
                tufts.retain(|&(ku, kv), _| (ku - cu).abs().max((kv - cv).abs()) <= 48);
            }
        }
        for gv in v0..(v0 + (SECTOR_SPAN as i32)) {
            for gu in u0..(u0 + (SECTOR_SPAN as i32)) {
                if gu < -half || gu >= half || gv < -half || gv >= half {
                    continue;
                }
                let cell = GridCell::new(gu, gv);
                let (cx, cy) = cell.center();
                let cell_up = frame.up_at(frame.point_at(cx, cy, 0.0));
                if continental {
                    let mut lifted = LiftedTarget { inner: target, lift: cell_up * 0.003 };
                    self.strata.emit_cell(cell, frame, relief, palette, half, &mut lifted);
                }
                if lod == 0 {
                    let entry = tufts.entry((gu, gv)).or_default();
                    if entry.is_empty() {
                        let mut cell_target = CellQuadTarget(entry);
                        let mut meadow = crate::NaturalMeadowTarget::with_lod(&mut cell_target, cell, cell_up, 0);
                        let mut ignored = NullMoldedTarget;
                        self.strata.emit_cell_layers(cell, frame, relief, palette, half, &mut ignored, &mut meadow);
                    }
                    for &quad in entry.iter() {
                        target.push_quad(quad);
                    }
                } else {
                    let mut meadow = crate::NaturalMeadowTarget::with_lod(target, cell, cell_up, lod);
                    let mut ignored = NullMoldedTarget;
                    self.strata.emit_cell_layers(cell, frame, relief, palette, half, &mut ignored, &mut meadow);
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
