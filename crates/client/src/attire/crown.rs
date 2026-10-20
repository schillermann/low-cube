use glam::Vec3;
use world::MeshTarget;
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

    /// Emits neck, sculpted 3-tier youthful head, facial features, collar, and hair/headgear.
    pub fn emit(
        self,
        pose: &BodyPose,
        complexion: ComplexionSwatch,
        pigment: PigmentSwatch,
        target: &mut dyn MeshTarget,
    ) {
        let frame = pose.crown_frame();
        let skin = complexion.skin();
        let shade_skin = skin * 0.91;
        let lip_col = skin * 0.75;
        let eye_col = Vec3::new(0.13, 0.11, 0.10);
        let hair_col = Vec3::new(0.28, 0.18, 0.11);
        let cloth = pigment.cloth();
        let trim = pigment.trim();

        // Neck column connecting skull base to upper torso collar
        TaperedPrism::hexagonal(
            Vec3::new(0.0, -0.15, -0.022),
            Vec3::new(0.0, 0.02, -0.008),
            pose.scale(0.056),
            pose.scale(0.050),
            shade_skin,
        )
        .emit(&frame, false, target);

        // Tier 1: Sculpted chin and jawline tapering upward to cheekbones
        TaperedPrism::new(8, Vec3::new(0.0, -0.015, 0.016), Vec3::new(0.0, 0.065, 0.008), 0.054, 0.066, 0.086, 0.090, skin)
            .emit(&frame, false, target);
        // Tier 2: Cheekbones and mid-face rising to brow ridge
        TaperedPrism::new(8, Vec3::new(0.0, 0.065, 0.008), Vec3::new(0.0, 0.145, 0.008), 0.086, 0.090, 0.088, 0.092, skin)
            .emit(&frame, false, target);
        // Tier 3: Forehead rounding inward into the upper cranium so hair/caps fully enclose the back crown
        TaperedPrism::new(8, Vec3::new(0.0, 0.145, 0.008), Vec3::new(0.0, 0.195, 0.008), 0.088, 0.092, 0.064, 0.068, skin)
            .emit(&frame, true, target);

        // Sculpted nose bridge and tip on the front (+Z) of the face
        TaperedPrism::new(4, Vec3::new(0.0, 0.064, 0.096), Vec3::new(0.0, 0.118, 0.090), 0.016, 0.018, 0.010, 0.012, shade_skin)
            .emit(&frame, true, target);
        // Subtle mouth / lip line below the nose
        TaperedPrism::new(4, Vec3::new(0.0, 0.024, 0.082), Vec3::new(0.0, 0.035, 0.082), 0.024, 0.009, 0.022, 0.009, lip_col)
            .emit(&frame, true, target);

        // Expressive eyes, eyebrows, and side ears
        for side in [-1.0f32, 1.0f32] {
            TaperedPrism::new(6, Vec3::new(side * 0.036, 0.098, 0.086), Vec3::new(side * 0.036, 0.120, 0.086), 0.014, 0.010, 0.013, 0.010, eye_col)
                .emit(&frame, true, target);
            TaperedPrism::new(4, Vec3::new(side * 0.037, 0.126, 0.090), Vec3::new(side * 0.037, 0.136, 0.090), 0.019, 0.010, 0.018, 0.010, hair_col)
                .emit(&frame, true, target);
            TaperedPrism::new(4, Vec3::new(side * 0.088, 0.068, -0.004), Vec3::new(side * 0.088, 0.120, -0.002), 0.012, 0.018, 0.014, 0.020, shade_skin)
                .emit(&frame, true, target);
        }

        // Tailored stand collar on upper torso
        let torso = pose.torso_frame();
        let collar_y = 1.52 - pose.crouch_compression() * 0.80;
        TaperedPrism::new(
            8,
            Vec3::new(0.0, collar_y - 0.02, -0.072),
            Vec3::new(0.0, collar_y + 0.045, -0.062),
            pose.scale(0.096),
            0.080,
            pose.scale(0.074),
            0.068,
            trim,
        )
        .emit(&torso, true, target);

        // Multi-layered hair or medieval headgear fully covering the rear cranium crown
        match self {
            Self::TousledMop => {
                TaperedPrism::new(8, Vec3::new(0.0, 0.045, -0.024), Vec3::new(0.0, 0.208, -0.022), 0.096, 0.088, 0.104, 0.100, hair_col)
                    .emit(&frame, true, target);
                TaperedPrism::new(8, Vec3::new(0.0, 0.150, -0.010), Vec3::new(0.0, 0.246, -0.006), 0.106, 0.112, 0.082, 0.090, hair_col)
                    .emit(&frame, true, target);
                TaperedPrism::new(6, Vec3::new(0.0, 0.148, 0.042), Vec3::new(0.0, 0.198, 0.032), 0.080, 0.064, 0.068, 0.052, hair_col)
                    .emit(&frame, true, target);
            }
            Self::SquireCrop => {
                let blonde = hair_col * 1.28;
                TaperedPrism::new(8, Vec3::new(0.0, 0.060, -0.022), Vec3::new(0.0, 0.206, -0.020), 0.096, 0.088, 0.102, 0.098, blonde)
                    .emit(&frame, true, target);
                TaperedPrism::new(8, Vec3::new(0.0, 0.155, -0.008), Vec3::new(0.0, 0.235, -0.006), 0.102, 0.108, 0.078, 0.086, blonde)
                    .emit(&frame, true, target);
                TaperedPrism::new(6, Vec3::new(0.0, 0.150, 0.038), Vec3::new(0.0, 0.190, 0.030), 0.078, 0.060, 0.068, 0.050, blonde)
                    .emit(&frame, true, target);
            }
            Self::MedievalGugel => {
                TaperedPrism::new(8, Vec3::new(0.0, -0.15, -0.024), Vec3::new(0.0, -0.01, -0.022), pose.scale(0.185), 0.112, 0.108, 0.092, trim)
                    .emit(&frame, false, target);
                TaperedPrism::new(8, Vec3::new(0.0, -0.01, -0.032), Vec3::new(0.0, 0.215, -0.026), 0.108, 0.094, 0.100, 0.094, trim)
                    .emit(&frame, true, target);
                TaperedPrism::new(8, Vec3::new(0.0, 0.19, -0.026), Vec3::new(0.0, 0.26, -0.088), 0.094, 0.088, 0.028, 0.038, trim)
                    .emit(&frame, true, target);
            }
            Self::MerchantCap => {
                TaperedPrism::new(8, Vec3::new(0.0, 0.060, -0.022), Vec3::new(0.0, 0.198, -0.020), 0.094, 0.086, 0.100, 0.094, hair_col)
                    .emit(&frame, true, target);
                TaperedPrism::new(8, Vec3::new(0.0, 0.150, -0.004), Vec3::new(0.0, 0.188, -0.004), 0.102, 0.108, 0.106, 0.110, trim)
                    .emit(&frame, true, target);
                TaperedPrism::new(8, Vec3::new(0.0, 0.180, -0.008), Vec3::new(0.0, 0.256, -0.012), 0.124, 0.122, 0.092, 0.092, cloth)
                    .emit(&frame, true, target);
            }
        }
    }
}
