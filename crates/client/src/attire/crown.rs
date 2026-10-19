use glam::{Mat4, Vec3};
use world::MeshTarget;
use crate::horizon::ViewFrame;
use crate::posture::AvatarPosture;
use super::limb::{FacetedSolid, TaperedPrism};
use super::palette::{ComplexionSwatch, PigmentSwatch};
use super::pose::BodyPose;

/// Head, hair, and medieval hood/cap style visible to multiplayer peers and in Looking-Glass mode.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CrownStyle {
    TousledMop,
    SquireCrop,
    MedievalGugel,
    MerchantCap,
}

impl CrownStyle {
    pub fn next(self) -> Self {
        match self {
            Self::TousledMop => Self::SquireCrop,
            Self::SquireCrop => Self::MedievalGugel,
            Self::MedievalGugel => Self::MerchantCap,
            Self::MerchantCap => Self::TousledMop,
        }
    }

    pub fn from_code(code: u8) -> Self {
        match code % 4 {
            0 => Self::TousledMop,
            1 => Self::SquireCrop,
            2 => Self::MedievalGugel,
            _ => Self::MerchantCap,
        }
    }

    pub fn code(self) -> u8 {
        match self {
            Self::TousledMop => 0,
            Self::SquireCrop => 1,
            Self::MedievalGugel => 2,
            Self::MerchantCap => 3,
        }
    }

    /// Emits neck, youthful head, upper shoulders, and hair/headgear for multiplayer and mirror view.
    pub fn emit(
        self,
        pose: &BodyPose,
        complexion: ComplexionSwatch,
        pigment: PigmentSwatch,
        target: &mut dyn MeshTarget,
    ) {
        let frame = pose.crown_frame();
        let skin = complexion.skin();
        let hair_col = Vec3::new(0.28, 0.18, 0.11);
        let cloth = pigment.cloth();

        // Neck column connecting crown to recessed sternum
        TaperedPrism::hexagonal(
            Vec3::new(0.0, -0.15, -0.03),
            Vec3::new(0.0, 0.02, 0.0),
            pose.scale(0.050),
            pose.scale(0.044),
            skin,
        )
        .emit(&frame, false, target);

        // Faceted youthful head (jaw to forehead)
        TaperedPrism::new(
            8,
            Vec3::new(0.0, 0.01, 0.01),
            Vec3::new(0.0, 0.20, 0.01),
            0.075,
            0.082,
            0.092,
            0.098,
            skin,
        )
        .emit(&frame, true, target);

        // Collar trim on upper torso in observer/mirror view
        let torso = pose.torso_frame();
        let crouch = pose.crouch_compression();
        let collar_y = 1.54 - crouch * 0.84;
        TaperedPrism::hexagonal(
            Vec3::new(0.0, collar_y - 0.02, -0.086),
            Vec3::new(0.0, collar_y + 0.04, -0.078),
            pose.scale(0.092),
            pose.scale(0.066),
            pigment.trim(),
        )
        .emit(&torso, true, target);

        // Hair or medieval headgear
        match self {
            Self::TousledMop => {
                TaperedPrism::new(
                    8,
                    Vec3::new(0.0, 0.14, -0.005),
                    Vec3::new(0.0, 0.25, 0.01),
                    0.098,
                    0.105,
                    0.065,
                    0.075,
                    hair_col,
                )
                .emit(&frame, true, target);
            }
            Self::SquireCrop => {
                TaperedPrism::new(
                    8,
                    Vec3::new(0.0, 0.16, -0.005),
                    Vec3::new(0.0, 0.23, 0.0),
                    0.095,
                    0.100,
                    0.055,
                    0.060,
                    hair_col * 1.25,
                )
                .emit(&frame, true, target);
            }
            Self::MedievalGugel => {
                TaperedPrism::new(
                    8,
                    Vec3::new(0.0, -0.06, -0.01),
                    Vec3::new(0.0, 0.26, -0.02),
                    pose.scale(0.135),
                    0.115,
                    0.040,
                    0.065,
                    pigment.trim(),
                )
                .emit(&frame, true, target);
            }
            Self::MerchantCap => {
                TaperedPrism::new(
                    8,
                    Vec3::new(0.0, 0.17, 0.0),
                    Vec3::new(0.0, 0.26, -0.01),
                    0.105,
                    0.108,
                    0.085,
                    0.090,
                    cloth,
                )
                .emit(&frame, true, target);
            }
        }
    }
}

/// Polymorphic camera sight mode switching between pure First-Person and Looking-Glass mirror character creation.
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
                let torso = pose.torso_frame();
                let mirror_eye = torso.point(0.0, 1.20, 1.65);
                let target = torso.point(0.0, 0.96, 0.0);
                let up = torso.point(0.0, 1.0, 0.0) - torso.point(0.0, 0.0, 0.0);
                Mat4::look_at_rh(mirror_eye, target, up.normalize())
            }
        }
    }
}
