use glam::Vec3;
use crate::{MeshTarget, SurfaceFrame};
use super::conical_skirt::ConicalSkirt;
use super::depository::TimberDepository;
use super::facet::ArborealFacet;
use super::prism_trunk::PrismTrunk;
use super::species::TreeSpecies;
use super::timber_yield::TimberYield;

/// Steppe savanna umbrella acacia with bent trunk and parasol disc crown.
pub struct SteppeAcacia;

impl SteppeAcacia {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self
    }
}

impl TreeSpecies for SteppeAcacia {
    fn initial_vitality(&self) -> u32 {
        3
    }

    fn yield_harvest(&self, recipient: &mut dyn TimberDepository) {
        recipient.deposit_timber(TimberYield::SteppeDrywood, 4);
        recipient.deposit_timber(TimberYield::Twig, 2);
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
        let trunk_col = Vec3::new(0.46, 0.34, 0.24);
        let leaf_col = Vec3::new(0.44, 0.54, 0.18);

        PrismTrunk::vertical(6, 0.28, 0.20, 1.2, trunk_col).emit(frame, u, v, elevation, target);
        PrismTrunk::vertical(5, 0.20, 0.16, 1.2, trunk_col).emit(frame, u + 0.15, v + 0.05, elevation + 1.1, target);
        PrismTrunk::vertical(5, 0.16, 0.12, 1.0, trunk_col).emit(frame, u - 0.05, v - 0.1, elevation + 2.2, target);

        ConicalSkirt::new(10, 2.4, 1.8, 0.5, 3.2, leaf_col).emit(frame, u, v, elevation, target);
        ConicalSkirt::new(8, 1.8, 0.6, 0.35, 3.6, leaf_col).emit(frame, u, v, elevation, target);
    }

    fn clone_species(&self) -> Box<dyn TreeSpecies> {
        Box::new(Self::new())
    }

    fn canopy_drag(&self, distance: f32, relative_height: f32) -> f32 {
        if (2.8..=4.5).contains(&relative_height) && distance <= 2.6 {
            0.85
        } else {
            0.0
        }
    }
}
