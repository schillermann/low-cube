use glam::Vec3;
use world::MeshTarget;
use super::crown::HeadSculpture;
use super::limb::{AxialLimb, FacetedSolid, LimbFrame, TaperedPrism};
use super::pose::BodyPose;

/// Anatomical facial hair cut leaving cheekbones, lips, and facial planes open and expressive.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BeardCut {
    FullJawline,
    PointedGoatee,
    RangerStubble,
    BraidedFork,
    GarrisonMustache,
}

impl BeardCut {
    fn emit_winged_mustache(pose: &BodyPose, frame: &LimbFrame, tint: Vec3, span: f32, target: &mut dyn MeshTarget) {
        let hs = |y: f32| pose.crown_size(y);
        let hw = |r: f32| pose.crown_span(r);
        for side in [-1.0_f32, 1.0_f32] {
            AxialLimb::new(
                Vec3::new(side * hw(0.005), hs(0.050), hw(0.094)),
                Vec3::new(side * hw(0.044 * span), hs(0.036), hw(0.079)),
                hw(0.012),
                hw(0.009),
                tint,
            )
            .emit(frame, true, target);
        }
    }

    fn emit(self, pose: &BodyPose, frame: &LimbFrame, (beard_col, mustache_col): (Vec3, Vec3), target: &mut dyn MeshTarget) {
        let hs = |y: f32| pose.crown_size(y);
        let hw = |r: f32| pose.crown_span(r);
        let deep = beard_col * 0.84;
        let brass = Vec3::new(0.84, 0.68, 0.26);

        match self {
            Self::FullJawline => {
                TaperedPrism::new(8, Vec3::new(0.0, hs(-0.070), hw(0.044)), Vec3::new(0.0, hs(-0.020), hw(0.032)), hw(0.045), hw(0.048), hw(0.082), hw(0.076), deep)
                    .emit(frame, true, target);
                TaperedPrism::new(8, Vec3::new(0.0, hs(-0.022), hw(0.026)), Vec3::new(0.0, hs(0.017), hw(0.020)), hw(0.084), hw(0.076), hw(0.090), hw(0.074), beard_col)
                    .emit(frame, true, target);
                TaperedPrism::new(4, Vec3::new(0.0, hs(0.012), hw(0.086)), Vec3::new(0.0, hs(0.023), hw(0.088)), hw(0.010), hw(0.009), hw(0.009), hw(0.008), mustache_col)
                    .emit(frame, true, target);
                Self::emit_winged_mustache(pose, frame, mustache_col, 1.0, target);
                for side in [-1.0_f32, 1.0_f32] {
                    AxialLimb::new(Vec3::new(side * hw(0.072), hs(0.008), hw(0.048)), Vec3::new(side * hw(0.084), hs(0.098), hw(0.014)), hw(0.021), hw(0.015), beard_col)
                        .emit(frame, true, target);
                    AxialLimb::new(Vec3::new(side * hw(0.040), hs(0.036), hw(0.078)), Vec3::new(side * hw(0.062), hs(0.014), hw(0.064)), hw(0.010), hw(0.013), beard_col)
                        .emit(frame, true, target);
                }
            }
            Self::PointedGoatee => {
                Self::emit_winged_mustache(pose, frame, mustache_col, 0.88, target);
                TaperedPrism::new(6, Vec3::new(0.0, hs(-0.052), hw(0.070)), Vec3::new(0.0, hs(0.016), hw(0.078)), hw(0.014), hw(0.016), hw(0.032), hw(0.022), beard_col)
                    .emit(frame, true, target);
                TaperedPrism::new(4, Vec3::new(0.0, hs(0.012), hw(0.086)), Vec3::new(0.0, hs(0.022), hw(0.088)), hw(0.009), hw(0.008), hw(0.008), hw(0.008), mustache_col)
                    .emit(frame, true, target);
            }
            Self::RangerStubble => {
                Self::emit_winged_mustache(pose, frame, mustache_col, 0.90, target);
                TaperedPrism::new(8, Vec3::new(0.0, hs(-0.034), hw(0.032)), Vec3::new(0.0, hs(0.012), hw(0.022)), hw(0.046), hw(0.056), hw(0.086), hw(0.078), deep)
                    .emit(frame, true, target);
            }
            Self::BraidedFork => {
                Self::FullJawline.emit(pose, frame, (beard_col, mustache_col), target);
                for side in [-1.0_f32, 1.0_f32] {
                    TaperedPrism::new(6, Vec3::new(side * hw(0.030), hs(-0.132), hw(0.052)), Vec3::new(side * hw(0.036), hs(-0.018), hw(0.042)), hw(0.020), hw(0.022), hw(0.034), hw(0.034), beard_col)
                        .emit(frame, true, target);
                    TaperedPrism::new(6, Vec3::new(side * hw(0.031), hs(-0.092), hw(0.049)), Vec3::new(side * hw(0.033), hs(-0.072), hw(0.047)), hw(0.025), hw(0.025), hw(0.025), hw(0.025), brass)
                        .emit(frame, true, target);
                }
            }
            Self::GarrisonMustache => {
                Self::emit_winged_mustache(pose, frame, mustache_col, 1.15, target);
                TaperedPrism::new(6, Vec3::new(0.0, hs(-0.032), hw(0.068)), Vec3::new(0.0, hs(0.012), hw(0.074)), hw(0.018), hw(0.016), hw(0.026), hw(0.020), beard_col)
                    .emit(frame, true, target);
            }
        }
    }
}

/// `HeadSculpture` decorator adding sculpted anatomical facial hair to an inner head sculpture.
#[derive(Clone, Copy, Debug)]
pub struct BeardedHead<S> {
    origin: S,
    cut: BeardCut,
    tints: (Vec3, Vec3),
}

impl<S> BeardedHead<S> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(origin: S, cut: BeardCut, tints: (Vec3, Vec3)) -> Self {
        Self { origin, cut, tints }
    }
}

impl<S: HeadSculpture> HeadSculpture for BeardedHead<S> {
    fn emit(&self, pose: &BodyPose, target: &mut dyn MeshTarget) {
        self.origin.emit(pose, target);
        let frame = pose.crown_frame();
        self.cut.emit(pose, &frame, self.tints, target);
    }
}
