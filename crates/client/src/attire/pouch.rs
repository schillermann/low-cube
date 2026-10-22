use glam::Vec3;
use world::{BlockMold, MeshTarget, PlanetaryWorld};
use super::limb::LimbFrame;
use super::pose::BodyPose;
use super::relic::HeldRelic;
use super::YouthAttire;

/// Shared satchel inventory with invisible stowed storage, WASD inspection grid, and dual-hand block molds.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BeltSatchel {
    right_blocks: u32, left_blocks: u32, stowed: [u32; 12],
    right_mold: BlockMold, left_mold: BlockMold, active_side: f32, cursor: usize, open: bool,
}

impl BeltSatchel {
    #[allow(clippy::too_many_arguments)]
    pub const fn new(
        right_blocks: u32, left_blocks: u32, stowed: [u32; 12],
        right_mold: BlockMold, left_mold: BlockMold, active_side: f32, cursor: usize, open: bool,
    ) -> Self { Self { right_blocks, left_blocks, stowed, right_mold, left_mold, active_side, cursor, open } }

    pub const fn empty() -> Self { Self::new(0, 0, [0; 12], BlockMold::cube(), BlockMold::cube(), 1.0, 0, false) }
    pub const fn open(&self) -> bool { self.open }
    pub const fn with_open(mut self, open: bool) -> Self { self.open = open; self }

    pub fn resource_tally(&self, relic: HeldRelic, attire: YouthAttire) -> u32 {
        let (r, l) = attire.held_pair();
        u32::from(r == relic) + u32::from(l == relic) + self.stowed[relic.code() as usize]
    }

    fn packed_slots(&self) -> ([HeldRelic; 9], usize) {
        let (mut slots, mut len) = ([HeldRelic::EmptyPalm; 9], 0usize);
        for mat in [HeldRelic::ExcavatedEarth, HeldRelic::QuarriedStone, HeldRelic::TimberBlock, HeldRelic::PlankBlock, HeldRelic::DeadwoodBough, HeldRelic::FlintStone] {
            if self.stowed[mat.code() as usize] > 0 && len < 9 { slots[len] = mat; len += 1; }
        }
        for relic in [HeldRelic::IronSpade, HeldRelic::WoodmanAxe, HeldRelic::KnightlySword, HeldRelic::WoodenMallet, HeldRelic::TimberTorch] {
            for _ in 0..self.stowed[relic.code() as usize] { if len < 9 { slots[len] = relic; len += 1; } }
        }
        (slots, len)
    }

    fn target_mold_sides(&self, attire: YouthAttire) -> (bool, bool) {
        let (r, l) = attire.held_pair();
        let any_stowed_blocks = self.stowed[1] > 0 || self.stowed[7] > 0 || self.stowed[10] > 0 || self.stowed[11] > 0;
        if r.placeable_block() || l.placeable_block() { (r.placeable_block(), l.placeable_block()) }
        else if self.right_blocks == 0 && self.left_blocks == 0 && !any_stowed_blocks { (true, true) }
        else { (self.active_side > 0.0, self.active_side < 0.0) }
    }

    pub fn map_mold(mut self, attire: YouthAttire, op: fn(BlockMold) -> BlockMold) -> Self {
        let (r, l) = self.target_mold_sides(attire);
        if r { self.right_mold = op(self.right_mold); }
        if l { self.left_mold = op(self.left_mold); }
        self
    }

    pub fn reconcile_attire(mut self, attire: YouthAttire) -> Self {
        let (r, l) = attire.held_pair();
        if r.placeable_block() && l.placeable_block() && self.right_blocks == 0 && self.left_blocks > 0 { self.right_mold = self.left_mold; }
        else if r.placeable_block() && l.placeable_block() && self.left_blocks == 0 && self.right_blocks > 0 { self.left_mold = self.right_mold; }
        self.right_blocks = if r.placeable_block() { self.right_blocks.max(1) } else { 0 };
        self.left_blocks = if l.placeable_block() { self.left_blocks.max(1) } else { 0 };
        self
    }

    fn stow_relic(&mut self, relic: HeldRelic, count: u32) {
        if relic.gripped() {
            self.stowed[relic.code() as usize] = self.stowed[relic.code() as usize].saturating_add(count.max(1));
            let (slots, len) = self.packed_slots();
            if let Some(idx) = slots[..len].iter().position(|&r| r == relic) { self.cursor = idx; }
        }
    }

    fn take_stowed(&mut self, relic: HeldRelic) -> bool {
        let slot = &mut self.stowed[relic.code() as usize];
        if *slot > 0 { *slot -= 1; true } else { false }
    }

