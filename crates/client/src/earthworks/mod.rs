mod debris;

use glam::Vec3;
use world::{BlockMedium, BlockMold, MeshTarget, PlanetaryWorld, SurfaceFind};
use crate::attire::{
    ArtisanTome, BeltSatchel, BodyPose, CameraMount, HeldRelic, SatchelWardrobe,
    SightMode, StrideRhythm, Wardrobe, YouthAttire,
};
use crate::cardiac::{AcousticWave, PalmScrape, SpadeStrike};
use crate::controls::{ActionBindings, ControlAction, InputTrigger};
pub use crate::grip::{DigGrip, DigStroke};
use self::debris::{DebrisSplash, DigOperation};

const EXCAVATION_THRESHOLD: u32 = 8;

/// Immutable dual-hand block inventory, satchel inspection state, crafting tome, per-hand block molds, and excavation progress.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PalmEarthworks {
    satchel: BeltSatchel,
    tome: ArtisanTome,
    dig: DigOperation,
}

impl PalmEarthworks {
    pub const fn with_tome_and_medium(
        satchel: BeltSatchel, tome: ArtisanTome, dig_progress: u32, grip: DigGrip, dig_site: Vec3, medium: BlockMedium, splash_active: bool,
    ) -> Self {
        Self { satchel, tome, dig: DigOperation::new(dig_progress, grip, DebrisSplash::new(dig_site, medium, splash_active)) }
    }

    pub const fn with_tome(
        satchel: BeltSatchel, tome: ArtisanTome, dig_progress: u32, grip: DigGrip, dig_site: Vec3, splash_active: bool,
    ) -> Self {
        Self::with_tome_and_medium(satchel, tome, dig_progress, grip, dig_site, BlockMedium::TurfEarth, splash_active)
    }

    pub const fn new(
        satchel: BeltSatchel, dig_progress: u32, grip: DigGrip, dig_site: Vec3, splash_active: bool,
    ) -> Self {
        Self::with_tome(satchel, ArtisanTome::closed(), dig_progress, grip, dig_site, splash_active)
    }

    pub const fn empty() -> Self { Self::new(BeltSatchel::empty(), 0, DigGrip::idle(), Vec3::ZERO, false) }
    pub const fn satchel_open(&self) -> bool { self.satchel.open() || self.tome.open() }
    pub const fn camera_mount(&self, attire: YouthAttire) -> CameraMount {
        if self.tome.open() { CameraMount::Unmounted } else { self.satchel.camera_mount(attire) }
    }
    pub fn placed_camera_lens(&self, attire: YouthAttire) -> Option<(Vec3, Vec3, Vec3)> {
        self.satchel.placed_camera_lens(attire)
    }
    fn with_satchel(self, satchel: BeltSatchel) -> Self { Self { satchel, ..self } }
    pub fn mold_next_form(self, attire: YouthAttire) -> Self { self.with_satchel(self.satchel.map_mold(attire, BlockMold::next_form)) }
    pub fn mold_prev_form(self, attire: YouthAttire) -> Self { self.with_satchel(self.satchel.map_mold(attire, BlockMold::prev_form)) }
    pub fn reconcile_attire(self, attire: YouthAttire) -> Self { self.with_satchel(self.satchel.reconcile_attire(attire)) }
    pub fn with_provisioned_satchel(self, provider: &dyn crate::server_satchel::StartingSatchel) -> Self {
        self.with_satchel(provider.provision(self.satchel))
    }

