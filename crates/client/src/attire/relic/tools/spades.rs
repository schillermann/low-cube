use glam::Vec3;
use world::MeshTarget;
use crate::attire::limb::{FacetedSolid, LimbFrame, TaperedPrism};
use super::super::temper::ToolCondition;
use super::super::HeldRelic;

/// Sculpted 3D digging spade geometry across Wood, Copper, Iron, and Titanium grades (`2` fields).
#[derive(Clone, Copy)]
pub(super) struct SpadeSculpture {
    relic: HeldRelic,
    condition: ToolCondition,
}

impl SpadeSculpture {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub(super) const fn new(relic: HeldRelic, condition: ToolCondition) -> Self {
        Self { relic, condition }
    }

    fn palette(self) -> (Vec3, Vec3, Vec3, Vec3, Vec3) {
        let raw = match self.relic {
            HeldRelic::WoodenSpade => (
                Vec3::new(0.58, 0.40, 0.22),
                Vec3::new(0.44, 0.29, 0.16),
                Vec3::new(0.46, 0.30, 0.16),
                Vec3::new(0.68, 0.48, 0.27),
                Vec3::new(0.80, 0.60, 0.36),
            ),
            HeldRelic::CopperSpade => (
                Vec3::new(0.54, 0.37, 0.20),
                Vec3::new(0.42, 0.28, 0.15),
                Vec3::new(0.52, 0.28, 0.15),
                Vec3::new(0.84, 0.46, 0.24),
                Vec3::new(0.98, 0.70, 0.46),
            ),
            HeldRelic::TitaniumSpade => (
                Vec3::new(0.24, 0.27, 0.32),
                Vec3::new(0.17, 0.19, 0.24),
                Vec3::new(0.42, 0.52, 0.64),
                Vec3::new(0.70, 0.78, 0.88),
                Vec3::new(0.94, 0.98, 1.00),
            ),
            _ => (
                Vec3::new(0.54, 0.37, 0.20),
                Vec3::new(0.42, 0.28, 0.15),
                Vec3::new(0.30, 0.33, 0.38),
                Vec3::new(0.54, 0.58, 0.64),
                Vec3::new(0.84, 0.88, 0.93),
            ),
        };
        (
            raw.0,
            raw.1,
            self.condition.weather_metal(raw.2),
            self.condition.weather_metal(raw.3),
            self.condition.weather_metal(raw.4),
        )
    }

    pub(super) fn emit(self, gx: f32, gz: f32, frame: &LimbFrame, target: &mut dyn MeshTarget) {
        let (haft, grip, socket, cheek, lip) = self.palette();
        let spade_frame = match self.condition {
            ToolCondition::Splintered => frame.pitched(0.08).rolled(-0.08),
            _ => *frame,
        };
        TaperedPrism::new(
            6,
            Vec3::new(gx, -0.164, gz),
            Vec3::new(gx, -0.144, gz),
            0.012,
            0.034,
            0.012,
            0.034,
            grip,
        )
        .emit(&spade_frame, true, target);
        TaperedPrism::hexagonal(
            Vec3::new(gx, -0.150, gz),
            Vec3::new(gx, 0.190, gz),
            0.012,
            0.014,
            haft,
        )
        .emit(&spade_frame, false, target);
        TaperedPrism::hexagonal(
            Vec3::new(gx, 0.185, gz),
            Vec3::new(gx, 0.235, gz + 0.004),
            0.016,
            0.020,
            socket,
        )
        .emit(&spade_frame, true, target);

        let (cheek_top, cheek_w, lip_top, lip_w, skew_x) = match self.condition {
            ToolCondition::Pristine => (0.345, 0.046, 0.395, 0.018, 0.0),
            ToolCondition::Scuffed => (0.338, 0.045, 0.376, 0.024, 0.0),
            ToolCondition::Chipped => (0.312, 0.041, 0.338, 0.030, 0.007),
            ToolCondition::Splintered => (0.278, 0.035, 0.296, 0.022, -0.011),
        };
        TaperedPrism::new(
            4,
            Vec3::new(gx, 0.230, gz + 0.008),
            Vec3::new(gx + skew_x * 0.5, cheek_top, gz + 0.016),
            0.007,
            0.052,
            0.006,
            cheek_w,
            cheek,
        )
        .emit(&spade_frame, true, target);
        TaperedPrism::new(
            4,
            Vec3::new(gx + skew_x * 0.5, cheek_top, gz + 0.016),
            Vec3::new(gx + skew_x, lip_top, gz + 0.020),
            0.006,
            cheek_w,
            0.003,
            lip_w,
            lip,
        )
        .emit(&spade_frame, true, target);

        if matches!(self.condition, ToolCondition::Chipped | ToolCondition::Splintered) {
            TaperedPrism::hexagonal(
                Vec3::new(gx, 0.105, gz),
                Vec3::new(gx, 0.132, gz),
                0.015,
                0.015,
                Vec3::new(0.19, 0.12, 0.07),
            )
            .emit(&spade_frame, true, target);
        }
    }
}