    pub fn toggle_open(mut self) -> (Self, bool) {
        let (_, len) = self.packed_slots();
        self.open = !self.open;
        if self.open { self.cursor = self.cursor.min(len.saturating_sub(1)); }
        (self, true)
    }

    pub fn navigate(mut self, dx: i32, dy: i32) -> (Self, bool) {
        if !self.open { return (self, false); }
        let (_, len) = self.packed_slots();
        if len > 0 { self.cursor = (self.cursor + if dy > 0 { 3 } else if dy < 0 { len.saturating_sub(3).max(1) } else if dx > 0 { 1 } else { len - 1 }) % len; }
        (self, true)
    }

    pub fn transfer_hand(self, side: f32, attire: YouthAttire) -> (Self, YouthAttire, bool) {
        let (mut state, (r, l)) = (self.reconcile_attire(attire), attire.held_pair());
        let (slots, len) = state.packed_slots();
        if state.open && len > 0 {
            let chosen = slots[state.cursor.min(len - 1)];
            let _ = state.take_stowed(chosen);
            if side > 0.0 {
                if r.gripped() { state.stow_relic(r, state.right_blocks); }
                if chosen.placeable_block() && l.placeable_block() { state.right_mold = state.left_mold; }
                state.right_blocks = u32::from(chosen.placeable_block()); state.active_side = 1.0; state.open = false;
                return (state, attire.with_relics(chosen, l), true);
            }
            if l.gripped() { state.stow_relic(l, state.left_blocks); }
            if chosen.placeable_block() && r.placeable_block() { state.left_mold = state.right_mold; }
            state.left_blocks = u32::from(chosen.placeable_block()); state.active_side = -1.0; state.open = false;
            return (state, attire.with_relics(r, chosen), true);
        }
        if (if side > 0.0 { r } else { l }).gripped() {
            state.open = false;
            if side > 0.0 { state.stow_relic(r, state.right_blocks); state.right_blocks = 0; state.active_side = 1.0; (state, attire.with_relics(HeldRelic::EmptyPalm, l), true) }
            else { state.stow_relic(l, state.left_blocks); state.left_blocks = 0; state.active_side = -1.0; (state, attire.with_relics(r, HeldRelic::EmptyPalm), true) }
        } else if state.open { state.open = false; (state, attire, true) }
        else if len > 0 { state.open = true; state.cursor = state.cursor.min(len - 1); state.active_side = side; (state, attire, true) }
        else { (state, attire, false) }
    }

    pub fn place_from_hand(mut self, side: f32, attire: YouthAttire, eye: Vec3, view_dir: Vec3, avatar_pos: Vec3, world: &mut dyn PlanetaryWorld) -> (Self, YouthAttire, bool) {
        let ((r, l), mold) = (attire.held_pair(), if side > 0.0 { self.right_mold } else { self.left_mold });
        let hand = if side > 0.0 { r } else { l };
        if !world.place_molded_surface_block(eye, view_dir, avatar_pos, mold.in_medium(hand.block_medium())) { return (self, attire, false); }
        self.active_side = side;
        let has_refill = self.take_stowed(hand);
        let next_hand = if has_refill { hand } else { HeldRelic::EmptyPalm };
        if side > 0.0 { self.right_blocks = u32::from(has_refill); (self, attire.with_relics(next_hand, l), true) }
        else { self.left_blocks = u32::from(has_refill); (self, attire.with_relics(r, next_hand), true) }
    }

    pub fn collect_excavated_block(mut self, side: f32, attire: YouthAttire) -> (Self, YouthAttire) {
        let (r, l) = attire.held_pair();
        self.active_side = side;
        if side > 0.0 {
            if r.gripped() && !r.placeable_block() { self.stow_relic(r, 0); }
            if l.placeable_block() { self.right_mold = self.left_mold; }
            if self.right_blocks == 0 { self.right_blocks = 1; } else { self.stow_relic(HeldRelic::ExcavatedEarth, 1); }
            return (self, attire.with_relics(if r.placeable_block() { r } else { HeldRelic::ExcavatedEarth }, l));
        }
        if l.gripped() && !l.placeable_block() { self.stow_relic(l, 0); }
        if r.placeable_block() { self.left_mold = self.right_mold; }
        if self.left_blocks == 0 { self.left_blocks = 1; } else { self.stow_relic(HeldRelic::ExcavatedEarth, 1); }
        (self, attire.with_relics(r, if l.placeable_block() { l } else { HeldRelic::ExcavatedEarth }))
    }

