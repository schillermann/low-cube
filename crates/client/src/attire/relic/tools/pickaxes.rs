use glam::Vec3;
use world::MeshTarget;
use crate::attire::limb::{FacetedSolid, LimbFrame, TaperedPrism};
use super::super::temper::ToolCondition;
use super::super::HeldRelic;

/// Sculpted 3D pickaxe geometry across Flint, Copper, Iron, and Titanium grades (`2` fields).
#[derive(Clone, Copy)]
pub(super) struct PickaxeSculpture {
    relic: HeldRelic,
    condition: ToolCondition,
}

impl PickaxeSculpture {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub(super) const fn new(relic: HeldRelic, condition: ToolCondition) -> Self {
        Self { relic, condition }
    }

    fn palette(self) -> (Vec3, Vec3, Vec3, Vec3, Vec3) {
        let raw = match self.relic {
            HeldRelic::FlintPickaxe => (
                Vec3::new(0.48, 0.31, 0.17),
                Vec3::new(0.38, 0.24, 0.13),
                Vec3::new(0.72, 0.57, 0.35),
                Vec3::new(0.22, 0.25, 0.32),
                Vec3::new(0.78, 0.84, 0.92),
            ),
            HeldRelic::CopperPickaxe => (
                Vec3::new(0.62, 0.43, 0.24),
                Vec3::new(0.50, 0.34, 0.18),
                Vec3::new(0.52, 0.28, 0.15),
                Vec3::new(0.84, 0.46, 0.23),
                Vec3::new(0.98, 0.70, 0.45),
            ),
            HeldRelic::TitaniumPickaxe => (
                Vec3::new(0.24, 0.27, 0.32),
                Vec3::new(0.17, 0.19, 0.24),
                Vec3::new(0.38, 0.46, 0.56),
                Vec3::new(0.68, 0.76, 0.86),
                Vec3::new(0.94, 0.98, 1.00),
            ),
            _ => (
                Vec3::new(0.62, 0.43, 0.24),
                Vec3::new(0.50, 0.34, 0.18),
                Vec3::new(0.30, 0.34, 0.40),
                Vec3::new(0.66, 0.71, 0.78),
                Vec3::new(0.92, 0.95, 0.99),
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
        let (haft, grip, collar, arm_col, tip_col) = self.palette();
        let pick_frame = match self.condition {
            ToolCondition::Splintered => frame.pitched(0.09).rolled(0.07),
            _ => *frame,
        };
        TaperedPrism::hexagonal(
            Vec3::new(gx, -0.155, gz - 0.006),
            Vec3::new(gx, -0.115, gz),
            0.016,
            0.012,
            grip,
        )
        .emit(&pick_frame, true, target);
        TaperedPrism::hexagonal(
            Vec3::new(gx, -0.115, gz),
            Vec3::new(gx, 0.045, gz),
            0.012,
            0.013,
            grip,
        )
        .emit(&pick_frame, false, target);
        TaperedPrism::hexagonal(
            Vec3::new(gx, 0.045, gz),
            Vec3::new(gx, 0.254, gz + 0.004),
            0.013,
            0.015,
            haft,
        )
        .emit(&pick_frame, true, target);
        TaperedPrism::new(
            6,
            Vec3::new(gx, 0.190, gz - 0.004),
            Vec3::new(gx, 0.242, gz + 0.005),
            0.019,
            0.026,
            0.019,
            0.026,
            collar,
        )
        .emit(&pick_frame, true, target);

        let (fwd_arm, fwd_tip, back_arm, back_tip, tip_r) = match self.condition {
            ToolCondition::Pristine => (0.104, 0.170, -0.096, -0.156, 0.003),
            ToolCondition::Scuffed => (0.098, 0.150, -0.092, -0.138, 0.006),
            ToolCondition::Chipped => (0.082, 0.114, -0.068, -0.090, 0.009),
            ToolCondition::Splintered => (0.056, 0.074, -0.042, -0.048, 0.012),
        };
        let sides = if self.relic == HeldRelic::FlintPickaxe { 5 } else { 4 };
        TaperedPrism::new(
            sides,
            Vec3::new(gx, 0.218, gz + 0.010),
            Vec3::new(gx, 0.208, gz + fwd_arm),
            0.014,
            0.019,
            0.010,
            0.014,
            arm_col,
        )
        .emit(&pick_frame, true, target);
        TaperedPrism::new(
            4,
            Vec3::new(gx, 0.208, gz + fwd_arm),
            Vec3::new(gx, 0.172, gz + fwd_tip),
            0.010,
            0.014,
            tip_r,
            tip_r + 0.002,
            tip_col,
        )
        .emit(&pick_frame, true, target);
        TaperedPrism::new(
            sides,
            Vec3::new(gx, 0.218, gz - 0.008),
            Vec3::new(gx, 0.208, gz + back_arm),
            0.014,
            0.019,
            0.010,
            0.014,
            arm_col,
        )
        .emit(&pick_frame, true, target);
        if !matches!(self.condition, ToolCondition::Splintered) {
            TaperedPrism::new(
                4,
                Vec3::new(gx, 0.208, gz + back_arm),
                Vec3::new(gx, 0.176, gz + back_tip),
                0.010,
                0.014,
                tip_r,
                tip_r + 0.002,
                tip_col,
            )
            .emit(&pick_frame, true, target);
        } else {
            TaperedPrism::hexagonal(
                Vec3::new(gx, 0.108, gz),
                Vec3::new(gx, 0.136, gz),
                0.015,
                0.015,
                Vec3::new(0.19, 0.12, 0.07),
            )
            .emit(&pick_frame, true, target);
        }
    }
}
