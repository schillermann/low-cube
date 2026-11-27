use std::f32::consts::TAU;
use glam::{Mat4, Quat, Vec3};
use world::MeshTarget;
use crate::attire::{
    BodyPose, CrewPhysique, EvaAttire, FacetedSolid, HeldRelic, LimbFrame, RocketControls,
    SightMode, StrideRhythm, TaperedPrism, Wardrobe,
};

/// Available crew EVA animations automatically demonstrated in sequence by the center studio figure.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StudioAnimation {
    StandingIdle,
    WalkingStride,
    RunningSprint,
    CrouchingSneak,
    SeatedDrive,
    RocketPilot,
    JumpingLeap,
    RigTorque,
    RadioCall,
    PalmDig,
}

impl StudioAnimation {
    pub fn sequence() -> [Self; 10] {
        [
            Self::StandingIdle,
            Self::WalkingStride,
            Self::RunningSprint,
            Self::CrouchingSneak,
            Self::SeatedDrive,
            Self::RocketPilot,
            Self::JumpingLeap,
            Self::RigTorque,
            Self::RadioCall,
            Self::PalmDig,
        ]
    }

    pub const fn title(self) -> &'static str {
        match self {
            Self::StandingIdle => "IDLE STAND",
            Self::WalkingStride => "WALKING",
            Self::RunningSprint => "SPRINTING",
            Self::CrouchingSneak => "SNEAKING",
            Self::SeatedDrive => "SEATED DRIVE",
            Self::RocketPilot => "ROCKET PILOT",
            Self::JumpingLeap => "JUMP LEAP",
            Self::RigTorque => "RIG WRENCH",
            Self::RadioCall => "SUIT RADIO",
            Self::PalmDig => "GLOVE DIG",
        }
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
        attire: &EvaAttire,
        target: &mut dyn MeshTarget,
    ) {
        let (lift, eye_height, seated, rhythm, staged_attire) = match self {
            Self::StandingIdle => (0.0, 0.80, 0.0, StrideRhythm::new((elapsed * 1.8) % TAU, 0.0), *attire),
            Self::WalkingStride => (0.0, 0.80, 0.0, StrideRhythm::new((elapsed * 8.4) % TAU, 0.45), *attire),
            Self::RunningSprint => (0.0, 0.80, 0.0, StrideRhythm::new((elapsed * 14.2) % TAU, 1.0), *attire),
            Self::CrouchingSneak => {
                let s = ((elapsed % 2.2) / 2.2 * std::f32::consts::PI).sin();
                (0.0, 0.80 - 0.44 * s * s, 0.0, StrideRhythm::new((elapsed * 5.4) % TAU, 0.38), *attire)
            }
            Self::SeatedDrive => (0.0, 0.46, 1.0, StrideRhythm::new((elapsed * 5.2) % TAU, 0.35), *attire),
            Self::RocketPilot => {
                let bare = attire.with_relics(HeldRelic::EmptyPalm, HeldRelic::EmptyPalm);
                (0.0, 0.46, 0.0, StrideRhythm::new((elapsed * 3.6) % TAU, 0.0), bare)
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
                (jump_lift, eye, 0.0, StrideRhythm::idle(), *attire)
            }
            Self::RigTorque => {
                let cleave = (elapsed * 1.15).fract();
                let armed = if attire.armed() { *attire } else { attire.with_relics(HeldRelic::RigWrench, HeldRelic::EmptyPalm) };
                (0.0, 0.80, 0.0, StrideRhythm::with_cleave((elapsed * 3.2) % TAU, 0.25, cleave), armed)
            }
            Self::RadioCall => {
                let cleave = (elapsed * 1.15).fract();
                let armed = if attire.armed() { *attire } else { attire.with_relics(HeldRelic::SuitRadio, HeldRelic::EmptyPalm) };
                (0.0, 0.80, 0.0, StrideRhythm::with_cleave((elapsed * 3.2) % TAU, 0.25, cleave), armed)
            }
            Self::PalmDig => {
                let cleave = (elapsed * 1.15).fract();
                let bare = attire.with_relics(HeldRelic::EmptyPalm, HeldRelic::EmptyPalm);
                (0.0, 0.56, 0.0, StrideRhythm::with_cleave((elapsed * 3.2) % TAU, 0.15, cleave), bare)
            }
        };

        let view_rot = Quat::from_rotation_y(std::f32::consts::PI + yaw_radians);
        let raw_pose = BodyPose::new(
            position + Vec3::Y * lift,
            view_rot,
            view_rot,
            eye_height,
            rhythm,
            CrewPhysique::OrbitalCadet,
        );
        let pose = if matches!(self, Self::RocketPilot) {
            raw_pose.with_rocket_controls(RocketControls::studio_showcase((elapsed % 2.2) / 2.2))
        } else if seated > 0.0 {
            raw_pose.with_seated(seated)
        } else {
            raw_pose
        };
        staged_attire.emit(&pose, SightMode::LookingGlassSight, target);
    }
}

/// Interactive 3D turntable stage displaying a single centered crew figure cycling through all animations.
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

    pub fn active_animation(&self) -> StudioAnimation {
        StudioAnimation::from_elapsed(self.elapsed)
    }

    pub fn next_animation(self) -> Self {
        let next_idx = (((self.elapsed / 2.2).max(0.0) as usize) + 1) % StudioAnimation::sequence().len();
        Self::new(self.yaw_radians, (next_idx as f32) * 2.2 + 0.35)
    }

    pub fn view_projection(&self, aspect: f32) -> Mat4 {
        let proj = Mat4::perspective_rh(45.0f32.to_radians(), aspect.max(0.5), 0.1, 30.0);
        let view = Mat4::look_at_rh(
            Vec3::new(0.0, 1.05, 4.65),
            Vec3::new(0.0, 1.02, 0.0),
            Vec3::Y,
        );
        proj * view
    }

    /// Emits only the single rotatable crew figure on the central turntable pedestal.
    pub fn emit_stage(&self, attire: &EvaAttire, target: &mut dyn MeshTarget) {
        let world_frame = LimbFrame::new(Vec3::ZERO, Vec3::X, Vec3::Y, Vec3::Z);
        let alloy = Vec3::new(0.28, 0.32, 0.38);
        let rim = Vec3::new(0.28, 0.72, 0.88);

        TaperedPrism::new(8, Vec3::new(0.0, -0.09, 0.0), Vec3::new(0.0, 0.015, 0.0), 0.56, 0.56, 0.50, 0.50, alloy)
            .emit(&world_frame, true, target);
        TaperedPrism::new(8, Vec3::new(0.0, -0.02, 0.0), Vec3::new(0.0, 0.022, 0.0), 0.52, 0.52, 0.47, 0.47, rim)
            .emit(&world_frame, true, target);

        let active_anim = StudioAnimation::from_elapsed(self.elapsed);
        active_anim.emit_figure(Vec3::new(0.0, 1.0, 0.0), self.yaw_radians, self.elapsed, attire, target);
    }
}
