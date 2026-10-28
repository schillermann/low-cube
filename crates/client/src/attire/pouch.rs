use glam::Vec3;
use world::{BlockMold, MeshTarget, PlanetaryWorld};
use super::limb::LimbFrame;
use super::pose::BodyPose;
use super::relic::HeldRelic;
use super::YouthAttire;

#[derive(Clone, Copy, Debug, PartialEq)]
struct PalmStack { right_blocks: u32, left_blocks: u32, right_mold: BlockMold, left_mold: BlockMold }

impl PalmStack {
    const fn blocks_on(&self, side: f32) -> u32 { if side > 0.0 { self.right_blocks } else { self.left_blocks } }
    const fn mold_on(&self, side: f32) -> BlockMold { if side > 0.0 { self.right_mold } else { self.left_mold } }
    fn set_blocks(&mut self, side: f32, blocks: u32) {
        if side > 0.0 { self.right_blocks = blocks; } else { self.left_blocks = blocks; }
    }
    fn sync_mold_from_opposite(&mut self, side: f32) {
        if side > 0.0 { self.right_mold = self.left_mold; } else { self.left_mold = self.right_mold; }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct SatchelTray { stowed: [u32; 12], active_side: f32, cursor: usize, open: bool }

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
            tray: SatchelTray { stowed, active_side, cursor, open },
        }
    }

    pub const fn empty() -> Self { Self::new(0, 0, [0; 12], BlockMold::cube(), BlockMold::cube(), 1.0, 0, false) }
    pub const fn open(&self) -> bool { self.tray.open }
    pub const fn with_open(mut self, open: bool) -> Self { self.tray.open = open; self }

    pub fn resource_tally(&self, relic: HeldRelic, attire: YouthAttire) -> u32 {
        let (r, l) = attire.held_pair();
        u32::from(r == relic) + u32::from(l == relic) + self.tray.stowed[relic.code() as usize]
    }

    fn packed_slots(&self) -> ([HeldRelic; 9], usize) {
        let (mut slots, mut len) = ([HeldRelic::EmptyPalm; 9], 0usize);
        let materials = [
            HeldRelic::ExcavatedEarth, HeldRelic::QuarriedStone, HeldRelic::TimberBlock,
            HeldRelic::PlankBlock, HeldRelic::DeadwoodBough, HeldRelic::FlintStone,
        ];
        for mat in materials {
            if self.tray.stowed[mat.code() as usize] > 0 && len < 9 { slots[len] = mat; len += 1; }
        }
        let tools = [
            HeldRelic::IronSpade, HeldRelic::WoodmanAxe, HeldRelic::KnightlySword,
            HeldRelic::WoodenMallet, HeldRelic::TimberTorch,
        ];
        for relic in tools {
            for _ in 0..self.tray.stowed[relic.code() as usize] {
                if len < 9 { slots[len] = relic; len += 1; }
            }
        }
        (slots, len)
    }

    pub fn map_mold(mut self, attire: YouthAttire, op: fn(BlockMold) -> BlockMold) -> Self {
        let (r, l) = attire.held_pair();
        let s = &self.tray.stowed;
        let any_stowed = s[1] > 0 || s[7] > 0 || s[10] > 0 || s[11] > 0;
        let (mr, ml) = if r.placeable_block() || l.placeable_block() {
            (r.placeable_block(), l.placeable_block())
        } else if self.palms.right_blocks == 0 && self.palms.left_blocks == 0 && !any_stowed {
            (true, true)
        } else {
            (self.tray.active_side > 0.0, self.tray.active_side < 0.0)
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

    fn stow_relic(&mut self, relic: HeldRelic, count: u32) {
        if relic.gripped() {
            let idx_code = relic.code() as usize;
            self.tray.stowed[idx_code] = self.tray.stowed[idx_code].saturating_add(count.max(1));
            let (slots, len) = self.packed_slots();
            if let Some(idx) = slots[..len].iter().position(|&r| r == relic) {
                self.tray.cursor = idx;
            }
        }
    }

    fn take_stowed(&mut self, relic: HeldRelic) -> bool {
        let slot = &mut self.tray.stowed[relic.code() as usize];
        if *slot > 0 {
            *slot -= 1;
            true
        } else {
            false
        }
    }

    fn with_hand_relic(attire: YouthAttire, side: f32, relic: HeldRelic) -> YouthAttire {
        let (r, l) = attire.held_pair();
        if side > 0.0 { attire.with_relics(relic, l) } else { attire.with_relics(r, relic) }
    }

    pub fn toggle_open(mut self) -> (Self, bool) {
        let (_, len) = self.packed_slots();
        self.tray.open = !self.tray.open;
        if self.tray.open {
            self.tray.cursor = self.tray.cursor.min(len.saturating_sub(1));
        }
        (self, true)
    }

    pub fn navigate(mut self, dx: i32, dy: i32) -> (Self, bool) {
        if !self.tray.open {
            return (self, false);
        }
        let (_, len) = self.packed_slots();
        if len > 0 {
            let step = if dy > 0 { 3 } else if dy < 0 { len.saturating_sub(3).max(1) } else if dx > 0 { 1 } else { len - 1 };
            self.tray.cursor = (self.tray.cursor + step) % len;
        }
        (self, true)
    }

    pub fn transfer_hand(self, side: f32, attire: YouthAttire) -> (Self, YouthAttire, bool) {
        let (mut state, (r, l)) = (self.reconcile_attire(attire), attire.held_pair());
        let (hand, other) = if side > 0.0 { (r, l) } else { (l, r) };
        let (slots, len) = state.packed_slots();
        if state.tray.open && len > 0 {
            let chosen = slots[state.tray.cursor.min(len - 1)];
            let _ = state.take_stowed(chosen);
            if hand.gripped() { state.stow_relic(hand, state.palms.blocks_on(side)); }
            if chosen.placeable_block() && other.placeable_block() { state.palms.sync_mold_from_opposite(side); }
            state.palms.set_blocks(side, u32::from(chosen.placeable_block()));
            state.tray.active_side = if side > 0.0 { 1.0 } else { -1.0 };
            state.tray.open = false;
            return (state, Self::with_hand_relic(attire, side, chosen), true);
        }
        if hand.gripped() {
            state.tray.open = false;
            state.stow_relic(hand, state.palms.blocks_on(side));
            state.palms.set_blocks(side, 0);
            state.tray.active_side = if side > 0.0 { 1.0 } else { -1.0 };
            (state, Self::with_hand_relic(attire, side, HeldRelic::EmptyPalm), true)
        } else if state.tray.open {
            state.tray.open = false;
            (state, attire, true)
        } else if len > 0 {
            state.tray.open = true;
            state.tray.cursor = state.tray.cursor.min(len - 1);
            state.tray.active_side = side;
            (state, attire, true)
        } else {
            (state, attire, false)
        }
    }

    pub fn place_from_hand(
        mut self, side: f32, attire: YouthAttire, eye: Vec3, view_dir: Vec3, avatar_pos: Vec3, world: &mut dyn PlanetaryWorld,
    ) -> (Self, YouthAttire, bool) {
        let (r, l) = attire.held_pair();
        let (mold, hand) = (self.palms.mold_on(side), if side > 0.0 { r } else { l });
        if !world.place_molded_surface_block(eye, view_dir, avatar_pos, mold.in_medium(hand.block_medium())) {
            return (self, attire, false);
        }
        self.tray.active_side = side;
        let has_refill = self.take_stowed(hand);
        let next_hand = if has_refill { hand } else { HeldRelic::EmptyPalm };
        self.palms.set_blocks(side, u32::from(has_refill));
        (self, Self::with_hand_relic(attire, side, next_hand), true)
    }

    pub fn collect_excavated_block(mut self, side: f32, relic: HeldRelic, attire: YouthAttire) -> (Self, YouthAttire) {
        let (r, l) = attire.held_pair();
        let (hand, other) = if side > 0.0 { (r, l) } else { (l, r) };
        if !hand.free_palm() && !other.free_palm() {
            self.tray.active_side = side;
            self.stow_relic(relic, 1);
            return (self, attire);
        }
        let target = if hand.free_palm() { side } else { -side };
        if hand.placeable_block() || other.placeable_block() { self.palms.sync_mold_from_opposite(target); }
        self.palms.set_blocks(target, 1);
        self.tray.active_side = target;
        (self, Self::with_hand_relic(attire, target, relic))
    }

    pub fn collect_foraged(mut self, side: f32, relic: HeldRelic, attire: YouthAttire) -> (Self, YouthAttire) {
        let (r, l) = attire.held_pair();
        self.tray.active_side = side;
        if (side > 0.0 && r.free_palm()) || (r.free_palm() && !l.free_palm()) {
            (self, attire.with_relics(relic, l))
        } else if l.free_palm() {
            (self, attire.with_relics(r, relic))
        } else {
            self.stow_relic(relic, 1);
            (self, attire)
        }
    }

    pub fn collect_felled_timber(mut self, side: f32, attire: YouthAttire) -> (Self, YouthAttire) {
        let (r, l) = attire.held_pair();
        let next_attire = if r.free_palm() {
            self.palms.right_blocks = 1;
            self.stow_relic(HeldRelic::TimberBlock, 2);
            attire.with_relics(HeldRelic::TimberBlock, l)
        } else if l.free_palm() {
            self.palms.left_blocks = 1;
            self.stow_relic(HeldRelic::TimberBlock, 2);
            attire.with_relics(r, HeldRelic::TimberBlock)
        } else {
            self.stow_relic(HeldRelic::TimberBlock, 3);
            attire
        };
        self.tray.active_side = side;
        (self.reconcile_attire(next_attire), next_attire)
    }

    pub fn consume_and_equip(
        mut self, ingredients: [(HeldRelic, u32); 2], product: HeldRelic, side: f32, mut attire: YouthAttire,
    ) -> (Self, YouthAttire) {
        for (ing, mut rem) in ingredients {
            while rem > 0 {
                let (r, l) = attire.held_pair();
                if !self.take_stowed(ing) {
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
        let (r, l) = attire.held_pair();
        if (side > 0.0 && r.gripped() && !l.free_palm()) || (side < 0.0 && l.gripped() && !r.free_palm()) {
            let hand = if side > 0.0 { r } else { l };
            self.stow_relic(hand, self.palms.blocks_on(side));
            self.palms.set_blocks(side, 0);
            attire = Self::with_hand_relic(attire, side, HeldRelic::EmptyPalm);
        }
        let (mut next, next_attire) = self.collect_foraged(side, product, attire);
        if product == HeldRelic::PlankBlock { next.stow_relic(HeldRelic::PlankBlock, 3); }
        (next.reconcile_attire(next_attire), next_attire)
    }

    pub fn emit_in_palm(&self, side: f32, hand_relic: HeldRelic, frame: &LimbFrame, target: &mut dyn MeshTarget) {
        hand_relic.emit_molded_in_palm(side, self.palms.mold_on(side), frame, target);
    }

    pub fn emit_on_waist(&self, pose: &BodyPose, target: &mut dyn MeshTarget) {
        if !self.tray.open { return; }
        let ((slots, len), tray, mold) = (self.packed_slots(), pose.view_frame().shifted(0.0, -0.02, 0.40), BlockMold::cube());
        let (cols, rows, sel) = (len.clamp(1, 3), (len.max(1) + 2) / 3, self.tray.cursor.min(len.saturating_sub(1)));
        if len == 0 {
            HeldRelic::EmptyPalm.emit_in_satchel_slot(mold, Vec3::new(0.42, 0.27, 0.16), &tray, target);
        }
        for (i, &item) in slots[..len].iter().enumerate() {
            let (col, row) = (i % 3, i / 3);
            let r_cols = if row + 1 == rows && len % 3 != 0 { len % 3 } else { cols };
            let sx = ((col as f32) - (r_cols as f32 - 1.0) * 0.5) * 0.115;
            let sy = ((rows as f32 - 1.0) * 0.5 - (row as f32)) * 0.115;
            let sz = if i == sel { -0.022 } else { 0.0 };
            let rim_col = if i == sel { Vec3::new(0.95, 0.78, 0.24) } else { Vec3::new(0.42, 0.27, 0.16) };
            item.emit_in_satchel_slot(mold, rim_col, &tray.shifted(sx, sy, sz), target);
        }
        let legend = if len > 0 { slots[sel] } else { HeldRelic::EmptyPalm };
        legend.emit_satchel_legend(&tray.shifted(0.0, -((rows as f32 - 1.0) * 0.5) * 0.115 - 0.090, 0.0), target);
    }
}
