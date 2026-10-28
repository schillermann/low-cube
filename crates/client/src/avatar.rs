use glam::{Mat4, Quat, Vec3};
#[cfg(test)]
use winit::keyboard::KeyCode;
use world::{MeshTarget, PlanetaryWorld};
use crate::attire::{BodyPose, SightMode, StrideRhythm, YouthAttire};
use crate::controls::{ActionBindings, InputTrigger, KeyboardControls};
use crate::earthworks::PalmEarthworks;
use crate::envelope::PhysicalEnvelope;
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
    fn dispatch_wardrobe_key(&mut self, code: KeyCode) -> bool { self.dispatch_trigger(InputTrigger::Key(code)) }
    #[cfg(test)]
    fn rotate_degrees(&mut self, yaw_deg: f32, pitch_deg: f32) {
        let px_per_deg = std::f32::consts::PI / (180.0 * 0.003);
        self.rotate_view(yaw_deg * px_per_deg, pitch_deg * px_per_deg);
    }
    fn step_motion(&mut self, dt: f32, controls: &dyn KeyboardControls, world: &dyn PlanetaryWorld);
    fn view_projection_matrix(&self, aspect: f32) -> Mat4;
    fn surface_rotation(&self) -> Quat;
    fn emit_appearance(&self, target: &mut dyn MeshTarget);
    fn emit_reticle(&self, target: &mut dyn MeshTarget) { ScreenPoint::center().emit_crosshair(target); }
}

struct AvatarBody { position: Vec3, flight: Box<dyn GravitationalFlight>, posture: AvatarPosture }
struct AvatarSight { surface_rot: Quat, camera_rot: Quat, look: SensitivityLook, lens: PerspectiveLens }
struct AvatarBearing { tilt_radians: f32, rhythm: StrideRhythm, sight: SightMode }
struct AvatarGear { attire: YouthAttire, earthworks: PalmEarthworks, bindings: ActionBindings }

/// Fully encapsulated player entity composed of body, sight line, kinematic bearing, and gear.
pub struct PlanetaryAvatar {
    body: AvatarBody,
    sight_line: AvatarSight,
    bearing: AvatarBearing,
    gear: AvatarGear,
}

