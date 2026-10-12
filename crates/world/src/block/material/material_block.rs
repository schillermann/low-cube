use glam::Vec3;
use crate::{ColoredMeshTarget, MeshTarget};
use crate::frame::SurfaceFrame;
use crate::block::Block;

/// Decorator applying surface material coloration to an underlying polymorphic Block.
pub struct MaterialBlock {
    block: Box<dyn Block>,
    color: Vec3,
}

impl MaterialBlock {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(block: Box<dyn Block>, color: Vec3) -> Self {
        Self { block, color }
    }
}

impl Block for MaterialBlock {
    fn constrain_surface(
        &self,
        base: Vec3,
        u: f32,
        v: f32,
        frame: &SurfaceFrame,
        distance: f32,
    ) -> Vec3 {
        self.block.constrain_surface(base, u, v, frame, distance)
    }

    fn emit_surface(&self, frame: &SurfaceFrame, target: &mut dyn MeshTarget) {
        let mut colored = ColoredMeshTarget::new(target, self.color);
        self.block.emit_surface(frame, &mut colored);
    }
}
