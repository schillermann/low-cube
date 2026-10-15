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

    /// Secondary constructor decorating block with fertile brown earth loam color.
    pub fn dirt(block: Box<dyn Block>) -> Self {
        Self::new(block, Vec3::new(0.48, 0.35, 0.22))
    }

    /// Secondary constructor decorating block with warm amber desert sand color.
    pub fn sand(block: Box<dyn Block>) -> Self {
        Self::new(block, Vec3::new(0.85, 0.75, 0.35))
    }

    /// Secondary constructor decorating block with crystalline alpine snow color.
    pub fn snow(block: Box<dyn Block>) -> Self {
        Self::new(block, Vec3::new(0.92, 0.95, 0.98))
    }

    /// Secondary constructor decorating block with standard alpine gray granite stone color.
    pub fn stone(block: Box<dyn Block>) -> Self {
        Self::new(block, Vec3::new(0.48, 0.50, 0.46))
    }

    /// Secondary constructor decorating block with dark shale rock color.
    pub fn dark_stone(block: Box<dyn Block>) -> Self {
        Self::new(block, Vec3::new(0.35, 0.42, 0.35))
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
