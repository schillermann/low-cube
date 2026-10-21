use glam::Vec3;
use world::{BlockMold, MeshTarget, PlanetaryWorld};
use super::limb::LimbFrame;
use super::pose::BodyPose;
use super::relic::HeldRelic;
use super::YouthAttire;

/// Shared satchel inventory with invisible stowed storage, WASD inspection grid, and dual-hand block molds.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BeltSatchel {
    right_blocks: u32, left_blocks: u32, stowed_earth: u32, stowed_stone: u32, stowed_tools: u8,
    right_mold: BlockMold, left_mold: BlockMold, active_side: f32, cursor: usize, open: bool,
}

impl BeltSatchel {
    /// Primary constructor doing nothing except assigning arguments to fields.
    #[allow(clippy::too_many_arguments)]
    pub const fn new(
        right_blocks: u32, left_blocks: u32, stowed_earth: u32, stowed_stone: u32, stowed_tools: u8,
        right_mold: BlockMold, left_mold: BlockMold, active_side: f32, cursor: usize, open: bool,
    ) -> Self {
        Self { right_blocks, left_blocks, stowed_earth, stowed_stone, stowed_tools, right_mold, left_mold, active_side, cursor, open }
    }

    pub const fn empty() -> Self { Self::new(0, 0, 0, 0, 0, BlockMold::cube(), BlockMold::cube(), 1.0, 0, false) }
    pub const fn open(&self) -> bool { self.open }

    fn packed_slots(&self) -> ([HeldRelic; 7], usize) {
        let (mut slots, mut len) = ([HeldRelic::EmptyPalm; 7], 0usize);
        if self.stowed_earth > 0 { slots[len] = HeldRelic::ExcavatedEarth; len += 1; }
        if self.stowed_stone > 0 { slots[len] = HeldRelic::QuarriedStone; len += 1; }
        for relic in [HeldRelic::IronSpade, HeldRelic::WoodmanAxe, HeldRelic::KnightlySword, HeldRelic::WoodenMallet, HeldRelic::TimberTorch] {
            if self.stowed_tools & (1 << relic.code()) != 0 { slots[len] = relic; len += 1; }
        }
        (slots, len)
    }

    fn target_mold_sides(&self, attire: YouthAttire) -> (bool, bool) {
        let (r, l) = attire.held_pair();
        if r.placeable_block() && !l.placeable_block() { (true, false) }
        else if l.placeable_block() && !r.placeable_block() { (false, true) }
        else if r.placeable_block() && l.placeable_block() { (self.active_side > 0.0, self.active_side < 0.0) }
        else if self.right_blocks == 0 && self.left_blocks == 0 && self.stowed_earth == 0 && self.stowed_stone == 0 { (true, true) }
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
        self.right_blocks = if r.placeable_block() { self.right_blocks.max(1) } else { 0 };
        self.left_blocks = if l.placeable_block() { self.left_blocks.max(1) } else { 0 };
        if r.gripped() && !r.placeable_block() { self.stowed_tools &= !(1 << r.code()); }
        if l.gripped() && !l.placeable_block() { self.stowed_tools &= !(1 << l.code()); }
        self
    }

    fn resolve_site_hand(&mut self, cur: HeldRelic, site: HeldRelic, blocks: &mut u32) -> HeldRelic {
        if site.gripped() {
            if self.stowed_tools & (1 << site.code()) != 0 {
                if *blocks > 0 { HeldRelic::ExcavatedEarth } else { HeldRelic::EmptyPalm }
            } else {
                if cur.placeable_block() { self.stow_relic(cur, *blocks); *blocks = 0; }
                site
            }
        } else if *blocks > 0 || cur.placeable_block() {
            *blocks = (*blocks).max(1);
            if cur.placeable_block() { cur } else { HeldRelic::ExcavatedEarth }
        } else { HeldRelic::EmptyPalm }
    }

    pub fn merge_site_relics(mut self, attire: YouthAttire, site_pair: (HeldRelic, HeldRelic)) -> (Self, YouthAttire) {
        let (cur_r, cur_l) = attire.held_pair();
        let (mut rb, mut lb) = (self.right_blocks, self.left_blocks);
        let hand_r = self.resolve_site_hand(cur_r, site_pair.0, &mut rb);
        let hand_l = self.resolve_site_hand(cur_l, site_pair.1, &mut lb);
        self.right_blocks = rb;
        self.left_blocks = lb;
        (self, attire.with_relics(hand_r, hand_l))
    }

    fn stow_relic(&mut self, relic: HeldRelic, count: u32) {
        match relic {
            HeldRelic::ExcavatedEarth => self.stowed_earth += count.max(1),
            HeldRelic::QuarriedStone => self.stowed_stone += count.max(1),
            _ if relic.gripped() => self.stowed_tools |= 1 << relic.code(),
            _ => {}
        }
        let (slots, len) = self.packed_slots();
        if let Some(idx) = slots[..len].iter().position(|&r| r == relic) { self.cursor = idx; }
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
        if len > 0 {
            let step = if dy > 0 { 3 } else if dy < 0 { len.saturating_sub(3).max(1) } else if dx > 0 { 1 } else { len - 1 };
            self.cursor = (self.cursor + step) % len;
        }
        (self, true)
    }

