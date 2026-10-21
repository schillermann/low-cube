use glam::Vec3;
use world::{BlockMold, MeshTarget, PlanetaryWorld};
use super::limb::{FacetedSolid, LimbFrame, TaperedPrism};
use super::pose::BodyPose;
use super::relic::HeldRelic;
use super::YouthAttire;

/// Immutable per-side hand block state, belt-pouch block reserve, stowed tool/weapon, and persistent block mold.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HipPouch {
    hand_blocks: u32,
    stowed_blocks: u32,
    block_kind: HeldRelic,
    stowed_tool: HeldRelic,
    prefer_tool: bool,
    mold: BlockMold,
}

impl HipPouch {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(hand_blocks: u32, stowed_blocks: u32, block_kind: HeldRelic, stowed_tool: HeldRelic, prefer_tool: bool, mold: BlockMold) -> Self {
        Self { hand_blocks, stowed_blocks, block_kind, stowed_tool, prefer_tool, mold }
    }

    pub const fn empty() -> Self { Self::new(0, 0, HeldRelic::ExcavatedEarth, HeldRelic::EmptyPalm, false, BlockMold::cube()) }
    pub const fn with_mold(self, mold: BlockMold) -> Self { Self::new(self.hand_blocks, self.stowed_blocks, self.block_kind, self.stowed_tool, self.prefer_tool, mold) }
    pub const fn mold_next_form(self) -> Self { self.with_mold(self.mold.next_form()) }
    pub const fn mold_prev_form(self) -> Self { self.with_mold(self.mold.prev_form()) }
    pub const fn turn_clockwise(self) -> Self { self.with_mold(self.mold.turn_clockwise()) }
    pub const fn turn_counter_clockwise(self) -> Self { self.with_mold(self.mold.turn_counter_clockwise()) }
    pub const fn unstocked(&self) -> bool { self.hand_blocks == 0 && self.stowed_blocks == 0 }
    pub const fn holding_stowed_blocks(&self) -> bool { self.stowed_blocks > 0 }

    pub fn reconcile_hand(self, relic: HeldRelic) -> Self {
        let (hand_blocks, block_kind) = if relic.placeable_block() { (self.hand_blocks.max(1), relic) } else { (0, self.block_kind) };
        let stowed_tool = if relic.gripped() && relic == self.stowed_tool { HeldRelic::EmptyPalm } else { self.stowed_tool };
        Self::new(hand_blocks, self.stowed_blocks, block_kind, stowed_tool, self.prefer_tool, self.mold)
    }

    pub fn merge_site_relic(self, current_hand: HeldRelic, site_relic: HeldRelic) -> (Self, HeldRelic) {
        if site_relic.gripped() {
            if self.stowed_tool == site_relic { (self, if self.hand_blocks > 0 { self.block_kind } else { HeldRelic::EmptyPalm }) }
            else { (Self::new(0, self.stowed_blocks + self.hand_blocks, self.block_kind, self.stowed_tool, self.prefer_tool, self.mold), site_relic) }
        } else if self.hand_blocks > 0 { (self, self.block_kind) }
        else if current_hand.placeable_block() { (self.reconcile_hand(current_hand), current_hand) }
        else {
            let stowed = if self.stowed_tool == current_hand { HeldRelic::EmptyPalm } else { self.stowed_tool };
            (Self::new(0, self.stowed_blocks, self.block_kind, stowed, self.prefer_tool, self.mold), HeldRelic::EmptyPalm)
        }
    }

    pub fn collect_excavated_block(self, current_hand: HeldRelic) -> (Self, HeldRelic) {
        let tool = if current_hand.gripped() && !current_hand.placeable_block() { current_hand } else { self.stowed_tool };
        let stowed = if self.hand_blocks == 0 { self.stowed_blocks } else { self.stowed_blocks + 1 };
        (Self::new(1, stowed, HeldRelic::ExcavatedEarth, tool, false, self.mold), HeldRelic::ExcavatedEarth)
    }

    pub fn place_from_hand(self, current_hand: HeldRelic, eye: Vec3, view_dir: Vec3, avatar_pos: Vec3, world: &mut dyn PlanetaryWorld) -> (Self, HeldRelic, bool) {
        if !world.place_molded_surface_block(eye, view_dir, avatar_pos, self.mold) { return (self, current_hand, false); }
        if self.stowed_blocks > 0 { (Self::new(1, self.stowed_blocks - 1, current_hand, self.stowed_tool, self.prefer_tool, self.mold), current_hand, true) }
        else if self.hand_blocks > 1 { (Self::new(self.hand_blocks - 1, 0, current_hand, self.stowed_tool, self.prefer_tool, self.mold), current_hand, true) }
        else { (Self::new(0, 0, self.block_kind, self.stowed_tool, self.prefer_tool, self.mold), HeldRelic::EmptyPalm, true) }
    }

