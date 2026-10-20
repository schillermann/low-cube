use glam::Vec3;
use world::MeshTarget;
use super::headgear::PolygonHeadgear;
use super::limb::{FacetedSolid, TaperedPrism};
use super::palette::{ComplexionSwatch, PigmentSwatch};
use super::pose::BodyPose;

/// Head, hair, beard, helmet, and hat style visible to multiplayer peers and in Looking-Glass / Studio view.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CrownStyle {
    TousledMop,
    SquireCrop,
    MedievalGugel,
    MerchantCap,
    VisoredHelm,
    SmithGoggles,
    KettleHelm,
}

impl CrownStyle {
    pub fn next(self) -> Self {
        Self::from_code(self.code() + 1)
    }

    pub fn from_code(code: u8) -> Self {
        match code % 7 {
            0 => Self::TousledMop,
            1 => Self::SquireCrop,
            2 => Self::MedievalGugel,
            3 => Self::MerchantCap,
            4 => Self::VisoredHelm,
            5 => Self::SmithGoggles,
            _ => Self::KettleHelm,
        }
    }

    pub fn code(self) -> u8 {
        match self {
            Self::TousledMop => 0,
            Self::SquireCrop => 1,
            Self::MedievalGugel => 2,
            Self::MerchantCap => 3,
            Self::VisoredHelm => 4,
            Self::SmithGoggles => 5,
            Self::KettleHelm => 6,
        }
    }

    /// Emits neck, sculpted Polygon-style head, detailed facial features, collar, and headgear.
    pub fn emit(
        self,
        pose: &BodyPose,
        complexion: ComplexionSwatch,
        pigment: PigmentSwatch,
        target: &mut dyn MeshTarget,
    ) {
        let frame = pose.crown_frame();
        let skin = complexion.skin();
        let shade_skin = skin * 0.89;
        let lip_col = skin * 0.72;
        let sclera = Vec3::new(0.95, 0.95, 0.96);
        let iris = Vec3::new(0.14, 0.12, 0.10);
        let hair_col = Vec3::new(0.28, 0.18, 0.11);
        let trim = pigment.trim();
        let hs = |y: f32| pose.crown_size(y);
        let hw = |r: f32| pose.crown_span(r);

        // Neck column connecting skull base to upper torso collar
        TaperedPrism::hexagonal(
            Vec3::new(0.0, hs(-0.15), hw(-0.022)),
            Vec3::new(0.0, hs(0.02), hw(-0.008)),
            pose.crown_girth(0.056),
            pose.crown_girth(0.050),
            shade_skin,
        )
        .emit(&frame, false, target);

        // Tier 1: Sculpted chin and angular Polygon-style jawline tapering upward to cheekbones
        TaperedPrism::new(8, Vec3::new(0.0, hs(-0.018), hw(0.018)), Vec3::new(0.0, hs(0.065), hw(0.008)), hw(0.052), hw(0.066), hw(0.088), hw(0.092), skin)
            .emit(&frame, false, target);
        // Tier 2: Cheekbones and mid-face rising to pronounced brow ridge
        TaperedPrism::new(8, Vec3::new(0.0, hs(0.065), hw(0.008)), Vec3::new(0.0, hs(0.146), hw(0.010)), hw(0.088), hw(0.092), hw(0.090), hw(0.094), skin)
            .emit(&frame, false, target);
        // Tier 3: Forehead rounding inward into the upper cranium
        TaperedPrism::new(8, Vec3::new(0.0, hs(0.146), hw(0.010)), Vec3::new(0.0, hs(0.198), hw(0.008)), hw(0.090), hw(0.094), hw(0.064), hw(0.068), skin)
            .emit(&frame, true, target);

        // Sculpted Polygon-style nose bridge and tip on the front (+Z) of the face
        TaperedPrism::new(4, Vec3::new(0.0, hs(0.062), hw(0.098)), Vec3::new(0.0, hs(0.120), hw(0.090)), hw(0.017), hw(0.020), hw(0.010), hw(0.012), shade_skin)
            .emit(&frame, true, target);
        // Mouth and lip line below the nose
        TaperedPrism::new(4, Vec3::new(0.0, hs(0.024), hw(0.084)), Vec3::new(0.0, hs(0.036), hw(0.084)), hw(0.025), hw(0.010), hw(0.022), hw(0.010), lip_col)
            .emit(&frame, true, target);

        // Expressive multi-part eyes (sclera + dark iris), eyebrows, and sculpted side ears
        for side in [-1.0f32, 1.0f32] {
            TaperedPrism::new(6, Vec3::new(side * hw(0.036), hs(0.095), hw(0.085)), Vec3::new(side * hw(0.036), hs(0.122), hw(0.085)), hw(0.016), hw(0.009), hw(0.015), hw(0.009), sclera)
                .emit(&frame, true, target);
            TaperedPrism::new(6, Vec3::new(side * hw(0.036), hs(0.099), hw(0.089)), Vec3::new(side * hw(0.036), hs(0.118), hw(0.089)), hw(0.010), hw(0.008), hw(0.010), hw(0.008), iris)
                .emit(&frame, true, target);
            TaperedPrism::new(4, Vec3::new(side * hw(0.037), hs(0.126), hw(0.091)), Vec3::new(side * hw(0.037), hs(0.138), hw(0.091)), hw(0.021), hw(0.011), hw(0.019), hw(0.011), hair_col)
                .emit(&frame, true, target);
            TaperedPrism::new(4, Vec3::new(side * hw(0.089), hs(0.066), hw(-0.004)), Vec3::new(side * hw(0.092), hs(0.122), hw(-0.002)), hw(0.013), hw(0.019), hw(0.015), hw(0.021), shade_skin)
                .emit(&frame, true, target);
        }

        // Tailored stand collar on upper torso
        let torso = pose.torso_frame();
        let collar_y = pose.torso_span(0.72 - pose.crouch_compression() * 0.42);
        TaperedPrism::new(
            8,
            Vec3::new(0.0, collar_y - 0.02, -0.072),
            Vec3::new(0.0, collar_y + 0.045, -0.062),
            pose.scale(0.096),
            pose.torso_depth(0.080),
            pose.scale(0.074),
            pose.torso_depth(0.068),
            trim,
        )
        .emit(&torso, true, target);

        PolygonHeadgear::new(self).emit(pose, &frame, pigment, target);
    }
}
