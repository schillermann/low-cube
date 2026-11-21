use glam::Vec3;
use world::MeshTarget;
use super::crown::HeadSculpture;
use super::limb::{AxialLimb, FacetedSolid, LimbFrame, TaperedPrism};
use super::pose::BodyPose;

/// Sculpted open-faced helmets, hoods, caps, diadems, and goggles applied by `CrownedHead`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BrowAdornment {
    SquireBeret,
    RangerHood,
    VelvetCap,
    RaisedSallet,
    BrowGoggles,
    KettleBrim,
    WingedDiadem,
    FloralWreath,
    BattleCirclet,
    RoyalTiara,
}

impl BrowAdornment {
    fn emit(self, pose: &BodyPose, frame: &LimbFrame, (cloth, trim): (Vec3, Vec3), target: &mut dyn MeshTarget) {
        let hs = |y: f32| pose.crown_size(y);
        let hw = |r: f32| pose.crown_span(r);
        let steel = Vec3::new(0.64, 0.68, 0.75);
        let dark_steel = Vec3::new(0.32, 0.36, 0.42);
        let brass = Vec3::new(0.86, 0.70, 0.28);
        let plume_white = Vec3::new(0.95, 0.93, 0.88);
        let lens_cyan = Vec3::new(0.28, 0.88, 0.96);

        match self {
            Self::SquireBeret => {
                TaperedPrism::new(8, Vec3::new(0.0, hs(0.156), hw(-0.004)), Vec3::new(0.0, hs(0.192), hw(-0.004)), hw(0.104), hw(0.108), hw(0.112), hw(0.114), brass)
                    .emit(frame, true, target);
                TaperedPrism::new(8, Vec3::new(hw(0.016), hs(0.186), hw(-0.006)), Vec3::new(hw(0.030), hs(0.264), hw(-0.010)), hw(0.140), hw(0.136), hw(0.094), hw(0.092), cloth)
                    .emit(frame, true, target);
                TaperedPrism::new(6, Vec3::new(hw(-0.075), hs(0.176), hw(0.075)), Vec3::new(hw(-0.075), hs(0.214), hw(0.082)), hw(0.021), hw(0.015), hw(0.019), hw(0.013), brass)
                    .emit(frame, true, target);
                TaperedPrism::new(4, Vec3::new(hw(-0.082), hs(0.205), hw(0.068)), Vec3::new(hw(-0.115), hs(0.362), hw(-0.035)), hw(0.016), hw(0.028), hw(0.006), hw(0.014), plume_white)
                    .emit(frame, true, target);
            }
            Self::RangerHood => {
                let neck = pose.neck_frame();
                TaperedPrism::new(8, Vec3::new(0.0, hs(-0.155), hw(-0.026)), Vec3::new(0.0, hs(-0.008), hw(-0.020)), pose.scale(0.192), hw(0.116), hw(0.110), hw(0.094), trim)
                    .emit(&neck, false, target);
                TaperedPrism::new(8, Vec3::new(0.0, hs(0.142), hw(-0.022)), Vec3::new(0.0, hs(0.236), hw(-0.012)), hw(0.112), hw(0.104), hw(0.104), hw(0.106), cloth)
                    .emit(frame, true, target);
                TaperedPrism::new(8, Vec3::new(0.0, hs(0.196), hw(-0.026)), Vec3::new(0.0, hs(0.286), hw(-0.138)), hw(0.096), hw(0.090), hw(0.022), hw(0.032), trim)
                    .emit(frame, true, target);
                for side in [-1.0_f32, 1.0_f32] {
                    AxialLimb::new(Vec3::new(side * hw(0.090), hs(0.005), hw(-0.012)), Vec3::new(side * hw(0.098), hs(0.156), hw(-0.006)), hw(0.022), hw(0.026), cloth)
                        .emit(frame, true, target);
                }
                TaperedPrism::new(4, Vec3::new(hw(0.096), hs(0.165), hw(0.010)), Vec3::new(hw(0.126), hs(0.312), hw(-0.075)), hw(0.012), hw(0.024), hw(0.005), hw(0.012), brass)
                    .emit(frame, true, target);
            }
            Self::VelvetCap => {
                TaperedPrism::new(8, Vec3::new(0.0, hs(0.146), hw(-0.010)), Vec3::new(0.0, hs(0.194), hw(-0.008)), hw(0.106), hw(0.110), hw(0.108), hw(0.112), trim)
                    .emit(frame, true, target);
                TaperedPrism::new(8, Vec3::new(0.0, hs(0.188), hw(-0.012)), Vec3::new(0.0, hs(0.274), hw(-0.028)), hw(0.102), hw(0.106), hw(0.058), hw(0.062), cloth)
                    .emit(frame, true, target);
            }
            Self::RaisedSallet => {
                TaperedPrism::new(8, Vec3::new(0.0, hs(0.138), hw(-0.010)), Vec3::new(0.0, hs(0.228), hw(-0.006)), hw(0.106), hw(0.112), hw(0.080), hw(0.090), steel)
                    .emit(frame, true, target);
                TaperedPrism::new(6, Vec3::new(0.0, hs(0.146), hw(0.048)), Vec3::new(0.0, hs(0.196), hw(0.068)), hw(0.088), hw(0.058), hw(0.072), hw(0.046), dark_steel)
                    .emit(frame, true, target);
                TaperedPrism::new(4, Vec3::new(0.0, hs(0.142), hw(0.108)), Vec3::new(0.0, hs(0.204), hw(0.102)), hw(0.014), hw(0.014), hw(0.012), hw(0.012), brass)
                    .emit(frame, true, target);
                for side in [-1.0_f32, 1.0_f32] {
                    AxialLimb::new(Vec3::new(side * hw(0.084), hs(0.016), hw(0.018)), Vec3::new(side * hw(0.094), hs(0.148), hw(-0.004)), hw(0.016), hw(0.024), steel)
                        .emit(frame, true, target);
                }
                TaperedPrism::new(6, Vec3::new(0.0, hs(0.218), hw(-0.010)), Vec3::new(0.0, hs(0.278), hw(-0.022)), hw(0.022), hw(0.056), hw(0.018), hw(0.068), brass)
                    .emit(frame, true, target);
                TaperedPrism::new(6, Vec3::new(0.0, hs(0.258), hw(-0.025)), Vec3::new(0.0, hs(0.362), hw(-0.096)), hw(0.026), hw(0.084), hw(0.015), hw(0.044), cloth)
                    .emit(frame, true, target);
            }
            Self::BrowGoggles => {
                TaperedPrism::new(8, Vec3::new(0.0, hs(0.150), hw(-0.008)), Vec3::new(0.0, hs(0.180), hw(-0.006)), hw(0.099), hw(0.103), hw(0.098), hw(0.102), dark_steel)
                    .emit(frame, true, target);
                for side in [-1.0_f32, 1.0_f32] {
                    TaperedPrism::new(8, Vec3::new(side * hw(0.040), hs(0.148), hw(0.088)), Vec3::new(side * hw(0.040), hs(0.184), hw(0.088)), hw(0.028), hw(0.018), hw(0.028), hw(0.018), brass)
                        .emit(frame, true, target);
                    TaperedPrism::new(6, Vec3::new(side * hw(0.040), hs(0.152), hw(0.096)), Vec3::new(side * hw(0.040), hs(0.180), hw(0.096)), hw(0.020), hw(0.012), hw(0.020), hw(0.012), lens_cyan)
                        .emit(frame, true, target);
                }
            }
            Self::KettleBrim => {
                for side in [-1.0_f32, 1.0_f32] {
                    AxialLimb::new(Vec3::new(side * hw(0.080), hs(0.004), hw(0.018)), Vec3::new(side * hw(0.088), hs(0.146), hw(0.004)), hw(0.010), hw(0.014), dark_steel)
                        .emit(frame, true, target);
                }
                TaperedPrism::new(8, Vec3::new(0.0, hs(0.138), hw(-0.006)), Vec3::new(0.0, hs(0.172), hw(-0.006)), hw(0.205), hw(0.215), hw(0.116), hw(0.120), steel)
                    .emit(frame, true, target);
                TaperedPrism::new(8, Vec3::new(0.0, hs(0.168), hw(-0.006)), Vec3::new(0.0, hs(0.256), hw(-0.008)), hw(0.112), hw(0.116), hw(0.066), hw(0.074), steel)
                    .emit(frame, true, target);
                TaperedPrism::new(4, Vec3::new(0.0, hs(0.232), hw(-0.008)), Vec3::new(0.0, hs(0.284), hw(-0.010)), hw(0.016), hw(0.080), hw(0.012), hw(0.062), brass)
                    .emit(frame, true, target);
            }
            Self::WingedDiadem => {
                TaperedPrism::new(8, Vec3::new(0.0, hs(0.148), hw(0.004)), Vec3::new(0.0, hs(0.188), hw(0.008)), hw(0.102), hw(0.106), hw(0.106), hw(0.110), brass)
                    .emit(frame, true, target);
                for side in [-1.0_f32, 1.0_f32] {
                    TaperedPrism::new(4, Vec3::new(side * hw(0.102), hs(0.165), hw(0.015)), Vec3::new(side * hw(0.165), hs(0.320), hw(-0.055)), hw(0.016), hw(0.042), hw(0.008), hw(0.022), plume_white)
                        .emit(frame, true, target);
                }
            }
            Self::FloralWreath => {
                TaperedPrism::new(8, Vec3::new(0.0, hs(0.150), hw(-0.004)), Vec3::new(0.0, hs(0.188), hw(-0.004)), hw(0.108), hw(0.112), hw(0.110), hw(0.114), trim)
                    .emit(frame, true, target);
                for side in [-1.0_f32, 1.0_f32] {
                    TaperedPrism::new(6, Vec3::new(side * hw(0.068), hs(0.162), hw(0.084)), Vec3::new(side * hw(0.068), hs(0.184), hw(0.084)), hw(0.016), hw(0.014), hw(0.016), hw(0.014), plume_white)
                        .emit(frame, true, target);
                }
            }
            Self::BattleCirclet => {
                TaperedPrism::new(8, Vec3::new(0.0, hs(0.146), hw(-0.002)), Vec3::new(0.0, hs(0.184), hw(-0.002)), hw(0.104), hw(0.108), hw(0.106), hw(0.110), steel)
                    .emit(frame, true, target);
                TaperedPrism::new(4, Vec3::new(0.0, hs(0.142), hw(0.108)), Vec3::new(0.0, hs(0.196), hw(0.108)), hw(0.020), hw(0.014), hw(0.014), hw(0.012), brass)
                    .emit(frame, true, target);
            }
            Self::RoyalTiara => {
                TaperedPrism::new(8, Vec3::new(0.0, hs(0.164), hw(-0.004)), Vec3::new(0.0, hs(0.224), hw(-0.002)), hw(0.096), hw(0.100), hw(0.104), hw(0.108), brass)
                    .emit(frame, true, target);
                TaperedPrism::new(6, Vec3::new(0.0, hs(-0.180), hw(-0.125)), Vec3::new(0.0, hs(0.215), hw(-0.055)), hw(0.124), hw(0.025), hw(0.074), hw(0.022), plume_white)
                    .emit(frame, true, target);
            }
        }
    }
}

/// `HeadSculpture` decorator crowning an inner head sculpture with hats, hoods, diadems, or helms.
#[derive(Clone, Copy, Debug)]
pub struct CrownedHead<S> {
    origin: S,
    adornment: BrowAdornment,
    palette: (Vec3, Vec3),
}

impl<S> CrownedHead<S> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(origin: S, adornment: BrowAdornment, palette: (Vec3, Vec3)) -> Self {
        Self { origin, adornment, palette }
    }
}

impl<S: HeadSculpture> HeadSculpture for CrownedHead<S> {
    fn emit(&self, pose: &BodyPose, target: &mut dyn MeshTarget) {
        self.origin.emit(pose, target);
        let frame = pose.crown_frame();
        self.adornment.emit(pose, &frame, self.palette, target);
    }
}
