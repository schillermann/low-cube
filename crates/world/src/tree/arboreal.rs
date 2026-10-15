use glam::Vec3;
use crate::{MeshTarget, SurfaceFrame};
use super::depository::TimberDepository;

/// Core interface representing an arboreal living or harvested entity.
pub trait Arboreal: Send + Sync {
    /// Strikes the entity with cutting impact, transferring yields and transitioning state.
    fn strike(&self, cleave: u32, recipient: &mut dyn TimberDepository) -> Box<dyn Arboreal>;

    /// Indicates whether this entity presents a solid physical obstacle.
    fn solid(&self) -> bool;

    /// Indicates whether this entity is actively standing as a full tree.
    fn standing(&self) -> bool;

    /// Constrains avatar surface traversal against the physical tree trunk.
    fn constrain_surface(
        &self,
        base: Vec3,
        u: f32,
        v: f32,
        frame: &SurfaceFrame,
        distance: f32,
    ) -> Vec3;

    /// Emits faceted geometric quads into a mesh target.
    fn emit_surface(&self, target: &mut dyn MeshTarget);

    /// Evaluates aerodynamic braking drag of canopy foliage at a given surface-relative point.
    fn canopy_drag(&self, u: f32, v: f32, height: f32) -> f32;
}
