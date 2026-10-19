use glam::Vec3;
use crate::{MeshTarget, SurfaceFrame};
use super::depository::TimberDepository;
use super::facet::ArborealFacet;
use super::prism_trunk::PrismTrunk;
use super::species::TreeSpecies;
use super::timber_yield::TimberYield;

/// Volcanic charred basalt spire with crystalline jagged spines.
pub struct VolcanicSpire;

impl VolcanicSpire {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self
    }
}

impl TreeSpecies for VolcanicSpire {
    fn initial_vitality(&self) -> u32 {
        4
    }

    fn yield_harvest(&self, recipient: &mut dyn TimberDepository) {
        recipient.deposit_timber(TimberYield::VolcanicBasaltwood, 4);
        recipient.deposit_timber(TimberYield::Twig, 2);
    }

    fn emit_tree(
        &self,
        frame: &SurfaceFrame,
        u: f32,
        v: f32,
        elevation: f32,
        target: &mut dyn MeshTarget,
    ) {
        let basalt_col = Vec3::new(0.16, 0.16, 0.18);
        let char_edge = Vec3::new(0.24, 0.22, 0.24);

        PrismTrunk::vertical(5, 0.40, 0.26, 2.0, basalt_col).emit(frame, u, v, elevation, target);
        PrismTrunk::vertical(4, 0.26, 0.06, 1.8, char_edge).emit(frame, u + 0.05, v - 0.05, elevation + 1.8, target);

        PrismTrunk::vertical(4, 0.12, 0.02, 1.2, basalt_col).emit(frame, u - 0.2, v + 0.15, elevation + 1.2, target);
        PrismTrunk::vertical(4, 0.10, 0.02, 1.0, basalt_col).emit(frame, u + 0.2, v - 0.15, elevation + 1.4, target);
    }

    fn clone_species(&self) -> Box<dyn TreeSpecies> {
        Box::new(Self::new())
    }

    fn canopy_drag(&self, _distance: f32, _relative_height: f32) -> f32 {
        0.0
    }
}
