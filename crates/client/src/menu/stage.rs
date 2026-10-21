use std::f32::consts::TAU;
use glam::{Mat4, Quat, Vec3};
use world::MeshTarget;
use crate::attire::{
    BodyPose, FacetedSolid, HeldRelic, LimbFrame, SightMode, StrideRhythm, TaperedPrism, Wardrobe,
    YouthAttire, YouthPhysique,
};

/// Available character animations automatically demonstrated in sequence by the center studio figure.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StudioAnimation {
    StandingIdle,
    WalkingStride,
    RunningSprint,
    CrouchingSneak,
    JumpingLeap,
    AxeCleave,
    SwordSlash,
    SpadeDig,
    PalmDig,
}

impl StudioAnimation {
    pub fn sequence() -> [Self; 9] {
        [
            Self::StandingIdle,
            Self::WalkingStride,
            Self::RunningSprint,
            Self::CrouchingSneak,
            Self::JumpingLeap,
            Self::AxeCleave,
            Self::SwordSlash,
            Self::SpadeDig,
            Self::PalmDig,
        ]
    }

    pub fn from_elapsed(elapsed: f32) -> Self {
        let list = Self::sequence();
        let idx = ((elapsed / 2.2).max(0.0) as usize) % list.len();
        list[idx]
    }

    pub fn emit_figure(
        self,
        position: Vec3,
        yaw_radians: f32,
        elapsed: f32,
        attire: &YouthAttire,
        target: &mut dyn MeshTarget,
    ) {
        let (lift, eye_height, rhythm, staged_attire) = match self {
            Self::StandingIdle => (
                0.0,
                0.80,
                StrideRhythm::new((elapsed * 1.8) % TAU, 0.0),
                *attire,
            ),
            Self::WalkingStride => (
                0.0,
                0.80,
                StrideRhythm::new((elapsed * 8.4) % TAU, 0.45),
                *attire,
            ),
            Self::RunningSprint => (
                0.0,
                0.80,
                StrideRhythm::new((elapsed * 14.2) % TAU, 1.0),
                *attire,
            ),
            Self::CrouchingSneak => {
                let s = ((elapsed % 2.2) / 2.2 * std::f32::consts::PI).sin();
                (
                    0.0,
                    0.80 - 0.44 * s * s,
                    StrideRhythm::new((elapsed * 5.4) % TAU, 0.38),
                    *attire,
                )
            }
            Self::JumpingLeap => {
                let u = (elapsed % 2.2) / 2.2;
                let (jump_lift, eye) = if u < 0.08 {
                    (0.0, 0.80)
                } else if u < 0.32 {
                    let s = ((u - 0.08) / 0.24 * std::f32::consts::PI).sin();
                    (0.0, 0.80 - 0.44 * s * s)
                } else if u < 0.72 {
                    let air = ((u - 0.32) / 0.40 * std::f32::consts::PI).sin();
                    (0.55 * air, 0.80 + 0.36 * air * air)
                } else {
                    let s = ((u - 0.72) / 0.28 * std::f32::consts::PI).sin();
                    (0.0, 0.80 - 0.40 * s * s)
                };
                (jump_lift, eye, StrideRhythm::idle(), *attire)
            }
            Self::AxeCleave => {
                let cleave = (elapsed * 1.15).fract();
                let armed = if attire.armed() {
                    *attire
                } else {
                    attire.with_relics(HeldRelic::WoodmanAxe, HeldRelic::EmptyPalm)
                };
                (0.0, 0.80, StrideRhythm::with_cleave((elapsed * 3.2) % TAU, 0.25, cleave), armed)
            }
            Self::SwordSlash => {
                let cleave = (elapsed * 1.15).fract();
                let armed = if attire.armed() {
                    *attire
                } else {
                    attire.with_relics(HeldRelic::KnightlySword, HeldRelic::EmptyPalm)
                };
                (0.0, 0.80, StrideRhythm::with_cleave((elapsed * 3.2) % TAU, 0.25, cleave), armed)
            }
            Self::SpadeDig => {
                let cleave = (elapsed * 1.15).fract();
                let armed = if attire.armed() { *attire } else { attire.with_relics(HeldRelic::IronSpade, HeldRelic::EmptyPalm) };
                (0.0, 0.68, StrideRhythm::with_dual_cleave((elapsed * 3.2) % TAU, 0.15, cleave, 0.0), armed)
            }
            Self::PalmDig => {
                let cleave = (elapsed * 1.15).fract();
                let bare = attire.with_relics(HeldRelic::EmptyPalm, HeldRelic::EmptyPalm);
                (0.0, 0.56, StrideRhythm::with_cleave((elapsed * 3.2) % TAU, 0.15, cleave), bare)
            }
        };

        let view_rot = Quat::from_rotation_y(std::f32::consts::PI + yaw_radians);
        let pose = BodyPose::new(
            position + Vec3::Y * lift,
            Quat::IDENTITY,
            view_rot,
            eye_height,
            rhythm,
            YouthPhysique::SlenderSquire,
        );
        staged_attire.emit(&pose, SightMode::LookingGlassSight, target);
    }
}

/// Interactive 3D turntable stage displaying a single centered character cycling through all animations.
#[derive(Clone, Copy, Debug)]
pub struct TurntableStage {
    yaw_radians: f32,
    elapsed: f32,
}

impl TurntableStage {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(yaw_radians: f32, elapsed: f32) -> Self {
        Self { yaw_radians, elapsed }
    }

    pub const fn front_showcase() -> Self {
        Self::new(0.0, 0.0)
    }

    pub fn step(self, dt: f32) -> Self {
        Self::new(self.yaw_radians, self.elapsed + dt)
    }

    pub fn rotated(self, delta_yaw: f32) -> Self {
        Self::new((self.yaw_radians + delta_yaw) % TAU, self.elapsed)
    }

    pub fn reset_angle(self) -> Self {
        Self::new(0.0, self.elapsed)
    }

    pub fn view_projection(&self, aspect: f32) -> Mat4 {
        let proj = Mat4::perspective_rh_gl(45.0f32.to_radians(), aspect.max(0.5), 0.1, 30.0);
        let view = Mat4::look_at_rh(
            Vec3::new(0.0, 1.05, 4.65),
            Vec3::new(0.0, 1.02, 0.0),
            Vec3::Y,
        );
        proj * view
    }

    /// Emits only the single rotatable character on the central turntable pedestal cycling through all animations.
    pub fn emit_stage(&self, attire: &YouthAttire, target: &mut dyn MeshTarget) {
        let world_frame = LimbFrame::new(Vec3::ZERO, Vec3::X, Vec3::Y, Vec3::Z);
        let stone = Vec3::new(0.30, 0.33, 0.38);
        let rim = Vec3::new(0.54, 0.48, 0.34);

        TaperedPrism::new(8, Vec3::new(0.0, -0.09, 0.0), Vec3::new(0.0, 0.015, 0.0), 0.56, 0.56, 0.50, 0.50, stone)
            .emit(&world_frame, true, target);
        TaperedPrism::new(8, Vec3::new(0.0, -0.02, 0.0), Vec3::new(0.0, 0.022, 0.0), 0.52, 0.52, 0.47, 0.47, rim)
            .emit(&world_frame, true, target);

        let active_anim = StudioAnimation::from_elapsed(self.elapsed);
        active_anim.emit_figure(Vec3::new(0.0, 1.0, 0.0), self.yaw_radians, self.elapsed, attire, target);
    }
}
