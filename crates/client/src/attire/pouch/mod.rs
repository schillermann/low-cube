mod palms;

use glam::Vec3;
use world::{BlockMold, MeshTarget, PlanetaryWorld};
use super::limb::LimbFrame;
use super::pose::BodyPose;
use super::relic::HeldRelic;
use super::sight::{CameraMount, TripodMount};
use super::tray::SatchelTray;
use super::YouthAttire;
use self::palms::PalmStack;

/// Shared satchel inventory with invisible stowed storage, WASD inspection grid, and dual-hand block molds.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BeltSatchel {
    palms: PalmStack,
    tray: SatchelTray,
    tripod: TripodMount,
}

impl BeltSatchel {
    #[allow(clippy::too_many_arguments)]
    pub const fn new(
        right_blocks: u32, left_blocks: u32, stowed: [u32; 22], right_mold: BlockMold, left_mold: BlockMold,
        active_side: f32, cursor: usize, open: bool,
    ) -> Self {
        Self {
            palms: PalmStack::new(right_blocks, left_blocks, right_mold, left_mold),
            tray: SatchelTray::new(stowed, active_side, cursor, open),
            tripod: TripodMount::Unmounted,
        }
    }

    pub const fn empty() -> Self {
        Self::new(0, 0, [0; 22], BlockMold::cube(), BlockMold::cube(), 1.0, 0, false)
    }

    pub fn with_stowed_relic(mut self, relic: HeldRelic, count: u32) -> Self {
        if count > 0 {
            self.tray.stow_relic(relic, count);
        }
        self
    }

    pub const fn open(&self) -> bool { self.tray.open() }
    pub const fn with_open(mut self, open: bool) -> Self { self.tray = self.tray.with_open(open); self }
    pub const fn camera_mount(&self, attire: YouthAttire) -> CameraMount {
        let mount = self.palms.camera_mount(attire);
        if matches!(mount, CameraMount::Handheld(_)) {
            mount
        } else if let Some(sight) = self.tripod.placed_sight() {
            CameraMount::Placed(sight)
        } else {
            mount
        }
    }

    pub fn placed_camera_lens(&self, attire: YouthAttire) -> Option<(Vec3, Vec3, Vec3)> {
        let (r, l) = attire.held_pair();
        if r.field_camera() || l.field_camera() {
            None
        } else {
            self.tripod.lens_ray()
        }
    }

    pub fn resource_tally(&self, relic: HeldRelic, attire: YouthAttire) -> u32 {
        let (r, l) = attire.held_pair();
        u32::from(r == relic) + u32::from(l == relic) + self.tray.count(relic)
    }

    pub fn map_mold(mut self, attire: YouthAttire, op: fn(BlockMold) -> BlockMold) -> Self {
        let (r, l) = attire.held_pair();
        let (pr, pl) = self.palms.placed_camera_sides();
        let (mr, ml) = if r.field_camera() || l.field_camera() {
            (r.field_camera(), l.field_camera())
        } else if r.rotatable_in_palm() || l.rotatable_in_palm() {
            (r.rotatable_in_palm(), l.rotatable_in_palm())
        } else if pr || pl || self.tripod.placed_sight().is_some() {
            self.tripod = self.tripod.map_mold(op);
            (pr, pl)
        } else if self.palms.empty_hands() && !self.tray.any_stowed_blocks() {
            (true, true)
        } else {
            (self.tray.active_side() > 0.0, self.tray.active_side() < 0.0)
        };
        self.palms.map_sides(attire, mr, ml, op);
        self
    }