    pub fn toggle_pocket(self, current_hand: HeldRelic) -> (Self, HeldRelic, bool) {
        if current_hand.placeable_block() {
            let next_stowed = self.stowed_blocks + self.hand_blocks.max(1);
            (Self::new(0, next_stowed, current_hand, self.stowed_tool, self.stowed_tool.gripped(), self.mold), HeldRelic::EmptyPalm, true)
        } else if current_hand.gripped() {
            (Self::new(0, self.stowed_blocks, self.block_kind, current_hand, self.stowed_blocks == 0, self.mold), HeldRelic::EmptyPalm, true)
        } else if self.stowed_tool.gripped() && (self.prefer_tool || self.stowed_blocks == 0) {
            (Self::new(0, self.stowed_blocks, self.block_kind, HeldRelic::EmptyPalm, false, self.mold), self.stowed_tool, true)
        } else if self.stowed_blocks > 0 {
            (Self::new(1, self.stowed_blocks - 1, self.block_kind, self.stowed_tool, true, self.mold), self.block_kind, true)
        } else {
            (self, HeldRelic::EmptyPalm, false)
        }
    }

    pub fn emit_in_palm(&self, side: f32, hand_relic: HeldRelic, frame: &LimbFrame, target: &mut dyn MeshTarget) {
        hand_relic.emit_molded_in_palm(side, self.mold, frame, target);
    }

    pub fn emit_at_hip(&self, side: f32, pose: &BodyPose, frame: &LimbFrame, target: &mut dyn MeshTarget) {
        let (waist_y, px, pz) = (pose.torso_span(0.26), side * pose.scale(if pose.feminine() { 0.126 } else { 0.162 }), 0.036);
        if self.stowed_blocks > 0 {
            let fill = (self.stowed_blocks.min(12) as f32) / 12.0;
            let (rx, rz, drop) = (0.038 + 0.038 * fill, 0.034 + 0.036 * fill, 0.076 + 0.064 * fill);
            let (bx, bz) = (px + side * (0.020 * fill), pz + 0.016 * fill);
            let (leather, dark_hide, brass) = (Vec3::new(0.44, 0.27, 0.15), Vec3::new(0.32, 0.19, 0.11), Vec3::new(0.82, 0.66, 0.24));
            TaperedPrism::new(8, Vec3::new(bx, waist_y - drop, bz), Vec3::new(bx, waist_y - drop * 0.42, bz), rx * 0.72, rz * 0.72, rx, rz, leather).emit(frame, true, target);
            TaperedPrism::new(8, Vec3::new(bx, waist_y - drop * 0.42, bz), Vec3::new(bx, waist_y + 0.010, bz), rx, rz, rx * 0.76, rz * 0.76, dark_hide).emit(frame, false, target);
            TaperedPrism::new(8, Vec3::new(bx, waist_y + 0.006, bz), Vec3::new(bx, waist_y + 0.018, bz), rx * 0.80, rz * 0.80, rx * 0.80, rz * 0.80, brass).emit(frame, true, target);
            let (base_col, top_col) = if matches!(self.block_kind, HeldRelic::QuarriedStone) {
                (Vec3::new(0.50, 0.51, 0.53), Vec3::new(0.62, 0.64, 0.66))
            } else {
                (Vec3::new(0.44, 0.31, 0.19), Vec3::new(0.52, 0.38, 0.24))
            };
            frame.emit_molded_block(Vec3::new(bx, waist_y + 0.020, bz), self.mold, 0.050 + 0.026 * fill, base_col, top_col, target);
        }
        if self.stowed_tool.gripped() {
            let loop_frame = frame.shifted(px + side * 0.032, waist_y - 0.04, pz - 0.018).pitched(-1.12).yawed(side * 0.22);
            self.stowed_tool.emit_molded_in_palm(side, self.mold, &loop_frame, target);
        }
    }
}

/// Dual-hip belt satchel pairing independent right (`KeyE`) and left (`KeyQ`) pouches and block molds.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BeltSatchel {
    right: HipPouch,
    left: HipPouch,
    active_side: f32,
}

