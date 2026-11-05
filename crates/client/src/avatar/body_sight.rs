use glam::{Mat4, Quat, Vec3};
use world::{MeshTarget, PlanetaryWorld, VehicleAssembly};
use crate::attire::{BodyPose, SightMode, StrideRhythm};
use crate::controls::KeyboardControls;
use crate::drive::{DrivenCar, VehiclePilot};
use crate::envelope::BodyEnvelope;
use crate::flight::GravitationalFlight;
use crate::heading::SurfaceHeading;
use crate::horizon::{CameraHorizon, HorizonTilt};
use crate::lens::{PerspectiveLens, ProjectionLens};
use crate::order::LocomotionOrder;
use crate::orientation::{SensitivityLook, ViewOrientation};
use crate::posture::AvatarPosture;

const EYE_SUPPORT_HEIGHT: f32 = 1.0;

/// Encapsulated physical avatar body state: world position, gravitational flight, posture, and pilot state.
pub struct AvatarBody {
    position: Vec3,
    flight: Box<dyn GravitationalFlight>,
    posture: AvatarPosture,
    pilot: VehiclePilot,
}

impl AvatarBody {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn with_pilot(position: Vec3, flight: Box<dyn GravitationalFlight>, posture: AvatarPosture, pilot: VehiclePilot) -> Self {
        Self { position, flight, posture, pilot }
    }

    pub fn new(position: Vec3, flight: Box<dyn GravitationalFlight>, posture: AvatarPosture) -> Self {
        Self::with_pilot(position, flight, posture, VehiclePilot::Pedestrian)
    }

    pub const fn position(&self) -> Vec3 {
        self.position
    }

    pub const fn posture(&self) -> AvatarPosture {
        self.posture
    }

    pub const fn driving(&self) -> bool {
        self.pilot.driving()
    }

    pub fn perished(&self) -> bool {
        self.posture.perished()
    }

    pub fn eye_offset(&self, terrain_up: Vec3) -> Vec3 {
        terrain_up * self.posture.eye_elevation()
    }

    pub fn relocate(&mut self, next_pos: Vec3) {
        self.position = next_pos;
    }

    pub fn respawn(&mut self, spawn: Vec3) {
        self.position = spawn;
        self.posture = AvatarPosture::standing();
        self.pilot = VehiclePilot::Pedestrian;
        self.flight.ground();
    }

    pub fn board_vehicle(&mut self, assembly: VehicleAssembly, world: &dyn PlanetaryWorld) {
        let car = DrivenCar::boarded(assembly);
        self.position = car.seat_position(world);
        self.posture = AvatarPosture::crouching();
        self.flight.ground();
        self.pilot = VehiclePilot::Seated(car);
    }

    pub fn dismount_vehicle(&mut self, world: &mut dyn PlanetaryWorld) -> bool {
        if self.pilot.dismount(world) {
            self.posture = AvatarPosture::standing();
            self.position = world.constrain_to_surface(self.position, EYE_SUPPORT_HEIGHT);
            self.flight.ground();
            true
        } else {
            false
        }
    }

    pub fn step_driving(&mut self, dt: f32, controls: &dyn KeyboardControls, world: &dyn PlanetaryWorld) -> Option<f32> {
        let (seat_pos, steer_px) = self.pilot.step(dt, controls, world)?;
        self.position = seat_pos;
        self.posture = self.posture.crouch_down(dt);
        Some(steer_px)
    }

    pub fn emit_vehicle(&self, target: &mut dyn MeshTarget) {
        self.pilot.emit(target);
    }

    pub fn advance_perish(&mut self, dt: f32) -> (bool, bool) {
        let (next_posture, beat) = self.posture.perish().advance_perish(dt);
        if next_posture.blackout_complete() {
            return (true, false);
        }
        self.posture = next_posture;
        (false, beat)
    }

    pub fn prepare_stride(&mut self, order: &LocomotionOrder, standing_env: BodyEnvelope, terrain_up: Vec3, dt: f32, world: &dyn PlanetaryWorld) -> BodyEnvelope {
        order.dispatch_jump(&mut *self.flight);
        let next_posture = order.advance_posture(self.posture, dt);
        self.posture = if standing_env.crust(world).overhead_obstructed(self.position, terrain_up, EYE_SUPPORT_HEIGHT) {
            self.posture.crouch_down(dt)
        } else {
            next_posture
        };
        let active_env = standing_env.with_crouch_and_facing(self.posture.crouch_drop(), Vec3::ZERO);
        self.flight.apply_envelope(active_env);
        active_env
    }

