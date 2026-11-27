mod intake;
mod palms;
mod temper;

use glam::Vec3;
use world::{BlockMold, MeshTarget, PalmMolds, PlanetaryWorld};
use super::burden::{CarryLoad, JoinedLoad, SatchelBurden};
use super::pose::BodyPose;
use super::relic::HeldRelic;
#[cfg(test)]
use super::relic::ToolCondition;
use super::sight::{CameraMount, TripodMount};
use super::tray::{SatchelTray, TrayState};
use super::EvaAttire;
use self::intake::SatchelIntake;
use self::palms::PalmStack;
use self::temper::SatchelTemper;

/// Shared EVA satchel inventory with stowed storage, WASD inspection grid, palm molds, and tool state (`4` fields).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BeltSatchel {
    palms: PalmStack,
    tray: SatchelTray,
    tripod: TripodMount,
    temper: SatchelTemper,
}

impl BeltSatchel {
    /// Primary constructor doing nothing except assigning arguments to fields.
    const fn new(palms: PalmStack, tray: SatchelTray, tripod: TripodMount, temper: SatchelTemper) -> Self {
        Self { palms, tray, tripod, temper }
    }

    pub const fn empty() -> Self {
        Self::new(
            PalmStack::new(0, 0, PalmMolds::cubes()),
            SatchelTray::new([0; HeldRelic::COUNT], 1.0, 0, TrayState::Closed),
            TripodMount::Unmounted,
            SatchelTemper::empty(),
        )
    }

    pub fn with_stowed_relic(self, relic: HeldRelic, count: u32) -> Self {
        if count == 0 {
            return self;
        }
        Self::new(self.palms, self.tray.stowed_with(relic, count), self.tripod, self.temper)
    }

    pub fn from_survivor_manifest(manifest: world::SatchelManifest, attire: EvaAttire) -> (Self, EvaAttire) {
        let empty_kit = Self::empty();
        let (p, t, tm, next_attire) = SatchelIntake::new(empty_kit.palms, empty_kit.tray, empty_kit.temper)
            .from_manifest(manifest, attire);
        (Self::new(p, t, empty_kit.tripod, tm), next_attire)
    }

    pub fn survivor_manifest(&self, attire: EvaAttire) -> world::SatchelManifest {
        SatchelIntake::new(self.palms, self.tray, self.temper).to_manifest(attire)
    }

    pub const fn open(&self) -> bool { self.tray.open() }
    pub const fn with_open(self, open: bool) -> Self { Self::new(self.palms, self.tray.with_open(open), self.tripod, self.temper) }
    #[cfg(test)]
    pub const fn palm_molds(&self) -> PalmMolds { self.palms.palm_molds() }
    pub fn wardrobe_molds(&self, attire: EvaAttire) -> PalmMolds {
        self.temper.wardrobe_molds(self.palms.palm_molds(), attire)
    }
    #[cfg(test)]
    pub const fn hand_condition(&self, side: f32, attire: EvaAttire) -> ToolCondition {
        self.temper.palm_condition(side, attire.hand_pair(side).0)
    }

    pub fn active_preview_mold(&self, attire: EvaAttire, pending_side: f32) -> Option<(f32, BlockMold)> {
        if self.tray.open() {
            return None;
        }
        if pending_side != 0.0 {
            let (hand, _) = attire.hand_pair(pending_side);
            if hand.placeable_block() {
                return Some((pending_side, self.palms.mold_on(pending_side).in_medium(hand.block_medium())));
            }
        }
        let (right, left) = attire.held_pair();
        if right.placeable_block() {
            Some((1.0, self.palms.mold_on(1.0).in_medium(right.block_medium())))
        } else if left.placeable_block() {
            Some((-1.0, self.palms.mold_on(-1.0).in_medium(left.block_medium())))
        } else {
            None
        }
    }

    pub const fn camera_mount(&self, attire: EvaAttire) -> CameraMount {
        let mount = self.palms.camera_mount(attire);
        if matches!(mount, CameraMount::Handheld(_)) {
            mount
        } else if let Some(sight) = self.tripod.placed_sight() {
            CameraMount::Placed(sight)
        } else {
            mount
        }
    }

    pub fn placed_camera_lens(&self, attire: EvaAttire) -> Option<(Vec3, Vec3, Vec3)> {
        let (r, l) = attire.held_pair();
        if r.field_camera() || l.field_camera() { None } else { self.tripod.lens_ray() }
    }

    pub fn resource_tally(&self, relic: HeldRelic, attire: EvaAttire) -> u32 {
        let (r, l) = attire.held_pair();
        u32::from(r == relic) + u32::from(l == relic) + self.tray.count(relic)
    }