    pub fn handle_trigger(
        self, trigger: InputTrigger, bindings: &ActionBindings, attire: YouthAttire,
    ) -> (Self, YouthAttire, bool) {
        if bindings.matches(ControlAction::CraftingBook, trigger)
            || (self.tome.open() && trigger == InputTrigger::Key(winit::keyboard::KeyCode::Escape))
        {
            let next_satchel = self.satchel.reconcile_attire(attire).with_open(false);
            return (Self::with_tome(next_satchel, self.tome.toggle(), 0, DigGrip::idle(), Vec3::ZERO, false), attire, true);
        }
        if self.tome.open() {
            let (next_tome, next_satchel, next_attire, ok) = self.tome.dispatch_trigger(trigger, bindings, self.satchel, attire);
            return (Self::with_tome(next_satchel, next_tome, 0, DigGrip::idle(), Vec3::ZERO, false), next_attire, ok);
        }
        if !self.satchel.open() && bindings.matches(ControlAction::MoldForm, trigger) {
            let (r, l) = attire.held_pair();
            if r.turnable_in_palm() || l.turnable_in_palm() {
                return (self.with_satchel(self.satchel.turn_mold(attire)), attire, true);
            }
        }
        if !self.satchel.open() && bindings.matches(ControlAction::MoldRight, trigger) {
            return (self.mold_next_form(attire), attire, true);
        }
        if !self.satchel.open() && bindings.matches(ControlAction::MoldLeft, trigger) {
            return (self.mold_prev_form(attire), attire, true);
        }
        if bindings.matches(ControlAction::Satchel, trigger)
            || (self.satchel.open() && trigger == InputTrigger::Key(winit::keyboard::KeyCode::Escape))
        {
            let (next_satchel, ok) = self.satchel.reconcile_attire(attire).toggle_open();
            return (Self::new(next_satchel, 0, DigGrip::idle(), Vec3::ZERO, false), attire, ok);
        }
        if bindings.matches(ControlAction::SatchelRight, trigger) || bindings.matches(ControlAction::SatchelLeft, trigger) {
            let side = if bindings.matches(ControlAction::SatchelRight, trigger) { 1.0 } else { -1.0 };
            let (next_satchel, next_attire, ok) = self.satchel.transfer_hand(side, attire);
            return (Self::new(next_satchel, 0, DigGrip::idle(), Vec3::ZERO, false), next_attire, ok);
        }
        let dx = if bindings.matches(ControlAction::MoveLeft, trigger) || bindings.matches(ControlAction::MoldLeft, trigger) { -1 } else if bindings.matches(ControlAction::MoveRight, trigger) || bindings.matches(ControlAction::MoldRight, trigger) { 1 } else { 0 };
        let dy = if bindings.matches(ControlAction::MoveForward, trigger) { -1 } else if bindings.matches(ControlAction::MoveBackward, trigger) { 1 } else { 0 };
        if dx != 0 || dy != 0 {
            let (next_satchel, ok) = self.satchel.navigate(dx, dy);
            return (self.with_satchel(next_satchel), attire, ok);
        }
        (self, attire, false)
    }

    pub fn try_adjust_rig(
        &self, trigger: InputTrigger, bindings: &ActionBindings, attire: YouthAttire, eye: Vec3, view_dir: Vec3, world: &mut dyn PlanetaryWorld,
    ) -> bool {
        let (r, l) = attire.held_pair();
        if self.satchel.open()
            || self.tome.open()
            || r.rotatable_in_palm()
            || l.rotatable_in_palm()
            || matches!(self.camera_mount(attire), CameraMount::Handheld(_))
        {
            return false;
        }
        if bindings.matches(ControlAction::MoldRight, trigger) {
            return world.adjust_rig(eye, view_dir, 1, false);
        }
        if bindings.matches(ControlAction::MoldLeft, trigger) {
            return world.adjust_rig(eye, view_dir, -1, false);
        }
        if bindings.matches(ControlAction::MoldForm, trigger) && matches!(self.camera_mount(attire), CameraMount::Unmounted) {
            return world.adjust_rig(eye, view_dir, 0, true);
        }
        false
    }

    pub fn emit_attire(&self, attire: &YouthAttire, pose: &BodyPose, sight: SightMode, target: &mut dyn MeshTarget) {
        let (dig_r, dig_l) = self.dig.bare_hand_digging(attire);
        SatchelWardrobe::new(*attire, self.satchel, dig_r, dig_l).emit(pose, sight, target);
        self.tome.emit_book(&self.satchel, *attire, pose, target);
    }

