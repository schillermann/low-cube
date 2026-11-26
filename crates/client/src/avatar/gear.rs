use glam::Vec3;
use world::{MeshTarget, PlanetaryWorld, PresenceAttire};
use crate::attire::{BodyPose, CameraMount, HeldRelic, SightMode, StrideRhythm, YouthAttire};
use crate::blueprint::BlueprintFolio;
use crate::controls::{ActionBindings, InputTrigger};
use crate::earthworks::{PalmEarthworks, SurfaceReach};
use crate::envelope::BodyEnvelope;
use crate::order::LocomotionOrder;
use crate::server_satchel::StartingSatchel;
use super::bearing::AvatarBearing;
use super::body::AvatarBody;
use super::sight::AvatarSight;

/// Encapsulated wardrobe attire, dual-hand earthworks, blueprint folio, and input bindings.
pub struct AvatarGear {
    attire: YouthAttire,
    earthworks: PalmEarthworks,
    folio: BlueprintFolio,
    bindings: ActionBindings,
}

impl AvatarGear {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn with_folio(
        attire: YouthAttire,
        earthworks: PalmEarthworks,
        folio: BlueprintFolio,
        bindings: ActionBindings,
    ) -> Self {
        Self { attire, earthworks, folio, bindings }
    }

    pub fn new(attire: YouthAttire, earthworks: PalmEarthworks, bindings: ActionBindings) -> Self {
        Self::with_folio(attire, earthworks, BlueprintFolio::profile(), bindings)
    }

    #[cfg(test)]
    pub fn install_blueprint_archive(&mut self, archive: Box<dyn crate::blueprint::BlueprintArchive>) {
        self.folio = BlueprintFolio::with_archive(archive);
    }

    pub const fn bindings(&self) -> &ActionBindings {
        &self.bindings
    }

    pub fn satchel_open(&self) -> bool {
        self.earthworks.satchel_open() || self.folio.open()
    }

    pub const fn typing_text(&self) -> bool {
        self.folio.typing_name()
    }

    pub fn accept_text(&mut self, text: &str) -> bool {
        self.folio.append_text(text)
    }

    fn active_camera_mount(&self) -> CameraMount {
        if self.folio.open() {
            CameraMount::Unmounted
        } else {
            self.earthworks.camera_mount(self.attire)
        }
    }

    pub fn placed_camera_lens(&self) -> Option<(Vec3, Vec3, Vec3)> {
        if self.folio.open() {
            None
        } else {
            self.earthworks.placed_camera_lens(self.attire)
        }
    }

    pub fn standing_envelope(&self, facing: Vec3) -> BodyEnvelope {
        self.attire.body_envelope().with_crouch_and_facing(0.0, facing)
    }

    pub fn encumber_order(&self, order: LocomotionOrder) -> LocomotionOrder {
        self.earthworks.encumber_order(self.attire, order)
    }

    pub fn cargo_ballast(&self) -> f32 { self.earthworks.cargo_ballast(self.attire) }

    pub fn shaft_yaw_play(&self) -> f32 {
        self.attire.shaft_yaw_play()
    }

    pub fn presence_attire(&self) -> PresenceAttire {
        self.earthworks.presence_attire(self.attire)
    }

    pub fn apply_attire(&mut self, attire: YouthAttire) {
        let (r, l) = self.attire.held_pair();
        let merged = if !attire.armed() && self.attire.armed() { attire.with_relics(r, l) } else { attire };
        (self.attire, self.earthworks) = (merged, self.earthworks.transition_attire(self.attire, merged));
    }

    pub fn apply_bindings(&mut self, bindings: ActionBindings) { self.bindings = bindings; }
    pub fn provision_satchel(&mut self, provider: &dyn StartingSatchel) {
        self.earthworks = self.earthworks.with_provisioned_satchel(provider);
    }
    pub fn apply_survivor_manifest(&mut self, manifest: world::SatchelManifest) {
        (self.earthworks, self.attire) = self.earthworks.apply_survivor_manifest(manifest, self.attire);
        self.folio.close();
    }
    pub fn drain_crafted_notes(&mut self, world: &mut dyn PlanetaryWorld) {
        self.earthworks = self.earthworks.drain_crafted_notes(world);
    }
    pub fn step_timers(&mut self, dt: f32) { self.earthworks = self.earthworks.step_toss(dt); }

