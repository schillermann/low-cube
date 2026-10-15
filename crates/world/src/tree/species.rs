use crate::{MeshTarget, SurfaceFrame};
use super::depository::TimberDepository;

/// Archetypal traits, resource yields, and geometry for an arboreal species.
pub trait TreeSpecies: Send + Sync {
    /// Initial mechanical vitality of a standing tree specimen.
    fn initial_vitality(&self) -> u32;

    /// Dispatches forest resource yields directly to a depository upon felling.
    fn yield_harvest(&self, recipient: &mut dyn TimberDepository);

    /// Emits faceted geometry of the full living tree.
    fn emit_tree(
        &self,
        frame: &SurfaceFrame,
        u: f32,
        v: f32,
        elevation: f32,
        target: &mut dyn MeshTarget,
    );

    /// Produces an independent clone of the species definition.
    fn clone_species(&self) -> Box<dyn TreeSpecies>;

    /// Evaluates aerodynamic braking drag of canopy foliage at a given distance and relative height.
    fn canopy_drag(&self, distance: f32, relative_height: f32) -> f32;
}
