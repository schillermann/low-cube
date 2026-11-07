mod bearing_gear;
mod body_sight;

use glam::{Mat4, Quat, Vec3};
#[cfg(test)]
use winit::keyboard::KeyCode;
use world::{MeshTarget, PlanetaryWorld};
use crate::attire::{BodyPose, PhotoShutter, SightMode, StrideRhythm, YouthAttire};
use crate::controls::{ActionBindings, InputTrigger, KeyboardControls};
use crate::earthworks::PalmEarthworks;
use crate::envelope::PhysicalEnvelope;
use crate::flight::GravitationalFlight;
#[cfg(test)]
use crate::flight::SurfaceFlight;
use crate::lens::PerspectiveLens;
use crate::menu::ScreenPoint;
use crate::order::LocomotionOrder;
use crate::orientation::{MouseLook, SensitivityLook};
use crate::posture::AvatarPosture;
use self::bearing_gear::{AvatarBearing, AvatarGear};
use self::body_sight::{AvatarBody, AvatarSight};

const EYE_SUPPORT_HEIGHT: f32 = 1.0;
const WALK_PACE_SPEED: f32 = 6.0;

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
    fn adjust_aimed_rig(&mut self, _trigger: InputTrigger, _world: &mut dyn PlanetaryWorld) -> bool { false }
    #[cfg(test)]
    fn dispatch_wardrobe_key(&mut self, code: KeyCode) -> bool { self.dispatch_trigger(InputTrigger::Key(code)) }
    #[cfg(test)]
    fn rotate_degrees(&mut self, yaw_deg: f32, pitch_deg: f32) {
        let px_per_deg = std::f32::consts::PI / (180.0 * 0.003);
        self.rotate_view(yaw_deg * px_per_deg, pitch_deg * px_per_deg);
    }
    fn step_motion(&mut self, dt: f32, controls: &dyn KeyboardControls, world: &dyn PlanetaryWorld);
    fn view_projection_matrix(&self, aspect: f32) -> Mat4;
    fn view_projection_for_sight(&self, aspect: f32, _sight: SightMode) -> Mat4 { self.view_projection_matrix(aspect) }
    fn surface_rotation(&self) -> Quat;
    fn emit_appearance(&self, target: &mut dyn MeshTarget);
    fn emit_appearance_for_sight(&self, _sight: SightMode, target: &mut dyn MeshTarget) { self.emit_appearance(target); }
    fn emit_surroundings(&self, _world: &dyn PlanetaryWorld, _target: &mut dyn MeshTarget) {}
    fn meadow_anchor(&self) -> (i32, i32, i32) { (0, 0, 0) }
    fn take_photo_shutter(&mut self) -> PhotoShutter { PhotoShutter::Idle }
    fn apply_polaroid_mosaic(&mut self, _mosaic: Vec<Vec3>) {}
    #[cfg(test)]
    fn polaroid_tilt(&self) -> f32 { 0.0 }
    fn emit_reticle(&self, target: &mut dyn MeshTarget) { ScreenPoint::center().emit_crosshair(target); }
}

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
            body: AvatarBody::new(position, flight, posture),
            sight_line: AvatarSight::new(surface_rot, camera_rot, look, lens),
            bearing: AvatarBearing::new(tilt_radians, rhythm, sight),
            gear: AvatarGear::new(attire, earthworks, bindings),
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

    pub fn with_starting_satchel(mut self, provider: &dyn crate::server_satchel::StartingSatchel) -> Self {
        self.gear.provision_satchel(provider);
        self
    }

    #[cfg(test)]
    pub fn with_test_engines(self) -> Self {
        self.with_starting_satchel(&crate::server_satchel::TestKitSatchel::new())
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
        self.body.shape_pose(self.sight_line.body_pose(self.body.position(), self.body.posture(), self.bearing.rhythm()))
    }

    fn hand_ray(&self) -> (Vec3, Vec3) {
        let (pose_eye, up, dir) = (self.current_pose().eye_origin(), self.sight_line.surface_up(), self.sight_line.view_direction());
        let pos = self.body.position();
        if dir.dot(up) > 0.975 { (pos + up * (pose_eye - pos).dot(up), up) } else { (pose_eye, dir) }
    }

    fn act_with_hand(&mut self, side: f32, world: &mut dyn PlanetaryWorld) -> bool {
        if self.body.perished() || world.lethal_at(self.body.position()) {
            return false;
        }
        let (eye, view_dir) = self.hand_ray();
        if !self.gear.satchel_open() {
            let allow_perch = !self.gear.holding_placeable_on(side);
            if let Some(outcome) = self.body.interact_vehicle(eye, view_dir, allow_perch, true, &mut self.sight_line, world) {
                self.bearing.calm();
                return outcome;
            }
        }
        self.gear.act_with_hand(side, &mut self.bearing, eye, view_dir, world)
    }

    fn step_perish(&mut self, dt: f32, world: &dyn PlanetaryWorld) {
        let (respawn_now, beat) = self.body.advance_perish(dt);
        if respawn_now {
            self.body.respawn(world);
            self.sight_line.reset_horizon();
            self.bearing.reset();
            return;
        }
        self.bearing.tilt_perish(dt);
        self.sight_line.transition_perish_lens(self.body.posture(), dt);
        if beat {
            use crate::cardiac::AcousticWave;
            crate::cardiac::MonitoredPulse::clinical().thump();
        }
    }
}

