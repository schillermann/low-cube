mod actions;
mod debris;
mod hearth;
mod reach;
mod toss;

use glam::Vec3;
use world::{BlockMold, MeshTarget, PlanetaryWorld, PresenceAttire};
use crate::attire::{
    ArtisanTome, BeltSatchel, BodyPose, CameraMount, MoldedWardrobe, SightMode,
    StrideRhythm, Wardrobe, YouthAttire,
};
use crate::blueprint::BlueprintFolio;
use crate::controls::{ActionBindings, ControlAction, InputTrigger};
pub use crate::grip::DigGrip;
use crate::order::LocomotionOrder;
use self::actions::PalmContact;
use self::debris::DigOperation;
pub use self::hearth::NourishedBody;
pub(crate) use self::hearth::{MealOutcome, WorkstationStation};
#[cfg(test)]
pub use self::hearth::{
    AtmosphericBloomery, ColdStone, CulinaryOven, HearthMedium, ThermalHearth,
    VacuumForge,
};
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
    pub fn encumber_order(&self, attire: YouthAttire, order: LocomotionOrder) -> LocomotionOrder {
        order.with_burden(self.satchel.carried_burden(attire))
    }
    pub fn cargo_ballast(&self, attire: YouthAttire) -> f32 {
        self.satchel.carried_burden(attire).ballast_tonnage()
    }
    pub fn step_toss(self, dt: f32) -> Self { Self { toss: self.toss.advance(dt), ..self } }
    fn with_satchel(self, satchel: BeltSatchel) -> Self { Self { satchel, ..self } }
    pub fn mold_next_form(self, attire: YouthAttire) -> Self { self.with_satchel(self.satchel.map_mold(attire, BlockMold::next_form)) }
    pub fn mold_prev_form(self, attire: YouthAttire) -> Self { self.with_satchel(self.satchel.map_mold(attire, BlockMold::prev_form)) }
    pub fn reconcile_attire(self, attire: YouthAttire) -> Self { self.with_satchel(self.satchel.reconcile_attire(attire)) }
    pub fn transition_attire(self, prev: YouthAttire, next: YouthAttire) -> Self {
        self.with_satchel(self.satchel.transition_attire(prev, next))
    }
    pub fn with_provisioned_satchel(self, provider: &dyn crate::server_satchel::StartingSatchel) -> Self {
        self.with_satchel(provider.provision(self.satchel))
    }

    pub fn apply_survivor_manifest(
        self,
        manifest: world::SatchelManifest,
        attire: YouthAttire,
    ) -> (Self, YouthAttire) {
        let (next_satchel, next_attire) = BeltSatchel::from_survivor_manifest(manifest, attire);
        (Self::with_dig(next_satchel, ArtisanTome::closed(), DigOperation::idle()), next_attire)
    }

    pub fn survivor_manifest(&self, attire: YouthAttire) -> world::SatchelManifest {
        self.satchel.survivor_manifest(attire)
    }

    pub fn drain_crafted_notes(mut self, world: &mut dyn PlanetaryWorld) -> Self {
        self.tome = self.tome.drain_crafted_notes(world);
        self
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
        self, trigger: InputTrigger, bindings: &ActionBindings, attire: YouthAttire,
        rhythm: StrideRhythm, eye: Vec3, view_dir: Vec3, world: &mut dyn PlanetaryWorld,
    ) -> (Self, StrideRhythm, bool) {
        if self.satchel.open() || self.tome.open() {
            return (self, rhythm, false);
        }
        let (ns, nt, nr, ok) = self.toss.try_cast(self.satchel, trigger, bindings, attire, rhythm, eye, view_dir, world);
        let next_dig = if ok { self.dig.with_grip(DigGrip::idle()) } else { self.dig };
        (Self::with_state(ns, self.tome, next_dig, nt), nr, ok)
    }

    pub fn wear_hand(
        self, side: f32, strain: u16, attire: YouthAttire, world: &mut dyn PlanetaryWorld,
    ) -> (Self, YouthAttire) {
        let (ns, na) = self.satchel.reconcile_attire(attire).strain_hand(side, strain, attire, world);
        (self.with_satchel(ns), na)
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

    pub fn emit_attire(
        &self, attire: &YouthAttire, pose: &BodyPose, sight: SightMode,
        inspection_open: bool, target: &mut dyn MeshTarget,
    ) {
        let active = self.dig.bare_hand_attire(*attire);
        MoldedWardrobe::new(active, self.satchel.wardrobe_molds(active), inspection_open).emit(pose, sight, target);
    }

    pub fn emit_inspection(&self, folio: &BlueprintFolio, attire: YouthAttire, pose: &BodyPose, target: &mut dyn MeshTarget) {
        self.satchel.emit_for_attire(pose, attire, target);
        self.tome.emit_book(&self.satchel, attire, pose, target);
        folio.emit(&self.satchel, attire, pose, target);
    }

    pub fn presence_attire(&self, attire: YouthAttire) -> PresenceAttire {
        let active = self.dig.bare_hand_attire(attire);
        PresenceAttire::new(active.multiplayer_bytes(), active.region_bytes(), self.satchel.wardrobe_molds(active))
    }

    pub fn emit_debris(&self, view_dir: Vec3, surface_up: Vec3, rhythm: StrideRhythm, target: &mut dyn MeshTarget) {
        self.dig.emit_debris(view_dir, surface_up, rhythm, target);
    }

    pub fn planned_mount(&self, reach: SurfaceReach, attire: YouthAttire, world: &dyn PlanetaryWorld) -> Option<world::PlannedMount> {
        if self.satchel.open() || self.tome.open() || !self.dig.untouched() {
            return None;
        }
        let (side, mold) = self.satchel.active_preview_mold(attire, self.dig.grip().pending_place_side())?;
        if let Some(aimed) = world.aimed_surface_medium(reach.eye(), reach.view_dir()) {
            if WorkstationStation::claims_hand(aimed, attire.hand_pair(side).0) {
                return None;
            }
        }
        world.plan_molded_block(reach.eye(), reach.view_dir(), reach.stance_origin(), mold)
    }

    pub const fn halt_dig(self) -> Self {
        Self::with_state(self.satchel.with_open(false), ArtisanTome::closed(), self.dig.halted(), TossCadence::Quiet)
    }

    pub const fn relinquish_perished(self) -> Self {
        let next_dig = if self.dig.untouched() { self.dig.halted() } else { self.dig };
        Self::with_state(self.satchel.with_open(false), ArtisanTome::closed(), next_dig, TossCadence::Quiet)
    }

    pub fn act_folio(
        self, folio: &mut BlueprintFolio, attire: YouthAttire, world: &mut dyn PlanetaryWorld,
    ) -> (Self, YouthAttire, bool, bool) {
        PalmContact::new(self).act_folio(folio, attire, world)
    }

    pub fn try_build_folio(
        self, folio: &mut BlueprintFolio, trigger: InputTrigger, bindings: &ActionBindings,
        attire: YouthAttire, world: &mut dyn PlanetaryWorld,
    ) -> (Self, YouthAttire, bool) {
        PalmContact::new(self).try_build_folio(folio, trigger, bindings, attire, world)
    }

    pub fn abort_hand(self, reach: SurfaceReach, world: &mut dyn PlanetaryWorld) -> (Self, bool) {
        let cleared = self.dig.clear_cracks(reach.eye(), reach.view_dir(), world);
        (Self::with_state(self.satchel, self.tome, self.dig.halted(), TossCadence::Quiet), cleared)
    }

    pub fn release_hand(
        self, reach: SurfaceReach, attire: YouthAttire, world: &mut dyn PlanetaryWorld,
    ) -> (Self, YouthAttire, bool, bool) {
        let state = self.reconcile_attire(attire);
        let (ns, na, nd, perch_ready, changed) = reach.release(state.satchel, attire, state.dig, world);
        (Self::with_state(ns, state.tome, nd, state.toss), na, perch_ready, changed)
    }

    pub fn continue_digging(
        mut self, reach: SurfaceReach, attire: YouthAttire, rhythm: StrideRhythm, world: &mut dyn PlanetaryWorld,
    ) -> (Self, YouthAttire, StrideRhythm, bool) {
        self.tome = self.tome.drain_crafted_notes(world);
        let (ns, na, nd, nr, changed) = reach.sustain(self.satchel, attire, rhythm, self.dig, world);
        (Self::with_state(ns, self.tome, nd, self.toss), na, nr, changed)
    }

    pub fn act_with_hand(
        self, side: f32, reach: SurfaceReach, attire: YouthAttire, rhythm: StrideRhythm, world: &mut dyn PlanetaryWorld,
    ) -> (Self, YouthAttire, StrideRhythm, bool, MealOutcome) {
        PalmContact::new(self).act_with_hand(side, reach, attire, rhythm, world)
    }
}
