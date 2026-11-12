use glam::Vec3;
use world::{CleaveOutcome, PlanetaryWorld, SurfaceFind};
use crate::attire::{BeltSatchel, HeldRelic, StrideRhythm, YouthAttire};
use crate::grip::{DigGrip, DigStroke};
use super::debris::{DebrisSplash, DigOperation};

const EXCAVATION_THRESHOLD: u32 = 8;

/// Encapsulated 3D eye-to-surface sightline and stance eligibility for hand contact operations.
#[derive(Clone, Copy, Debug)]
pub struct SurfaceReach {
    eye: Vec3,
    view_dir: Vec3,
    stance_origin: Vec3,
    perch_eligible: bool,
}

impl SurfaceReach {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(eye: Vec3, view_dir: Vec3, stance_origin: Vec3, perch_eligible: bool) -> Self {
        Self { eye, view_dir, stance_origin, perch_eligible }
    }

    pub const fn eye(self) -> Vec3 { self.eye }
    pub const fn view_dir(self) -> Vec3 { self.view_dir }

    pub fn swing_rhythm(attire: YouthAttire, rhythm: StrideRhythm, side: f32) -> StrideRhythm {
        let (right, left) = attire.held_pair();
        if right.free_palm() && left.free_palm() {
            return rhythm.trigger_right_strike().trigger_left_strike();
        }
        let (active, other) = if side > 0.0 { (right, left) } else { (left, right) };
        if active.gripped() || active.free_palm() {
            if side > 0.0 { rhythm.trigger_right_strike() } else { rhythm.trigger_left_strike() }
        } else if other.free_palm() {
            if side > 0.0 { rhythm.trigger_left_strike() } else { rhythm.trigger_right_strike() }
        } else {
            rhythm
        }
    }

    pub fn release(
        self, satchel: BeltSatchel, attire: YouthAttire, dig: DigOperation, world: &mut dyn PlanetaryWorld,
    ) -> (BeltSatchel, YouthAttire, DigOperation, bool, bool) {
        if dig.ready_to_perch() {
            dig.clear_cracks(self.eye, self.view_dir, world);
            return (satchel, attire, dig.with_grip(DigGrip::idle()), true, false);
        }
        if dig.ready_to_place(DebrisSplash::overhead_aim(self.eye, self.view_dir, world)) {
            dig.clear_cracks(self.eye, self.view_dir, world);
            let (ns, na, placed) = satchel.place_from_hand(dig.grip().pending_place_side(), attire, self.eye, self.view_dir, self.stance_origin, world);
            return (ns, na, dig.with_grip(DigGrip::idle()), false, placed);
        }
        let cleared = dig.clear_cracks(self.eye, self.view_dir, world);
        (satchel, attire, dig.halted(), false, cleared)
    }

    pub fn sustain(
        self, satchel: BeltSatchel, attire: YouthAttire, rhythm: StrideRhythm, dig: DigOperation, world: &mut dyn PlanetaryWorld,
    ) -> (BeltSatchel, YouthAttire, DigOperation, StrideRhythm, bool) {
        if !dig.grip().any_held() {
            return (satchel, attire, dig, rhythm, false);
        }
        let side = dig.grip().active_side();
        if dig.grip().pending_click_on(side) && dig.untouched() {
            if rhythm.cleave_surge(side) > 0.0 && rhythm.cleave_arc(side) < 0.0 {
                return (satchel, attire, dig, rhythm, false);
            }
            let next_grip = dig.grip().promoted_to_dig(side);
            let reconciled = satchel.reconcile_attire(attire);
            if rhythm.cleave_surge(side) > 0.0 {
                let active = attire.hand_pair(side).0;
                let step: u32 = if active.spade_blade() { 4 } else { 1 };
                if let Some(med) = world.loosen_surface(self.eye, self.view_dir, step.div_ceil(2).clamp(1, 3) as u8) {
                    let site = dig.resolve_site(self.eye, self.view_dir, world);
                    return (reconciled, attire, dig.advanced_stroke(step, next_grip, site, med, &active), rhythm, true);
                }
                return (reconciled, attire, dig.with_grip(next_grip), rhythm, false);
            }
            let promoted = dig.promoted(next_grip);
            let follow = Self::new(self.eye, self.view_dir, self.stance_origin, false);
            let (ns, na, nd, nr, excavated) = follow.strike(side, reconciled, attire, rhythm, promoted, world);
            return (ns, na, nd, nr, excavated || nd.scarred());
        }
        if rhythm.cleave_surge(1.0) > 0.0 || rhythm.cleave_surge(-1.0) > 0.0 {
            return (satchel, attire, dig, rhythm, false);
        }
        let follow = Self::new(self.eye, self.view_dir, self.stance_origin, false);
        let (ns, na, nd, nr, excavated) = follow.strike(side, satchel, attire, rhythm, dig, world);
        (ns, na, nd, nr, excavated || nd.progressed_from(dig))
    }

