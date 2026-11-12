mod debris;
mod reach;

use glam::Vec3;
use world::{BlockMedium, BlockMold, MeshTarget, PlanetaryWorld};
use crate::attire::{
    ArtisanTome, BeltSatchel, BodyPose, CameraMount, SatchelWardrobe,
    SightMode, StrideRhythm, Wardrobe, YouthAttire,
};
use crate::controls::{ActionBindings, ControlAction, InputTrigger};
pub use crate::grip::DigGrip;
use self::debris::{DebrisSplash, DigOperation};
pub use self::reach::SurfaceReach;

/// Immutable dual-hand block inventory, satchel inspection state, crafting tome, per-hand block molds, and excavation progress.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PalmEarthworks {
    satchel: BeltSatchel,
    tome: ArtisanTome,
    dig: DigOperation,
}

impl PalmEarthworks {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn with_dig(satchel: BeltSatchel, tome: ArtisanTome, dig: DigOperation) -> Self {
        Self { satchel, tome, dig }
    }

    pub const fn with_tome_and_medium(
        satchel: BeltSatchel, tome: ArtisanTome, dig_progress: u32, grip: DigGrip, dig_site: Vec3, medium: BlockMedium, splash_active: bool,
    ) -> Self {
        Self::with_dig(satchel, tome, DigOperation::new(dig_progress, grip, DebrisSplash::new(dig_site, medium, splash_active)))
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

    pub fn emit_debris(&self, view_dir: Vec3, surface_up: Vec3, rhythm: StrideRhythm, target: &mut dyn MeshTarget) {
        self.dig.emit_debris(view_dir, surface_up, rhythm, target);
    }

    pub fn release_hand(
        self, reach: SurfaceReach, attire: YouthAttire, world: &mut dyn PlanetaryWorld,
    ) -> (Self, YouthAttire, bool, bool) {
        let state = self.reconcile_attire(attire);
        let (ns, na, nd, perch_ready, changed) = reach.release(state.satchel, attire, state.dig, world);
        (Self::with_dig(ns, state.tome, nd), na, perch_ready, changed)
    }

    pub fn continue_digging(
        self, reach: SurfaceReach, attire: YouthAttire, rhythm: StrideRhythm, world: &mut dyn PlanetaryWorld,
    ) -> (Self, YouthAttire, StrideRhythm, bool) {
        let (ns, na, nd, nr, changed) = reach.sustain(self.satchel, attire, rhythm, self.dig, world);
        (Self::with_dig(ns, self.tome, nd), na, nr, changed)
    }

    pub fn act_with_hand(
        self, side: f32, reach: SurfaceReach, attire: YouthAttire, rhythm: StrideRhythm, world: &mut dyn PlanetaryWorld,
    ) -> (Self, YouthAttire, StrideRhythm, bool) {
        if self.satchel.open() {
            let (next_satchel, next_attire, _) = self.satchel.transfer_hand(side, attire);
            return (Self::new(next_satchel, 0, DigGrip::idle(), Vec3::ZERO, false), next_attire, rhythm, false);
        }
        if self.tome.open() {
            let (next_tome, next_satchel, next_attire, ok) = self.tome.craft_selected(side, self.satchel, attire);
            let next_rhythm = if ok { SurfaceReach::swing_rhythm(next_attire, rhythm, side) } else { rhythm };
            return (Self::with_tome(next_satchel, next_tome, 0, DigGrip::idle(), Vec3::ZERO, false), next_attire, next_rhythm, false);
        }
        let (ns, na, nd, nr, changed) = reach.strike(side, self.satchel, attire, rhythm, self.dig, world);
        (Self::with_dig(ns, self.tome, nd), na, nr, changed)
    }
}
