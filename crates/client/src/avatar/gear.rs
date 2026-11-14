use glam::Vec3;
use world::{MeshTarget, PlanetaryWorld};
use crate::attire::{BodyPose, CameraMount, SatchelBurden, SightMode, StrideRhythm, YouthAttire};
use crate::controls::{ActionBindings, InputTrigger};
use crate::earthworks::{PalmEarthworks, SurfaceReach};
use crate::envelope::BodyEnvelope;
use fauna::WoodlandFauna;
use crate::server_satchel::StartingSatchel;
use super::bearing::AvatarBearing;
use super::body::AvatarBody;
use super::sight::AvatarSight;

/// Encapsulated wardrobe attire, dual-hand earthworks, and input bindings.
pub struct AvatarGear {
    attire: YouthAttire,
    earthworks: PalmEarthworks,
    bindings: ActionBindings,
}

impl AvatarGear {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(attire: YouthAttire, earthworks: PalmEarthworks, bindings: ActionBindings) -> Self {
        Self { attire, earthworks, bindings }
    }

    pub fn satchel_open(&self) -> bool {
        self.earthworks.satchel_open()
    }

    pub fn placed_camera_lens(&self) -> Option<(Vec3, Vec3, Vec3)> {
        self.earthworks.placed_camera_lens(self.attire)
    }

    pub fn standing_envelope(&self, facing: Vec3) -> BodyEnvelope {
        self.attire.body_envelope().with_crouch_and_facing(0.0, facing)
    }

    pub fn carried_burden(&self) -> SatchelBurden {
        self.earthworks.carried_burden(self.attire)
    }

    pub fn shaft_yaw_play(&self) -> f32 {
        self.attire.shaft_yaw_play()
    }

    pub fn attire_wire(&self) -> ([u8; 9], [u8; 4]) {
        (self.attire.multiplayer_bytes(), self.attire.region_bytes())
    }

    pub fn apply_attire(&mut self, attire: YouthAttire) {
        let (r, l) = self.attire.held_pair();
        let merged = if !attire.armed() && self.attire.armed() { attire.with_relics(r, l) } else { attire };
        (self.attire, self.earthworks) = (merged, self.earthworks.reconcile_attire(merged));
    }

    pub fn apply_bindings(&mut self, bindings: ActionBindings) {
        self.bindings = bindings;
    }

    pub fn provision_satchel(&mut self, provider: &dyn StartingSatchel) {
        self.earthworks = self.earthworks.with_provisioned_satchel(provider);
    }

    pub fn step_timers(&mut self, dt: f32) {
        self.earthworks = self.earthworks.step_toss(dt);
    }

    pub fn dispatch_trigger(&mut self, trigger: InputTrigger, bearing: &mut AvatarBearing, perished: bool) -> bool {
        if trigger == InputTrigger::WheelClick
            && !self.satchel_open()
            && !perished
            && bearing.press_shutter(self.earthworks.camera_mount(self.attire))
        {
            return true;
        }
        let (ew, attire, handled) = self.earthworks.handle_trigger(trigger, &self.bindings, self.attire);
        if handled {
            (self.earthworks, self.attire) = (ew, attire);
            bearing.reconcile_mount(self.earthworks.camera_mount(self.attire));
        }
        handled
    }

    pub fn try_cast_relic(
        &mut self,
        trigger: InputTrigger,
        bearing: &mut AvatarBearing,
        eye: Vec3,
        view_dir: Vec3,
        world: &mut dyn PlanetaryWorld,
    ) -> bool {
        let (next_ew, next_rhythm, cast) =
            self.earthworks.try_cast_relic(trigger, &self.bindings, self.attire, bearing.rhythm(), eye, view_dir, world);
        self.earthworks = next_ew;
        if cast {
            bearing.apply_rhythm(next_rhythm);
            bearing.reconcile_mount(self.earthworks.camera_mount(self.attire));
        }
        cast
    }

    pub fn try_adjust_rig(&mut self, trigger: InputTrigger, bearing: &mut AvatarBearing, eye: Vec3, view_dir: Vec3, world: &mut dyn PlanetaryWorld) -> bool {
        let adjusted = self.earthworks.try_adjust_rig(trigger, &self.bindings, self.attire, eye, view_dir, world);
        if adjusted && matches!(self.earthworks.camera_mount(self.attire), CameraMount::Placed(_)) {
            let (ew, attire, _) = self.earthworks.handle_trigger(trigger, &self.bindings, self.attire);
            (self.earthworks, self.attire) = (ew, attire);
            bearing.reconcile_mount(self.earthworks.camera_mount(self.attire));
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
        let (next_ew, next_attire, next_rhythm, changed, nourished) =
            self.earthworks.act_with_hand(side, reach, self.attire, bearing.rhythm(), world);
        (self.earthworks, self.attire) = (next_ew, next_attire);
        if nourished {
            use acoustics::{AcousticWave, HeartyBite};
            body.nourish();
            HeartyBite::savory().thump();
        }
        bearing.apply_rhythm(next_rhythm);
        bearing.reconcile_mount(self.earthworks.camera_mount(self.attire));
        changed || nourished
    }

    pub fn strike_fauna(
        &self,
        side: f32,
        reach: SurfaceReach,
        bearing: &mut AvatarBearing,
        fauna: &mut WoodlandFauna,
        world: &mut dyn PlanetaryWorld,
    ) -> (bool, bool) {
        let (active, other) = self.attire.hand_pair(side);
        let (weapon, swing_side) = if active.hunting_power() > 1 {
            (active, side)
        } else if active.free_palm() && other.hunting_power() > 1 {
            (other, -side)
        } else {
            (active, side)
        };
        let (struck, dropped) = fauna.strike_boar(
            reach.eye(),
            reach.view_dir(),
            weapon.hunting_power(),
            weapon.hunting_reach(),
            world,
        );
        if struck {
            bearing.apply_rhythm(SurfaceReach::swing_rhythm(self.attire, bearing.rhythm(), swing_side));
        }
        (struck, dropped)
    }

    pub fn release_hand(
        &mut self,
        reach: SurfaceReach,
        body: &mut AvatarBody,
        sight_line: &mut AvatarSight,
        bearing: &mut AvatarBearing,
        world: &mut dyn PlanetaryWorld,
    ) -> bool {
        let (next_ew, next_attire, perch_ready, changed) =
            self.earthworks.release_hand(reach, self.attire, world);
        (self.earthworks, self.attire) = (next_ew, next_attire);
        bearing.reconcile_mount(self.earthworks.camera_mount(self.attire));
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
        self.earthworks.emit_attire(&self.attire, pose, sight, target);
        self.earthworks.emit_debris(view_dir, up, rhythm, target);
    }
}