    pub fn reconcile_attire(mut self, attire: YouthAttire) -> Self {
        self.palms.reconcile_attire(attire);
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
            if chosen.placeable_block() && other.placeable_block() && !chosen.field_camera() && !other.field_camera() {
                state.palms.sync_mold_from_opposite(side);
            } else if !chosen.placeable_block() || chosen.field_camera() || state.palms.mold_on(side).placed_camera() {
                state.palms.reset_mold(side);
            }
            state.palms.assign_blocks(side, u32::from(chosen.placeable_block()));
            state.tray.focus_side(side);
            state.tray.close();
            return (state, attire.with_hand_relic(side, chosen), true);
        }
        if hand.gripped() {
            state.tray.close();
            state.tray.stow_relic(hand, state.palms.blocks_on(side));
            state.palms.assign_blocks(side, 0);
            if hand.field_camera() { state.palms.reset_mold(side); }
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
        let placed_mold = self.palms.mold_on(side).in_medium(hand.block_medium());
        if !world.place_molded_surface_block(eye, view_dir, avatar_pos, placed_mold) {
            if hand != HeldRelic::SteeringWheel && hand != HeldRelic::CantileverCrane {
                self.tray.flag_steering_notice();
            }
            return (self, attire, false);
        }
        self.tray.close();
        if hand.field_camera() {
            self.tripod = world.placed_camera_tripod().map_or(
                TripodMount::Unmounted,
                |(hub, base_fwd, up)| TripodMount::Mounted {
                    hub,
                    base_fwd,
                    up,
                    turn: placed_mold.camera_turn(),
                },
            );
        }
        self.palms.assign_mold(side, placed_mold);
        self.tray.focus_side(side);
        let has_refill = !hand.field_camera() && self.tray.take_stowed(hand);
        let next_hand = if has_refill { hand } else { HeldRelic::EmptyPalm };
        self.palms.assign_blocks(side, u32::from(has_refill));
        (self, attire.with_hand_relic(side, next_hand), true)
    }

    pub fn collect_excavated_block(mut self, side: f32, relic: HeldRelic, attire: YouthAttire) -> (Self, YouthAttire) {
        if relic == HeldRelic::FieldCamera {
            self.tripod = TripodMount::Unmounted;
            self.palms.clear_placed_cameras();
        }
        if relic == HeldRelic::CantileverCrane {
            for (part, qty) in [
                (HeldRelic::FieldCamera, 1),
                (HeldRelic::RocketEngine, 1),
                (HeldRelic::AircraftEngine, 1),
                (HeldRelic::SteeringWheel, 1),
                (HeldRelic::CarWheel, 8),
                (HeldRelic::CrossAxle, 4),
                (HeldRelic::CabinSeat, 2),
                (HeldRelic::PedalCluster, 2),
            ] {
                if self.resource_tally(part, attire) == 0 {
                    self.tray.stow_relic(part, qty);
                }
            }
        }
        let (hand, other) = attire.hand_pair(side);
        if hand.free_palm() || other.free_palm() {
            let (target, opp) = if hand.free_palm() { (side, other) } else { (-side, hand) };
            if opp == relic { self.palms.sync_mold_from_opposite(target); } else { self.palms.reset_mold(target); }
            self.palms.assign_blocks(target, 1);
            self.tray.focus_side(target);
            return (self, attire.with_hand_relic(target, relic));
        }
        let target = if hand.placeable_block() { side } else if other.placeable_block() { -side } else { 0.0 };
        let held = if target > 0.0 { attire.held_pair().0 } else { attire.held_pair().1 };
        if target != 0.0 && held != relic {
            self.tray.stow_relic(held, self.palms.blocks_on(target));
            self.palms.assign_blocks(target, 1);
            self.palms.reset_mold(target);
            self.tray.focus_side(target);
            return (self, attire.with_hand_relic(target, relic));
        }
        self.tray.focus_side(if target != 0.0 { target } else { side });
        self.tray.stow_relic(relic, 1);
        (self, attire)
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
            self.palms.assign_blocks(1.0, 1);
            self.tray.stow_relic(HeldRelic::TimberBlock, 2);
            attire.with_relics(HeldRelic::TimberBlock, l)
        } else if l.free_palm() {
            self.palms.assign_blocks(-1.0, 1);
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
                        self.palms.assign_blocks(1.0, 0);
                        attire = attire.with_relics(HeldRelic::EmptyPalm, l);
                    } else if l == ing {
                        self.palms.assign_blocks(-1.0, 0);
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
        hand_relic.figure(self.palms.mold_on(side)).emit_in_palm(side, frame, target);
    }

    pub fn emit_on_waist(&self, pose: &BodyPose, target: &mut dyn MeshTarget) {
        self.tray.emit_on_waist(pose, target);
    }
}
