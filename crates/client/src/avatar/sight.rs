use glam::{Mat4, Quat, Vec3};
use crate::attire::{BodyPose, SightMode, StrideRhythm};
use crate::controls::KeyboardControls;
use crate::earthworks::SurfaceReach;
use crate::heading::SurfaceHeading;
use crate::horizon::{CameraHorizon, HorizonTilt};
use crate::lens::{PerspectiveLens, ProjectionLens};
use crate::order::LocomotionOrder;
use crate::orientation::{SensitivityLook, ViewOrientation};
use crate::posture::AvatarPosture;

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
            let rate = if posture.shock_active() { 18.0 } else { 8.0 };
            self.lens = self.lens.transition_with_rate(posture.cardiac_fov(), rate, dt);
        }
    }

    pub fn transition_perish_lens(&mut self, posture: AvatarPosture, dt: f32) {
        self.lens = self.lens.transition_to(posture.cardiac_fov(), dt);
    }

    pub fn constrain_shoulders(&mut self, wedged: bool, yaw_play: f32) {
        self.look.constrain_shoulders(wedged, yaw_play);
    }

    pub fn anchor_vehicle_heading(&mut self, heading: Vec3) {
        let up = self.surface_up();
        let flat = (heading - up * heading.dot(up)).normalize_or_zero();
        let base_fwd = (self.surface_rot * Vec3::NEG_Z).normalize_or_zero();
        let base_right = (self.surface_rot * Vec3::X).normalize_or_zero();
        let car_yaw = (-flat.dot(base_right)).atan2(flat.dot(base_fwd));
        self.look.anchor_heading(car_yaw, 1.42);
    }

    pub fn steer_vehicle(&mut self, steer_px: f32) {
        self.look.shift_anchor(steer_px);
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

    pub fn project_lens_view(&self, aspect: f32, eye: Vec3, forward: Vec3, up: Vec3) -> Mat4 {
        self.lens.projection_matrix(aspect) * Mat4::look_to_rh(eye, forward, up)
    }

    pub fn surface_reach(&self, pose_eye: Vec3, pos: Vec3, standing: bool, driving: bool) -> SurfaceReach {
        let (up, dir) = (self.surface_up(), self.view_direction());
        let d_up = dir.dot(up);
        let (eye, ray_dir) = if d_up > 0.975 {
            (pos + up * (pose_eye - pos).dot(up), up)
        } else if !standing && !driving && d_up < -0.815 {
            (pos + up * (pose_eye - pos).dot(up), -up)
        } else {
            (pose_eye, dir)
        };
        SurfaceReach::new(eye, ray_dir, pos, standing)
    }
}
