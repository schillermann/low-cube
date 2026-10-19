use glam::Vec3;
use crate::{MeshTarget, SurfaceFrame};
use super::depository::TimberDepository;
use super::facet::ArborealFacet;
use super::polyhedral_crown::PolyhedralCrown;
use super::prism_trunk::PrismTrunk;
use super::species::TreeSpecies;
use super::timber_yield::TimberYield;

/// Desert Joshua tree and succulent branched column with physically connected limbs.
pub struct DesertSucculent;

impl DesertSucculent {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self
    }
}

impl TreeSpecies for DesertSucculent {
    fn initial_vitality(&self) -> u32 {
        2
    }

    fn yield_harvest(&self, recipient: &mut dyn TimberDepository) {
        recipient.deposit_timber(TimberYield::SucculentFiber, 3);
        recipient.deposit_timber(TimberYield::SucculentFluid, 2);
        recipient.deposit_timber(TimberYield::Sapling, 1);
    }

    fn emit_tree(
        &self,
        frame: &SurfaceFrame,
        u: f32,
        v: f32,
        elevation: f32,
        target: &mut dyn MeshTarget,
    ) {
        let stem_col = Vec3::new(0.34, 0.52, 0.24);
        let tip_col = Vec3::new(0.58, 0.65, 0.28);

        // Central vertical trunk
        PrismTrunk::vertical(6, 0.30, 0.22, 2.0, stem_col).emit(frame, u, v, elevation, target);
        PolyhedralCrown::new(0.0, 0.0, 2.15, 0.38, tip_col).emit(frame, u, v, elevation, target);

        // Left connected arm (angled outward from trunk, then vertical)
        PrismTrunk::new(5, 0.18, 0.15, -0.45, 0.10, 0.5, stem_col).emit(frame, u, v, elevation + 1.0, target);
        PrismTrunk::vertical(5, 0.15, 0.12, 0.9, stem_col).emit(frame, u - 0.45, v + 0.10, elevation + 1.5, target);
        PolyhedralCrown::new(-0.45, 0.10, 2.5, 0.36, tip_col).emit(frame, u, v, elevation, target);

        // Right connected arm (angled outward from trunk, then vertical)
        PrismTrunk::new(5, 0.18, 0.15, 0.45, -0.10, 0.5, stem_col).emit(frame, u, v, elevation + 1.2, target);
        PrismTrunk::vertical(5, 0.15, 0.12, 0.8, stem_col).emit(frame, u + 0.45, v - 0.10, elevation + 1.7, target);
        PolyhedralCrown::new(0.45, -0.10, 2.6, 0.36, tip_col).emit(frame, u, v, elevation, target);
    }

    fn clone_species(&self) -> Box<dyn TreeSpecies> {
        Box::new(Self::new())
    }

    fn canopy_drag(&self, distance: f32, relative_height: f32) -> f32 {
        if (1.5..=3.2).contains(&relative_height) && distance <= 1.2 {
            0.60
        } else {
            0.0
        }
    }
}
