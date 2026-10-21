use glam::{Mat4, Vec3};
use world::MeshTarget;
use crate::horizon::ViewFrame;
use crate::posture::AvatarPosture;
use super::crown::CrownStyle;
use super::limb::LimbFrame;
use super::palette::{ComplexionSwatch, PigmentSwatch};
use super::pose::BodyPose;
use super::relic::MartialStyle;

/// Polymorphic camera sight mode switching between pure First-Person and Looking-Glass mirror/peer display.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SightMode {
    FirstPersonSight,
    LookingGlassSight,
}

impl SightMode {
    pub fn toggle(self) -> Self {
        match self {
            Self::FirstPersonSight => Self::LookingGlassSight,
            Self::LookingGlassSight => Self::FirstPersonSight,
        }
    }

    /// Resolves shoulder, elbow, wrist-aligned LimbFrame, and forearm length polymorphically; raises hand only when holding an item.
    pub fn arm_joints(self, pose: &BodyPose, side: f32, holding: bool, martial: MartialStyle) -> (Vec3, Vec3, LimbFrame, f32) {
        match self {
            Self::FirstPersonSight => {
                if holding {
                    pose.first_person_arm(side, martial)
                } else {
                    pose.external_arm(side, false, MartialStyle::Unarmed)
                }
            }
            Self::LookingGlassSight => pose.external_arm(side, holding, martial),
        }
    }

    pub fn emit_crown(
        self,
        crown: CrownStyle,
        pose: &BodyPose,
        complexion: ComplexionSwatch,
        pigment: PigmentSwatch,
        target: &mut dyn MeshTarget,
    ) {
        match self {
            Self::FirstPersonSight => {}
            Self::LookingGlassSight => crown.emit(pose, complexion, pigment, target),
        }
    }

    pub fn view_matrix(
        self,
        frame: &dyn ViewFrame,
        posture: AvatarPosture,
        position: Vec3,
        pose: &BodyPose,
    ) -> Mat4 {
        match self {
            Self::FirstPersonSight => posture.view_matrix(frame, position),
            Self::LookingGlassSight => {
                let torso = pose.stance_frame();
                let mirror_eye = torso.point(0.0, 1.20, 1.65);
                let target = torso.point(0.0, 0.96, 0.0);
                let up = torso.point(0.0, 1.0, 0.0) - torso.point(0.0, 0.0, 0.0);
                Mat4::look_at_rh(mirror_eye, target, up.normalize())
            }
        }
    }
}