    pub fn trigger_hand_swing(attire: YouthAttire, rhythm: StrideRhythm, side: f32) -> StrideRhythm {
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

    pub fn emit_debris(&self, view_dir: Vec3, surface_up: Vec3, rhythm: StrideRhythm, target: &mut dyn MeshTarget) {
        self.dig.emit_debris(view_dir, surface_up, rhythm, target);
    }

    pub fn release_hand(
        self, attire: YouthAttire, eye: Vec3, view_dir: Vec3, avatar_pos: Vec3, world: &mut dyn PlanetaryWorld,
    ) -> (Self, YouthAttire, bool) {
        let state = self.reconcile_attire(attire);
        let d = state.dig;
        if d.ready_to_place(DebrisSplash::overhead_aim(eye, view_dir, world)) {
            d.clear_cracks(eye, view_dir, world);
            let (next_satchel, next_attire, placed) = state.satchel.place_from_hand(d.grip().pending_place_side(), attire, eye, view_dir, avatar_pos, world);
            return (Self::new(next_satchel, 0, DigGrip::idle(), Vec3::ZERO, false), next_attire, placed);
        }
        let cleared = d.clear_cracks(eye, view_dir, world);
        (Self { satchel: state.satchel, tome: state.tome, dig: DigOperation::new(0, DigGrip::idle(), d.splash().halted()) }, attire, cleared)
    }

    pub fn continue_digging(
        self, attire: YouthAttire, rhythm: StrideRhythm, eye: Vec3, view_dir: Vec3, world: &mut dyn PlanetaryWorld,
    ) -> (Self, YouthAttire, StrideRhythm, bool) {
        let d = self.dig;
        if !d.grip().any_held() { return (self, attire, rhythm, false); }
        let side = d.grip().active_side();
        if d.grip().pending_place_on(side) && d.progress() == 0 {
            if rhythm.cleave_surge(side) > 0.0 && rhythm.cleave_arc(side) < 0.0 {
                return (self, attire, rhythm, false);
            }
            let promoted = DigOperation::new(d.progress(), d.grip().promoted_to_dig(side), d.splash());
            let state = Self { satchel: self.satchel, tome: self.tome, dig: promoted }.reconcile_attire(attire);
            if rhythm.cleave_surge(side) > 0.0 {
                if let Some(med) = world.loosen_surface(eye, view_dir, 1) {
                    PalmScrape::earthy().thump();
                    return (Self::with_tome_and_medium(state.satchel, state.tome, 1, state.dig.grip(), d.resolve_site(eye, view_dir, world), med, true), attire, rhythm, true);
                }
                return (Self::with_tome_and_medium(state.satchel, state.tome, 0, state.dig.grip(), Vec3::ZERO, BlockMedium::TurfEarth, false), attire, rhythm, false);
            }
            let (next, next_attire, next_rhythm, excavated) = state.act_with_hand(side, attire, rhythm, eye, view_dir, world);
            return (next, next_attire, next_rhythm, excavated || next.dig.progress() > 0);
        }
        if rhythm.cleave_surge(1.0) > 0.0 || rhythm.cleave_surge(-1.0) > 0.0 {
            return (self, attire, rhythm, false);
        }
        let prev = d.progress();
        let (next, next_attire, next_rhythm, excavated) = self.act_with_hand(side, attire, rhythm, eye, view_dir, world);
        (next, next_attire, next_rhythm, excavated || next.dig.progress() != prev)
    }

    pub fn act_with_hand(
        self, side: f32, attire: YouthAttire, rhythm: StrideRhythm, eye: Vec3, view_dir: Vec3, world: &mut dyn PlanetaryWorld,
    ) -> (Self, YouthAttire, StrideRhythm, bool) {
        if self.satchel.open() {
            let (next_satchel, next_attire, _) = self.satchel.transfer_hand(side, attire);
            return (Self::new(next_satchel, 0, DigGrip::idle(), Vec3::ZERO, false), next_attire, rhythm, false);
        }
        if self.tome.open() {
            let (next_tome, next_satchel, next_attire, ok) = self.tome.craft_selected(side, self.satchel, attire);
            let next_rhythm = if ok { Self::trigger_hand_swing(next_attire, rhythm, side) } else { rhythm };
            return (Self::with_tome(next_satchel, next_tome, 0, DigGrip::idle(), Vec3::ZERO, false), next_attire, next_rhythm, false);
        }
        let (state, find) = (self.reconcile_attire(attire), world.forage_surface(eye, view_dir));
        let d = state.dig;
        if find.gathered() {
            let gathered = if matches!(find, SurfaceFind::DeadwoodBough) { HeldRelic::DeadwoodBough } else { HeldRelic::FlintStone };
            let (next_satchel, next_attire) = state.satchel.collect_foraged(side, gathered, attire);
            return (Self::new(next_satchel, 0, DigGrip::idle(), Vec3::ZERO, false), next_attire, rhythm, true);
        }
        let (right, left) = attire.held_pair();
        let (active, other) = if side > 0.0 { (right, left) } else { (left, right) };
        if d.progress() == 0 && !d.grip().any_held() {
            if let Some(med) = world.loosen_surface(eye, view_dir, 1) {
                if med.instant_pickup() && world.harvest_surface(eye, view_dir) == Some(med) {
                    let (ns, na) = state.satchel.collect_excavated_block(side, HeldRelic::from_medium(med), attire);
                    return (Self::new(ns, 0, DigGrip::idle(), Vec3::ZERO, false), na, rhythm, true);
                }
                world.loosen_surface(eye, view_dir, 0);
            }
        }
        if d.progress() == 0 && !d.grip().any_held() && active != HeldRelic::WoodenMallet && world.lower_rig(eye, view_dir) {
            return (Self::new(state.satchel, 0, DigGrip::idle(), Vec3::ZERO, false), attire, Self::trigger_hand_swing(attire, rhythm, side), true);
        }
        let weightless = DebrisSplash::overhead_aim(eye, view_dir, world);
        if active.placeable_block() && !weightless && !d.grip().held_on(side) && !d.grip().pending_place_any() && d.progress() == 0 {
            let next_rhythm = if side > 0.0 { rhythm.trigger_right_strike() } else { rhythm.trigger_left_strike() };
            return (Self::new(state.satchel, 0, DigGrip::with_pending_place(side), Vec3::ZERO, false), attire, next_rhythm, false);
        }
        let deferred = u32::from(d.grip().pending_place_on(side));
        let w_place = weightless && active.placeable_block() && !d.grip().held_on(side) && d.progress() == 0;
        let both_dig = (right.free_palm() || right.placeable_block()) && (left.free_palm() || left.placeable_block());
        let stroke = if active.spade_blade() {
            DigStroke::spade(side, other.free_palm() || other.placeable_block(), rhythm)
        } else if both_dig && d.grip().combined_both_held(side) {
            DigStroke::dual_palm(side, d.grip(), active.free_palm() || !other.free_palm(), weightless, w_place, rhythm)
        } else if active.free_palm() || active.placeable_block() {
            let outcome = if other.axe_blade() { world.strike_timber(eye, view_dir) } else { world::CleaveOutcome::Missed };
            if outcome.struck() {
                let (ns, na) = if outcome.felled() { state.satchel.collect_felled_timber(side, attire) } else { (state.satchel, attire) };
                return (Self::new(ns, 0, DigGrip::with_dig_on(side, false), Vec3::ZERO, false), na, Self::trigger_hand_swing(attire, rhythm, -side), true);
            }
            DigStroke::single_palm(side, deferred, weightless, w_place, rhythm)
        } else {
            if active == HeldRelic::WoodenMallet && world.adjust_rig(eye, view_dir, 1, false) {
                return (Self::new(state.satchel, 0, DigGrip::idle(), Vec3::ZERO, false), attire, Self::trigger_hand_swing(attire, rhythm, side), true);
            }
            let outcome = if active.axe_blade() { world.strike_timber(eye, view_dir) } else { world::CleaveOutcome::Missed };
            if outcome.struck() {
                let (ns, na) = if outcome.felled() { state.satchel.collect_felled_timber(-side, attire) } else { (state.satchel, attire) };
                let grip = if active.axe_blade() { DigGrip::with_dig_on(side, false) } else { DigGrip::idle() };
                return (Self::new(ns, 0, grip, Vec3::ZERO, false), na, Self::trigger_hand_swing(attire, rhythm, side), true);
            }
            DigStroke::single_palm(side, deferred, weightless, w_place, rhythm)
        };
        let (site, progress, next_grip, next_rhythm) = (d.resolve_site(eye, view_dir, world), stroke.advance_progress(d.progress()), stroke.next_grip(), stroke.next_rhythm());
        if progress >= EXCAVATION_THRESHOLD {
            if let Some(med) = world.harvest_surface(eye, view_dir) {
                if active.spade_blade() { SpadeStrike::forged().thump(); } else { PalmScrape::earthy().thump(); }
                let harvested = HeldRelic::from_medium(med);
                let (next_satchel, next_attire) = state.satchel.collect_excavated_block(stroke.recipient_side(), harvested, attire);
                return (Self::with_tome_and_medium(next_satchel, state.tome, 0, next_grip.without_pending_place(), site, med, true), next_attire, next_rhythm, true);
            }
            return (Self::new(state.satchel, 0, next_grip.without_pending_place(), Vec3::ZERO, false), attire, next_rhythm, false);
        }
        if let Some(med) = world.loosen_surface(eye, view_dir, progress.div_ceil(2).clamp(1, 3) as u8) {
            if progress > d.progress() {
                if active.spade_blade() { SpadeStrike::forged().thump(); } else { PalmScrape::earthy().thump(); }
            }
            return (Self::with_tome_and_medium(state.satchel, state.tome, progress, next_grip, site, med, true), attire, next_rhythm, false);
        }
        let fallback_grip = if w_place { DigGrip::with_pending_place(side) } else { next_grip.without_pending_place() };
        (Self::new(state.satchel, 0, fallback_grip, Vec3::ZERO, false), attire, next_rhythm, false)
    }
}