    pub fn carried_burden(&self, attire: EvaAttire) -> SatchelBurden {
        let (r, l) = attire.held_pair();
        JoinedLoad::new(JoinedLoad::new(r, l), self.tray).fold_burden(SatchelBurden::weightless())
    }

    pub fn map_mold(mut self, attire: EvaAttire, op: fn(BlockMold) -> BlockMold) -> Self {
        let (r, l) = attire.held_pair();
        let (mr, ml) = if r.field_camera() || l.field_camera() {
            (r.field_camera(), l.field_camera())
        } else if r.rotatable_in_palm() || l.rotatable_in_palm() {
            (r.rotatable_in_palm(), l.rotatable_in_palm())
        } else if self.tripod.placed_sight().is_some() {
            self.tripod = self.tripod.map_mold(op);
            (false, false)
        } else if self.palms.empty_hands() && !self.tray.any_stowed_blocks() {
            (true, true)
        } else {
            self.tray.focused_sides()
        };
        self.palms.map_sides(attire, mr, ml, op);
        self
    }

    pub fn turn_mold(mut self, attire: EvaAttire) -> Self {
        let (r, l) = attire.held_pair();
        self.palms.map_sides(attire, r.turnable_in_palm(), l.turnable_in_palm(), BlockMold::turn_clockwise);
        self
    }

    pub fn reconcile_attire(mut self, attire: EvaAttire) -> Self {
        self.palms.reconcile_attire(attire);
        self.temper.reconcile_attire(attire);
        self
    }

    pub fn transition_attire(mut self, prev: EvaAttire, next: EvaAttire) -> Self {
        let ((pr, pl), (nr, nl)) = (prev.held_pair(), next.held_pair());
        if nr != pr && nr.independent_palm_mold() { self.palms.reset_mold(1.0); }
        if nl != pl && nl.independent_palm_mold() { self.palms.reset_mold(-1.0); }
        self.temper.transition_attire(prev, next);
        self.reconcile_attire(next)
    }

    pub fn toggle_open(self) -> (Self, bool) {
        (Self::new(self.palms, self.tray.toggled_open(), self.tripod, self.temper), true)
    }

    pub const fn navigate(self, dx: i32, dy: i32) -> (Self, bool) {
        let (next_tray, ok) = self.tray.navigated(dx, dy);
        (Self::new(self.palms, next_tray, self.tripod, self.temper), ok)
    }

    pub fn transfer_hand(self, side: f32, attire: EvaAttire) -> (Self, EvaAttire, bool) {
        let (mut state, (hand, other)) = (self.reconcile_attire(attire), attire.hand_pair(side));
        if state.tray.open() {
            let (chosen, chosen_rank) = state.tray.selected_slot();
            if chosen.gripped() {
                let drawn_wear = state.temper.take_stowed_wear(chosen, chosen_rank);
                let (mut next_tray, _) = state.tray.taken_stowed(chosen);
                if hand.gripped() {
                    state.temper.push_stowed_wear(hand, state.temper.palm_wear(side));
                    next_tray = next_tray.stowed_with(hand, state.palms.blocks_on(side));
                }
                state.temper.assign_palm_wear(side, drawn_wear);
                if chosen.placeable_block() && other.placeable_block() && !chosen.independent_palm_mold() && !other.independent_palm_mold() {
                    state.palms.sync_mold_from_opposite(side);
                } else if !chosen.placeable_block() || chosen.independent_palm_mold() {
                    state.palms.reset_mold(side);
                }
                state.palms.assign_blocks(side, u32::from(chosen.placeable_block()));
                state.tray = next_tray.focused_on(side).closed();
                return (state, attire.with_hand_relic(side, chosen), true);
            }
        }
        if hand.gripped() {
            state.temper.push_stowed_wear(hand, state.temper.palm_wear(side));
            state.temper.assign_palm_wear(side, 0);
            state.tray = state.tray.closed().stowed_with(hand, state.palms.blocks_on(side)).focused_on(side);
            state.palms.assign_blocks(side, 0);
            if hand.independent_palm_mold() { state.palms.reset_mold(side); }
            (state, attire.with_hand_relic(side, HeldRelic::EmptyPalm), true)
        } else if state.tray.open() {
            state.tray = state.tray.closed();
            (state, attire, true)
        } else {
            let (next_tray, opened) = state.tray.opened_if_populated(side);
            state.tray = next_tray;
            (state, attire, opened)
        }
    }

