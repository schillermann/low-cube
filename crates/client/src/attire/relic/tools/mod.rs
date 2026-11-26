mod pickaxes;
mod spades;

use glam::Vec3;
use world::MeshTarget;
use super::super::limb::{FacetedSolid, LimbFrame, TaperedPrism};
use super::temper::ToolCondition;
use super::HeldRelic;
use self::pickaxes::PickaxeSculpture;
use self::spades::SpadeSculpture;

/// Sculpted 3D hafted tool and blade geometry reflecting material grade and physical wear (`2` fields).
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

    fn emit_mallet(self, gx: f32, gz: f32, frame: &LimbFrame, target: &mut dyn MeshTarget) {
        let haft = self.condition.weather_metal(Vec3::new(0.48, 0.31, 0.16));
        let head = self.condition.weather_metal(Vec3::new(0.36, 0.24, 0.13));
        let crack = Vec3::new(0.18, 0.11, 0.06);
        let head_frame = match self.condition {
            ToolCondition::Splintered => frame.pitched(0.08).rolled(-0.07),
            _ => *frame,
        };
        TaperedPrism::hexagonal(Vec3::new(gx, -0.10, gz), Vec3::new(gx, 0.18, gz), 0.013, 0.014, haft)
            .emit(&head_frame, true, target);
        let (z_back, z_front, r_front) = match self.condition {
            ToolCondition::Pristine => (-0.040, 0.050, 0.052),
            ToolCondition::Scuffed => (-0.038, 0.046, 0.049),
            ToolCondition::Chipped => (-0.034, 0.038, 0.043),
            ToolCondition::Splintered => (-0.024, 0.028, 0.034),
        };
        TaperedPrism::new(
            6,
            Vec3::new(gx, 0.14, gz + z_back),
            Vec3::new(gx, 0.20, gz + z_front),
            0.032,
            0.052,
            0.028,
            r_front,
            head,
        )
        .emit(&head_frame, true, target);
        if matches!(self.condition, ToolCondition::Chipped | ToolCondition::Splintered) {
            TaperedPrism::hexagonal(Vec3::new(gx, 0.08, gz), Vec3::new(gx, 0.11, gz), 0.015, 0.015, crack)
                .emit(&head_frame, true, target);
        }
    }

    fn emit_axe(self, gx: f32, gz: f32, frame: &LimbFrame, target: &mut dyn MeshTarget) {
        let (ash_wood, worn_ash, forged_iron, blade_steel, bright_edge) = (
            Vec3::new(0.64, 0.45, 0.25),
            Vec3::new(0.52, 0.35, 0.19),
            self.condition.weather_metal(Vec3::new(0.42, 0.46, 0.52)),
            self.condition.weather_metal(Vec3::new(0.68, 0.73, 0.79)),
            self.condition.weather_metal(Vec3::new(0.92, 0.95, 0.99)),
        );
        let axe_frame = match self.condition {
            ToolCondition::Splintered => frame.pitched(0.07).rolled(0.06),
            _ => *frame,
        };
        TaperedPrism::hexagonal(Vec3::new(gx, -0.155, gz - 0.008), Vec3::new(gx, -0.115, gz), 0.016, 0.012, worn_ash)
            .emit(&axe_frame, true, target);
        TaperedPrism::hexagonal(Vec3::new(gx, -0.115, gz), Vec3::new(gx, 0.045, gz), 0.012, 0.013, worn_ash)
            .emit(&axe_frame, false, target);
        TaperedPrism::hexagonal(Vec3::new(gx, 0.045, gz), Vec3::new(gx, 0.252, gz + 0.004), 0.013, 0.015, ash_wood)
            .emit(&axe_frame, true, target);
        TaperedPrism::new(6, Vec3::new(gx, 0.184, gz - 0.004), Vec3::new(gx, 0.244, gz - 0.002), 0.017, 0.025, 0.018, 0.026, forged_iron)
            .emit(&axe_frame, true, target);
        TaperedPrism::new(4, Vec3::new(gx, 0.194, gz + 0.032), Vec3::new(gx, 0.242, gz + 0.034), 0.013, 0.024, 0.014, 0.026, forged_iron)
            .emit(&axe_frame, true, target);
        let bit_z = match self.condition {
            ToolCondition::Splintered => 0.052,
            _ => 0.064,
        };
        TaperedPrism::new(4, Vec3::new(gx, 0.128, gz + bit_z), Vec3::new(gx, 0.194, gz + bit_z - 0.002), 0.008, 0.020, 0.011, 0.026, blade_steel)
            .emit(&axe_frame, true, target);
        if !matches!(self.condition, ToolCondition::Splintered) {
            TaperedPrism::new(4, Vec3::new(gx, 0.194, gz + 0.062), Vec3::new(gx, 0.254, gz + 0.064), 0.011, 0.026, 0.010, 0.024, blade_steel)
                .emit(&axe_frame, true, target);
        }
        match self.condition {
            ToolCondition::Pristine | ToolCondition::Scuffed => {
                let bevel = if matches!(self.condition, ToolCondition::Pristine) { 0.096 } else { 0.088 };
                TaperedPrism::new(4, Vec3::new(gx, 0.124, gz + 0.084), Vec3::new(gx, 0.172, gz + bevel - 0.002), 0.003, 0.010, 0.004, 0.014, bright_edge)
                    .emit(&axe_frame, true, target);
                TaperedPrism::new(4, Vec3::new(gx, 0.172, gz + bevel - 0.002), Vec3::new(gx, 0.218, gz + bevel), 0.004, 0.014, 0.004, 0.014, bright_edge)
                    .emit(&axe_frame, true, target);
                TaperedPrism::new(4, Vec3::new(gx, 0.218, gz + bevel), Vec3::new(gx, 0.258, gz + 0.086), 0.004, 0.014, 0.003, 0.010, bright_edge)
                    .emit(&axe_frame, true, target);
            }
            ToolCondition::Chipped => {
                TaperedPrism::new(4, Vec3::new(gx, 0.128, gz + 0.078), Vec3::new(gx, 0.168, gz + 0.084), 0.003, 0.009, 0.004, 0.011, bright_edge)
                    .emit(&axe_frame, true, target);
                TaperedPrism::new(4, Vec3::new(gx, 0.218, gz + 0.084), Vec3::new(gx, 0.252, gz + 0.078), 0.004, 0.011, 0.003, 0.009, bright_edge)
                    .emit(&axe_frame, true, target);
            }
            ToolCondition::Splintered => {
                TaperedPrism::hexagonal(Vec3::new(gx, 0.110, gz), Vec3::new(gx, 0.138, gz), 0.015, 0.015, Vec3::new(0.20, 0.12, 0.07))
                    .emit(&axe_frame, true, target);
            }
        }
    }

    fn emit_sword(self, gx: f32, gz: f32, frame: &LimbFrame, target: &mut dyn MeshTarget) {
        let (grip_leather, guard_iron, fuller_steel, honed_edge) = (
            Vec3::new(0.26, 0.15, 0.09),
            self.condition.weather_metal(Vec3::new(0.32, 0.35, 0.40)),
            self.condition.weather_metal(Vec3::new(0.56, 0.60, 0.66)),
            self.condition.weather_metal(Vec3::new(0.84, 0.88, 0.94)),
        );
        TaperedPrism::new(8, Vec3::new(gx, -0.106, gz), Vec3::new(gx, -0.068, gz), 0.015, 0.024, 0.015, 0.024, guard_iron)
            .emit(frame, true, target);
        TaperedPrism::new(6, Vec3::new(gx, -0.068, gz), Vec3::new(gx, 0.048, gz), 0.011, 0.015, 0.012, 0.016, grip_leather)
            .emit(frame, false, target);
        TaperedPrism::new(6, Vec3::new(gx, 0.046, gz), Vec3::new(gx, 0.066, gz), 0.013, 0.082, 0.012, 0.076, guard_iron)
            .emit(frame, true, target);
        let (body_top, tip_top, tip_w) = match self.condition {
            ToolCondition::Pristine => (0.420, 0.680, 0.002),
            ToolCondition::Scuffed => (0.410, 0.650, 0.005),
            ToolCondition::Chipped => (0.370, 0.535, 0.009),
            ToolCondition::Splintered => (0.285, 0.365, 0.014),
        };
        TaperedPrism::new(4, Vec3::new(gx, 0.066, gz), Vec3::new(gx, body_top, gz), 0.005, 0.025, 0.004, 0.020, honed_edge)
            .emit(frame, false, target);
        TaperedPrism::new(4, Vec3::new(gx, 0.068, gz), Vec3::new(gx, body_top - 0.025, gz), 0.006, 0.011, 0.005, 0.008, fuller_steel)
            .emit(frame, true, target);
        TaperedPrism::new(4, Vec3::new(gx, body_top, gz), Vec3::new(gx, tip_top, gz), 0.004, 0.020, 0.002, tip_w, honed_edge)
            .emit(frame, true, target);
    }

    pub(super) fn emit_in_palm(self, gx: f32, gz: f32, frame: &LimbFrame, target: &mut dyn MeshTarget) {
        match self.relic {
            HeldRelic::TimberTorch => {
                TaperedPrism::hexagonal(Vec3::new(gx, -0.10, gz), Vec3::new(gx, 0.18, gz + 0.01), 0.013, 0.017, Vec3::new(0.45, 0.28, 0.14))
                    .emit(frame, false, target);
                TaperedPrism::hexagonal(Vec3::new(gx, 0.18, gz + 0.01), Vec3::new(gx, 0.25, gz + 0.015), 0.022, 0.008, Vec3::new(0.96, 0.52, 0.12))
                    .emit(frame, true, target);
            }
            HeldRelic::WoodenMallet => self.emit_mallet(gx, gz, frame, target),
            HeldRelic::WoodmanAxe => self.emit_axe(gx, gz, frame, target),
            HeldRelic::KnightlySword => self.emit_sword(gx, gz, frame, target),
            HeldRelic::WoodenSpade
            | HeldRelic::CopperSpade
            | HeldRelic::IronSpade
            | HeldRelic::TitaniumSpade => {
                SpadeSculpture::new(self.relic, self.condition).emit(gx, gz, frame, target);
            }
            HeldRelic::FlintPickaxe
            | HeldRelic::CopperPickaxe
            | HeldRelic::IronPickaxe
            | HeldRelic::TitaniumPickaxe => {
                PickaxeSculpture::new(self.relic, self.condition).emit(gx, gz, frame, target);
            }
            _ => {}
        }
    }
}