    pub fn apply_flight(&mut self, active_env: BodyEnvelope, order: &LocomotionOrder, dt: f32, terrain_up: Vec3, pace: f32, world: &dyn PlanetaryWorld) -> Vec3 {
        self.flight.apply_envelope(active_env);
        order.advance_flight(&mut *self.flight, dt, self.position, terrain_up, pace, world)
    }

    pub fn advance_leap_and_hover(&mut self, next_pos: Vec3, terrain_up: Vec3, weightless: bool, core_closeness: f32, dt: f32) -> bool {
        let (next_posture, beat) = self
            .posture
            .advance_leap(self.position, next_pos, terrain_up, dt)
            .advance_core_hover(weightless, core_closeness, dt);
        self.posture = next_posture;
        beat
    }
}

/// Encapsulated camera orientation, surface alignment, look sensitivity, and projection lens.
pub struct AvatarSight {
    surface_rot: Quat,
    camera_rot: Quat,
    look: SensitivityLook,
    lens: PerspectiveLens,
}

impl AvatarSight {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(surface_rot: Quat, camera_rot: Quat, look: SensitivityLook, lens: PerspectiveLens) -> Self {
        Self { surface_rot, camera_rot, look, lens }
    }

    pub const fn surface_rotation(&self) -> Quat {
        self.surface_rot
    }

    pub fn surface_up(&self) -> Vec3 {
        self.surface_rot * Vec3::Y
    }

    pub fn view_direction(&self) -> Vec3 {
        (self.look.combine_with_surface(self.surface_rot) * Vec3::NEG_Z).normalize()
    }

    pub fn rotate(&mut self, dx: f32, dy: f32) {
        self.look.rotate(dx, dy);
    }

    pub fn reset_horizon(&mut self) {
        self.surface_rot = Quat::IDENTITY;
        self.camera_rot = Quat::IDENTITY;
        self.lens = PerspectiveLens::human_eye();
        self.look.constrain_shoulders(false, 0.0);
    }

    pub fn reorient_surface(&mut self, target_up: Vec3, dt: f32) {
        let current_up = self.surface_up();
        let dot = current_up.dot(target_up);
        if dot <= 0.9999 {
            let rot_to_target = Quat::from_rotation_arc(current_up, target_up);
            let step_rot = if dot < 0.7 {
                rot_to_target
            } else {
                Quat::IDENTITY.slerp(rot_to_target, 1.0 - (-8.0 * dt).exp())
            };
            self.surface_rot = (step_rot * self.surface_rot).normalize();
        }
        if self.camera_rot.dot(self.surface_rot).abs() > 0.9999 {
            self.camera_rot = self.surface_rot;
        } else {
            self.camera_rot = self.camera_rot.slerp(self.surface_rot, 1.0 - (-4.0 * dt).exp()).normalize();
        }
    }

    pub fn drive_order(&self, controls: &dyn KeyboardControls, order: &mut LocomotionOrder) {
        SurfaceHeading::aligned(self.surface_rot, &self.look).drive_locomotion(controls, order);
    }

    pub fn advance_lens(&mut self, order: &LocomotionOrder, posture: AvatarPosture, dt: f32) {
        self.lens = order.advance_lens(self.lens, dt);
        if posture.cardiac_constriction() > 0.0 {
            self.lens = self.lens.transition_to(posture.cardiac_fov(), dt);
        }
    }

    pub fn transition_perish_lens(&mut self, posture: AvatarPosture, dt: f32) {
        self.lens = self.lens.transition_to(posture.cardiac_fov(), dt);
    }

    pub fn constrain_shoulders(&mut self, wedged: bool, yaw_play: f32) {
        self.look.constrain_shoulders(wedged, yaw_play);
    }

    pub fn body_pose(&self, position: Vec3, posture: AvatarPosture, rhythm: StrideRhythm) -> BodyPose {
        let view_rot = self.look.combine_with_surface(self.camera_rot);
        posture.body_pose(position, self.camera_rot, view_rot, rhythm)
    }

    pub fn project_view(&self, aspect: f32, tilt_radians: f32, sight: SightMode, posture: AvatarPosture, position: Vec3, pose: &BodyPose) -> Mat4 {
        let horizon = CameraHorizon::aligned(self.camera_rot, &self.look);
        let frame = HorizonTilt::new(&horizon, tilt_radians);
        self.lens.projection_matrix(aspect) * sight.view_matrix(&frame, posture, position, pose)
    }
}
