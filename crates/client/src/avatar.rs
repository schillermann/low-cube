use glam::{Mat4, Quat, Vec3};
#[cfg(test)]
use winit::keyboard::KeyCode;
use world::{MeshTarget, PlanetaryWorld};
use crate::attire::{BodyPose, SightMode, StrideRhythm, YouthAttire};
use crate::controls::{ActionBindings, InputTrigger, KeyboardControls};
use crate::earthworks::PalmEarthworks;
use crate::flight::GravitationalFlight;
#[cfg(test)]
use crate::flight::SurfaceFlight;
use crate::heading::SurfaceHeading;
use crate::horizon::{CameraHorizon, HorizonTilt};
use crate::lens::{PerspectiveLens, ProjectionLens};
use crate::menu::ScreenPoint;
use crate::order::LocomotionOrder;
use crate::orientation::{MouseLook, SensitivityLook, ViewOrientation};
use crate::posture::AvatarPosture;

/// Interface for player avatars acting within planetary gravity.
pub trait Avatar {
    fn rotate_view(&mut self, dx: f32, dy: f32);
    fn use_left_hand(&mut self, world: &mut dyn PlanetaryWorld) -> bool;
    fn use_right_hand(&mut self, world: &mut dyn PlanetaryWorld) -> bool;
    fn release_hand(&mut self, _world: &mut dyn PlanetaryWorld) -> bool { false }
    fn step_digging(&mut self, _world: &mut dyn PlanetaryWorld) -> bool { false }
    fn apply_attire(&mut self, attire: YouthAttire);
    fn apply_bindings(&mut self, _bindings: ActionBindings) {}
    fn dispatch_trigger(&mut self, _trigger: InputTrigger) -> bool { false }
    #[cfg(test)]
    fn dispatch_wardrobe_key(&mut self, code: KeyCode) -> bool {
        self.dispatch_trigger(InputTrigger::Key(code))
    }
    fn step_motion(&mut self, dt: f32, controls: &dyn KeyboardControls, world: &dyn PlanetaryWorld);
    fn view_projection_matrix(&self, aspect: f32) -> Mat4;
    fn surface_rotation(&self) -> Quat;
    fn emit_appearance(&self, target: &mut dyn MeshTarget);
    fn emit_reticle(&self, target: &mut dyn MeshTarget) { ScreenPoint::center().emit_crosshair(target); }
}

/// Fully encapsulated player entity composed of position, gravity alignment, look, lens, flight, posture, medieval attire, and bindings.
pub struct PlanetaryAvatar {
    position: Vec3, surface_rot: Quat, camera_rot: Quat, look: SensitivityLook, lens: PerspectiveLens,
    flight: Box<dyn GravitationalFlight>, posture: AvatarPosture, tilt_radians: f32,
    rhythm: StrideRhythm, attire: YouthAttire, sight: SightMode, earthworks: PalmEarthworks,
    bindings: ActionBindings,
}

impl PlanetaryAvatar {
    /// Primary constructor doing nothing except assigning arguments to fields.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        position: Vec3, surface_rot: Quat, camera_rot: Quat, look: SensitivityLook, lens: PerspectiveLens,
        flight: Box<dyn GravitationalFlight>, posture: AvatarPosture, tilt_radians: f32,
        rhythm: StrideRhythm, attire: YouthAttire, sight: SightMode, earthworks: PalmEarthworks,
        bindings: ActionBindings,
    ) -> Self {
        Self { position, surface_rot, camera_rot, look, lens, flight, posture, tilt_radians, rhythm, attire, sight, earthworks, bindings }
    }

    pub fn with_attire_and_bindings(initial_position: Vec3, flight: Box<dyn GravitationalFlight>, attire: YouthAttire, bindings: ActionBindings) -> Self {
        Self::new(
            initial_position, Quat::IDENTITY, Quat::IDENTITY,
            SensitivityLook::calibrated(Box::new(MouseLook::idle())),
            PerspectiveLens::human_eye(), flight, AvatarPosture::standing(), 0.0,
            StrideRhythm::idle(), attire, SightMode::FirstPersonSight, PalmEarthworks::empty().reconcile_attire(attire), bindings,
        )
    }

    #[cfg(test)]
    pub fn with_attire(initial_position: Vec3, flight: Box<dyn GravitationalFlight>, attire: YouthAttire) -> Self {
        Self::with_attire_and_bindings(initial_position, flight, attire, ActionBindings::seeded())
    }

    #[cfg(test)]
    pub fn with_flight(initial_position: Vec3, flight: Box<dyn GravitationalFlight>) -> Self { Self::with_attire(initial_position, flight, YouthAttire::squire()) }
    #[cfg(test)]
    pub fn at(initial_position: Vec3) -> Self { Self::with_flight(initial_position, Box::new(SurfaceFlight::terrestrial(1.0))) }

    fn current_pose(&self) -> BodyPose {
        let view_rot = self.look.combine_with_surface(self.camera_rot);
        self.posture.body_pose(self.position, self.camera_rot, view_rot, self.rhythm)
    }

    fn surface_up(&self) -> Vec3 { self.surface_rot * Vec3::Y }

    fn reorient_surface(&mut self, target_up: Vec3, dt: f32) {
        let current_up = self.surface_up();
        let dot = current_up.dot(target_up);
        if dot <= 0.9999 {
            let rot_to_target = Quat::from_rotation_arc(current_up, target_up);
            let step_rot = if dot < 0.7 { rot_to_target } else { Quat::IDENTITY.slerp(rot_to_target, 1.0 - (-8.0 * dt).exp()) };
            self.surface_rot = (step_rot * self.surface_rot).normalize();
        }
        if self.camera_rot.dot(self.surface_rot).abs() > 0.9999 { self.camera_rot = self.surface_rot; }
        else { self.camera_rot = self.camera_rot.slerp(self.surface_rot, 1.0 - (-4.0 * dt).exp()).normalize(); }
    }

    fn act_with_hand(&mut self, side: f32, world: &mut dyn PlanetaryWorld) -> bool {
        if self.earthworks.satchel_open() { return false; }
        let eye = self.current_pose().eye_origin();
        let view_dir = (self.look.combine_with_surface(self.surface_rot) * Vec3::NEG_Z).normalize();
        let (next_ew, next_attire, next_rhythm, changed) =
            self.earthworks.act_with_hand(side, self.attire, self.rhythm, eye, view_dir, world);
        (self.earthworks, self.attire, self.rhythm) = (next_ew, next_attire, next_rhythm);
        changed
    }
}

