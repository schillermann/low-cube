use glam::Vec3;
use world::MeshTarget;
use super::crown::HeadSculpture;
use super::limb::{AxialLimb, FacetedSolid, LimbFrame, TaperedPrism};
use super::pose::BodyPose;

/// Sculpted hairstyle archetype applied by the `TressedHead` decorator across male and female characters.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HairCoiffure {
    PartedWave,
    CroppedTemples,
    FlowingBraids,
    SilkTopknot,
}

impl HairCoiffure {
    fn emit(self, pose: &BodyPose, frame: &LimbFrame, (root_col, highlight_col): (Vec3, Vec3), target: &mut dyn MeshTarget) {
        let hs = |y: f32| pose.crown_size(y);
        let hw = |r: f32| pose.crown_span(r);
        let brass = Vec3::new(0.86, 0.70, 0.28);

        match self {
            Self::PartedWave => {
                TaperedPrism::new(8, Vec3::new(0.0, hs(0.038), hw(-0.028)), Vec3::new(0.0, hs(0.196), hw(-0.020)), hw(0.094), hw(0.084), hw(0.102), hw(0.096), root_col)
                    .emit(frame, true, target);
                TaperedPrism::new(8, Vec3::new(hw(0.008), hs(0.158), hw(-0.006)), Vec3::new(hw(0.014), hs(0.236), hw(0.004)), hw(0.104), hw(0.106), hw(0.080), hw(0.088), root_col)
                    .emit(frame, true, target);
                AxialLimb::new(Vec3::new(hw(-0.048), hs(0.196), hw(0.056)), Vec3::new(hw(0.058), hs(0.228), hw(0.072)), hw(0.028), hw(0.036), highlight_col)
                    .emit(frame, true, target);
                AxialLimb::new(Vec3::new(hw(-0.018), hs(0.218), hw(0.022)), Vec3::new(hw(0.068), hs(0.204), hw(0.046)), hw(0.032), hw(0.024), highlight_col)
                    .emit(frame, true, target);
                for side in [-1.0_f32, 1.0_f32] {
                    AxialLimb::new(Vec3::new(side * hw(0.086), hs(0.092), hw(0.012)), Vec3::new(side * hw(0.090), hs(0.172), hw(0.018)), hw(0.016), hw(0.024), root_col)
                        .emit(frame, true, target);
                }
            }
            Self::CroppedTemples => {
                TaperedPrism::new(8, Vec3::new(0.0, hs(0.052), hw(-0.026)), Vec3::new(0.0, hs(0.168), hw(-0.018)), hw(0.093), hw(0.084), hw(0.099), hw(0.092), root_col)
                    .emit(frame, false, target);
                for side in [-1.0_f32, 1.0_f32] {
                    AxialLimb::new(Vec3::new(side * hw(0.084), hs(0.094), hw(0.010)), Vec3::new(side * hw(0.088), hs(0.156), hw(0.016)), hw(0.014), hw(0.020), highlight_col)
                        .emit(frame, true, target);
                }
            }
            Self::FlowingBraids => {
                TaperedPrism::new(8, Vec3::new(0.0, hs(0.048), hw(-0.024)), Vec3::new(0.0, hs(0.214), hw(-0.014)), hw(0.098), hw(0.092), hw(0.104), hw(0.100), root_col)
                    .emit(frame, true, target);
                TaperedPrism::new(8, Vec3::new(0.0, hs(-0.275), hw(-0.092)), Vec3::new(0.0, hs(0.148), hw(-0.046)), hw(0.110), hw(0.056), hw(0.102), hw(0.072), root_col)
                    .emit(frame, true, target);
                for side in [-1.0_f32, 1.0_f32] {
                    AxialLimb::new(
                        Vec3::new(side * hw(0.014), hs(0.194), hw(0.074)),
                        Vec3::new(side * hw(0.082), hs(0.132), hw(0.056)),
                        hw(0.024),
                        hw(0.020),
                        highlight_col,
                    )
                    .emit(frame, true, target);
                    TaperedPrism::new(6, Vec3::new(side * hw(0.082), hs(-0.235), hw(0.042)), Vec3::new(side * hw(0.086), hs(0.125), hw(0.026)), hw(0.024), hw(0.024), hw(0.036), hw(0.036), root_col)
                        .emit(frame, true, target);
                    TaperedPrism::new(6, Vec3::new(side * hw(0.082), hs(-0.182), hw(0.042)), Vec3::new(side * hw(0.082), hs(-0.162), hw(0.042)), hw(0.029), hw(0.029), hw(0.029), hw(0.029), brass)
                        .emit(frame, true, target);
                }
            }
            Self::SilkTopknot => {
                Self::FlowingBraids.emit(pose, frame, (root_col, highlight_col), target);
                TaperedPrism::new(8, Vec3::new(0.0, hs(0.196), hw(-0.032)), Vec3::new(0.0, hs(0.296), hw(-0.042)), hw(0.070), hw(0.070), hw(0.054), hw(0.054), highlight_col)
                    .emit(frame, true, target);
                TaperedPrism::new(8, Vec3::new(0.0, hs(0.190), hw(-0.032)), Vec3::new(0.0, hs(0.214), hw(-0.032)), hw(0.076), hw(0.076), hw(0.076), hw(0.076), brass)
                    .emit(frame, true, target);
            }
        }
    }
}

/// `HeadSculpture` decorator layering sculpted hair locks, curtain bangs, or parted waves onto an inner head.
#[derive(Clone, Copy, Debug)]
pub struct TressedHead<S> {
    origin: S,
    coiffure: HairCoiffure,
    tints: (Vec3, Vec3),
}

impl<S> TressedHead<S> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(origin: S, coiffure: HairCoiffure, tints: (Vec3, Vec3)) -> Self {
        Self { origin, coiffure, tints }
    }
}

impl<S: HeadSculpture> HeadSculpture for TressedHead<S> {
    fn emit(&self, pose: &BodyPose, target: &mut dyn MeshTarget) {
        self.origin.emit(pose, target);
        let frame = pose.crown_frame();
        self.coiffure.emit(pose, &frame, self.tints, target);
    }
}
