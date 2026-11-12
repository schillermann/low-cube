use glam::Vec3;
use crate::MeshTarget;
use crate::block::{ElevatedShape, GridCell};
use crate::frame::SurfaceFrame;
use super::super::turf_habitat::TurfHabitat;
use super::super::turf_scars::TurfScars;
use super::NICHE_CONTACT_TOLERANCE;

const BLOCK_STEP: f32 = GridCell::SPAN;

/// Vertical slice of elevated blocks evaluating grass-sod exposure and neighbor colonization.
pub(super) struct SodColumn<'a> {
    blocks: &'a [ElevatedShape],
}

impl<'a> SodColumn<'a> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub(super) const fn new(blocks: &'a [ElevatedShape]) -> Self {
        Self { blocks }
    }

    pub(super) fn covers_ground(&self, ground_top: f32) -> bool {
        self.blocks.iter().any(|b| b.smothers_turf_below(ground_top))
    }

    pub(super) fn bare_surface(&self, ground_bare: bool) -> bool {
        self.blocks.last().map_or(ground_bare, |top| top.bare_surface(ground_bare))
    }

    pub(super) fn covered_at(&self, idx: usize) -> bool {
        idx + 1 < self.blocks.len() && self.blocks[idx + 1].smothers_turf_below(self.blocks[idx].ceiling())
    }

    pub(super) fn verdant_near_step(&self, target_step: i32) -> bool {
        self.blocks.iter().enumerate().any(|(idx, block)| {
            !self.covered_at(idx)
                && block.verdant_turf()
                && (target_step - (block.ceiling() / BLOCK_STEP).round() as i32).abs() <= 1
        })
    }

    pub(super) fn collect_colonization(
        &self,
        cell: (i32, i32),
        habitat: &dyn TurfHabitat,
        out: &mut Vec<((i32, i32), usize, u32)>,
    ) {
        let (u, v) = cell;
        for (idx, block) in self.blocks.iter().enumerate() {
            if self.covered_at(idx) || !block.bare_turf() {
                continue;
            }
            let h = (block.ceiling() / BLOCK_STEP).round() as i32;
            let count = habitat.verdant_neighbors(u, v, h);
            if count > 0 {
                out.push(((u, v), idx, count));
            }
        }
    }

    pub(super) fn emit(
        &self,
        cell: GridCell,
        frame: &SurfaceFrame,
        colors: (Vec3, Vec3),
        scars: &TurfScars,
        target: &mut dyn MeshTarget,
    ) -> bool {
        let (u, v) = cell.feed_coordinates(|gu, gv| (gu, gv));
        let (top_col, wall_col) = colors;
        for (idx, block) in self.blocks.iter().enumerate() {
            let stage = scars.block_stage(u, v, block);
            let covered_above = self.covered_at(idx);
            let flank_only = self.blocks.len() > 2
                && idx > 0
                && idx + 1 < self.blocks.len()
                && block.full_cube_envelope()
                && self.blocks[idx - 1].full_cube_envelope()
                && self.blocks[idx + 1].full_cube_envelope()
                && (block.floor() - self.blocks[idx - 1].ceiling()).abs() < NICHE_CONTACT_TOLERANCE
                && (self.blocks[idx + 1].floor() - block.ceiling()).abs() < NICHE_CONTACT_TOLERANCE;
            block.emit_terraced(cell, frame, top_col, wall_col, covered_above, flank_only, stage, target);
        }
        !self.blocks.is_empty()
    }
}