    pub fn place_from_hand(
        mut self, side: f32, attire: EvaAttire, eye: Vec3, view_dir: Vec3, avatar_pos: Vec3, world: &mut dyn PlanetaryWorld,
    ) -> (Self, EvaAttire, bool) {
        let (hand, _) = attire.hand_pair(side);
        let placed_mold = self.palms.mold_on(side).in_medium(hand.block_medium());
        if !world.place_molded_surface_block(eye, view_dir, avatar_pos, placed_mold) {
            return (self, attire, false);
        }
        if hand.field_camera() {
            self.tripod = world.placed_camera_tripod().map_or(
                TripodMount::Unmounted,
                |(hub, base_fwd, up)| TripodMount::Mounted { hub, base_fwd, up, turn: placed_mold.camera_turn() },
            );
            self.palms.reset_mold(side);
        } else {
            self.palms.assign_mold(side, placed_mold);
        }
        let focused = self.tray.closed().focused_on(side);
        let (next_tray, refilled) = if hand.field_camera() { (focused, false) } else { focused.taken_stowed(hand) };
        self.tray = next_tray;
        let next_hand = if refilled { hand } else { HeldRelic::EmptyPalm };
        self.palms.assign_blocks(side, u32::from(refilled));
        (self, attire.with_hand_relic(side, next_hand), true)
    }

    pub fn strain_hand(
        mut self, side: f32, strain: u16, attire: EvaAttire, world: &mut dyn PlanetaryWorld,
    ) -> (Self, EvaAttire) {
        let (hand, _) = attire.hand_pair(side);
        if self.temper.strain_palm(side, hand, strain) {
            world.note_consumed_provision(hand.code());
            self.palms.assign_blocks(side, 0);
            (self, attire.with_hand_relic(side, HeldRelic::EmptyPalm))
        } else {
            (self, attire)
        }
    }

    pub fn collect_excavated_block(mut self, side: f32, relic: HeldRelic, attire: EvaAttire) -> (Self, EvaAttire) {
        if relic == HeldRelic::FieldCamera { self.tripod = TripodMount::Unmounted; }
        let (p, t, tm, na) = SatchelIntake::new(self.palms, self.tray, self.temper).excavated_block(side, relic, attire);
        (Self::new(p, t, self.tripod, tm), na)
    }

    pub fn collect_foraged(mut self, side: f32, relic: HeldRelic, attire: EvaAttire) -> (Self, EvaAttire) {
        if relic == HeldRelic::FieldCamera { self.tripod = TripodMount::Unmounted; }
        let (p, t, tm, na) = SatchelIntake::new(self.palms, self.tray, self.temper).foraged(side, relic, attire);
        (Self::new(p, t, self.tripod, tm), na)
    }

    pub fn take_cast_relic(mut self, relic: HeldRelic) -> (Self, bool) {
        if !relic.gripped() { return (self, false); }
        let (next_tray, ok) = self.tray.taken_stowed(relic);
        if ok { let _ = self.temper.take_stowed_wear(relic, 0); }
        (Self::new(self.palms, next_tray, self.tripod, self.temper), ok)
    }

    pub fn collect_felled_timber(self, side: f32, attire: EvaAttire) -> (Self, EvaAttire) {
        let (p, t, tm, na) = SatchelIntake::new(self.palms, self.tray, self.temper).felled_timber(side, attire);
        (Self::new(p, t, self.tripod, tm), na)
    }

    #[cfg(test)]
    pub fn consume_and_equip(
        self, ingredients: [(HeldRelic, u32); 2], product: HeldRelic, batch_yield: u32, side: f32, attire: EvaAttire,
    ) -> (Self, EvaAttire) {
        let (p, t, tm, na) = SatchelIntake::new(self.palms, self.tray, self.temper).crafted_batch(ingredients, product, batch_yield, side, attire);
        (Self::new(p, t, self.tripod, tm), na)
    }

    pub fn consume_manifest(self, manifest: &[(HeldRelic, u32)], attire: EvaAttire) -> Option<(Self, EvaAttire)> {
        let state = self.reconcile_attire(attire);
        let (p, t, tm, na) = SatchelIntake::new(state.palms, state.tray, state.temper).consume_manifest(manifest, attire)?;
        Some((Self::new(p, t, state.tripod, tm), na))
    }

    pub fn emit_for_attire(&self, pose: &BodyPose, attire: EvaAttire, target: &mut dyn MeshTarget) {
        let scaled = attire.scaled_pose(pose);
        let temper = self.temper;
        self.tray.emit_with_burden(&scaled, self.carried_burden(attire), |item, rank| temper.stowed_condition(item, rank), target);
    }
}
