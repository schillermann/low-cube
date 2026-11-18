use crate::Mesh;

/// Partitioned `16×16`-cell sector GPU mesh updating individual patches in `O(1)` without rebuilding the face.
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
    fn draw(&self, gl: &glow::Context) {
        for patch in &self.patches {
            patch.draw(gl);
        }
    }

    fn release(&self, gl: &glow::Context) {
        for patch in &self.patches {
            patch.release(gl);
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
