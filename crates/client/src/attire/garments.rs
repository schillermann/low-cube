use glam::Vec3;
use world::MeshTarget;
use super::accoutrement::PolygonAccoutrement;
use super::limb::{FacetedSolid, TaperedPrism};
use super::palette::PigmentSwatch;
use super::pose::BodyPose;

/// Polygon-style upper-body tunic, jerkin, cuirass, robe, or tabard style.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TunicStyle {
    LinenDoublet,
    QuiltedJerkin,
    PeasantShirt,
    PlateCuirass,
    ForgeApron,
    HeraldicTabard,
}

impl TunicStyle {
    pub fn next(self) -> Self {
        Self::from_code(self.code() + 1)
    }

    pub fn from_code(code: u8) -> Self {
        match code % 6 {
            0 => Self::LinenDoublet,
            1 => Self::QuiltedJerkin,
            2 => Self::PeasantShirt,
            3 => Self::PlateCuirass,
            4 => Self::ForgeApron,
            _ => Self::HeraldicTabard,
        }
    }

    pub fn code(self) -> u8 {
        match self {
            Self::LinenDoublet => 0,
            Self::QuiltedJerkin => 1,
            Self::PeasantShirt => 2,
            Self::PlateCuirass => 3,
            Self::ForgeApron => 4,
            Self::HeraldicTabard => 5,
        }
    }

    /// Emits the full-volume 3D athletic torso, flared tunic skirt, shoulder caps, and Polygon-style body details.
    pub fn emit(self, pose: &BodyPose, pigment: PigmentSwatch, target: &mut dyn MeshTarget) {
        let frame = pose.torso_frame();
        let hip_y = pose.hip_elevation();
        let waist_y = pose.torso_span(0.26);
        let chest_y = pose.torso_span(0.54);
        let sternum_y = pose.torso_span(0.72);

        let cloth = pigment.cloth();
        let trim = pigment.trim();
        let vest = Vec3::new(0.46, 0.28, 0.16);
        let steel = Vec3::new(0.58, 0.62, 0.68);
        let (skirt_col, chest_col, cap_col) = match self {
            Self::LinenDoublet => (cloth, cloth, trim),
            Self::QuiltedJerkin => (vest, vest, cloth),
            Self::PeasantShirt => (cloth, cloth * 0.94, trim),
            Self::PlateCuirass => (cloth, steel, steel),
            Self::ForgeApron => (vest, cloth, vest),
            Self::HeraldicTabard => (trim, cloth, steel),
        };

        let fem = pose.feminine();
        let (skirt_y0, hip_rx, hip_rz, waist_rx, waist_rz, chest_rx, sternum_rx) = if fem {
            (hip_y - 0.24, pose.scale(0.242), pose.torso_depth(0.158), pose.scale(0.104), pose.torso_depth(0.080), pose.scale(0.184), pose.scale(0.178))
        } else {
            (hip_y, pose.scale(0.204), pose.torso_depth(0.126), pose.scale(0.148), pose.torso_depth(0.100), pose.scale(0.215), pose.scale(0.236))
        };

        // Wide-flared tunic or A-line feminine dress skirt draping over the pelvis and thighs
        TaperedPrism::new(8, Vec3::new(0.0, skirt_y0, -0.030), Vec3::new(0.0, waist_y, -0.048), hip_rx, hip_rz, waist_rx, waist_rz, skirt_col)
            .emit(&frame, false, target);

        // Athletic or feminine hourglass lower and mid ribcage broadening into the chest
        TaperedPrism::new(8, Vec3::new(0.0, waist_y, -0.048), Vec3::new(0.0, chest_y, -0.064), waist_rx * 0.98, waist_rz * 0.98, chest_rx, pose.torso_depth(0.118), chest_col)
            .emit(&frame, false, target);

        // Upper back and collarbone shoulder yoke recessed behind the forward camera eye
        TaperedPrism::new(8, Vec3::new(0.0, chest_y, -0.064), Vec3::new(0.0, sternum_y, -0.076), chest_rx, pose.torso_depth(0.118), sternum_rx, pose.torso_depth(0.106), chest_col)
            .emit(&frame, true, target);

        if fem {
            TaperedPrism::new(8, Vec3::new(0.0, chest_y - pose.torso_size(0.09), 0.032), Vec3::new(0.0, chest_y + pose.torso_size(0.05), 0.022), pose.scale(0.152), pose.torso_depth(0.076), pose.scale(0.162), pose.torso_depth(0.088), chest_col)
                .emit(&frame, true, target);
            for side in [-1.0f32, 1.0f32] {
                let bx = side * pose.scale(0.066);
                TaperedPrism::new(6, Vec3::new(bx, chest_y - pose.torso_size(0.07), 0.052), Vec3::new(bx, chest_y + pose.torso_size(0.05), 0.042), pose.scale(0.078), pose.torso_depth(0.068), pose.scale(0.068), pose.torso_depth(0.058), chest_col)
                    .emit(&frame, true, target);
            }
        }

        // Left and right tailored medieval shoulder caps / deltoid yokes
        for side in [-1.0f32, 1.0f32] {
            let sx = side * pose.scale(if fem { 0.178 } else { 0.226 });
            TaperedPrism::new(
                6,
                Vec3::new(sx, sternum_y - pose.torso_size(0.12), -0.066),
                Vec3::new(sx * 0.95, sternum_y - pose.torso_size(0.008), -0.066),
                pose.scale(if fem { 0.056 } else { 0.076 }),
                pose.torso_depth(0.078),
                pose.scale(if fem { 0.048 } else { 0.066 }),
                pose.torso_depth(0.068),
                cap_col,
            )
            .emit(&frame, true, target);
        }

        PolygonAccoutrement::emit_tunic_details(self, pose, &frame, pigment, target);
    }
}

