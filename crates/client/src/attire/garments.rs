use glam::Vec3;
use world::MeshTarget;
use super::limb::{FacetedSolid, TaperedPrism};
use super::palette::PigmentSwatch;
use super::pose::BodyPose;

/// Medieval upper-body tunic or jerkin style.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TunicStyle {
    LinenDoublet,
    QuiltedJerkin,
    PeasantShirt,
}

impl TunicStyle {
    pub fn next(self) -> Self {
        match self {
            Self::LinenDoublet => Self::QuiltedJerkin,
            Self::QuiltedJerkin => Self::PeasantShirt,
            Self::PeasantShirt => Self::LinenDoublet,
        }
    }

    pub fn from_code(code: u8) -> Self {
        match code % 3 {
            0 => Self::LinenDoublet,
            1 => Self::QuiltedJerkin,
            _ => Self::PeasantShirt,
        }
    }

    pub fn code(self) -> u8 {
        match self {
            Self::LinenDoublet => 0,
            Self::QuiltedJerkin => 1,
            Self::PeasantShirt => 2,
        }
    }

    /// Emits the flat-front athletic medieval torso with wide, First-Person-visible hips.
    pub fn emit(self, pose: &BodyPose, pigment: PigmentSwatch, target: &mut dyn MeshTarget) {
        let frame = pose.torso_frame();
        let crouch = pose.crouch_compression();
        let hip_y = 0.85 - crouch * 0.40;
        let waist_y = 1.08 - crouch * 0.52;
        let chest_y = 1.38 - crouch * 0.72;
        let sternum_y = 1.54 - crouch * 0.84;

        let cloth = pigment.cloth();
        let vest = Vec3::new(0.46, 0.28, 0.16);
        let (skirt_col, chest_col) = match self {
            Self::LinenDoublet => (cloth, cloth),
            Self::QuiltedJerkin => (vest, vest),
            Self::PeasantShirt => (cloth, cloth * 0.92),
        };

        // Wide-flared tunic skirt from broad hip sockets (0.85m) to waist (1.08m), stepped forward so it is clearly visible when looking down
        TaperedPrism::new(
            8,
            Vec3::new(0.0, hip_y, -0.022),
            Vec3::new(0.0, waist_y, -0.048),
            pose.scale(0.168),
            0.060,
            pose.scale(0.106),
            0.044,
            skirt_col,
        )
        .emit(&frame, false, target);

        // Flat-front ribcage recessed behind chin line so it never eclipses the waist or hips from First-Person camera
        TaperedPrism::new(
            8,
            Vec3::new(0.0, waist_y, -0.048),
            Vec3::new(0.0, chest_y, -0.072),
            pose.scale(0.106),
            0.044,
            pose.scale(0.118),
            0.044,
            chest_col,
        )
        .emit(&frame, false, target);

        // Upper sternum and collarbone yoke recessed under the neck
        TaperedPrism::new(
            8,
            Vec3::new(0.0, chest_y, -0.072),
            Vec3::new(0.0, sternum_y, -0.088),
            pose.scale(0.118),
            0.044,
            pose.scale(0.098),
            0.038,
            chest_col,
        )
        .emit(&frame, true, target);
    }
}

/// Medieval waist belt and pouch style.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GirdleStyle {
    KnottedCord,
    PouchedGirdle,
    UnbeltedWaist,
}

impl GirdleStyle {
    pub fn next(self) -> Self {
        match self {
            Self::KnottedCord => Self::PouchedGirdle,
            Self::PouchedGirdle => Self::UnbeltedWaist,
            Self::UnbeltedWaist => Self::KnottedCord,
        }
    }

    pub fn from_code(code: u8) -> Self {
        match code % 3 {
            0 => Self::KnottedCord,
            1 => Self::PouchedGirdle,
            _ => Self::UnbeltedWaist,
        }
    }

    pub fn code(self) -> u8 {
        match self {
            Self::KnottedCord => 0,
            Self::PouchedGirdle => 1,
            Self::UnbeltedWaist => 2,
        }
    }

    /// Emits the waist belt and optional leather pouch polymorphically according to this girdle style.
    pub fn emit(self, pose: &BodyPose, target: &mut dyn MeshTarget) {
        let frame = pose.torso_frame();
        let waist_y = 1.08 - pose.crouch_compression() * 0.52;
        let hide = Vec3::new(0.34, 0.21, 0.12);
        let vest = Vec3::new(0.46, 0.28, 0.16);
        let rx = pose.scale(0.114);

        match self {
            Self::UnbeltedWaist => {}
            Self::KnottedCord => {
                TaperedPrism::new(
                    8,
                    Vec3::new(0.0, waist_y - 0.022, -0.046),
                    Vec3::new(0.0, waist_y + 0.022, -0.046),
                    rx,
                    0.048,
                    rx,
                    0.048,
                    hide,
                )
                .emit(&frame, false, target);
            }
            Self::PouchedGirdle => {
                TaperedPrism::new(
                    8,
                    Vec3::new(0.0, waist_y - 0.022, -0.046),
                    Vec3::new(0.0, waist_y + 0.022, -0.046),
                    rx,
                    0.048,
                    rx,
                    0.048,
                    hide,
                )
                .emit(&frame, false, target);
                let px = pose.scale(0.122);
                TaperedPrism::hexagonal(
                    Vec3::new(px, waist_y - 0.06, -0.022),
                    Vec3::new(px, waist_y + 0.01, -0.022),
                    0.032,
                    0.025,
                    vest,
                )
                .emit(&frame, true, target);
            }
        }
    }
}
