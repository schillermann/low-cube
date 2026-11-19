use world::{BlockMold, PalmMolds};
use super::super::sight::{CameraMount, SightMode};
use super::super::YouthAttire;

/// Dual-hand block count and mold orientation state for the right and left palms.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct PalmStack {
    right_blocks: u32,
    left_blocks: u32,
    molds: PalmMolds,
}

impl PalmStack {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub(super) const fn new(right_blocks: u32, left_blocks: u32, molds: PalmMolds) -> Self {
        Self { right_blocks, left_blocks, molds }
    }

    pub(super) const fn blocks_on(&self, side: f32) -> u32 {
        if side > 0.0 { self.right_blocks } else { self.left_blocks }
    }

    pub(super) const fn mold_on(&self, side: f32) -> BlockMold {
        self.molds.on_side(side)
    }

    pub(super) const fn palm_molds(&self) -> PalmMolds {
        self.molds
    }

    pub(super) const fn empty_hands(&self) -> bool {
        self.right_blocks == 0 && self.left_blocks == 0
    }

    pub(super) const fn camera_mount(&self, attire: YouthAttire) -> CameraMount {
        let (r, l) = attire.held_pair();
        if r.field_camera() {
            CameraMount::Handheld(SightMode::from_camera_turn(self.molds.on_side(1.0).camera_turn()))
        } else if l.field_camera() {
            CameraMount::Handheld(SightMode::from_camera_turn(self.molds.on_side(-1.0).camera_turn()))
        } else {
            CameraMount::Unmounted
        }
    }

    pub(super) fn assign_blocks(&mut self, side: f32, blocks: u32) {
        if side > 0.0 { self.right_blocks = blocks; } else { self.left_blocks = blocks; }
    }

    pub(super) fn assign_mold(&mut self, side: f32, mold: BlockMold) {
        self.molds = self.molds.with_side(side, mold);
    }

    pub(super) fn reset_mold(&mut self, side: f32) {
        self.molds = self.molds.with_side(side, BlockMold::cube());
    }

    pub(super) fn sync_mold_from_opposite(&mut self, side: f32) {
        self.molds = self.molds.synced_from_opposite(side);
    }

    pub(super) fn map_sides(&mut self, attire: YouthAttire, mr: bool, ml: bool, op: fn(BlockMold) -> BlockMold) {
        let (r, l) = attire.held_pair();
        if mr {
            let cur = self.molds.on_side(1.0);
            let m = if r.placeable_block() { cur.in_medium(r.block_medium()) } else { cur };
            self.molds = self.molds.with_side(1.0, op(m));
        }
        if ml {
            let cur = self.molds.on_side(-1.0);
            let m = if l.placeable_block() { cur.in_medium(l.block_medium()) } else { cur };
            self.molds = self.molds.with_side(-1.0, op(m));
        }
    }

    pub(super) fn reconcile_attire(&mut self, attire: YouthAttire) {
        let (r, l) = attire.held_pair();
        if r.independent_palm_mold() && self.right_blocks == 0 {
            self.reset_mold(1.0);
        }
        if l.independent_palm_mold() && self.left_blocks == 0 {
            self.reset_mold(-1.0);
        }
        if r.placeable_block() && l.placeable_block() && !r.independent_palm_mold() && !l.independent_palm_mold() {
            if self.right_blocks == 0 && self.left_blocks > 0 {
                self.molds = self.molds.synced_from_opposite(1.0);
            } else if self.left_blocks == 0 && self.right_blocks > 0 {
                self.molds = self.molds.synced_from_opposite(-1.0);
            }
        }
        self.right_blocks = if r.placeable_block() { self.right_blocks.max(1) } else { 0 };
        self.left_blocks = if l.placeable_block() { self.left_blocks.max(1) } else { 0 };
    }
}
