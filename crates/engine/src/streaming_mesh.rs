use world::{MeshTarget, Quad};
use crate::quad_batch::QuadBatch;
use crate::{FramePass, Mesh};

/// Dynamic actor/plume/UI quad stream backed by persistent `wgpu` storage buffers (`1` private field).
pub struct StreamingMesh {
    batch: QuadBatch,
}

impl StreamingMesh {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(batch: QuadBatch) -> Self {
        Self { batch }
    }

    pub fn empty() -> Self {
        Self::new(QuadBatch::empty())
    }

    pub fn with_quad_capacity(quads: usize) -> Self {
        Self::new(QuadBatch::with_quad_capacity(quads))
    }

    pub fn clear(&mut self) {
        self.batch.clear();
    }

    pub fn batch(&self) -> &QuadBatch {
        &self.batch
    }
}

impl MeshTarget for StreamingMesh {
    fn push_quad(&mut self, quad: Quad) {
        self.batch.push_quad(quad);
    }
}

impl Mesh for StreamingMesh {
    fn draw(&self, pass: &mut dyn FramePass) {
        if !self.batch.vacant() {
            pass.stream_quads(&self.batch);
        }
    }
}
