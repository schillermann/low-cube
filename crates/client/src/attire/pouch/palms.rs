use world::BlockMold;
use super::super::sight::{CameraMount, SightMode};
use super::super::YouthAttire;

/// Dual-hand block count and mold orientation state for the right and left palms.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct PalmStack {
    right_blocks: u32,
    left_blocks: u32,
    right_mold: BlockMold,
    left_mold: BlockMold,
}

impl PalmStack {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub(super) const fn new(
        right_blocks: u32,
        left_blocks: u32,
        right_mold: BlockMold,
        left_mold: BlockMold,
    ) -> Self {
        Self { right_blocks, left_blocks, right_mold, left_mold }
    }

    pub(super) const fn blocks_on(&self, side: f32) -> u32 {
        if side > 0.0 { self.right_blocks } else { self.left_blocks }
    }

    pub(super) const fn mold_on(&self, side: f32) -> BlockMold {
        if side > 0.0 { self.right_mold } else { self.left_mold }
    }

    pub(super) const fn empty_hands(&self) -> bool {
        self.right_blocks == 0 && self.left_blocks == 0
    }

    pub(super) const fn placed_camera_sides(&self) -> (bool, bool) {
        (self.right_mold.placed_camera(), self.left_mold.placed_camera())
    }

    pub(super) const fn camera_mount(&self, attire: YouthAttire) -> CameraMount {
        let (r, l) = attire.held_pair();
        if r.field_camera() {
            CameraMount::Handheld(SightMode::from_camera_turn(self.right_mold.camera_turn()))
        } else if l.field_camera() {
            CameraMount::Handheld(SightMode::from_camera_turn(self.left_mold.camera_turn()))
        } else if self.right_mold.placed_camera() {
            CameraMount::Placed(SightMode::from_placed_camera_turn(self.right_mold.camera_turn()))
        } else if self.left_mold.placed_camera() {
            CameraMount::Placed(SightMode::from_placed_camera_turn(self.left_mold.camera_turn()))
        } else {
            CameraMount::Unmounted
        }
    }

    pub(super) fn assign_blocks(&mut self, side: f32, blocks: u32) {
        if side > 0.0 { self.right_blocks = blocks; } else { self.left_blocks = blocks; }
    }

    pub(super) fn assign_mold(&mut self, side: f32, mold: BlockMold) {
        if side > 0.0 { self.right_mold = mold; } else { self.left_mold = mold; }
    }

    pub(super) fn reset_mold(&mut self, side: f32) {
        if side > 0.0 { self.right_mold = BlockMold::cube(); } else { self.left_mold = BlockMold::cube(); }
    }

    pub(super) fn clear_placed_cameras(&mut self) {
        if self.right_mold.placed_camera() { self.right_mold = BlockMold::cube(); }
        if self.left_mold.placed_camera() { self.left_mold = BlockMold::cube(); }
    }

    pub(super) fn sync_mold_from_opposite(&mut self, side: f32) {
        if side > 0.0 { self.right_mold = self.left_mold; } else { self.left_mold = self.right_mold; }
    }

    pub(super) fn map_sides(&mut self, attire: YouthAttire, mr: bool, ml: bool, op: fn(BlockMold) -> BlockMold) {
        let (r, l) = attire.held_pair();
        if mr {
            let m = if r.placeable_block() { self.right_mold.in_medium(r.block_medium()) } else { self.right_mold };
            self.right_mold = op(m);
        }
        if ml {
            let m = if l.placeable_block() { self.left_mold.in_medium(l.block_medium()) } else { self.left_mold };
            self.left_mold = op(m);
        }
    }

    pub(super) fn reconcile_attire(&mut self, attire: YouthAttire) {
        let (r, l) = attire.held_pair();
        if r.placeable_block() && l.placeable_block() && !r.field_camera() && !l.field_camera() {
            if self.right_blocks == 0 && self.left_blocks > 0 {
                self.right_mold = self.left_mold;
            } else if self.left_blocks == 0 && self.right_blocks > 0 {
                self.left_mold = self.right_mold;
            }
        }
        self.right_blocks = if r.placeable_block() { self.right_blocks.max(1) } else { 0 };
        self.left_blocks = if l.placeable_block() { self.left_blocks.max(1) } else { 0 };
    }
}
