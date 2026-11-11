use crate::MeshTarget;
use crate::block::GridCell;
use crate::frame::SurfaceFrame;
use super::elevation::ReliefElevation;
use super::facet::ReliefFacet;
use super::crust_vault::CrustVault;
use super::palette::ReliefPalette;

/// Encapsulated facet assembler generating authentic watertight voxel columns and slopes.
pub struct ReliefStrata {
    facet: ReliefFacet,
}

impl ReliefStrata {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new() -> Self {
        Self {
            facet: ReliefFacet::new(),
        }
    }

    /// Emits all surface and subterranean facets for a single discrete grid cell ("Tell, Don't Ask").
    pub fn emit_cell(
        &self,
        cell: GridCell,
        frame: &SurfaceFrame,
        relief: &dyn ReliefElevation,
        palette: &ReliefPalette,
        half_blocks: i32,
        target: &mut dyn MeshTarget,
    ) {
        let vault = CrustVault::beneath(half_blocks);
        self.facet.emit_with_vault(
            cell, frame, relief, half_blocks,
            &vault, palette, target,
        );
    }
}