    pub fn toggle_pocket(self, side: f32, attire: YouthAttire) -> (Self, YouthAttire, bool) {
        let mut state = self.reconcile_attire(attire);
        let (r, l) = attire.held_pair();
        let (slots, len) = state.packed_slots();
        if state.open && len > 0 {
            let chosen = slots[state.cursor.min(len - 1)];
            match chosen {
                HeldRelic::ExcavatedEarth => state.stowed_earth = state.stowed_earth.saturating_sub(1),
                HeldRelic::QuarriedStone => state.stowed_stone = state.stowed_stone.saturating_sub(1),
                _ => state.stowed_tools &= !(1 << chosen.code()),
            }
            if side > 0.0 {
                if r.gripped() { state.stow_relic(r, state.right_blocks); }
                state.right_blocks = if chosen.placeable_block() { 1 } else { 0 };
                state.active_side = 1.0; state.open = false;
                return (state, attire.with_relics(chosen, l), true);
            }
            if l.gripped() { state.stow_relic(l, state.left_blocks); }
            state.left_blocks = if chosen.placeable_block() { 1 } else { 0 };
            state.active_side = -1.0; state.open = false;
            return (state, attire.with_relics(r, chosen), true);
        }
        let hand = if side > 0.0 { r } else { l };
        if hand.gripped() {
            state.open = false;
            if side > 0.0 { state.stow_relic(r, state.right_blocks); state.right_blocks = 0; state.active_side = 1.0; (state, attire.with_relics(HeldRelic::EmptyPalm, l), true) }
            else { state.stow_relic(l, state.left_blocks); state.left_blocks = 0; state.active_side = -1.0; (state, attire.with_relics(r, HeldRelic::EmptyPalm), true) }
        } else if state.open {
            state.open = false; (state, attire, true)
        } else if len > 0 {
            state.open = true; state.cursor = state.cursor.min(len - 1); state.active_side = side; (state, attire, true)
        } else { (state, attire, false) }
    }

    pub fn place_from_hand(mut self, side: f32, attire: YouthAttire, eye: Vec3, view_dir: Vec3, avatar_pos: Vec3, world: &mut dyn PlanetaryWorld) -> (Self, YouthAttire, bool) {
        let (r, l) = attire.held_pair();
        let (hand, mold) = if side > 0.0 { (r, self.right_mold) } else { (l, self.left_mold) };
        if !world.place_molded_surface_block(eye, view_dir, avatar_pos, mold) { return (self, attire, false); }
        self.active_side = side;
        let has_refill = if matches!(hand, HeldRelic::QuarriedStone) {
            if self.stowed_stone > 0 { self.stowed_stone -= 1; true } else { false }
        } else if self.stowed_earth > 0 { self.stowed_earth -= 1; true } else { false };
        let next_hand = if has_refill { hand } else { HeldRelic::EmptyPalm };
        if side > 0.0 { self.right_blocks = if has_refill { 1 } else { 0 }; (self, attire.with_relics(next_hand, l), true) }
        else { self.left_blocks = if has_refill { 1 } else { 0 }; (self, attire.with_relics(r, next_hand), true) }
    }

    pub fn collect_excavated_block(mut self, side: f32, attire: YouthAttire) -> (Self, YouthAttire) {
        let (r, l) = attire.held_pair();
        self.active_side = side;
        if side > 0.0 {
            if r.gripped() && !r.placeable_block() { self.stow_relic(r, 0); }
            if self.right_blocks == 0 { self.right_blocks = 1; } else { self.stowed_earth += 1; }
            (self, attire.with_relics(HeldRelic::ExcavatedEarth, l))
        } else {
            if l.gripped() && !l.placeable_block() { self.stow_relic(l, 0); }
            if self.left_blocks == 0 { self.left_blocks = 1; } else { self.stowed_earth += 1; }
            (self, attire.with_relics(r, HeldRelic::ExcavatedEarth))
        }
    }

    pub fn emit_in_palm(&self, side: f32, hand_relic: HeldRelic, frame: &LimbFrame, target: &mut dyn MeshTarget) {
        hand_relic.emit_molded_in_palm(side, if side > 0.0 { self.right_mold } else { self.left_mold }, frame, target);
    }

    pub fn emit_on_waist(&self, pose: &BodyPose, target: &mut dyn MeshTarget) {
        if !self.open { return; }
        let (slots, len) = self.packed_slots();
        let tray = pose.view_frame().shifted(0.0, -0.02, 0.40);
        let mold = if self.active_side > 0.0 { self.right_mold } else { self.left_mold };
        if len == 0 { HeldRelic::EmptyPalm.emit_in_satchel_slot(mold, Vec3::new(0.42, 0.27, 0.16), &tray, target); return; }
        let (cols, rows, sel) = (len.clamp(1, 3), (len + 2) / 3, self.cursor.min(len - 1));
        for (i, &item) in slots[..len].iter().enumerate() {
            let (col, row) = (i % 3, i / 3);
            let r_cols = if row + 1 == rows && len % 3 != 0 { len % 3 } else { cols };
            let (sx, sy, sz) = (((col as f32) - (r_cols as f32 - 1.0) * 0.5) * 0.115, ((rows as f32 - 1.0) * 0.5 - (row as f32)) * 0.115, if i == sel { -0.022 } else { 0.0 });
            let rim = if i == sel { Vec3::new(0.95, 0.78, 0.24) } else { Vec3::new(0.42, 0.27, 0.16) };
            item.emit_in_satchel_slot(mold, rim, &tray.shifted(sx, sy, sz), target);
        }
    }
}
