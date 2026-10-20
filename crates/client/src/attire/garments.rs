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

    /// Emits the full-volume 3D athletic medieval torso, flared tunic skirt, and tailored shoulder caps.
    pub fn emit(self, pose: &BodyPose, pigment: PigmentSwatch, target: &mut dyn MeshTarget) {
        let frame = pose.torso_frame();
        let crouch = pose.crouch_compression();
        let hip_y = 0.80 - crouch * 0.38;
        let waist_y = 1.06 - crouch * 0.50;
        let chest_y = 1.34 - crouch * 0.68;
        let sternum_y = 1.52 - crouch * 0.80;

        let cloth = pigment.cloth();
        let trim = pigment.trim();
        let vest = Vec3::new(0.46, 0.28, 0.16);
        let (skirt_col, chest_col, cap_col) = match self {
            Self::LinenDoublet => (cloth, cloth, trim),
            Self::QuiltedJerkin => (vest, vest, cloth),
            Self::PeasantShirt => (cloth, cloth * 0.94, cloth * 0.88),
        };

        // Wide-flared tunic skirt draping cleanly over the pelvis and upper thighs without leg clipping
        TaperedPrism::new(
            8,
            Vec3::new(0.0, hip_y, -0.030),
            Vec3::new(0.0, waist_y, -0.048),
            pose.scale(0.204),
            0.126,
            pose.scale(0.148),
            0.100,
            skirt_col,
        )
        .emit(&frame, false, target);

        // Athletic V-taper lower and mid ribcage broadening strongly into the lats and chest
        TaperedPrism::new(
            8,
            Vec3::new(0.0, waist_y, -0.048),
            Vec3::new(0.0, chest_y, -0.064),
            pose.scale(0.146),
            0.098,
            pose.scale(0.215),
            0.120,
            chest_col,
        )
        .emit(&frame, false, target);

        // Broad upper back (Kreuz) and collarbone shoulder yoke recessed behind the forward camera eye
        TaperedPrism::new(
            8,
            Vec3::new(0.0, chest_y, -0.064),
            Vec3::new(0.0, sternum_y, -0.076),
            pose.scale(0.215),
            0.120,
            pose.scale(0.236),
            0.112,
            chest_col,
        )
        .emit(&frame, true, target);

        // Left and right tailored medieval shoulder caps / deltoid yokes
        for side in [-1.0f32, 1.0f32] {
            let sx = side * pose.scale(0.226);
            TaperedPrism::new(
                6,
                Vec3::new(sx, sternum_y - 0.12, -0.066),
                Vec3::new(sx * 0.95, sternum_y - 0.008, -0.066),
                pose.scale(0.076),
                0.082,
                pose.scale(0.066),
                0.072,
                cap_col,
            )
            .emit(&frame, true, target);
        }
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

    /// Emits the 3D waist belt, brass buckle, and optional leather pouch polymorphically.
    pub fn emit(self, pose: &BodyPose, target: &mut dyn MeshTarget) {
        let frame = pose.torso_frame();
        let waist_y = 1.06 - pose.crouch_compression() * 0.50;
        let hide = Vec3::new(0.34, 0.21, 0.12);
        let brass = Vec3::new(0.78, 0.62, 0.24);
        let vest = Vec3::new(0.46, 0.28, 0.16);
        let rx = pose.scale(0.156);
        let rz = 0.104;

        match self {
            Self::UnbeltedWaist => {}
            Self::KnottedCord => {
                TaperedPrism::new(8, Vec3::new(0.0, waist_y - 0.025, -0.048), Vec3::new(0.0, waist_y + 0.025, -0.048), rx, rz, rx, rz, hide)
                    .emit(&frame, false, target);
                TaperedPrism::new(4, Vec3::new(0.0, waist_y - 0.028, 0.052), Vec3::new(0.0, waist_y + 0.028, 0.052), 0.026, 0.014, 0.026, 0.014, brass)
                    .emit(&frame, true, target);
            }
            Self::PouchedGirdle => {
                TaperedPrism::new(8, Vec3::new(0.0, waist_y - 0.025, -0.048), Vec3::new(0.0, waist_y + 0.025, -0.048), rx, rz, rx, rz, hide)
                    .emit(&frame, false, target);
                TaperedPrism::new(4, Vec3::new(0.0, waist_y - 0.028, 0.052), Vec3::new(0.0, waist_y + 0.028, 0.052), 0.026, 0.014, 0.026, 0.014, brass)
                    .emit(&frame, true, target);
                let px = pose.scale(0.158);
                TaperedPrism::hexagonal(
                    Vec3::new(px, waist_y - 0.08, -0.008),
                    Vec3::new(px, waist_y + 0.01, -0.008),
                    0.038,
                    0.028,
                    vest,
                )
                .emit(&frame, true, target);
            }
        }
    }
}
