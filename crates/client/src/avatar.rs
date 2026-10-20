use glam::{Mat4, Quat, Vec3};
use winit::keyboard::KeyCode;
use world::{MeshTarget, PlanetaryWorld};
use crate::attire::{BodyPose, HeldRelic, SightMode, StrideRhythm, Wardrobe, YouthAttire};
use crate::axe::RelicSite;
use crate::controls::KeyboardControls;
use crate::flight::GravitationalFlight;
#[cfg(test)]
use crate::flight::SurfaceFlight;
use crate::heading::SurfaceHeading;
use crate::horizon::{CameraHorizon, HorizonTilt};
use crate::lens::{PerspectiveLens, ProjectionLens};
use crate::order::LocomotionOrder;
use crate::orientation::{InvertibleLook, MouseLook, SensitivityLook, ViewOrientation};
use crate::posture::AvatarPosture;

/// Interface for player avatars acting within planetary gravity.
pub trait Avatar {
    fn rotate_view(&mut self, dx: f32, dy: f32);
    fn toggle_invert_y(&mut self);
    fn toggle_invert_x(&mut self);
    fn trigger_strike(&mut self);
    fn dispatch_wardrobe_key(&mut self, code: KeyCode) -> bool;
    fn step_motion(&mut self, dt: f32, controls: &dyn KeyboardControls, world: &dyn PlanetaryWorld);
    fn sync_relic_site(&mut self, site: &mut dyn RelicSite, dt: f32, world: &dyn PlanetaryWorld) -> Option<(HeldRelic, HeldRelic)>;
    fn view_projection_matrix(&self, aspect: f32) -> Mat4;
    fn surface_rotation(&self) -> Quat;
    fn emit_appearance(&self, target: &mut dyn MeshTarget);
}

/// Fully encapsulated player entity composed of position, gravity alignment, look, lens, flight, posture, and medieval attire.
pub struct PlanetaryAvatar {
    position: Vec3,
    surface_rot: Quat,
    camera_rot: Quat,
    look: InvertibleLook,
    lens: PerspectiveLens,
    flight: Box<dyn GravitationalFlight>,
    posture: AvatarPosture,
    tilt_radians: f32,
    rhythm: StrideRhythm,
    attire: YouthAttire,
    sight: SightMode,
}

impl PlanetaryAvatar {
    /// Primary constructor doing nothing except assigning arguments to fields.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        position: Vec3, surface_rot: Quat, camera_rot: Quat, look: InvertibleLook, lens: PerspectiveLens,
        flight: Box<dyn GravitationalFlight>, posture: AvatarPosture, tilt_radians: f32,
        rhythm: StrideRhythm, attire: YouthAttire, sight: SightMode,
    ) -> Self {
        Self { position, surface_rot, camera_rot, look, lens, flight, posture, tilt_radians, rhythm, attire, sight }
    }

    /// Secondary constructor initializing avatar with explicit gravitational flight and medieval attire.
    pub fn with_attire(initial_position: Vec3, flight: Box<dyn GravitationalFlight>, attire: YouthAttire) -> Self {
        Self::new(
            initial_position, Quat::IDENTITY, Quat::IDENTITY,
            InvertibleLook::direct(Box::new(SensitivityLook::calibrated(Box::new(MouseLook::idle())))),
            PerspectiveLens::human_eye(), flight, AvatarPosture::standing(), 0.0,
            StrideRhythm::idle(), attire, SightMode::FirstPersonSight,
        )
    }

    /// Secondary constructor initializing avatar with explicit gravitational flight mechanics.
    #[cfg(test)]
    pub fn with_flight(initial_position: Vec3, flight: Box<dyn GravitationalFlight>) -> Self {
        Self::with_attire(initial_position, flight, YouthAttire::squire())
    }

    /// Secondary constructor initializing avatar at an initial position with terrestrial flight.
    #[cfg(test)]
    pub fn at(initial_position: Vec3) -> Self {
        Self::with_flight(initial_position, Box::new(SurfaceFlight::terrestrial(1.0)))
    }

    fn current_pose(&self) -> BodyPose {
        let view_rot = self.look.combine_with_surface(self.camera_rot);
        self.posture.body_pose(
            self.position,
            self.camera_rot,
            view_rot,
            self.rhythm,
        )
    }

    fn surface_up(&self) -> Vec3 {
        self.surface_rot * Vec3::Y
    }

    fn reorient_surface(&mut self, target_up: Vec3, dt: f32) {
        let current_up = self.surface_up();
        let dot = current_up.dot(target_up);
        if dot <= 0.9999 {
            let rot_to_target = Quat::from_rotation_arc(current_up, target_up);
            let step_rot = if dot < 0.7 {
                rot_to_target
            } else {
                let factor = 1.0 - (-8.0 * dt).exp();
                Quat::IDENTITY.slerp(rot_to_target, factor)
            };
            self.surface_rot = (step_rot * self.surface_rot).normalize();
        }
        self.converge_camera(dt);
    }

    fn converge_camera(&mut self, dt: f32) {
        if self.camera_rot.dot(self.surface_rot).abs() > 0.9999 {
            self.camera_rot = self.surface_rot;
            return;
        }
        let factor = 1.0 - (-4.0 * dt).exp();
        self.camera_rot = self.camera_rot.slerp(self.surface_rot, factor).normalize();
    }
}

impl Avatar for PlanetaryAvatar {
    fn rotate_view(&mut self, dx: f32, dy: f32) {
        self.look.rotate(dx, dy);
    }

    fn toggle_invert_y(&mut self) {
        self.look.toggle_y();
    }

    fn toggle_invert_x(&mut self) {
        self.look.toggle_x();
    }

    fn trigger_strike(&mut self) {
        if self.attire.armed() {
            self.rhythm = self.rhythm.trigger_strike();
        }
    }

    fn dispatch_wardrobe_key(&mut self, code: KeyCode) -> bool {
        if code == KeyCode::KeyM {
            self.sight = self.sight.toggle();
            return true;
        }
        if let Some(updated) = self.attire.cycled_by_key(code) {
            self.attire = updated;
            return true;
        }
        false
    }

    fn step_motion(&mut self, dt: f32, controls: &dyn KeyboardControls, world: &dyn PlanetaryWorld) {
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
        self.rhythm = self.rhythm.advance(self.position, next_pos, terrain_up, dt);
        self.position = next_pos;
    }

    fn sync_relic_site(&mut self, site: &mut dyn RelicSite, dt: f32, world: &dyn PlanetaryWorld) -> Option<(HeldRelic, HeldRelic)> {
        let view_rot = self.look.combine_with_surface(self.surface_rot);
        let changed = site.step_near_avatar(self.position, self.surface_rot, view_rot, dt, world);
        if let Some((right, left)) = changed {
            self.attire = self.attire.with_relics(right, left);
        }
        changed
    }

    fn view_projection_matrix(&self, aspect: f32) -> Mat4 {
        let horizon = CameraHorizon::aligned(self.camera_rot, &self.look);
        let frame = HorizonTilt::new(&horizon, self.tilt_radians);
        let proj = self.lens.projection_matrix(aspect);
        let pose = self.current_pose();
        let view = self.sight.view_matrix(&frame, self.posture, self.position, &pose);
        proj * view
    }

    fn surface_rotation(&self) -> Quat {
        self.surface_rot
    }

    fn emit_appearance(&self, target: &mut dyn MeshTarget) {
        let pose = self.current_pose();
        self.attire.emit(&pose, self.sight, target);
    }
}