    pub fn calm_hands(&mut self) {
        self.earthworks = self.earthworks.halt_dig();
        self.folio.close();
    }

    pub fn relinquish_perished(&mut self) {
        self.earthworks = self.earthworks.relinquish_perished();
        self.folio.close();
    }

    pub fn abort_hands(&mut self, reach: SurfaceReach, world: &mut dyn PlanetaryWorld) -> bool {
        let (next_ew, cleared) = self.earthworks.abort_hand(reach, world);
        self.earthworks = next_ew;
        cleared
    }

    pub fn dispatch_trigger(&mut self, trigger: InputTrigger, bearing: &mut AvatarBearing, perished: bool) -> bool {
        if perished {
            return false;
        }
        if self.folio.handle_trigger(trigger, &self.bindings) {
            bearing.reconcile_mount(self.active_camera_mount());
            return true;
        }
        if trigger == InputTrigger::WheelClick && !self.satchel_open() {
            let (r, l) = self.attire.held_pair();
            if r == HeldRelic::BlackBoxPod || l == HeldRelic::BlackBoxPod {
                bearing.trigger_log_sheet(1800, 1);
                return true;
            }
            if bearing.press_shutter(self.active_camera_mount()) {
                return true;
            }
        }
        let (ew, attire, handled) = self.earthworks.handle_trigger(trigger, &self.bindings, self.attire);
        if handled {
            (self.earthworks, self.attire) = (ew, attire);
            if self.earthworks.satchel_open() {
                self.folio.close();
            }
            bearing.reconcile_mount(self.active_camera_mount());
        }
        handled
    }

    pub fn try_cast_relic(
        &mut self, trigger: InputTrigger, bearing: &mut AvatarBearing,
        eye: Vec3, view_dir: Vec3, world: &mut dyn PlanetaryWorld,
    ) -> bool {
        if self.folio.open() {
            return false;
        }
        let (next_ew, next_rhythm, cast) =
            self.earthworks.try_cast_relic(trigger, &self.bindings, self.attire, bearing.rhythm(), eye, view_dir, world);
        self.earthworks = next_ew;
        if cast {
            bearing.apply_rhythm(next_rhythm);
            bearing.reconcile_mount(self.active_camera_mount());
        }
        cast
    }

    pub fn try_adjust_rig(&mut self, trigger: InputTrigger, bearing: &mut AvatarBearing, eye: Vec3, view_dir: Vec3, world: &mut dyn PlanetaryWorld) -> bool {
        if self.folio.open() {
            let (next_ew, next_attire, built) =
                self.earthworks.try_build_folio(&mut self.folio, trigger, &self.bindings, self.attire, world);
            (self.earthworks, self.attire) = (next_ew, next_attire);
            if built {
                bearing.apply_rhythm(SurfaceReach::swing_rhythm(self.attire, bearing.rhythm(), 1.0));
                bearing.reconcile_mount(self.active_camera_mount());
            }
            return built;
        }
        let adjusted = self.earthworks.try_adjust_rig(trigger, &self.bindings, self.attire, eye, view_dir, world);
        if adjusted && matches!(self.active_camera_mount(), CameraMount::Placed(_)) {
            let (ew, attire, _) = self.earthworks.handle_trigger(trigger, &self.bindings, self.attire);
            (self.earthworks, self.attire) = (ew, attire);
            bearing.reconcile_mount(self.active_camera_mount());
        }
        adjusted
    }

