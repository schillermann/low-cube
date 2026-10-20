use glam::Vec3;
use world::MeshTarget;
use super::crown::CrownStyle;
use super::limb::{FacetedSolid, LimbFrame, TaperedPrism};
use super::palette::PigmentSwatch;
use super::pose::BodyPose;

/// Detailed low-poly Polygon-style headgear, helmets, hats, masks, spectacles, and facial hair.
pub struct PolygonHeadgear {
    style: CrownStyle,
}

impl PolygonHeadgear {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(style: CrownStyle) -> Self {
        Self { style }
    }

    pub fn emit(&self, pose: &BodyPose, frame: &LimbFrame, pigment: PigmentSwatch, target: &mut dyn MeshTarget) {
        let hs = |y: f32| pose.crown_size(y);
        let hw = |r: f32| pose.crown_span(r);
        let cloth = pigment.cloth();
        let trim = pigment.trim();
        let hair_dark = Vec3::new(0.24, 0.15, 0.09);
        let hair_auburn = Vec3::new(0.38, 0.20, 0.10);
        let steel = Vec3::new(0.62, 0.66, 0.72);
        let dark_steel = Vec3::new(0.34, 0.37, 0.42);
        let brass = Vec3::new(0.84, 0.68, 0.26);
        let plume_white = Vec3::new(0.94, 0.92, 0.86);

        match self.style {
            CrownStyle::TousledMop => {
                TaperedPrism::new(8, Vec3::new(0.0, hs(0.040), hw(-0.024)), Vec3::new(0.0, hs(0.208), hw(-0.022)), hw(0.096), hw(0.088), hw(0.105), hw(0.102), hair_dark).emit(frame, true, target);
                TaperedPrism::new(8, Vec3::new(0.0, hs(0.150), hw(-0.010)), Vec3::new(0.0, hs(0.248), hw(-0.004)), hw(0.108), hw(0.114), hw(0.084), hw(0.092), hair_dark).emit(frame, true, target);
                TaperedPrism::new(6, Vec3::new(0.0, hs(0.146), hw(0.044)), Vec3::new(0.0, hs(0.202), hw(0.036)), hw(0.082), hw(0.066), hw(0.068), hw(0.052), hair_dark).emit(frame, true, target);
            }
            CrownStyle::SquireCrop => {
                // Noble coiffure, sculpted goatee, velvet beret (Chaperon), gold jewel brooch, and tall feather plume
                TaperedPrism::new(8, Vec3::new(0.0, hs(0.058), hw(-0.020)), Vec3::new(0.0, hs(0.195), hw(-0.018)), hw(0.095), hw(0.088), hw(0.100), hw(0.096), hair_auburn).emit(frame, true, target);
                TaperedPrism::new(4, Vec3::new(0.0, hs(-0.032), hw(0.072)), Vec3::new(0.0, hs(0.018), hw(0.084)), hw(0.018), hw(0.016), hw(0.026), hw(0.018), hair_auburn).emit(frame, true, target);
                TaperedPrism::new(8, Vec3::new(0.0, hs(0.155), hw(-0.004)), Vec3::new(0.0, hs(0.192), hw(-0.004)), hw(0.104), hw(0.108), hw(0.112), hw(0.114), brass).emit(frame, true, target);
                TaperedPrism::new(8, Vec3::new(hw(0.016), hs(0.185), hw(-0.006)), Vec3::new(hw(0.028), hs(0.262), hw(-0.010)), hw(0.142), hw(0.138), hw(0.096), hw(0.094), cloth).emit(frame, true, target);
                TaperedPrism::new(6, Vec3::new(hw(-0.075), hs(0.175), hw(0.075)), Vec3::new(hw(-0.075), hs(0.215), hw(0.082)), hw(0.022), hw(0.016), hw(0.020), hw(0.014), brass).emit(frame, true, target);
                TaperedPrism::new(4, Vec3::new(hw(-0.082), hs(0.205), hw(0.068)), Vec3::new(hw(-0.115), hs(0.365), hw(-0.035)), hw(0.016), hw(0.028), hw(0.006), hw(0.014), plume_white).emit(frame, true, target);
            }
            CrownStyle::MedievalGugel => {
                // Ranger pointed hood, stealth lower-face mask over nose/chin, and side hunting feather
                TaperedPrism::new(8, Vec3::new(0.0, hs(-0.022), hw(0.018)), Vec3::new(0.0, hs(0.082), hw(0.026)), hw(0.064), hw(0.076), hw(0.092), hw(0.096), dark_steel).emit(frame, true, target);
                TaperedPrism::new(8, Vec3::new(0.0, hs(-0.155), hw(-0.024)), Vec3::new(0.0, hs(0.005), hw(-0.018)), pose.scale(0.195), hw(0.118), hw(0.114), hw(0.098), trim).emit(frame, false, target);
                TaperedPrism::new(8, Vec3::new(0.0, hs(-0.005), hw(-0.028)), Vec3::new(0.0, hs(0.228), hw(-0.014)), hw(0.114), hw(0.102), hw(0.106), hw(0.108), cloth).emit(frame, true, target);
                TaperedPrism::new(6, Vec3::new(0.0, hs(0.145), hw(0.042)), Vec3::new(0.0, hs(0.215), hw(0.068)), hw(0.094), hw(0.065), hw(0.068), hw(0.045), cloth).emit(frame, true, target);
                TaperedPrism::new(8, Vec3::new(0.0, hs(0.195), hw(-0.026)), Vec3::new(0.0, hs(0.285), hw(-0.135)), hw(0.096), hw(0.090), hw(0.022), hw(0.032), trim).emit(frame, true, target);
                TaperedPrism::new(4, Vec3::new(hw(0.095), hs(0.165), hw(0.010)), Vec3::new(hw(0.125), hs(0.310), hw(-0.075)), hw(0.012), hw(0.024), hw(0.005), hw(0.012), brass).emit(frame, true, target);
            }
            CrownStyle::MerchantCap => {
                // Woodsman full 3D bushy beard, handlebar mustache, thick brows, and folded wool beanie
                TaperedPrism::new(8, Vec3::new(0.0, hs(-0.095), hw(0.038)), Vec3::new(0.0, hs(-0.015), hw(0.028)), hw(0.048), hw(0.052), hw(0.088), hw(0.084), hair_auburn).emit(frame, true, target);
                TaperedPrism::new(8, Vec3::new(0.0, hs(-0.018), hw(0.022)), Vec3::new(0.0, hs(0.062), hw(0.016)), hw(0.092), hw(0.086), hw(0.096), hw(0.082), hair_auburn).emit(frame, true, target);
                TaperedPrism::new(6, Vec3::new(0.0, hs(0.036), hw(0.086)), Vec3::new(0.0, hs(0.064), hw(0.092)), hw(0.056), hw(0.022), hw(0.044), hw(0.020), hair_dark).emit(frame, true, target);
                TaperedPrism::new(8, Vec3::new(0.0, hs(0.138), hw(-0.012)), Vec3::new(0.0, hs(0.192), hw(-0.010)), hw(0.106), hw(0.110), hw(0.108), hw(0.112), trim).emit(frame, true, target);
                TaperedPrism::new(8, Vec3::new(0.0, hs(0.185), hw(-0.014)), Vec3::new(0.0, hs(0.275), hw(-0.028)), hw(0.102), hw(0.106), hw(0.058), hw(0.062), cloth).emit(frame, true, target);
            }
            CrownStyle::VisoredHelm => {
                // Armored Knight full visored plate helm with eye slit, brass cross-trim, and flowing plume crest
                TaperedPrism::new(8, Vec3::new(0.0, hs(-0.035), hw(-0.004)), Vec3::new(0.0, hs(0.085), hw(0.012)), hw(0.088), hw(0.096), hw(0.104), hw(0.112), steel).emit(frame, false, target);
                TaperedPrism::new(8, Vec3::new(0.0, hs(0.080), hw(0.004)), Vec3::new(0.0, hs(0.225), hw(-0.004)), hw(0.106), hw(0.114), hw(0.082), hw(0.092), steel).emit(frame, true, target);
                TaperedPrism::new(6, Vec3::new(0.0, hs(0.020), hw(0.065)), Vec3::new(0.0, hs(0.145), hw(0.062)), hw(0.078), hw(0.068), hw(0.082), hw(0.064), dark_steel).emit(frame, true, target);
                TaperedPrism::new(4, Vec3::new(0.0, hs(0.096), hw(0.116)), Vec3::new(0.0, hs(0.116), hw(0.116)), hw(0.068), hw(0.016), hw(0.068), hw(0.016), Vec3::new(0.08, 0.09, 0.11)).emit(frame, true, target);
                TaperedPrism::new(4, Vec3::new(0.0, hs(0.025), hw(0.120)), Vec3::new(0.0, hs(0.165), hw(0.114)), hw(0.012), hw(0.014), hw(0.012), hw(0.014), brass).emit(frame, true, target);
                TaperedPrism::new(6, Vec3::new(0.0, hs(0.215), hw(-0.010)), Vec3::new(0.0, hs(0.275), hw(-0.020)), hw(0.024), hw(0.056), hw(0.020), hw(0.068), brass).emit(frame, true, target);
                TaperedPrism::new(6, Vec3::new(0.0, hs(0.255), hw(-0.025)), Vec3::new(0.0, hs(0.365), hw(-0.095)), hw(0.028), hw(0.085), hw(0.016), hw(0.045), cloth).emit(frame, true, target);
                TaperedPrism::new(6, Vec3::new(0.0, hs(0.155), hw(-0.115)), Vec3::new(0.0, hs(0.315), hw(-0.085)), hw(0.018), hw(0.036), hw(0.024), hw(0.055), cloth).emit(frame, true, target);
            }
            CrownStyle::SmithGoggles => {
                // Blacksmith braided twin-fork forge beard, leather head-strap, and brass-rimmed smithing goggles on brow
                for side in [-1.0f32, 1.0f32] {
                    TaperedPrism::new(6, Vec3::new(side * hw(0.032), hs(-0.125), hw(0.048)), Vec3::new(side * hw(0.040), hs(-0.010), hw(0.036)), hw(0.022), hw(0.024), hw(0.038), hw(0.038), hair_dark).emit(frame, true, target);
                    TaperedPrism::new(6, Vec3::new(side * hw(0.032), hs(-0.085), hw(0.044)), Vec3::new(side * hw(0.034), hs(-0.065), hw(0.042)), hw(0.026), hw(0.026), hw(0.026), hw(0.026), brass).emit(frame, true, target);
                    TaperedPrism::new(6, Vec3::new(side * hw(0.042), hs(0.145), hw(0.088)), Vec3::new(side * hw(0.042), hs(0.185), hw(0.088)), hw(0.028), hw(0.022), hw(0.028), hw(0.022), brass).emit(frame, true, target);
                }
                TaperedPrism::new(8, Vec3::new(0.0, hs(0.148), hw(-0.008)), Vec3::new(0.0, hs(0.182), hw(-0.008)), hw(0.098), hw(0.102), hw(0.096), hw(0.100), dark_steel).emit(frame, true, target);
                TaperedPrism::new(8, Vec3::new(0.0, hs(0.045), hw(-0.024)), Vec3::new(0.0, hs(0.205), hw(-0.022)), hw(0.094), hw(0.088), hw(0.098), hw(0.094), hair_dark).emit(frame, true, target);
            }
            CrownStyle::KettleHelm => {
                // Sentinel wide-brimmed steel Kettle Hat (Eisenhut), cheek straps, and handlebar mustache
                TaperedPrism::new(6, Vec3::new(0.0, hs(0.038), hw(0.086)), Vec3::new(0.0, hs(0.062), hw(0.092)), hw(0.062), hw(0.020), hw(0.046), hw(0.018), hair_dark).emit(frame, true, target);
                for side in [-1.0f32, 1.0f32] {
                    TaperedPrism::new(4, Vec3::new(side * hw(0.082), hs(-0.005), hw(0.015)), Vec3::new(side * hw(0.088), hs(0.145), hw(0.005)), hw(0.010), hw(0.022), hw(0.012), hw(0.026), dark_steel).emit(frame, true, target);
                }
                TaperedPrism::new(8, Vec3::new(0.0, hs(0.135), hw(-0.006)), Vec3::new(0.0, hs(0.172), hw(-0.006)), hw(0.215), hw(0.225), hw(0.118), hw(0.122), steel).emit(frame, true, target);
                TaperedPrism::new(8, Vec3::new(0.0, hs(0.168), hw(-0.006)), Vec3::new(0.0, hs(0.255), hw(-0.008)), hw(0.114), hw(0.118), hw(0.068), hw(0.076), steel).emit(frame, true, target);
                TaperedPrism::new(4, Vec3::new(0.0, hs(0.235), hw(-0.008)), Vec3::new(0.0, hs(0.285), hw(-0.010)), hw(0.016), hw(0.082), hw(0.012), hw(0.064), brass).emit(frame, true, target);
            }
        }
    }
}
