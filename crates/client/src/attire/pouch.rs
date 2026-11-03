use glam::Vec3;
use world::{BlockMold, MeshTarget, PlanetaryWorld};
use super::limb::LimbFrame;
use super::pose::BodyPose;
use super::relic::HeldRelic;
use super::tray::SatchelTray;
use super::YouthAttire;

#[derive(Clone, Copy, Debug, PartialEq)]
struct PalmStack { right_blocks: u32, left_blocks: u32, right_mold: BlockMold, left_mold: BlockMold }

impl PalmStack {
    const fn blocks_on(&self, side: f32) -> u32 { if side > 0.0 { self.right_blocks } else { self.left_blocks } }
    const fn mold_on(&self, side: f32) -> BlockMold { if side > 0.0 { self.right_mold } else { self.left_mold } }
    fn assign_blocks(&mut self, side: f32, blocks: u32) {
        if side > 0.0 { self.right_blocks = blocks; } else { self.left_blocks = blocks; }
    }
    fn sync_mold_from_opposite(&mut self, side: f32) {
        if side > 0.0 { self.right_mold = self.left_mold; } else { self.left_mold = self.right_mold; }
    }
}

/// Shared satchel inventory with invisible stowed storage, WASD inspection grid, and dual-hand block molds.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BeltSatchel {
    palms: PalmStack,
    tray: SatchelTray,
}

impl BeltSatchel {
    #[allow(clippy::too_many_arguments)]
    pub const fn new(
        right_blocks: u32, left_blocks: u32, stowed: [u32; 12], right_mold: BlockMold, left_mold: BlockMold,
        active_side: f32, cursor: usize, open: bool,
    ) -> Self {
        Self {
            palms: PalmStack { right_blocks, left_blocks, right_mold, left_mold },
            tray: SatchelTray::new(stowed, active_side, cursor, open),
        }
    }

    pub const fn empty() -> Self { Self::new(0, 0, [0; 12], BlockMold::cube(), BlockMold::cube(), 1.0, 0, false) }
    pub const fn open(&self) -> bool { self.tray.open() }
    pub const fn with_open(mut self, open: bool) -> Self { self.tray = self.tray.with_open(open); self }

    pub fn resource_tally(&self, relic: HeldRelic, attire: YouthAttire) -> u32 {
        let (r, l) = attire.held_pair();
        u32::from(r == relic) + u32::from(l == relic) + self.tray.count(relic)
    }

    pub fn map_mold(mut self, attire: YouthAttire, op: fn(BlockMold) -> BlockMold) -> Self {
        let (r, l) = attire.held_pair();
        let any_stowed = self.tray.any_stowed_blocks();
        let (mr, ml) = if r.placeable_block() || l.placeable_block() {
            (r.placeable_block(), l.placeable_block())
        } else if self.palms.right_blocks == 0 && self.palms.left_blocks == 0 && !any_stowed {
            (true, true)
        } else {
            (self.tray.active_side() > 0.0, self.tray.active_side() < 0.0)
        };
        if mr { self.palms.right_mold = op(self.palms.right_mold); }
        if ml { self.palms.left_mold = op(self.palms.left_mold); }
        self
    }

    pub fn reconcile_attire(mut self, attire: YouthAttire) -> Self {
        let (r, l) = attire.held_pair();
        if r.placeable_block() && l.placeable_block() && self.palms.right_blocks == 0 && self.palms.left_blocks > 0 {
            self.palms.right_mold = self.palms.left_mold;
        } else if r.placeable_block() && l.placeable_block() && self.palms.left_blocks == 0 && self.palms.right_blocks > 0 {
            self.palms.left_mold = self.palms.right_mold;
        }
        self.palms.right_blocks = if r.placeable_block() { self.palms.right_blocks.max(1) } else { 0 };
        self.palms.left_blocks = if l.placeable_block() { self.palms.left_blocks.max(1) } else { 0 };
        self
    }

    pub fn toggle_open(mut self) -> (Self, bool) {
        self.tray.toggle_open();
        (self, true)
    }

    pub fn navigate(mut self, dx: i32, dy: i32) -> (Self, bool) {
        let ok = self.tray.navigate(dx, dy);
        (self, ok)
    }

    pub fn transfer_hand(self, side: f32, attire: YouthAttire) -> (Self, YouthAttire, bool) {
        let (mut state, (hand, other)) = (self.reconcile_attire(attire), attire.hand_pair(side));
        let (slots, len) = state.tray.packed_slots();
        if state.tray.open() && len > 0 {
            let chosen = slots[state.tray.cursor().min(len - 1)];
            let _ = state.tray.take_stowed(chosen);
            if hand.gripped() { state.tray.stow_relic(hand, state.palms.blocks_on(side)); }
            if chosen.placeable_block() && other.placeable_block() { state.palms.sync_mold_from_opposite(side); }
            state.palms.assign_blocks(side, u32::from(chosen.placeable_block()));
            state.tray.focus_side(side);
            state.tray.close();
            return (state, attire.with_hand_relic(side, chosen), true);
        }
        if hand.gripped() {
            state.tray.close();
            state.tray.stow_relic(hand, state.palms.blocks_on(side));
            state.palms.assign_blocks(side, 0);
            state.tray.focus_side(side);
            (state, attire.with_hand_relic(side, HeldRelic::EmptyPalm), true)
        } else if state.tray.open() {
            state.tray.close();
            (state, attire, true)
        } else if len > 0 {
            state.tray.open_for_side(side, len);
            (state, attire, true)
        } else {
            (state, attire, false)
        }
    }

