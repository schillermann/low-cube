mod intake;
mod palms;

use glam::Vec3;
use world::{BlockMold, MeshTarget, PlanetaryWorld};
use super::burden::{CarryLoad, JoinedLoad, SatchelBurden};
use super::limb::LimbFrame;
use super::pose::BodyPose;
use super::relic::HeldRelic;
use super::sight::{CameraMount, TripodMount};
use super::tray::{SatchelTray, TrayState};
use super::YouthAttire;
use self::intake::SatchelIntake;
use self::palms::PalmStack;

/// Shared satchel inventory with invisible stowed storage, WASD inspection grid, and dual-hand block molds.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BeltSatchel {
    palms: PalmStack,
    tray: SatchelTray,
    tripod: TripodMount,
}

impl BeltSatchel {
    /// Primary constructor doing nothing except assigning arguments to fields.
    const fn new(palms: PalmStack, tray: SatchelTray, tripod: TripodMount) -> Self {
        Self { palms, tray, tripod }
    }

    pub const fn empty() -> Self {
        Self::new(
            PalmStack::new(0, 0, BlockMold::cube(), BlockMold::cube()),
            SatchelTray::new([0; 22], 1.0, 0, TrayState::Closed),
            TripodMount::Unmounted,
        )
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

    pub fn carried_burden(&self, attire: YouthAttire) -> SatchelBurden {
        let (r, l) = attire.held_pair();
        JoinedLoad::new(JoinedLoad::new(r, l), self.tray).fold_burden(SatchelBurden::weightless())
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
            self.tray.focused_sides()
        };
        self.palms.map_sides(attire, mr, ml, op);
        self
    }

    pub fn turn_mold(mut self, attire: YouthAttire) -> Self {
        let (r, l) = attire.held_pair();
        self.palms.map_sides(attire, r.turnable_in_palm(), l.turnable_in_palm(), BlockMold::turn_clockwise);
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
        if state.tray.open() {
            let chosen = state.tray.selected_relic();
            if chosen.gripped() {
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
        } else if state.tray.open_if_populated(side) {
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
            return (self, attire, false);
        }
        self.tray.close();
        if hand.field_camera() {
            self.tripod = world.placed_camera_tripod().map_or(
                TripodMount::Unmounted,
                |(hub, base_fwd, up)| TripodMount::Mounted { hub, base_fwd, up, turn: placed_mold.camera_turn() },
            );
        }
        self.palms.assign_mold(side, placed_mold);
        self.tray.focus_side(side);
        let refilled = !hand.field_camera() && self.tray.take_stowed(hand);
        let next_hand = if refilled { hand } else { HeldRelic::EmptyPalm };
        self.palms.assign_blocks(side, u32::from(refilled));
        (self, attire.with_hand_relic(side, next_hand), true)
    }

    pub fn collect_excavated_block(mut self, side: f32, relic: HeldRelic, attire: YouthAttire) -> (Self, YouthAttire) {
        if relic == HeldRelic::FieldCamera {
            self.tripod = TripodMount::Unmounted;
        }
        let (palms, tray, next_attire) = SatchelIntake::new(self.palms, self.tray).excavated_block(side, relic, attire);
        (Self::new(palms, tray, self.tripod), next_attire)
    }

    pub fn collect_foraged(self, side: f32, relic: HeldRelic, attire: YouthAttire) -> (Self, YouthAttire) {
        let (palms, tray, next_attire) = SatchelIntake::new(self.palms, self.tray).foraged(side, relic, attire);
        (Self::new(palms, tray, self.tripod), next_attire)
    }

    pub fn collect_felled_timber(self, side: f32, attire: YouthAttire) -> (Self, YouthAttire) {
        let (palms, tray, next_attire) = SatchelIntake::new(self.palms, self.tray).felled_timber(side, attire);
        (Self::new(palms, tray, self.tripod), next_attire)
    }

    pub fn consume_and_equip(
        self, ingredients: [(HeldRelic, u32); 2], product: HeldRelic, batch_yield: u32, side: f32, attire: YouthAttire,
    ) -> (Self, YouthAttire) {
        let (palms, tray, next_attire) = SatchelIntake::new(self.palms, self.tray).crafted_batch(ingredients, product, batch_yield, side, attire);
        (Self::new(palms, tray, self.tripod), next_attire)
    }

    pub fn emit_in_palm(&self, side: f32, hand_relic: HeldRelic, frame: &LimbFrame, target: &mut dyn MeshTarget) {
        hand_relic.figure(self.palms.mold_on(side)).emit_in_palm(side, frame, target);
    }

    pub fn emit_for_attire(&self, pose: &BodyPose, attire: YouthAttire, target: &mut dyn MeshTarget) {
        self.tray.emit_with_burden(pose, self.carried_burden(attire), target);
    }
}
