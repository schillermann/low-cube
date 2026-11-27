use crate::upload_ring::BufferSlot;
use crate::{FramePass, Mesh, SlotReclamation};

/// Paired GPU storage-buffer handles for `8-Byte` `PackedQuad`s and index-free `FreeQuad`s (`2` private fields).
#[derive(Clone, Copy, Debug)]
pub struct SlotPair {
    packed: (BufferSlot, u32),
    free: (BufferSlot, u32),
}

impl SlotPair {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(packed: (BufferSlot, u32), free: (BufferSlot, u32)) -> Self {
        Self { packed, free }
    }
}

/// Uploaded GPU patch drawing `8-Byte` vertex-pulled `PackedQuad`s and optional `FreeQuad`s (`2` private fields).
pub struct UploadedPatchMesh {
    slots: SlotPair,
    origin: [f32; 4],
}

impl UploadedPatchMesh {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(slots: SlotPair, origin: [f32; 4]) -> Self {
        Self { slots, origin }
    }
}

impl Mesh for UploadedPatchMesh {
    fn draw(&self, pass: &mut dyn FramePass) {
        if self.slots.packed.1 > 0 {
            pass.draw_packed_quads(self.slots.packed.0, self.slots.packed.1, self.origin);
        }
        if self.slots.free.1 > 0 {
            pass.draw_free_quads(self.slots.free.0, self.slots.free.1);
        }
    }

    fn release(&self, reclaim: &mut dyn SlotReclamation) {
        if self.slots.packed.1 > 0 {
            reclaim.reclaim_slot(self.slots.packed.0);
        }
        if self.slots.free.1 > 0 {
            reclaim.reclaim_slot(self.slots.free.0);
        }
    }
}
