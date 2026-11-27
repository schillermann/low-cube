use crate::{FramePass, Mesh, SlotReclamation};

/// Partitioned `16×16`-cell sector GPU mesh updating individual patches in `O(1)` (`1` private field).
pub struct SectorGridMesh {
    patches: Vec<Box<dyn Mesh>>,
}

impl SectorGridMesh {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(patches: Vec<Box<dyn Mesh>>) -> Self {
        Self { patches }
    }
}

impl Mesh for SectorGridMesh {
    fn draw(&self, pass: &mut dyn FramePass) {
        for patch in &self.patches {
            patch.draw(pass);
        }
    }

    fn release(&self, reclaim: &mut dyn SlotReclamation) {
        for patch in &self.patches {
            patch.release(reclaim);
        }
    }

    fn patch_at(&self, slot: usize) -> Option<&dyn Mesh> {
        self.patches.get(slot).map(Box::as_ref)
    }

    fn store_patch(&mut self, slot: usize, patch: Box<dyn Mesh>) {
        if slot < self.patches.len() {
            self.patches[slot] = patch;
        }
    }
}
