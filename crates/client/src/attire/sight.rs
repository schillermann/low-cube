use glam::{Mat4, Quat, Vec3};
use world::{MeshTarget, MoldArchetype};
use crate::horizon::ViewFrame;
use crate::posture::AvatarPosture;
use super::crown::CrownStyle;
use super::limb::LimbFrame;
use super::palette::{ComplexionSwatch, PigmentSwatch};
use super::pose::BodyPose;
use super::relic::MartialStyle;

/// Polymorphic camera sight mode switching between First-Person and external Front, Angled, and Side views.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SightMode {
    FirstPersonSight,
    ForwardAngledSight,
    ProfileSight,
    AngledSight,
    LookingGlassSight,
    LeftAngledSight,
    LeftProfileSight,
    LeftForwardAngledSight,
}

/// Polymorphic shutter state for diegetic field camera exposures.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PhotoShutter {
    Idle,
    Triggered(SightMode),
}

/// Polymorphic field-camera mount state: unmounted (Null Object), handheld (instant shutter), or placed on a block (6s self-timer).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CameraMount {
    Unmounted,
    Handheld(SightMode),
    Placed(SightMode),
}

/// World-space tripod lens anchor for a field camera placed on a block.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TripodMount {
    Unmounted,
    Mounted {
        hub: Vec3,
        base_fwd: Vec3,
        up: Vec3,
        turn: u8,
    },
}

impl TripodMount {
    pub const fn placed_sight(self) -> Option<SightMode> {
        match self {
            Self::Unmounted => None,
            Self::Mounted { turn, .. } => Some(SightMode::from_placed_camera_turn(turn)),
        }
    }

    pub fn map_mold(self, op: fn(world::BlockMold) -> world::BlockMold) -> Self {
        match self {
            Self::Unmounted => Self::Unmounted,
            Self::Mounted { hub, base_fwd, up, turn } => {
                let mold = world::BlockMold::with_medium(
                    MoldArchetype::from_code(turn),
                    world::CardinalTurn::Forward,
                    world::BlockMedium::FieldCamera,
                );
                Self::Mounted {
                    hub,
                    base_fwd,
                    up,
                    turn: op(mold).camera_turn(),
                }
            }
        }
    }

    pub fn lens_ray(self) -> Option<(Vec3, Vec3, Vec3)> {
        match self {
            Self::Unmounted => None,
            Self::Mounted { hub, base_fwd, up, turn } => {
                let yaw = MoldArchetype::from_code(turn).placed_camera_yaw();
                let lens_dir = (Quat::from_axis_angle(up, yaw) * base_fwd).normalize_or_zero();
                let lens_eye = hub + lens_dir * 0.16 + up * 0.015;
                Some((lens_eye, lens_dir, up))
            }
        }
    }
}

impl SightMode {
    pub const fn from_camera_turn(camera_turn: u8) -> Self {
        match camera_turn % 8 {
            0 => Self::FirstPersonSight,
            1 => Self::ForwardAngledSight,
            2 => Self::ProfileSight,
            3 => Self::AngledSight,
            4 => Self::LookingGlassSight,
            5 => Self::LeftAngledSight,
            6 => Self::LeftProfileSight,
            _ => Self::LeftForwardAngledSight,
        }
    }

    pub const fn from_placed_camera_turn(camera_turn: u8) -> Self {
        match camera_turn % 8 {
            0 | 4 => Self::LookingGlassSight,
            1 => Self::ForwardAngledSight,
            2 => Self::ProfileSight,
            3 => Self::AngledSight,
            5 => Self::LeftAngledSight,
            6 => Self::LeftProfileSight,
            _ => Self::LeftForwardAngledSight,
        }
    }

    pub const fn photo_label(self) -> &'static str {
        match self {
            Self::FirstPersonSight => "landscape",
            Self::LookingGlassSight => "front",
            Self::ForwardAngledSight
            | Self::LeftForwardAngledSight
            | Self::AngledSight
            | Self::LeftAngledSight => "angled",
            Self::ProfileSight | Self::LeftProfileSight => "side",
        }
    }

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
                } else if (holding || pose.striking_on(side)) && pose.driving_factor() < 0.5 {
                    pose.arm(side, martial).first_person()
                } else {
                    pose.arm(side, MartialStyle::Unarmed).external(false)
                }
            }
            _ => pose.arm(side, martial).external(holding),
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
        if !matches!(self, Self::FirstPersonSight) {
            crown.emit(pose, complexion, pigment, target);
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
            _ => {
                let torso = pose.stance_frame();
                let up = (torso.point(0.0, 1.0, 0.0) - torso.point(0.0, 0.0, 0.0)).normalize();
                let (hand_pitch, crouch_shift) = pose.held_camera_elevation();
                let eye_y = (1.35 + hand_pitch * 0.90 + crouch_shift * 0.80).clamp(0.35, 2.30);
                let target_y = (1.26 + hand_pitch * 0.30 + crouch_shift * 0.75).clamp(0.80, 1.65);
                let dist = 1.85;
                let diag = dist * 0.72;
                let eye = match self {
                    Self::LookingGlassSight => torso.point(0.0, eye_y, dist),
                    Self::ForwardAngledSight => torso.point(diag, eye_y, -diag),
                    Self::ProfileSight => torso.point(dist, eye_y, 0.0),
                    Self::AngledSight => torso.point(diag, eye_y, diag),
                    Self::LeftAngledSight => torso.point(-diag, eye_y, diag),
                    Self::LeftProfileSight => torso.point(-dist, eye_y, 0.0),
                    _ => torso.point(-diag, eye_y, -diag),
                };
                let target = torso.point(0.0, target_y, 0.0);
                Mat4::look_at_rh(eye, target, up)
            }
        }
    }
}