impl Avatar for PlanetaryAvatar {
    fn rotate_view(&mut self, dx: f32, dy: f32) {
        if !self.gear.satchel_open() && !self.body.perished() {
            self.sight_line.rotate(dx, dy);
        }
    }

    fn use_left_hand(&mut self, world: &mut dyn PlanetaryWorld) -> bool { self.act_with_hand(-1.0, world) }
    fn use_right_hand(&mut self, world: &mut dyn PlanetaryWorld) -> bool { self.act_with_hand(1.0, world) }

    fn release_hand(&mut self, world: &mut dyn PlanetaryWorld) -> bool {
        if self.body.perished() || self.gear.satchel_open() { return false; }
        let (eye, dir) = self.hand_ray();
        self.gear.release_hand(&mut self.bearing, eye, dir, self.body.position(), world)
    }

    fn step_digging(&mut self, world: &mut dyn PlanetaryWorld) -> bool {
        if self.body.perished() || self.gear.satchel_open() { return false; }
        let (eye, dir) = self.hand_ray();
        let (next_rhythm, changed) = self.gear.continue_digging(self.bearing.rhythm(), eye, dir, world);
        self.bearing.apply_rhythm(next_rhythm);
        changed
    }

    fn apply_attire(&mut self, attire: YouthAttire) { self.gear.apply_attire(attire); }
    fn apply_bindings(&mut self, bindings: ActionBindings) { self.gear.apply_bindings(bindings); }
    fn dispatch_trigger(&mut self, trigger: InputTrigger) -> bool {
        self.gear.dispatch_trigger(trigger, &mut self.bearing, self.body.perished())
    }
    fn adjust_aimed_rig(&mut self, trigger: InputTrigger, world: &mut dyn PlanetaryWorld) -> bool {
        if self.body.perished() { return false; }
        let (eye, dir) = self.hand_ray();
        self.gear.try_adjust_rig(trigger, &mut self.bearing, eye, dir, world)
    }

