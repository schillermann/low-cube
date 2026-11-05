use glam::Vec3;
use world::{MeshTarget, PlanetaryWorld};
use crate::attire::{BodyPose, SightMode, StrideRhythm, YouthAttire};
use crate::controls::{ActionBindings, InputTrigger};
use crate::earthworks::PalmEarthworks;
use crate::envelope::BodyEnvelope;

/// Encapsulated kinematic bearing: horizon tilt, stride rhythm, and sight mode.
pub struct AvatarBearing {
    tilt_radians: f32,
    rhythm: StrideRhythm,
    sight: SightMode,
}

impl AvatarBearing {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(tilt_radians: f32, rhythm: StrideRhythm, sight: SightMode) -> Self {
        Self { tilt_radians, rhythm, sight }
    }

    pub const fn rhythm(&self) -> StrideRhythm {
        self.rhythm
    }

    pub const fn sight(&self) -> SightMode {
        self.sight
    }

    pub const fn tilt_radians(&self) -> f32 {
        self.tilt_radians
    }

    pub fn calm(&mut self) {
        self.rhythm = StrideRhythm::idle();
    }

    pub fn reset(&mut self) {
        self.tilt_radians = 0.0;
        self.rhythm = StrideRhythm::idle();
    }

    pub fn apply_rhythm(&mut self, rhythm: StrideRhythm) {
        self.rhythm = rhythm;
    }

    pub fn tilt_perish(&mut self, dt: f32) {
        self.rhythm = StrideRhythm::idle();
        self.tilt_radians += (0.22 - self.tilt_radians) * (4.0 * dt).clamp(0.0, 1.0);
    }

    pub fn decay_tilt(&mut self, dt: f32) {
        self.tilt_radians *= 1.0 - (8.0 * dt).min(1.0);
    }

    pub fn advance_stride(&mut self, prev_pos: Vec3, next_pos: Vec3, terrain_up: Vec3, dt: f32) {
        self.rhythm = self.rhythm.advance(prev_pos, next_pos, terrain_up, dt);
    }
}

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

    pub fn blocks_hand_action(&self) -> bool {
        self.earthworks.satchel_open() && !self.earthworks.tome_open()
    }

    pub fn standing_envelope(&self, facing: Vec3) -> BodyEnvelope {
        self.attire.body_envelope().with_crouch_and_facing(0.0, facing)
    }

    pub fn shaft_yaw_play(&self) -> f32 {
        self.attire.shaft_yaw_play()
    }

    pub fn apply_attire(&mut self, attire: YouthAttire) {
        let (r, l) = self.attire.held_pair();
        let merged = if !attire.armed() && self.attire.armed() { attire.with_relics(r, l) } else { attire };
        (self.attire, self.earthworks) = (merged, self.earthworks.reconcile_attire(merged));
    }

    pub fn apply_bindings(&mut self, bindings: ActionBindings) {
        self.bindings = bindings;
    }

    pub fn seed_test_engines(&mut self) {
        self.earthworks = self.earthworks.with_test_engines();
    }

    pub fn dispatch_trigger(&mut self, trigger: InputTrigger) -> bool {
        let (ew, attire, handled) = self.earthworks.handle_trigger(trigger, &self.bindings, self.attire);
        if handled {
            (self.earthworks, self.attire) = (ew, attire);
        }
        handled
    }

    pub fn try_adjust_rig(&self, trigger: InputTrigger, eye: Vec3, view_dir: Vec3, world: &mut dyn PlanetaryWorld) -> bool {
        self.earthworks.try_adjust_rig(trigger, &self.bindings, eye, view_dir, world)
    }

    pub fn act_with_hand(&mut self, side: f32, rhythm: StrideRhythm, eye: Vec3, view_dir: Vec3, world: &mut dyn PlanetaryWorld) -> (StrideRhythm, bool) {
        let (next_ew, next_attire, next_rhythm, changed) =
            self.earthworks.act_with_hand(side, self.attire, rhythm, eye, view_dir, world);
        (self.earthworks, self.attire) = (next_ew, next_attire);
        (next_rhythm, changed)
    }

    pub fn release_hand(&mut self, eye: Vec3, dir: Vec3, position: Vec3, world: &mut dyn PlanetaryWorld) -> bool {
        let (next_ew, next_attire, changed) = self.earthworks.release_hand(self.attire, eye, dir, position, world);
        (self.earthworks, self.attire) = (next_ew, next_attire);
        changed
    }

    pub fn continue_digging(&mut self, rhythm: StrideRhythm, eye: Vec3, dir: Vec3, world: &mut dyn PlanetaryWorld) -> (StrideRhythm, bool) {
        let (next_ew, next_attire, next_rhythm, changed) =
            self.earthworks.continue_digging(self.attire, rhythm, eye, dir, world);
        (self.earthworks, self.attire) = (next_ew, next_attire);
        (next_rhythm, changed)
    }

    pub fn emit_appearance(&self, pose: &BodyPose, sight: SightMode, view_dir: Vec3, up: Vec3, rhythm: StrideRhythm, target: &mut dyn MeshTarget) {
        self.earthworks.emit_attire(&self.attire, pose, sight, target);
        self.earthworks.emit_debris(view_dir, up, rhythm, target);
    }
}