impl PlanetaryAvatar {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        position: Vec3, surface_rot: Quat, camera_rot: Quat, look: SensitivityLook, lens: PerspectiveLens,
        flight: Box<dyn GravitationalFlight>, posture: AvatarPosture, tilt_radians: f32, rhythm: StrideRhythm,
        attire: YouthAttire, sight: SightMode, earthworks: PalmEarthworks, bindings: ActionBindings,
    ) -> Self {
        Self {
            body: AvatarBody { position, flight, posture },
            sight_line: AvatarSight { surface_rot, camera_rot, look, lens },
            bearing: AvatarBearing { tilt_radians, rhythm, sight },
            gear: AvatarGear { attire, earthworks, bindings },
        }
    }

    pub fn with_attire_and_bindings(
        initial_position: Vec3, flight: Box<dyn GravitationalFlight>, attire: YouthAttire, bindings: ActionBindings,
    ) -> Self {
        Self::new(
            initial_position, Quat::IDENTITY, Quat::IDENTITY,
            SensitivityLook::calibrated(Box::new(MouseLook::idle())), PerspectiveLens::human_eye(),
            flight, AvatarPosture::standing(), 0.0, StrideRhythm::idle(), attire, SightMode::FirstPersonSight,
            PalmEarthworks::empty().reconcile_attire(attire), bindings,
        )
    }

    #[cfg(test)]
    pub fn with_attire(initial_position: Vec3, flight: Box<dyn GravitationalFlight>, attire: YouthAttire) -> Self {
        Self::with_attire_and_bindings(initial_position, flight, attire, ActionBindings::seeded())
    }
    #[cfg(test)]
    pub fn with_flight(initial_position: Vec3, flight: Box<dyn GravitationalFlight>) -> Self {
        Self::with_attire(initial_position, flight, YouthAttire::squire())
    }
    #[cfg(test)]
    pub fn at(initial_position: Vec3) -> Self {
        Self::with_flight(initial_position, Box::new(SurfaceFlight::terrestrial(1.0)))
    }

    fn current_pose(&self) -> BodyPose {
        let view_rot = self.sight_line.look.combine_with_surface(self.sight_line.camera_rot);
        self.body.posture.body_pose(self.body.position, self.sight_line.camera_rot, view_rot, self.bearing.rhythm)
    }

    fn surface_up(&self) -> Vec3 { self.sight_line.surface_rot * Vec3::Y }

    fn reorient_surface(&mut self, target_up: Vec3, dt: f32) {
        let current_up = self.surface_up();
        let dot = current_up.dot(target_up);
        if dot <= 0.9999 {
            let rot_to_target = Quat::from_rotation_arc(current_up, target_up);
            let step_rot = if dot < 0.7 { rot_to_target } else { Quat::IDENTITY.slerp(rot_to_target, 1.0 - (-8.0 * dt).exp()) };
            self.sight_line.surface_rot = (step_rot * self.sight_line.surface_rot).normalize();
        }
        if self.sight_line.camera_rot.dot(self.sight_line.surface_rot).abs() > 0.9999 {
            self.sight_line.camera_rot = self.sight_line.surface_rot;
        } else {
            self.sight_line.camera_rot = self.sight_line.camera_rot.slerp(self.sight_line.surface_rot, 1.0 - (-4.0 * dt).exp()).normalize();
        }
    }

    fn hand_ray(&self) -> (Vec3, Vec3) {
        let (pose_eye, up) = (self.current_pose().eye_origin(), self.surface_up());
        let dir = (self.sight_line.look.combine_with_surface(self.sight_line.surface_rot) * Vec3::NEG_Z).normalize();
        if dir.dot(up) > 0.975 {
            (self.body.position + up * (pose_eye - self.body.position).dot(up), up)
        } else {
            (pose_eye, dir)
        }
    }

    fn act_with_hand(&mut self, side: f32, world: &mut dyn PlanetaryWorld) -> bool {
        if self.body.posture.perished() || world.lethal_at(self.body.position) || (self.gear.earthworks.satchel_open() && !self.gear.earthworks.tome_open()) {
            return false;
        }
        let (eye, view_dir) = self.hand_ray();
        let (next_ew, next_attire, next_rhythm, changed) =
            self.gear.earthworks.act_with_hand(side, self.gear.attire, self.bearing.rhythm, eye, view_dir, world);
        (self.gear.earthworks, self.gear.attire, self.bearing.rhythm) = (next_ew, next_attire, next_rhythm);
        changed
    }

    fn step_perish(&mut self, dt: f32, world: &dyn PlanetaryWorld) {
        let (next_posture, beat) = self.body.posture.perish().advance_perish(dt);
        if next_posture.blackout_complete() {
            let spawn = [(0.25, 0.25), (2.25, 0.25), (-2.25, 0.25), (0.25, 2.25), (0.25, -2.25)]
                .into_iter()
                .map(|(x, z)| world.constrain_to_surface(Vec3::new(x, 1024.0, z), 1.0))
                .fold(world.constrain_to_surface(Vec3::new(0.25, 1024.0, 0.25), 1.0), |acc, c| if c.y > acc.y + 1.5 { c } else { acc });
            (self.body.position, self.sight_line.surface_rot, self.sight_line.camera_rot) = (spawn, Quat::IDENTITY, Quat::IDENTITY);
            self.bearing.tilt_radians = 0.0;
            (self.body.posture, self.bearing.rhythm, self.sight_line.lens) = (AvatarPosture::standing(), StrideRhythm::idle(), PerspectiveLens::human_eye());
            self.body.flight.ground();
            self.sight_line.look.constrain_shoulders(false, 0.0);
            return;
        }
        (self.body.posture, self.bearing.rhythm) = (next_posture, StrideRhythm::idle());
        self.sight_line.lens = self.sight_line.lens.transition_to(self.body.posture.cardiac_fov(), dt);
        self.bearing.tilt_radians += (0.22 - self.bearing.tilt_radians) * (4.0 * dt).clamp(0.0, 1.0);
        if beat {
            crate::cardiac::CardiacPulse::studio().thump();
        }
    }
}

impl Avatar for PlanetaryAvatar {
    fn rotate_view(&mut self, dx: f32, dy: f32) {
        if !self.gear.earthworks.satchel_open() && !self.body.posture.perished() {
            self.sight_line.look.rotate(dx, dy);
        }
    }

    fn use_left_hand(&mut self, world: &mut dyn PlanetaryWorld) -> bool { self.act_with_hand(-1.0, world) }
    fn use_right_hand(&mut self, world: &mut dyn PlanetaryWorld) -> bool { self.act_with_hand(1.0, world) }

    fn release_hand(&mut self, world: &mut dyn PlanetaryWorld) -> bool {
        if self.body.posture.perished() || self.gear.earthworks.satchel_open() { return false; }
        let (eye, dir) = self.hand_ray();
        let (next_ew, next_attire, changed) =
            self.gear.earthworks.release_hand(self.gear.attire, eye, dir, self.body.position, world);
        (self.gear.earthworks, self.gear.attire) = (next_ew, next_attire);
        changed
    }

    fn step_digging(&mut self, world: &mut dyn PlanetaryWorld) -> bool {
        if self.body.posture.perished() || self.gear.earthworks.satchel_open() { return false; }
        let (eye, dir) = self.hand_ray();
        let (next_ew, next_attire, next_rhythm, changed) =
            self.gear.earthworks.continue_digging(self.gear.attire, self.bearing.rhythm, eye, dir, world);
        (self.gear.earthworks, self.gear.attire, self.bearing.rhythm) = (next_ew, next_attire, next_rhythm);
        changed
    }

    fn apply_attire(&mut self, attire: YouthAttire) {
        let (r, l) = self.gear.attire.held_pair();
        let merged = if !attire.armed() && self.gear.attire.armed() { attire.with_relics(r, l) } else { attire };
        (self.gear.attire, self.gear.earthworks) = (merged, self.gear.earthworks.reconcile_attire(merged));
    }

