mod actions;
mod debris;
mod reach;
mod scale_drag;
mod toss;

use glam::Vec3;
use world::{BlockMold, MeshTarget, PlanetaryWorld, PresenceAttire};
use crate::attire::{
    BeltSatchel, BodyPose, CameraMount, EvaAttire, MoldedWardrobe, SightMode,
    StrideRhythm, Wardrobe,
};
use crate::blueprint::BlueprintFolio;
use crate::controls::{ActionBindings, ControlAction, InputTrigger};
pub use crate::grip::DigGrip;
use crate::order::LocomotionOrder;
use self::actions::PalmContact;
use self::debris::DigOperation;
pub use self::reach::SurfaceReach;
pub use self::scale_drag::ScaleDrag;
use self::toss::TossCadence;

/// Immutable dual-hand EVA inventory, satchel inspection state, excavation progress, and drag-scaling state (`4` fields).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PalmEarthworks {
    satchel: BeltSatchel,
    dig: DigOperation,
    toss: TossCadence,
    drag: ScaleDrag,
}

impl PalmEarthworks {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn with_full_state(
        satchel: BeltSatchel,
        dig: DigOperation,
        toss: TossCadence,
        drag: ScaleDrag,
    ) -> Self {
        Self { satchel, dig, toss, drag }
    }

    pub const fn with_state(satchel: BeltSatchel, dig: DigOperation, toss: TossCadence) -> Self {
        Self::with_full_state(satchel, dig, toss, ScaleDrag::Idle)
    }

    pub const fn with_dig(satchel: BeltSatchel, dig: DigOperation) -> Self {
        Self::with_state(satchel, dig, TossCadence::Quiet)
    }

    pub const fn empty() -> Self {
        Self::with_dig(BeltSatchel::empty(), DigOperation::idle())
    }
    pub const fn satchel_open(&self) -> bool { self.satchel.open() }
    pub const fn camera_mount(&self, attire: EvaAttire) -> CameraMount {
        self.satchel.camera_mount(attire)
    }
    pub fn placed_camera_lens(&self, attire: EvaAttire) -> Option<(Vec3, Vec3, Vec3)> {
        self.satchel.placed_camera_lens(attire)
    }
    pub fn encumber_order(&self, attire: EvaAttire, order: LocomotionOrder) -> LocomotionOrder {
        order.with_burden(self.satchel.carried_burden(attire))
    }
    pub fn cargo_ballast(&self, attire: EvaAttire) -> f32 {
        self.satchel.carried_burden(attire).ballast_tonnage()
    }
    pub fn step_toss(self, dt: f32) -> Self { Self { toss: self.toss.advance(dt), ..self } }
    fn with_satchel(self, satchel: BeltSatchel) -> Self { Self { satchel, ..self } }
    pub fn mold_next_form(self, attire: EvaAttire) -> Self { self.with_satchel(self.satchel.map_mold(attire, BlockMold::next_form)) }
    pub fn mold_prev_form(self, attire: EvaAttire) -> Self { self.with_satchel(self.satchel.map_mold(attire, BlockMold::prev_form)) }
    pub fn reconcile_attire(self, attire: EvaAttire) -> Self { self.with_satchel(self.satchel.reconcile_attire(attire)) }
    pub fn transition_attire(self, prev: EvaAttire, next: EvaAttire) -> Self {
        self.with_satchel(self.satchel.transition_attire(prev, next))
    }
    pub fn with_provisioned_satchel(self, provider: &dyn crate::server_satchel::StartingSatchel) -> Self {
        self.with_satchel(provider.provision(self.satchel))
    }

    pub fn apply_survivor_manifest(
        self,
        manifest: world::SatchelManifest,
        attire: EvaAttire,
    ) -> (Self, EvaAttire) {
        let (next_satchel, next_attire) = BeltSatchel::from_survivor_manifest(manifest, attire);
        (Self::with_dig(next_satchel, DigOperation::idle()), next_attire)
    }