/// Polygon-style waist belt, pouch, quiver, backpack, or satchel style.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GirdleStyle {
    KnottedCord,
    PouchedGirdle,
    UnbeltedWaist,
    KnightlyBelt,
    ForgeBelt,
    GarrisonBelt,
}

impl GirdleStyle {
    pub fn next(self) -> Self {
        Self::from_code(self.code() + 1)
    }

    pub fn from_code(code: u8) -> Self {
        match code % 6 {
            0 => Self::KnottedCord,
            1 => Self::PouchedGirdle,
            2 => Self::UnbeltedWaist,
            3 => Self::KnightlyBelt,
            4 => Self::ForgeBelt,
            _ => Self::GarrisonBelt,
        }
    }

    pub fn code(self) -> u8 {
        match self {
            Self::KnottedCord => 0,
            Self::PouchedGirdle => 1,
            Self::UnbeltedWaist => 2,
            Self::KnightlyBelt => 3,
            Self::ForgeBelt => 4,
            Self::GarrisonBelt => 5,
        }
    }

    /// Emits the 3D waist belt, brass buckle, pouches, and Polygon-style back/hip gear.
    pub fn emit(self, pose: &BodyPose, target: &mut dyn MeshTarget) {
        let frame = pose.torso_frame();
        let waist_y = pose.torso_span(0.26);
        let hide = Vec3::new(0.34, 0.21, 0.12);
        let brass = Vec3::new(0.78, 0.62, 0.24);
        let vest = Vec3::new(0.46, 0.28, 0.16);
        let rx = pose.scale(if pose.feminine() { 0.112 } else { 0.156 });
        let rz = pose.torso_depth(if pose.feminine() { 0.084 } else { 0.104 });
        let buckle_z = -0.048 + rz - 0.004;

        TaperedPrism::new(8, Vec3::new(0.0, waist_y - 0.025, -0.048), Vec3::new(0.0, waist_y + 0.025, -0.048), rx, rz, rx, rz, hide)
            .emit(&frame, false, target);
        TaperedPrism::new(4, Vec3::new(0.0, waist_y - 0.028, buckle_z), Vec3::new(0.0, waist_y + 0.028, buckle_z), 0.028, 0.015, 0.028, 0.015, brass)
            .emit(&frame, true, target);

        if matches!(self, Self::PouchedGirdle | Self::GarrisonBelt) {
            let px = pose.scale(0.158);
            TaperedPrism::hexagonal(Vec3::new(px, waist_y - 0.08, -0.008), Vec3::new(px, waist_y + 0.01, -0.008), 0.038, 0.028, vest)
                .emit(&frame, true, target);
        }

        PolygonAccoutrement::emit_girdle_details(self, pose, &frame, target);
    }
}
