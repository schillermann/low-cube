use glam::Vec3;
use world::MeshTarget;
use super::super::limb::{AxialLimb, FacetedSolid, LimbFrame, TaperedPrism};
use super::temper::ToolCondition;
use super::HeldRelic;

/// Sculpted 3D Space-Age EVA tool and transceiver geometry (`RigWrench` & `SuitRadio`, `2` fields).
#[derive(Clone, Copy)]
pub(super) struct ToolSculpture {
    relic: HeldRelic,
    condition: ToolCondition,
}

impl ToolSculpture {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub(super) const fn new(relic: HeldRelic, condition: ToolCondition) -> Self {
        Self { relic, condition }
    }

    fn emit_rig_wrench(self, gx: f32, gz: f32, frame: &LimbFrame, target: &mut dyn MeshTarget) {
        let grip = Vec3::new(0.18, 0.22, 0.28);
        let shaft = self.condition.weather_metal(Vec3::new(0.68, 0.75, 0.84));
        let jaw = self.condition.weather_metal(Vec3::new(0.86, 0.92, 0.98));
        let torque_cyan = Vec3::new(0.24, 0.88, 0.98);
        let amber_band = Vec3::new(0.96, 0.72, 0.20);

        TaperedPrism::hexagonal(Vec3::new(gx, -0.140, gz), Vec3::new(gx, 0.035, gz), 0.015, 0.014, grip)
            .emit(frame, true, target);
        TaperedPrism::hexagonal(Vec3::new(gx, 0.035, gz), Vec3::new(gx, 0.062, gz), 0.017, 0.017, amber_band)
            .emit(frame, true, target);
        TaperedPrism::hexagonal(Vec3::new(gx, 0.060, gz), Vec3::new(gx, 0.195, gz), 0.013, 0.015, shaft)
            .emit(frame, true, target);
        TaperedPrism::new(8, Vec3::new(gx, 0.188, gz), Vec3::new(gx, 0.228, gz), 0.022, 0.036, 0.022, 0.036, torque_cyan)
            .emit(frame, true, target);
        // Upper and lower C-spanner jaws + rear crust-breaker spike
        TaperedPrism::new(4, Vec3::new(gx, 0.222, gz + 0.016), Vec3::new(gx, 0.278, gz + 0.048), 0.011, 0.018, 0.008, 0.012, jaw)
            .emit(frame, true, target);
        TaperedPrism::new(4, Vec3::new(gx, 0.184, gz + 0.016), Vec3::new(gx, 0.214, gz + 0.056), 0.011, 0.018, 0.008, 0.012, jaw)
            .emit(frame, true, target);
        TaperedPrism::new(4, Vec3::new(gx, 0.208, gz - 0.014), Vec3::new(gx, 0.194, gz - 0.088), 0.011, 0.016, 0.004, 0.006, jaw)
            .emit(frame, true, target);
    }

    fn emit_suit_radio(gx: f32, gz: f32, frame: &LimbFrame, target: &mut dyn MeshTarget) {
        let chassis = Vec3::new(0.20, 0.25, 0.32);
        let bezel = Vec3::new(0.56, 0.64, 0.74);
        let screen = Vec3::new(0.22, 0.88, 0.62);
        let whip = Vec3::new(0.84, 0.88, 0.94);

        TaperedPrism::new(4, Vec3::new(gx, -0.045, gz), Vec3::new(gx, 0.115, gz), 0.028, 0.018, 0.028, 0.018, chassis)
            .emit(frame, true, target);
        TaperedPrism::new(4, Vec3::new(gx, 0.038, gz + 0.014), Vec3::new(gx, 0.092, gz + 0.014), 0.020, 0.006, 0.020, 0.006, screen)
            .emit(frame, true, target);
        TaperedPrism::hexagonal(Vec3::new(gx + 0.012, 0.114, gz), Vec3::new(gx + 0.012, 0.132, gz), 0.008, 0.007, bezel)
            .emit(frame, true, target);
        AxialLimb::new(Vec3::new(gx - 0.014, 0.114, gz), Vec3::new(gx - 0.016, 0.258, gz - 0.006), 0.0045, 0.0022, whip)
            .emit(frame, true, target);
    }

    pub(super) fn emit_in_palm(self, gx: f32, gz: f32, frame: &LimbFrame, target: &mut dyn MeshTarget) {
        match self.relic {
            HeldRelic::RigWrench => self.emit_rig_wrench(gx, gz, frame, target),
            HeldRelic::SuitRadio => Self::emit_suit_radio(gx, gz, frame, target),
            _ => {}
        }
    }
}