    pub fn strike(
        self, side: f32, satchel: BeltSatchel, attire: YouthAttire, rhythm: StrideRhythm, dig: DigOperation, world: &mut dyn PlanetaryWorld,
    ) -> (BeltSatchel, YouthAttire, DigOperation, StrideRhythm, bool) {
        let (reconciled, find) = (satchel.reconcile_attire(attire), world.forage_surface(self.eye, self.view_dir));
        if find.gathered() {
            let gathered = if matches!(find, SurfaceFind::DeadwoodBough) { HeldRelic::DeadwoodBough } else { HeldRelic::FlintStone };
            let (ns, na) = reconciled.collect_foraged(side, gathered, attire);
            return (ns, na, dig.with_grip(DigGrip::idle()), rhythm, true);
        }
        let (right, left) = attire.held_pair();
        let (active, other) = if side > 0.0 { (right, left) } else { (left, right) };
        if dig.untouched() && !dig.grip().any_held() {
            if let Some(med) = world.loosen_surface(self.eye, self.view_dir, 1) {
                if med.instant_pickup() && world.harvest_surface(self.eye, self.view_dir) == Some(med) {
                    let (ns, na) = reconciled.collect_excavated_block(side, HeldRelic::from_medium(med), attire);
                    return (ns, na, dig.with_grip(DigGrip::idle()), rhythm, true);
                }
                world.loosen_surface(self.eye, self.view_dir, 0);
            }
        }
        if dig.untouched() && !dig.grip().any_held() && active == HeldRelic::WoodenMallet && world.adjust_rig(self.eye, self.view_dir, 1, false) {
            return (reconciled, attire, dig.with_grip(DigGrip::idle()), Self::swing_rhythm(attire, rhythm, side), true);
        }
        if dig.untouched() && !dig.grip().any_held() && active != HeldRelic::WoodenMallet && world.lower_rig(self.eye, self.view_dir) {
            return (reconciled, attire, dig.with_grip(DigGrip::idle()), Self::swing_rhythm(attire, rhythm, side), true);
        }
        if self.perch_eligible && !active.blocks_perch() && !dig.grip().held_on(side) && !dig.grip().pending_click_any() && dig.untouched() && world.board_vehicle(self.eye, self.view_dir, false).is_some() {
            let nr = if side > 0.0 { rhythm.trigger_right_strike() } else { rhythm.trigger_left_strike() };
            return (reconciled, attire, dig.with_grip(DigGrip::with_pending_perch(side)), nr, false);
        }
        let weightless = DebrisSplash::overhead_aim(self.eye, self.view_dir, world);
        if active.placeable_block() && !weightless && !dig.grip().held_on(side) && !dig.grip().pending_click_any() && dig.untouched() {
            let nr = if side > 0.0 { rhythm.trigger_right_strike() } else { rhythm.trigger_left_strike() };
            return (reconciled, attire, dig.with_grip(DigGrip::with_pending_place(side)), nr, false);
        }
        let deferred = u32::from(dig.grip().pending_click_on(side));
        let w_place = weightless && active.placeable_block() && !dig.grip().held_on(side) && dig.untouched();
        let both_dig = (right.free_palm() || right.placeable_block()) && (left.free_palm() || left.placeable_block());
        let stroke = if active.spade_blade() {
            DigStroke::spade(side, other.free_palm() || other.placeable_block(), rhythm)
        } else if both_dig && dig.grip().combined_both_held(side) {
            DigStroke::dual_palm(side, dig.grip(), active.free_palm() || !other.free_palm(), weightless, w_place, rhythm)
        } else if active.free_palm() || active.placeable_block() {
            let outcome = if other.axe_blade() { world.strike_timber(self.eye, self.view_dir) } else { CleaveOutcome::Missed };
            if outcome.struck() {
                let (ns, na) = if outcome.felled() { reconciled.collect_felled_timber(side, attire) } else { (reconciled, attire) };
                return (ns, na, dig.with_grip(DigGrip::with_dig_on(side, false)), Self::swing_rhythm(attire, rhythm, -side), true);
            }
            DigStroke::single_palm(side, deferred, weightless, w_place, rhythm)
        } else {
            let outcome = if active.axe_blade() { world.strike_timber(self.eye, self.view_dir) } else { CleaveOutcome::Missed };
            if outcome.struck() {
                let (ns, na) = if outcome.felled() { reconciled.collect_felled_timber(-side, attire) } else { (reconciled, attire) };
                let grip = if active.axe_blade() { DigGrip::with_dig_on(side, false) } else { DigGrip::idle() };
                return (ns, na, dig.with_grip(grip), Self::swing_rhythm(attire, rhythm, side), true);
            }
            DigStroke::single_palm(side, deferred, weightless, w_place, rhythm)
        };
        let (site, progress, next_grip, next_rhythm) = (dig.resolve_site(self.eye, self.view_dir, world), dig.step_progress(stroke), stroke.next_grip(), stroke.next_rhythm());
        if progress >= EXCAVATION_THRESHOLD {
            if let Some(med) = world.harvest_surface(self.eye, self.view_dir) {
                let (ns, na) = reconciled.collect_excavated_block(stroke.recipient_side(), HeldRelic::from_medium(med), attire);
                let harvested_dig = dig.advanced_stroke(0, next_grip.without_pending_place(), site, med, &active);
                return (ns, na, harvested_dig, next_rhythm, true);
            }
            return (reconciled, attire, dig.with_grip(next_grip.without_pending_place()), next_rhythm, false);
        }
        if let Some(med) = world.loosen_surface(self.eye, self.view_dir, progress.div_ceil(2).clamp(1, 3) as u8) {
            return (reconciled, attire, dig.loosen_stroke(progress, next_grip, site, med, &active), next_rhythm, false);
        }
        let fallback = if w_place { DigGrip::with_pending_place(side) } else { next_grip.without_pending_place() };
        (reconciled, attire, dig.with_grip(fallback), next_rhythm, false)
    }
}