impl Avatar for PlanetaryAvatar {
    fn rotate_view(&mut self, dx: f32, dy: f32) {
        if !self.earthworks.satchel_open() { self.look.rotate(dx, dy); }
    }
    fn use_left_hand(&mut self, world: &mut dyn PlanetaryWorld) -> bool { self.act_with_hand(-1.0, world) }
    fn use_right_hand(&mut self, world: &mut dyn PlanetaryWorld) -> bool { self.act_with_hand(1.0, world) }
    fn release_hand(&mut self, world: &mut dyn PlanetaryWorld) -> bool {
        if self.earthworks.satchel_open() { return false; }
        let (eye, dir) = (self.current_pose().eye_origin(), (self.look.combine_with_surface(self.surface_rot) * Vec3::NEG_Z).normalize());
        let (next_ew, next_attire, changed) = self.earthworks.release_hand(self.attire, eye, dir, self.position, world);
        (self.earthworks, self.attire) = (next_ew, next_attire);
        changed
    }
    fn step_digging(&mut self, world: &mut dyn PlanetaryWorld) -> bool {
        if self.earthworks.satchel_open() { return false; }
        let (eye, dir) = (self.current_pose().eye_origin(), (self.look.combine_with_surface(self.surface_rot) * Vec3::NEG_Z).normalize());
        let (next_ew, next_attire, next_rhythm, changed) = self.earthworks.continue_digging(self.attire, self.rhythm, eye, dir, world);
        (self.earthworks, self.attire, self.rhythm) = (next_ew, next_attire, next_rhythm);
        changed
    }
    fn apply_attire(&mut self, attire: YouthAttire) {
        let (r, l) = self.attire.held_pair();
        let merged = if !attire.armed() && self.attire.armed() { attire.with_relics(r, l) } else { attire };
        self.attire = merged;
        self.earthworks = self.earthworks.reconcile_attire(merged);
    }
    fn apply_bindings(&mut self, bindings: ActionBindings) { self.bindings = bindings; }

    fn dispatch_trigger(&mut self, trigger: InputTrigger) -> bool {
        let (ew, attire, handled) = self.earthworks.handle_trigger(trigger, &self.bindings, self.attire);
        if handled { (self.earthworks, self.attire) = (ew, attire); return true; }
        false
    }

    fn step_motion(&mut self, dt: f32, controls: &dyn KeyboardControls, world: &dyn PlanetaryWorld) {
        if self.earthworks.satchel_open() {
            self.rhythm = StrideRhythm::idle();
            return;
        }
        let terrain_up = world.surface_up_at(self.position);
        self.reorient_surface(terrain_up, dt);

        let mut order = LocomotionOrder::idle();
        let heading = SurfaceHeading::aligned(self.surface_rot, &self.look);
        heading.drive_locomotion(controls, &mut order);

        order.dispatch_jump(&mut *self.flight);
        self.posture = order.advance_posture(self.posture, dt);
        self.lens = order.advance_lens(self.lens, dt);
        self.tilt_radians *= 1.0 - (8.0 * dt).min(1.0);
        let next_pos = order.advance_flight(&mut *self.flight, dt, self.position, terrain_up, 6.0, world);
        self.posture = self.posture.advance_leap(self.position, next_pos, terrain_up, dt);
        self.rhythm = self.rhythm.advance(self.position, next_pos, terrain_up, dt);
        self.position = next_pos;
    }

    fn view_projection_matrix(&self, aspect: f32) -> Mat4 {
        let horizon = CameraHorizon::aligned(self.camera_rot, &self.look);
        let frame = HorizonTilt::new(&horizon, self.tilt_radians);
        let proj = self.lens.projection_matrix(aspect);
        let pose = self.current_pose();
        let view = self.sight.view_matrix(&frame, self.posture, self.position, &pose);
        proj * view
    }

    fn surface_rotation(&self) -> Quat { self.surface_rot }

    fn emit_appearance(&self, target: &mut dyn MeshTarget) {
        let pose = self.current_pose();
        self.earthworks.emit_attire(&self.attire, &pose, self.sight, target);
        let view_dir = (self.look.combine_with_surface(self.surface_rot) * Vec3::NEG_Z).normalize();
        self.earthworks.emit_debris(view_dir, self.surface_up(), self.rhythm, target);
    }

    fn emit_reticle(&self, target: &mut dyn MeshTarget) {
        if !self.earthworks.satchel_open() { ScreenPoint::center().emit_crosshair(target); }
    }
}