    fn step_motion(&mut self, dt: f32, controls: &dyn KeyboardControls, world: &dyn PlanetaryWorld) {
        self.bearing.step_polaroid(dt);
        let pos = self.body.position();
        if self.gear.satchel_open() || self.body.perished() || world.lethal_at(pos) {
            if self.body.perished() || world.lethal_at(pos) {
                self.step_perish(dt, world);
            } else {
                self.bearing.calm();
            }
            return;
        }
        let terrain_up = world.surface_up_at(pos);
        self.sight_line.reorient_surface(terrain_up, dt);
        if let Some(steer_px) = self.body.step_driving(dt, controls, world) {
            if steer_px.abs() > 1e-5 {
                self.sight_line.steer_vehicle(steer_px);
            }
            let next_up = world.surface_up_at(self.body.position());
            self.sight_line.reorient_surface(next_up, dt);
            self.bearing.calm();
            self.bearing.decay_tilt(dt);
            return;
        }
        let mut order = LocomotionOrder::idle();
        self.sight_line.drive_order(controls, &mut order);
        let facing = self.sight_line.view_direction();
        let standing_env = self.gear.standing_envelope(facing);
        let in_upward_shaft = standing_env.crust(world).in_upward_shaft(pos, terrain_up, EYE_SUPPORT_HEIGHT);
        order.calm_beneath_crust(world.weightless_at(pos) || in_upward_shaft);
        let _ = self.body.prepare_stride(&order, standing_env, terrain_up, dt, world);
        let active_env = standing_env.with_crouch_and_facing(self.body.posture().crouch_drop(), facing);
        let active_crust = active_env.crust(world);
        self.bearing.decay_tilt(dt);
        let raw_next = self.body.apply_flight(active_env, &order, dt, terrain_up, WALK_PACE_SPEED, world);
        if world.lethal_at(raw_next) {
            self.body.relocate(raw_next);
            self.step_perish(dt, world);
            return;
        }
        let weightless = world.weightless_at(raw_next) || world.weightless_at(pos);
        let in_shaft = weightless || active_crust.in_deep_shaft(raw_next, terrain_up, EYE_SUPPORT_HEIGHT);
        let pre_eye = if weightless { Vec3::ZERO } else { self.body.eye_offset(terrain_up) };
        let next_pos = if in_shaft { world.confine_beneath_crust(raw_next + pre_eye) - pre_eye } else { active_crust.confine(raw_next, pos, terrain_up) };
        let core_closeness = if weightless {
            let core_pt = world.constrain_to_surface(next_pos, EYE_SUPPORT_HEIGHT);
            let span = (core_pt.length() * 0.36).clamp(6.4, 13.0);
            (1.0 - (next_pos.length() - core_pt.length()).max(0.0) / span).clamp(0.0, 1.0)
        } else {
            0.0
        };
        let beat = self.body.advance_leap_and_hover(next_pos, terrain_up, weightless, core_closeness, dt);
        self.sight_line.advance_lens(&order, self.body.posture(), dt);
        if beat {
            use crate::cardiac::AcousticWave;
            crate::cardiac::CardiacPulse::studio().thump();
        }
        self.bearing.advance_stride(pos, next_pos, terrain_up, dt);
        let post_eye = self.body.eye_offset(terrain_up);
        let final_pos = if in_shaft { world.confine_beneath_crust(next_pos + post_eye) - post_eye } else { active_crust.confine(next_pos, pos, terrain_up) };
        self.body.relocate(final_pos);
        self.sight_line.constrain_shoulders(
            active_env.wedged_in_shaft(final_pos, terrain_up, EYE_SUPPORT_HEIGHT, world),
            self.gear.shaft_yaw_play(),
        );
    }

    fn view_projection_matrix(&self, aspect: f32) -> Mat4 {
        self.sight_line.project_view(aspect, self.bearing.tilt_radians(), self.bearing.sight(), self.body.posture(), self.body.position(), &self.current_pose())
    }

    fn view_projection_for_sight(&self, aspect: f32, sight: SightMode) -> Mat4 {
        if let Some((eye, fwd, up)) = self.gear.placed_camera_lens() {
            return self.sight_line.project_lens_view(aspect, eye, fwd, up);
        }
        self.sight_line.project_view(aspect, self.bearing.tilt_radians(), sight, self.body.posture(), self.body.position(), &self.current_pose())
    }

    fn surface_rotation(&self) -> Quat { self.sight_line.surface_rotation() }
    fn emit_appearance(&self, target: &mut dyn MeshTarget) {
        self.body.emit_vehicle(target);
        self.gear.emit_appearance(&self.current_pose(), self.bearing.sight(), self.sight_line.view_direction(), self.sight_line.surface_up(), self.bearing.rhythm(), target);
    }
    fn emit_appearance_for_sight(&self, sight: SightMode, target: &mut dyn MeshTarget) {
        self.body.emit_vehicle(target);
        if !matches!(sight, SightMode::FirstPersonSight) {
            self.gear.emit_appearance(&self.current_pose(), sight, self.sight_line.view_direction(), self.sight_line.surface_up(), self.bearing.rhythm(), target);
        }
    }
    fn emit_surroundings(&self, world: &dyn PlanetaryWorld, target: &mut dyn MeshTarget) {
        world.emit_meadow_around(self.body.position(), self.sight_line.view_direction(), target);
    }
    fn meadow_anchor(&self) -> (i32, i32, i32) {
        let p = self.body.position();
        (((p.x * 0.25).floor() as i32) * 8, ((p.y * 0.25).floor() as i32) * 8, ((p.z * 0.25).floor() as i32) * 8)
    }
    fn take_photo_shutter(&mut self) -> PhotoShutter { self.bearing.take_shutter() }
    fn apply_polaroid_mosaic(&mut self, mosaic: Vec<Vec3>) { self.bearing.apply_polaroid_mosaic(mosaic); }
    #[cfg(test)]
    fn polaroid_tilt(&self) -> f32 { self.bearing.polaroid_tilt() }
    fn emit_reticle(&self, target: &mut dyn MeshTarget) {
        if !self.gear.satchel_open() {
            self.body.posture().emit_reticle(target);
            self.bearing.emit_polaroid(target);
        }
    }
}
