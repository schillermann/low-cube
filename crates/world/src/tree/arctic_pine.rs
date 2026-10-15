use glam::Vec3;
use crate::{MeshTarget, SurfaceFrame};
use super::conical_skirt::ConicalSkirt;
use super::depository::TimberDepository;
use super::facet::ArborealFacet;
use super::prism_trunk::PrismTrunk;
use super::species::TreeSpecies;
use super::timber_yield::TimberYield;

/// Arctic alpine pine species with tiered snow-capped conical skirts.
pub struct ArcticPine;

impl ArcticPine {
    pub fn new() -> Self {
        Self
    }
}

impl TreeSpecies for ArcticPine {
    fn initial_vitality(&self) -> u32 {
        3
    }

    fn yield_harvest(&self, recipient: &mut dyn TimberDepository) {
        recipient.deposit_timber(TimberYield::AlpineHardwood, 4);
        recipient.deposit_timber(TimberYield::PineCone, 2);
        recipient.deposit_timber(TimberYield::Resin, 1);
    }

    fn emit_tree(
        &self,
        frame: &SurfaceFrame,
        u: f32,
        v: f32,
        elevation: f32,
        target: &mut dyn MeshTarget,
    ) {
        let trunk_col = Vec3::new(0.32, 0.24, 0.18);
        let pine_col = Vec3::new(0.18, 0.38, 0.32);
        let snow_col = Vec3::new(0.90, 0.95, 0.98);

        PrismTrunk::vertical(6, 0.28, 0.18, 1.2, trunk_col).emit(frame, u, v, elevation, target);
        ConicalSkirt::new(8, 1.6, 0.6, 1.0, 0.6, pine_col).emit(frame, u, v, elevation, target);
        ConicalSkirt::new(8, 1.3, 0.5, 0.9, 1.4, snow_col).emit(frame, u, v, elevation, target);
        ConicalSkirt::new(8, 1.0, 0.35, 0.85, 2.1, pine_col).emit(frame, u, v, elevation, target);
        ConicalSkirt::new(8, 0.75, 0.2, 0.8, 2.8, snow_col).emit(frame, u, v, elevation, target);
        ConicalSkirt::new(8, 0.5, 0.02, 0.9, 3.4, pine_col).emit(frame, u, v, elevation, target);
    }

    fn clone_species(&self) -> Box<dyn TreeSpecies> {
        Box::new(Self::new())
    }

    fn canopy_drag(&self, distance: f32, relative_height: f32) -> f32 {
        if relative_height >= 0.6 && relative_height <= 4.5 {
            let max_radius = 1.8 * (1.0 - (relative_height - 0.6) / 4.0);
            if distance <= max_radius.max(0.3) {
                return 0.85;
            }
        }
        0.0
    }
}
