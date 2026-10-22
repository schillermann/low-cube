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
    /// Resolves shoulder, elbow, wrist-aligned LimbFrame, and forearm length polymorphically; frames open satchel without overlap.
    pub fn arm_joints(
        self,
        pose: &BodyPose,
        side: f32,
        holding: bool,
        martial: MartialStyle,
        satchel_open: bool,
    ) -> (Vec3, Vec3, LimbFrame, f32) {
        match self {
            Self::FirstPersonSight => {
                if satchel_open && holding {
                    let vf = pose.view_frame();
                    let cradle = matches!(martial, MartialStyle::PalmCradle);
                    let raw = vf.shifted(side * 0.420, -0.260, 0.560).yawed(-side * 0.05);
                    let shoulder_w = vf.point(side * 0.500, -0.560, 0.420);
                    let elbow_w = raw.point(side * 0.032, -0.155, -0.140);
                    let wrist_w = raw.point(0.0, 0.0, 0.0);
                    let hand_dir = (wrist_w - raw.point(side * 0.010, 0.030, -0.220)).normalize();
                    let (rx, ry) = if cradle { (0.0, -side) } else { (1.0, -side * 0.06) };
                    let hand_up = (raw.point(rx, ry, 0.0) - wrist_w).cross(hand_dir).normalize();
                    let frame = LimbFrame::new(wrist_w, hand_dir.cross(hand_up).normalize(), hand_up, hand_dir)
                        .pitched(if cradle { -0.16 } else { 0.0 });
                    (shoulder_w, elbow_w, frame, (wrist_w - elbow_w).length())
                } else if holding || pose.striking_on(side) {
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
