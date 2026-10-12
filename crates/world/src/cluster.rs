use glam::Vec3;
use crate::{Face, MeshTarget};
use crate::block::Block;
use crate::frame::SurfaceFrame;

/// Composite decorator placing a collection of polymorphic discrete voxel blocks onto a planetary face.
pub struct VoxelCluster {
    face: Box<dyn Face>,
    frame: SurfaceFrame,
    blocks: Vec<Box<dyn Block>>,
}

impl VoxelCluster {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(face: Box<dyn Face>, frame: SurfaceFrame, blocks: Vec<Box<dyn Block>>) -> Self {
        Self { face, frame, blocks }
    }
}

impl Face for VoxelCluster {
    fn matches(&self, point: Vec3) -> bool {
        self.face.matches(point)
    }

    fn surface_up(&self) -> Vec3 {
        self.face.surface_up()
    }

    fn pull_direction(&self) -> Vec3 {
        self.face.pull_direction()
    }

    fn constrain(&self, point: Vec3, distance: f32) -> Vec3 {
        let mut position = self.face.constrain(point, distance);
        let (u, v) = self.frame.decompose(point);

        for block in &self.blocks {
            position = block.constrain_surface(position, u, v, &self.frame, distance);
        }

        position
    }

    fn emit_surface(&self, target: &mut dyn MeshTarget) {
        self.face.emit_surface(target);
        for block in &self.blocks {
            block.emit_surface(&self.frame, target);
        }
    }
}