impl BeltSatchel {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(right: HipPouch, left: HipPouch, active_side: f32) -> Self { Self { right, left, active_side } }
    pub const fn empty() -> Self { Self::new(HipPouch::empty(), HipPouch::empty(), 1.0) }

    fn target_mold_sides(&self, attire: YouthAttire) -> (bool, bool) {
        let (r, l) = attire.held_pair();
        if r.placeable_block() && !l.placeable_block() { (true, false) }
        else if l.placeable_block() && !r.placeable_block() { (false, true) }
        else if r.placeable_block() && l.placeable_block() { (self.active_side > 0.0, self.active_side < 0.0) }
        else if self.right.unstocked() && self.left.unstocked() { (true, true) }
        else if self.right.holding_stowed_blocks() && !self.left.holding_stowed_blocks() { (true, false) }
        else if self.left.holding_stowed_blocks() && !self.right.holding_stowed_blocks() { (false, true) }
        else { (self.active_side > 0.0, self.active_side < 0.0) }
    }

    pub fn map_mold(self, attire: YouthAttire, op: fn(HipPouch) -> HipPouch) -> Self {
        let (r, l) = self.target_mold_sides(attire);
        Self::new(if r { op(self.right) } else { self.right }, if l { op(self.left) } else { self.left }, self.active_side)
    }

    pub fn reconcile_attire(self, attire: YouthAttire) -> Self {
        let (r, l) = attire.held_pair();
        Self::new(self.right.reconcile_hand(r), self.left.reconcile_hand(l), self.active_side)
    }

    pub fn merge_site_relics(self, attire: YouthAttire, site_pair: (HeldRelic, HeldRelic)) -> (Self, YouthAttire) {
        let (cur_r, cur_l) = attire.held_pair();
        let (next_r, hand_r) = self.right.merge_site_relic(cur_r, site_pair.0);
        let (next_l, hand_l) = self.left.merge_site_relic(cur_l, site_pair.1);
        (Self::new(next_r, next_l, self.active_side), attire.with_relics(hand_r, hand_l))
    }

    pub fn toggle_pocket(self, side: f32, attire: YouthAttire) -> (Self, YouthAttire, bool) {
        let state = self.reconcile_attire(attire);
        let (r, l) = attire.held_pair();
        if side > 0.0 {
            let (next_r, hand_r, ok) = state.right.toggle_pocket(r);
            (Self::new(next_r, state.left, 1.0), attire.with_relics(hand_r, l), ok)
        } else {
            let (next_l, hand_l, ok) = state.left.toggle_pocket(l);
            (Self::new(state.right, next_l, -1.0), attire.with_relics(r, hand_l), ok)
        }
    }

    pub fn place_from_hand(self, side: f32, attire: YouthAttire, eye: Vec3, view_dir: Vec3, avatar_pos: Vec3, world: &mut dyn PlanetaryWorld) -> (Self, YouthAttire, bool) {
        let (r, l) = attire.held_pair();
        if side > 0.0 {
            let (next_r, hand_r, placed) = self.right.place_from_hand(r, eye, view_dir, avatar_pos, world);
            (Self::new(next_r, self.left, 1.0), attire.with_relics(hand_r, l), placed)
        } else {
            let (next_l, hand_l, placed) = self.left.place_from_hand(l, eye, view_dir, avatar_pos, world);
            (Self::new(self.right, next_l, -1.0), attire.with_relics(r, hand_l), placed)
        }
    }

    pub fn collect_excavated_block(self, side: f32, attire: YouthAttire) -> (Self, YouthAttire) {
        let (r, l) = attire.held_pair();
        if side > 0.0 {
            let (next_r, hand_r) = self.right.collect_excavated_block(r);
            (Self::new(next_r, self.left, 1.0), attire.with_relics(hand_r, l))
        } else {
            let (next_l, hand_l) = self.left.collect_excavated_block(l);
            (Self::new(self.right, next_l, -1.0), attire.with_relics(r, hand_l))
        }
    }

    pub fn emit_in_palm(&self, side: f32, hand_relic: HeldRelic, frame: &LimbFrame, target: &mut dyn MeshTarget) {
        if side > 0.0 { self.right.emit_in_palm(side, hand_relic, frame, target); } else { self.left.emit_in_palm(side, hand_relic, frame, target); }
    }

    pub fn emit_on_waist(&self, pose: &BodyPose, target: &mut dyn MeshTarget) {
        let frame = pose.torso_frame();
        self.left.emit_at_hip(-1.0, pose, &frame, target);
        self.right.emit_at_hip(1.0, pose, &frame, target);
    }
}