    pub fn act_with_hand(
        &mut self,
        side: f32,
        reach: SurfaceReach,
        body: &mut AvatarBody,
        bearing: &mut AvatarBearing,
        world: &mut dyn PlanetaryWorld,
    ) -> bool {
        if self.folio.open() {
            let (next_ew, next_attire, built, _) = self.earthworks.act_folio(&mut self.folio, self.attire, world);
            (self.earthworks, self.attire) = (next_ew, next_attire);
            if built {
                bearing.apply_rhythm(SurfaceReach::swing_rhythm(self.attire, bearing.rhythm(), side));
            }
            bearing.reconcile_mount(self.active_camera_mount());
            return built;
        }
        if !self.earthworks.satchel_open() && self.attire.hand_pair(side).0 == HeldRelic::VehicleBlueprint {
            if let Some(captured) = world.capture_vehicle_blueprint(reach.eye(), reach.view_dir()) {
                self.earthworks = self.earthworks.halt_dig();
                self.folio.open_naming(captured);
                bearing.apply_rhythm(SurfaceReach::swing_rhythm(self.attire, bearing.rhythm(), side));
                bearing.reconcile_mount(self.active_camera_mount());
                return false;
            }
            if world.aimed_empty_rig(reach.eye(), reach.view_dir(), reach.stance_origin()) {
                self.earthworks = self.earthworks.halt_dig();
                self.folio.open_rig_catalog(reach.eye(), reach.view_dir());
                bearing.apply_rhythm(SurfaceReach::swing_rhythm(self.attire, bearing.rhythm(), side));
                bearing.reconcile_mount(self.active_camera_mount());
                return false;
            }
        }
        let (next_ew, next_attire, next_rhythm, changed, nourished) =
            self.earthworks.act_with_hand(side, reach, self.attire, bearing.rhythm(), world);
        (self.earthworks, self.attire) = (next_ew, next_attire);
        if nourished {
            use acoustics::{AcousticWave, HeartyBite};
            body.nourish();
            HeartyBite::savory().thump();
        }
        bearing.apply_rhythm(next_rhythm);
        bearing.reconcile_mount(self.active_camera_mount());
        if !changed && !nourished && self.attire.hand_pair(side).0 == HeldRelic::BlackBoxPod {
            bearing.trigger_log_sheet(1800, 1);
            return true;
        }
        changed || nourished
    }

    pub fn strike_fauna(
        &mut self, side: f32, reach: SurfaceReach, bearing: &mut AvatarBearing, world: &mut dyn PlanetaryWorld,
    ) -> (bool, bool) {
        let (weapon, swing_side) = self.attire.hunting_hand(side);
        let (struck, dropped) =
            world.strike_boar(reach.eye(), reach.view_dir(), weapon.hunting_power(), weapon.hunting_reach());
        if struck {
            bearing.apply_rhythm(SurfaceReach::swing_rhythm(self.attire, bearing.rhythm(), swing_side));
            (self.earthworks, self.attire) = self.earthworks.wear_hand(swing_side, 1, self.attire, world);
        }
        (struck, dropped)
    }

    pub fn release_hand(
        &mut self, reach: SurfaceReach, body: &mut AvatarBody, sight_line: &mut AvatarSight, bearing: &mut AvatarBearing, world: &mut dyn PlanetaryWorld,
    ) -> bool {
        let (next_ew, next_attire, perch_ready, changed) =
            self.earthworks.release_hand(reach, self.attire, world);
        (self.earthworks, self.attire) = (next_ew, next_attire);
        bearing.reconcile_mount(self.active_camera_mount());
        if perch_ready && body.perch_vehicle(reach.eye(), reach.view_dir(), sight_line, world) {
            bearing.calm();
            return true;
        }
        changed
    }

    pub fn continue_digging(&mut self, reach: SurfaceReach, rhythm: StrideRhythm, world: &mut dyn PlanetaryWorld) -> (StrideRhythm, bool) {
        let (next_ew, next_attire, next_rhythm, changed) =
            self.earthworks.continue_digging(reach, self.attire, rhythm, world);
        (self.earthworks, self.attire) = (next_ew, next_attire);
        (next_rhythm, changed)
    }

    pub fn emit_appearance(&self, pose: &BodyPose, sight: SightMode, view_dir: Vec3, up: Vec3, rhythm: StrideRhythm, target: &mut dyn MeshTarget) {
        self.earthworks.emit_attire(&self.attire, pose, sight, self.satchel_open(), target);
        self.earthworks.emit_debris(view_dir, up, rhythm, target);
    }

    pub fn emit_inspection(&self, pose: &BodyPose, target: &mut dyn MeshTarget) {
        self.earthworks.emit_inspection(&self.folio, self.attire, pose, target);
    }

    pub fn planned_mount(&self, reach: SurfaceReach, world: &dyn PlanetaryWorld) -> Option<world::PlannedMount> {
        (!self.folio.open()).then(|| self.earthworks.planned_mount(reach, self.attire, world)).flatten()
    }

    pub fn emit_intercom(&self, aspect: f32, head_angles: (f32, f32), target: &mut dyn MeshTarget) -> glam::Mat4 {
        crate::attire::IntercomBust::new(self.attire, head_angles).emit(aspect, target)
    }
}