    pub fn collect_foraged(mut self, side: f32, relic: HeldRelic, attire: YouthAttire) -> (Self, YouthAttire) {
        let (r, l) = attire.held_pair();
        self.active_side = side;
        if (side > 0.0 && r.free_palm()) || (r.free_palm() && !l.free_palm()) { (self, attire.with_relics(relic, l)) }
        else if l.free_palm() { (self, attire.with_relics(r, relic)) }
        else { self.stow_relic(relic, 1); (self, attire) }
    }

    pub fn collect_felled_timber(mut self, side: f32, attire: YouthAttire) -> (Self, YouthAttire) {
        let (r, l) = attire.held_pair();
        let next_attire = if r.free_palm() { self.right_blocks = 1; self.stow_relic(HeldRelic::TimberBlock, 2); attire.with_relics(HeldRelic::TimberBlock, l) }
        else if l.free_palm() { self.left_blocks = 1; self.stow_relic(HeldRelic::TimberBlock, 2); attire.with_relics(r, HeldRelic::TimberBlock) }
        else { self.stow_relic(HeldRelic::TimberBlock, 3); attire };
        self.active_side = side;
        (self.reconcile_attire(next_attire), next_attire)
    }

    pub fn consume_and_equip(mut self, ingredients: [(HeldRelic, u32); 2], product: HeldRelic, side: f32, mut attire: YouthAttire) -> (Self, YouthAttire) {
        for (ing, mut rem) in ingredients {
            while rem > 0 {
                let (r, l) = attire.held_pair();
                if !self.take_stowed(ing) {
                    if r == ing { self.right_blocks = 0; attire = attire.with_relics(HeldRelic::EmptyPalm, l); }
                    else if l == ing { self.left_blocks = 0; attire = attire.with_relics(r, HeldRelic::EmptyPalm); }
                }
                rem -= 1;
            }
        }
        let (r, l) = attire.held_pair();
        if side > 0.0 && r.gripped() && !l.free_palm() { self.stow_relic(r, self.right_blocks); self.right_blocks = 0; attire = attire.with_relics(HeldRelic::EmptyPalm, l); }
        else if side < 0.0 && l.gripped() && !r.free_palm() { self.stow_relic(l, self.left_blocks); self.left_blocks = 0; attire = attire.with_relics(r, HeldRelic::EmptyPalm); }
        let (mut next, next_attire) = self.collect_foraged(side, product, attire);
        if product == HeldRelic::PlankBlock { next.stow_relic(HeldRelic::PlankBlock, 3); }
        (next.reconcile_attire(next_attire), next_attire)
    }

    pub fn emit_in_palm(&self, side: f32, hand_relic: HeldRelic, frame: &LimbFrame, target: &mut dyn MeshTarget) {
        hand_relic.emit_molded_in_palm(side, if side > 0.0 { self.right_mold } else { self.left_mold }, frame, target);
    }

    pub fn emit_on_waist(&self, pose: &BodyPose, target: &mut dyn MeshTarget) {
        if !self.open { return; }
        let ((slots, len), tray, mold) = (self.packed_slots(), pose.view_frame().shifted(0.0, -0.02, 0.40), BlockMold::cube());
        let (cols, rows, sel) = (len.clamp(1, 3), (len.max(1) + 2) / 3, self.cursor.min(len.saturating_sub(1)));
        if len == 0 { HeldRelic::EmptyPalm.emit_in_satchel_slot(mold, Vec3::new(0.42, 0.27, 0.16), &tray, target); }
        for (i, &item) in slots[..len].iter().enumerate() {
            let (col, row) = (i % 3, i / 3);
            let r_cols = if row + 1 == rows && len % 3 != 0 { len % 3 } else { cols };
            let (sx, sy, sz) = (((col as f32) - (r_cols as f32 - 1.0) * 0.5) * 0.115, ((rows as f32 - 1.0) * 0.5 - (row as f32)) * 0.115, if i == sel { -0.022 } else { 0.0 });
            item.emit_in_satchel_slot(mold, if i == sel { Vec3::new(0.95, 0.78, 0.24) } else { Vec3::new(0.42, 0.27, 0.16) }, &tray.shifted(sx, sy, sz), target);
        }
        (if len > 0 { slots[sel] } else { HeldRelic::EmptyPalm }).emit_satchel_legend(&tray.shifted(0.0, -((rows as f32 - 1.0) * 0.5) * 0.115 - 0.090, 0.0), target);
    }
}