    fn apply_bindings(&mut self, bindings: ActionBindings) { self.gear.bindings = bindings; }

    fn dispatch_trigger(&mut self, trigger: InputTrigger) -> bool {
        let (ew, attire, handled) = self.gear.earthworks.handle_trigger(trigger, &self.gear.bindings, self.gear.attire);
        if handled {
            (self.gear.earthworks, self.gear.attire) = (ew, attire);
        }
        handled
    }

    fn step_motion(&mut self, dt: f32, controls: &dyn KeyboardControls, world: &dyn PlanetaryWorld) {
        if self.gear.earthworks.satchel_open() || self.body.posture.perished() || world.lethal_at(self.body.position) {
            if self.body.posture.perished() || world.lethal_at(self.body.position) {
                self.step_perish(dt, world);
            } else {
                self.bearing.rhythm = StrideRhythm::idle();
            }
            return;
        }
        let terrain_up = world.surface_up_at(self.body.position);
        self.reorient_surface(terrain_up, dt);
        let mut order = LocomotionOrder::idle();
        SurfaceHeading::aligned(self.sight_line.surface_rot, &self.sight_line.look).drive_locomotion(controls, &mut order);
        order.calm_beneath_crust(world.weightless_at(self.body.position));
        order.dispatch_jump(&mut *self.body.flight);
        let facing = (self.sight_line.look.combine_with_surface(self.sight_line.surface_rot) * Vec3::NEG_Z).normalize();
        let standing_env = self.gear.attire.body_envelope().with_crouch_and_facing(0.0, facing);
        let next_posture = order.advance_posture(self.body.posture, dt);
        self.body.posture = if standing_env.overhead_obstructed(self.body.position, terrain_up, 1.0, world) {
            self.body.posture.transition_to(0.36, dt)
        } else {
            next_posture
        };
        let crouch_drop = (0.80 - self.body.posture.eye_elevation()).clamp(0.0, 0.44);
        let active_env = standing_env.with_crouch_and_facing(crouch_drop, facing);
        self.body.flight.apply_envelope(active_env);
        self.sight_line.lens = order.advance_lens(self.sight_line.lens, dt);
        self.bearing.tilt_radians *= 1.0 - (8.0 * dt).min(1.0);
        let next_pos = order.advance_flight(&mut *self.body.flight, dt, self.body.position, terrain_up, 6.0, world);
        if world.lethal_at(next_pos) {
            self.body.position = next_pos;
            self.step_perish(dt, world);
            return;
        }
        let weightless = world.weightless_at(next_pos) || world.weightless_at(self.body.position);
        let (next_posture, beat) = self.body.posture
            .advance_leap(self.body.position, next_pos, terrain_up, dt)
            .advance_hover(weightless, dt);
        self.body.posture = next_posture;
        if self.body.posture.cardiac_constriction() > 0.0 {
            self.sight_line.lens = self.sight_line.lens.transition_to(self.body.posture.cardiac_fov(), dt);
        }
        if beat {
            crate::cardiac::CardiacPulse::studio().thump();
        }
        self.bearing.rhythm = self.bearing.rhythm.advance(self.body.position, next_pos, terrain_up, dt);
        let eye_off = terrain_up * self.body.posture.eye_elevation();
        let in_shaft = weightless || (next_pos - active_env.constrain_with_height(next_pos, 1.0, world)).dot(terrain_up) > 2.5;
        self.body.position = if in_shaft { world.confine_beneath_crust(next_pos + eye_off) - eye_off } else { next_pos };
        self.sight_line.look.constrain_shoulders(
            active_env.wedged_in_shaft(self.body.position, terrain_up, 1.0, world),
            self.gear.attire.shaft_yaw_play(),
        );
    }

    fn view_projection_matrix(&self, aspect: f32) -> Mat4 {
        let horizon = CameraHorizon::aligned(self.sight_line.camera_rot, &self.sight_line.look);
        let frame = HorizonTilt::new(&horizon, self.bearing.tilt_radians);
        self.sight_line.lens.projection_matrix(aspect)
            * self.bearing.sight.view_matrix(&frame, self.body.posture, self.body.position, &self.current_pose())
    }

    fn surface_rotation(&self) -> Quat { self.sight_line.surface_rot }

    fn emit_appearance(&self, target: &mut dyn MeshTarget) {
        self.gear.earthworks.emit_attire(&self.gear.attire, &self.current_pose(), self.bearing.sight, target);
        let view_dir = (self.sight_line.look.combine_with_surface(self.sight_line.surface_rot) * Vec3::NEG_Z).normalize();
        self.gear.earthworks.emit_debris(view_dir, self.surface_up(), self.bearing.rhythm, target);
    }

    fn emit_reticle(&self, target: &mut dyn MeshTarget) {
        if !self.gear.earthworks.satchel_open() { self.body.posture.emit_reticle(target); }
    }
}
