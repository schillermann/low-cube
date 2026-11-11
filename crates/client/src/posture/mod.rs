mod vital;

use glam::{Mat4, Quat, Vec3};
use crate::attire::{BodyPose, StrideRhythm, YouthPhysique};
use crate::horizon::ViewFrame;
pub use self::vital::VitalStrain;

const STANDING_EYE_OFFSET: f32 = 0.80;
const CROUCHING_EYE_OFFSET: f32 = 0.36;
const MAX_CROUCH_DROP: f32 = STANDING_EYE_OFFSET - CROUCHING_EYE_OFFSET;
const POSTURE_BLEND_RATE: f32 = 15.0;

/// Takeoff knee compression, airborne leg extension, and landing spring absorption.
#[derive(Clone, Copy, Debug)]
pub struct LeapSpring {
    flex: f32,
    air_timer: f32,
    land_timer: f32,
}

impl LeapSpring {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(flex: f32, air_timer: f32, land_timer: f32) -> Self {
        Self { flex, air_timer, land_timer }
    }

    pub const fn grounded() -> Self {
        Self::new(0.0, 0.0, 0.0)
    }

    pub const fn flex(self) -> f32 {
        self.flex
    }

    pub fn advance(self, previous: Vec3, next: Vec3, terrain_up: Vec3, dt: f32) -> Self {
        let v_up = if dt > 1e-5 { (next - previous).dot(terrain_up) / dt } else { 0.0 };
        let (target, rate, next_air, next_land) = if self.air_timer > 0.0 || v_up.abs() > 0.35 {
            let air_time = self.air_timer + dt;
            let landed = v_up.abs() <= 1e-3 && (air_time > 0.42 || self.flex > 0.08);
            if landed {
                (-0.40, 26.0, 0.0, 0.16)
            } else if air_time < 0.08 && v_up > 2.2 {
                (-0.38, 32.0, air_time, 0.0)
            } else {
                (0.36, 12.0, air_time, 0.0)
            }
        } else if self.land_timer > 0.0 {
            (-0.40, 26.0, 0.0, (self.land_timer - dt).max(0.0))
        } else {
            (0.0, 10.0, 0.0, 0.0)
        };
        let next_flex = self.flex + (target - self.flex) * (rate * dt).clamp(0.0, 1.0);
        Self::new(next_flex, next_air, next_land)
    }
}

/// Kinematic posture composing vertical eye elevation, leap spring compression, and vital core-hover state.
#[derive(Clone, Copy, Debug)]
pub struct AvatarPosture {
    elevation: f32,
    leap: LeapSpring,
    vital: VitalStrain,
}

impl AvatarPosture {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn with_components(elevation: f32, leap: LeapSpring, vital: VitalStrain) -> Self {
        Self { elevation, leap, vital }
    }

    pub const fn new(elevation: f32) -> Self {
        Self::with_components(elevation, LeapSpring::grounded(), VitalStrain::calm())
    }

    pub const fn standing() -> Self {
        Self::new(STANDING_EYE_OFFSET)
    }

    pub const fn crouching() -> Self {
        Self::new(CROUCHING_EYE_OFFSET)
    }

    pub fn crouch_down(&self, dt: f32) -> Self {
        self.transition_to(CROUCHING_EYE_OFFSET, dt)
    }

    pub fn crouch_drop(&self) -> f32 {
        (STANDING_EYE_OFFSET - self.eye_elevation()).clamp(0.0, MAX_CROUCH_DROP)
    }

    pub const fn eye_elevation(&self) -> f32 {
        self.elevation
    }

    pub fn perish(self) -> Self {
        Self::with_components(self.elevation, LeapSpring::grounded(), self.vital.perish())
    }

    pub const fn perished(&self) -> bool {
        self.vital.perished()
    }

    pub fn blackout_complete(&self) -> bool {
        self.vital.blackout_complete()
    }

    pub fn cardiac_constriction(&self) -> f32 {
        self.vital.cardiac_constriction()
    }

    pub fn cardiac_fov(&self) -> f32 {
        self.vital.cardiac_fov()
    }

    pub fn emit_reticle(&self, target: &mut dyn world::MeshTarget) {
        self.vital.emit_reticle(target);
    }

    pub fn advance_perish(self, dt: f32) -> (Self, bool) {
        let (next_vital, beat) = self.vital.advance_perish(dt);
        (Self::with_components(self.elevation, LeapSpring::grounded(), next_vital), beat)
    }

    pub fn transition_to(&self, target_elevation: f32, dt: f32) -> Self {
        if self.perished() {
            return *self;
        }
        let factor = (POSTURE_BLEND_RATE * dt).clamp(0.0, 1.0);
        let updated = self.elevation + (target_elevation - self.elevation) * factor;
        Self::with_components(updated, self.leap, self.vital)
    }

    pub fn advance_leap(self, previous: Vec3, next: Vec3, terrain_up: Vec3, dt: f32) -> Self {
        Self::with_components(self.elevation, self.leap.advance(previous, next, terrain_up, dt), self.vital)
    }

    pub fn view_matrix(&self, frame: &dyn ViewFrame, position: Vec3) -> Mat4 {
        frame.view_matrix_at(position, self.eye_elevation())
    }

    pub fn body_pose(&self, position: Vec3, surface_rot: Quat, view_rot: Quat, rhythm: StrideRhythm) -> BodyPose {
        BodyPose::new(position, surface_rot, view_rot, self.eye_elevation(), rhythm, YouthPhysique::SlenderSquire)
            .with_leap(self.leap.flex())
    }

    #[cfg(test)]
    pub fn crouched(&self) -> bool {
        self.elevation < 0.6
    }
}