    pub fn place_from_hand(
        mut self, side: f32, attire: YouthAttire, eye: Vec3, view_dir: Vec3, avatar_pos: Vec3, world: &mut dyn PlanetaryWorld,
    ) -> (Self, YouthAttire, bool) {
        let (hand, _) = attire.hand_pair(side);
        let mold = self.palms.mold_on(side);
        if !world.place_molded_surface_block(eye, view_dir, avatar_pos, mold.in_medium(hand.block_medium())) {
            return (self, attire, false);
        }
        self.tray.focus_side(side);
        let has_refill = self.tray.take_stowed(hand);
        let next_hand = if has_refill { hand } else { HeldRelic::EmptyPalm };
        self.palms.assign_blocks(side, u32::from(has_refill));
        (self, attire.with_hand_relic(side, next_hand), true)
    }

    pub fn collect_excavated_block(mut self, side: f32, relic: HeldRelic, attire: YouthAttire) -> (Self, YouthAttire) {
        let (hand, other) = attire.hand_pair(side);
        if !hand.free_palm() && !other.free_palm() {
            self.tray.focus_side(side);
            self.tray.stow_relic(relic, 1);
            return (self, attire);
        }
        let target = if hand.free_palm() { side } else { -side };
        if hand.placeable_block() || other.placeable_block() { self.palms.sync_mold_from_opposite(target); }
        self.palms.assign_blocks(target, 1);
        self.tray.focus_side(target);
        (self, attire.with_hand_relic(target, relic))
    }

    pub fn collect_foraged(mut self, side: f32, relic: HeldRelic, attire: YouthAttire) -> (Self, YouthAttire) {
        let (r, l) = attire.held_pair();
        self.tray.focus_side(side);
        if (side > 0.0 && r.free_palm()) || (r.free_palm() && !l.free_palm()) {
            (self, attire.with_relics(relic, l))
        } else if l.free_palm() {
            (self, attire.with_relics(r, relic))
        } else {
            self.tray.stow_relic(relic, 1);
            (self, attire)
        }
    }

    pub fn collect_felled_timber(mut self, side: f32, attire: YouthAttire) -> (Self, YouthAttire) {
        let (r, l) = attire.held_pair();
        let next_attire = if r.free_palm() {
            self.palms.right_blocks = 1;
            self.tray.stow_relic(HeldRelic::TimberBlock, 2);
            attire.with_relics(HeldRelic::TimberBlock, l)
        } else if l.free_palm() {
            self.palms.left_blocks = 1;
            self.tray.stow_relic(HeldRelic::TimberBlock, 2);
            attire.with_relics(r, HeldRelic::TimberBlock)
        } else {
            self.tray.stow_relic(HeldRelic::TimberBlock, 3);
            attire
        };
        self.tray.focus_side(side);
        (self.reconcile_attire(next_attire), next_attire)
    }

    pub fn consume_and_equip(
        mut self, ingredients: [(HeldRelic, u32); 2], product: HeldRelic, batch_yield: u32, side: f32, mut attire: YouthAttire,
    ) -> (Self, YouthAttire) {
        for (ing, mut rem) in ingredients {
            while rem > 0 {
                let (r, l) = attire.held_pair();
                if !self.tray.take_stowed(ing) {
                    if r == ing {
                        self.palms.right_blocks = 0;
                        attire = attire.with_relics(HeldRelic::EmptyPalm, l);
                    } else if l == ing {
                        self.palms.left_blocks = 0;
                        attire = attire.with_relics(r, HeldRelic::EmptyPalm);
                    }
                }
                rem -= 1;
            }
        }
        let (hand, other) = attire.hand_pair(side);
        if hand.gripped() && !other.free_palm() {
            self.tray.stow_relic(hand, self.palms.blocks_on(side));
            self.palms.assign_blocks(side, 0);
            attire = attire.with_hand_relic(side, HeldRelic::EmptyPalm);
        }
        let (mut next, next_attire) = self.collect_foraged(side, product, attire);
        if batch_yield > 1 {
            next.tray.stow_relic(product, batch_yield - 1);
        }
        (next.reconcile_attire(next_attire), next_attire)
    }

    pub fn emit_in_palm(&self, side: f32, hand_relic: HeldRelic, frame: &LimbFrame, target: &mut dyn MeshTarget) {
        hand_relic.emit_molded_in_palm(side, self.palms.mold_on(side), frame, target);
    }

    pub fn emit_on_waist(&self, pose: &BodyPose, target: &mut dyn MeshTarget) {
        self.tray.emit_on_waist(pose, target);
    }
}