    pub fn survivor_manifest(&self, attire: EvaAttire) -> world::SatchelManifest {
        self.satchel.survivor_manifest(attire)
    }

    pub fn handle_trigger(
        self, trigger: InputTrigger, bindings: &ActionBindings, attire: EvaAttire,
    ) -> (Self, EvaAttire, bool) {
        if !self.satchel.open() && bindings.matches(ControlAction::HelmetLamp, trigger) {
            return (self, attire.with_toggled_lamp(), true);
        }
        if !self.satchel.open() && bindings.matches(ControlAction::MoldForm, trigger) {
            let (r, l) = attire.held_pair();
            if r.turnable_in_palm() || l.turnable_in_palm() {
                return (self.with_satchel(self.satchel.turn_mold(attire)), attire, true);
            }
        }
        if !self.satchel.open() && bindings.matches(ControlAction::MoldRight, trigger) {
            if let (next_drag, true) = self.drag.step_height(1) {
                return (Self { drag: next_drag, ..self }, attire, true);
            }
            return (self.mold_next_form(attire), attire, true);
        }
        if !self.satchel.open() && bindings.matches(ControlAction::MoldLeft, trigger) {
            if let (next_drag, true) = self.drag.step_height(-1) {
                return (Self { drag: next_drag, ..self }, attire, true);
            }
            return (self.mold_prev_form(attire), attire, true);
        }
        if bindings.matches(ControlAction::Satchel, trigger)
            || (self.satchel.open() && trigger == InputTrigger::Key(winit::keyboard::KeyCode::Escape))
        {
            let (next_satchel, ok) = self.satchel.reconcile_attire(attire).toggle_open();
            return (Self::with_dig(next_satchel, self.dig.with_grip(DigGrip::idle())), attire, ok);
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
            return (Self::with_state(next_satchel, self.dig.with_grip(DigGrip::idle()), next_toss), next_attire, ok);
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
        self, trigger: InputTrigger, bindings: &ActionBindings, attire: EvaAttire,
        rhythm: StrideRhythm, eye: Vec3, view_dir: Vec3, world: &mut dyn PlanetaryWorld,
    ) -> (Self, StrideRhythm, bool) {
        if self.satchel.open() {
            return (self, rhythm, false);
        }
        let (ns, nt, nr, ok) = self.toss.try_cast(self.satchel, trigger, bindings, attire, rhythm, eye, view_dir, world);
        let next_dig = if ok { self.dig.with_grip(DigGrip::idle()) } else { self.dig };
        (Self::with_state(ns, next_dig, nt), nr, ok)
    }

    pub fn try_adjust_rig(
        &self, trigger: InputTrigger, bindings: &ActionBindings, attire: EvaAttire, eye: Vec3, view_dir: Vec3, world: &mut dyn PlanetaryWorld,
    ) -> bool {
        let (r, l) = attire.held_pair();
        if self.satchel.open()
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
        &self, attire: &EvaAttire, pose: &BodyPose, sight: SightMode,
        inspection_open: bool, target: &mut dyn MeshTarget,
    ) {
        let active = self.dig.bare_hand_attire(*attire);
        MoldedWardrobe::new(active, self.satchel.wardrobe_molds(active), inspection_open).emit(pose, sight, target);
    }

    pub fn emit_inspection(&self, folio: &BlueprintFolio, attire: EvaAttire, pose: &BodyPose, target: &mut dyn MeshTarget) {
        self.satchel.emit_for_attire(pose, attire, target);
        folio.emit(&self.satchel, attire, pose, target);
    }

    pub fn presence_attire(&self, attire: EvaAttire) -> PresenceAttire {
        let active = self.dig.bare_hand_attire(attire);
        PresenceAttire::new(active.multiplayer_bytes(), active.region_bytes(), self.satchel.wardrobe_molds(active))
    }

    pub fn emit_debris(&self, view_dir: Vec3, surface_up: Vec3, rhythm: StrideRhythm, target: &mut dyn MeshTarget) {
        self.dig.emit_debris(view_dir, surface_up, rhythm, target);
    }

    pub fn planned_mount(&self, reach: SurfaceReach, attire: EvaAttire, world: &dyn PlanetaryWorld) -> Option<world::PlannedMount> {
        if self.satchel.open() || !self.dig.untouched() {
            return None;
        }
        let (_side, mold) = self.satchel.active_preview_mold(attire, self.dig.grip().pending_place_side())?;
        world.plan_molded_block(reach.eye(), reach.view_dir(), reach.stance_origin(), mold)
    }

    pub fn emit_preview(&self, reach: SurfaceReach, attire: EvaAttire, world: &dyn PlanetaryWorld, target: &mut dyn MeshTarget) {
        if self.satchel.open() {
            return;
        }
        if self.drag.emit_preview(reach, &self.satchel, attire, world, target) {
            return;
        }
        if let Some(planned) = self.planned_mount(reach, attire, world) {
            planned.emit_hologram(target);
        }
    }

    pub const fn halt_dig(self) -> Self {
        Self::with_state(self.satchel.with_open(false), self.dig.halted(), TossCadence::Quiet)
    }

    pub const fn relinquish_perished(self) -> Self {
        let next_dig = if self.dig.untouched() { self.dig.halted() } else { self.dig };
        Self::with_state(self.satchel.with_open(false), next_dig, TossCadence::Quiet)
    }

    pub fn act_folio(
        self, folio: &mut BlueprintFolio, attire: EvaAttire, world: &mut dyn PlanetaryWorld,
    ) -> (Self, EvaAttire, bool, bool) {
        PalmContact::new(self).act_folio(folio, attire, world)
    }

    pub fn try_build_folio(
        self, folio: &mut BlueprintFolio, trigger: InputTrigger, bindings: &ActionBindings,
        attire: EvaAttire, world: &mut dyn PlanetaryWorld,
    ) -> (Self, EvaAttire, bool) {
        PalmContact::new(self).try_build_folio(folio, trigger, bindings, attire, world)
    }

    pub fn abort_hand(self, reach: SurfaceReach, world: &mut dyn PlanetaryWorld) -> (Self, bool) {
        let cleared = self.dig.clear_cracks(reach.eye(), reach.view_dir(), world);
        (Self::with_state(self.satchel, self.dig.halted(), TossCadence::Quiet), cleared)
    }

    pub fn release_hand(
        self, reach: SurfaceReach, attire: EvaAttire, world: &mut dyn PlanetaryWorld,
    ) -> (Self, EvaAttire, bool, bool) {
        let state = self.reconcile_attire(attire);
        if let Some((ns, na, placed)) = state.drag.commit(reach, state.satchel, attire, world) {
            return (Self::with_state(ns, state.dig.with_grip(DigGrip::idle()), state.toss), na, false, placed);
        }
        let (ns, na, nd, perch_ready, changed) = reach.release(state.satchel, attire, state.dig, world);
        (Self::with_state(ns, nd, state.toss), na, perch_ready, changed)
    }

    pub fn continue_digging(
        self, reach: SurfaceReach, attire: EvaAttire, rhythm: StrideRhythm, world: &mut dyn PlanetaryWorld,
    ) -> (Self, EvaAttire, StrideRhythm, bool) {
        if self.drag.active() {
            return (self, attire, rhythm, false);
        }
        let (ns, na, nd, nr, changed) = reach.sustain(self.satchel, attire, rhythm, self.dig, world);
        (Self::with_state(ns, nd, self.toss), na, nr, changed)
    }

    pub fn act_with_hand(
        self, side: f32, reach: SurfaceReach, attire: EvaAttire, rhythm: StrideRhythm, world: &mut dyn PlanetaryWorld,
    ) -> (Self, EvaAttire, StrideRhythm, bool) {
        PalmContact::new(self).act_with_hand(side, reach, attire, rhythm, world)
    }
}
