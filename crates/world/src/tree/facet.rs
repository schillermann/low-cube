use crate::{MeshTarget, SurfaceFrame};

/// Geometric facet component of an arboreal organism.
pub trait ArborealFacet: Send + Sync {
    /// Emits faceted quads relative to root coordinates in tangent space.
    fn emit(
        &self,
        frame: &SurfaceFrame,
        u: f32,
        v: f32,
        elevation: f32,
        target: &mut dyn MeshTarget,
    );
}
