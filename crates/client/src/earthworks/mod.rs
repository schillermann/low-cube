mod debris;
mod reach;
mod toss;

use glam::Vec3;
use world::{BlockMold, MeshTarget, PlanetaryWorld};
use crate::attire::{
    ArtisanTome, BeltSatchel, BodyPose, CameraMount, HeldRelic, SatchelBurden, SatchelWardrobe,
    SightMode, StrideRhythm, Wardrobe, YouthAttire,
};
use crate::controls::{ActionBindings, ControlAction, InputTrigger};
pub use crate::grip::DigGrip;
use self::debris::DigOperation;
pub use self::reach::SurfaceReach;
use self::toss::TossCadence;

/// Immutable dual-hand block inventory, satchel inspection state, crafting tome, per-hand block molds, and excavation progress.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PalmEarthworks {
    satchel: BeltSatchel,
    tome: ArtisanTome,
    dig: DigOperation,
    toss: TossCadence,
}

impl PalmEarthworks {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn with_state(
        satchel: BeltSatchel,
        tome: ArtisanTome,
        dig: DigOperation,
        toss: TossCadence,
    ) -> Self {
        Self { satchel, tome, dig, toss }
    }

    pub const fn with_dig(satchel: BeltSatchel, tome: ArtisanTome, dig: DigOperation) -> Self {
        Self::with_state(satchel, tome, dig, TossCadence::Quiet)
    }

