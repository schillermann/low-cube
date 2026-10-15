use glam::Vec3;
use crate::{MeshTarget, SurfaceFrame};
use super::depository::TimberDepository;
use super::facet::ArborealFacet;
use super::polyhedral_crown::PolyhedralCrown;
use super::prism_trunk::PrismTrunk;
use super::species::TreeSpecies;
use super::timber_yield::TimberYield;

/// Temperate deciduous oak species with faceted polyhedral crown.
pub struct TemperateOak;

impl TemperateOak {
    pub fn new() -> Self {
        Self
    }
}

impl TreeSpecies for TemperateOak {
    fn initial_vitality(&self) -> u32 {
        3
    }

    fn yield_harvest(&self, recipient: &mut dyn TimberDepository) {
        recipient.deposit_timber(TimberYield::StandardLumber, 4);
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
        let trunk_col = Vec3::new(0.45, 0.30, 0.18);
        let leaf_col = Vec3::new(0.22, 0.52, 0.16);
        let leaf_top = Vec3::new(0.28, 0.60, 0.20);

        PrismTrunk::vertical(6, 0.35, 0.22, 2.0, trunk_col).emit(frame, u, v, elevation, target);
        PolyhedralCrown::new(0.0, 0.0, 3.2, 1.35, leaf_top).emit(frame, u, v, elevation, target);
        PolyhedralCrown::new(-0.6, 0.2, 2.4, 0.90, leaf_col).emit(frame, u, v, elevation, target);
        PolyhedralCrown::new(0.6, -0.2, 2.3, 0.85, leaf_col).emit(frame, u, v, elevation, target);
    }

    fn clone_species(&self) -> Box<dyn TreeSpecies> {
        Box::new(Self::new())
    }

    fn canopy_drag(&self, distance: f32, relative_height: f32) -> f32 {
        if relative_height >= 1.8 && relative_height <= 4.8 && distance <= 2.2 {
            0.90
        } else {
            0.0
        }
    }
}
