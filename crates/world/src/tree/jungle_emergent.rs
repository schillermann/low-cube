use glam::Vec3;
use crate::{MeshTarget, SurfaceFrame};
use super::conical_skirt::ConicalSkirt;
use super::depository::TimberDepository;
use super::facet::ArborealFacet;
use super::prism_trunk::PrismTrunk;
use super::species::TreeSpecies;
use super::timber_yield::TimberYield;

/// Jungle emergent giant with wide flared columnar trunk and lofty umbrella canopy.
pub struct JungleEmergent;

impl JungleEmergent {
    pub fn new() -> Self {
        Self
    }
}

impl TreeSpecies for JungleEmergent {
    fn initial_vitality(&self) -> u32 {
        4
    }

    fn yield_harvest(&self, recipient: &mut dyn TimberDepository) {
        recipient.deposit_timber(TimberYield::TropicalHardwood, 6);
        recipient.deposit_timber(TimberYield::JungleFruit, 2);
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
        let bark_col = Vec3::new(0.42, 0.36, 0.28);
        let leaf_col = Vec3::new(0.14, 0.48, 0.18);
        let leaf_top = Vec3::new(0.18, 0.56, 0.22);

        PrismTrunk::vertical(8, 0.65, 0.22, 4.5, bark_col).emit(frame, u, v, elevation, target);
        ConicalSkirt::new(10, 2.5, 1.5, 0.65, 4.3, leaf_col).emit(frame, u, v, elevation, target);
        ConicalSkirt::new(8, 1.6, 0.2, 0.45, 4.8, leaf_top).emit(frame, u, v, elevation, target);
    }

    fn clone_species(&self) -> Box<dyn TreeSpecies> {
        Box::new(Self::new())
    }

    fn canopy_drag(&self, distance: f32, relative_height: f32) -> f32 {
        if relative_height >= 3.8 && relative_height <= 5.8 && distance <= 2.8 {
            0.95
        } else {
            0.0
        }
    }
}