    pub const fn empty() -> Self {
        Self::with_dig(BeltSatchel::empty(), ArtisanTome::closed(), DigOperation::idle())
    }
    pub const fn satchel_open(&self) -> bool { self.satchel.open() || self.tome.open() }
    pub const fn camera_mount(&self, attire: YouthAttire) -> CameraMount {
        if self.tome.open() { CameraMount::Unmounted } else { self.satchel.camera_mount(attire) }
    }
    pub fn placed_camera_lens(&self, attire: YouthAttire) -> Option<(Vec3, Vec3, Vec3)> {
        self.satchel.placed_camera_lens(attire)
    }
    pub fn carried_burden(&self, attire: YouthAttire) -> SatchelBurden {
        self.satchel.carried_burden(attire)
    }
    pub fn step_toss(self, dt: f32) -> Self { Self { toss: self.toss.advance(dt), ..self } }
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
            return (Self::with_dig(next_satchel, self.tome.toggle(), self.dig.with_grip(DigGrip::idle())), attire, true);
        }
        if self.tome.open() {
            let (next_tome, next_satchel, next_attire, ok) = self.tome.dispatch_trigger(trigger, bindings, self.satchel, attire);
            return (Self::with_dig(next_satchel, next_tome, self.dig.with_grip(DigGrip::idle())), next_attire, ok);
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
            return (Self::with_dig(next_satchel, self.tome, self.dig.with_grip(DigGrip::idle())), attire, ok);
        }
        if bindings.matches(ControlAction::SatchelRight, trigger) || bindings.matches(ControlAction::SatchelLeft, trigger) {
            let side = if bindings.matches(ControlAction::SatchelRight, trigger) { 1.0 } else { -1.0 };
            let (hand, _) = attire.hand_pair(side);
            let next_toss = if !self.satchel.open() && hand.gripped() {
                TossCadence::prime(side, hand)
            } else {
                TossCadence::Quiet
            };
            let (next_satchel, next_attire, ok) = self.satchel.transfer_hand(side, attire);
            return (Self::with_state(next_satchel, self.tome, self.dig.with_grip(DigGrip::idle()), next_toss), next_attire, ok);
        }
        let dx = if bindings.matches(ControlAction::MoveLeft, trigger) || bindings.matches(ControlAction::MoldLeft, trigger) { -1 } else if bindings.matches(ControlAction::MoveRight, trigger) || bindings.matches(ControlAction::MoldRight, trigger) { 1 } else { 0 };
        let dy = if bindings.matches(ControlAction::MoveForward, trigger) { -1 } else if bindings.matches(ControlAction::MoveBackward, trigger) { 1 } else { 0 };
        if dx != 0 || dy != 0 {
            let (next_satchel, ok) = self.satchel.navigate(dx, dy);
            return (self.with_satchel(next_satchel), attire, ok);
        }
        (self, attire, false)
    }

    pub fn try_cast_relic(
        self,
        trigger: InputTrigger,
        bindings: &ActionBindings,
        attire: YouthAttire,
        rhythm: StrideRhythm,
        eye: Vec3,
        view_dir: Vec3,
        world: &mut dyn PlanetaryWorld,
    ) -> (Self, StrideRhythm, bool) {
        if self.satchel.open() || self.tome.open() {
            return (self, rhythm, false);
        }
        let side = if bindings.matches(ControlAction::SatchelRight, trigger) {
            1.0
        } else if bindings.matches(ControlAction::SatchelLeft, trigger) {
            -1.0
        } else {
            return (self, rhythm, false);
        };
        let Some(relic) = self.toss.ready_on(side) else {
            return (self, rhythm, false);
        };
        let (next_satchel, taken) = self.satchel.take_cast_relic(relic);
        if !taken || !world.cast_surface_find(eye, view_dir, relic.surface_find()) {
            return (Self { toss: TossCadence::Quiet, ..self }, rhythm, false);
        }
        let next_rhythm = SurfaceReach::swing_rhythm(attire, rhythm, side);
        (
            Self::with_state(next_satchel, self.tome, self.dig.with_grip(DigGrip::idle()), TossCadence::Quiet),
            next_rhythm,
            true,
        )
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

    pub fn emit_debris(&self, view_dir: Vec3, surface_up: Vec3, rhythm: StrideRhythm, target: &mut dyn MeshTarget) {
        self.dig.emit_debris(view_dir, surface_up, rhythm, target);
    }

    pub fn release_hand(
        self, reach: SurfaceReach, attire: YouthAttire, world: &mut dyn PlanetaryWorld,
    ) -> (Self, YouthAttire, bool, bool) {
        let state = self.reconcile_attire(attire);
        let (ns, na, nd, perch_ready, changed) = reach.release(state.satchel, attire, state.dig, world);
        (Self::with_state(ns, state.tome, nd, state.toss), na, perch_ready, changed)
    }

    pub fn continue_digging(
        self, reach: SurfaceReach, attire: YouthAttire, rhythm: StrideRhythm, world: &mut dyn PlanetaryWorld,
    ) -> (Self, YouthAttire, StrideRhythm, bool) {
        let (ns, na, nd, nr, changed) = reach.sustain(self.satchel, attire, rhythm, self.dig, world);
        (Self::with_state(ns, self.tome, nd, self.toss), na, nr, changed)
    }

    pub fn act_with_hand(
        self, side: f32, reach: SurfaceReach, attire: YouthAttire, rhythm: StrideRhythm, world: &mut dyn PlanetaryWorld,
    ) -> (Self, YouthAttire, StrideRhythm, bool, bool) {
        if self.satchel.open() {
            let (next_satchel, next_attire, _) = self.satchel.transfer_hand(side, attire);
            return (Self::with_dig(next_satchel, self.tome, self.dig.with_grip(DigGrip::idle())), next_attire, rhythm, false, false);
        }
        if self.tome.open() {
            let (next_tome, next_satchel, next_attire, ok) = self.tome.craft_selected(side, self.satchel, attire);
            let next_rhythm = if ok { SurfaceReach::swing_rhythm(next_attire, rhythm, side) } else { rhythm };
            return (Self::with_dig(next_satchel, next_tome, self.dig.with_grip(DigGrip::idle())), next_attire, next_rhythm, false, false);
        }
        if attire.hand_pair(side).0.edible_provision() && self.dig.untouched() && !self.dig.grip().any_held() {
            let find = world.forage_surface(reach.eye(), reach.view_dir());
            if find.gathered() {
                let (ns, na) = self.satchel.reconcile_attire(attire).collect_foraged(side, HeldRelic::from_surface_find(find), attire);
                return (Self::with_dig(ns, self.tome, self.dig.with_grip(DigGrip::idle())), na, rhythm, true, false);
            }
            let (ns, na, ate) = self.satchel.reconcile_attire(attire).consume_provision(side, attire);
            if ate {
                let nr = SurfaceReach::swing_rhythm(attire, rhythm, side);
                return (Self::with_dig(ns, self.tome, self.dig.with_grip(DigGrip::idle())), na, nr, false, true);
            }
        }
        let (ns, na, nd, nr, changed) = reach.strike(side, self.satchel, attire, rhythm, self.dig, world);
        (Self::with_dig(ns, self.tome, nd), na, nr, changed, false)
    }
}
