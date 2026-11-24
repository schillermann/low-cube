use glam::Vec3;
use world::MeshTarget;
use super::super::limb::{FacetedSolid, LimbFrame, TaperedPrism};
use super::HeldRelic;

/// Sculpted 3D hafted tool and blade geometry held in an avatar's palm.
#[derive(Clone, Copy)]
pub(super) struct ToolSculpture {
    relic: HeldRelic,
}

impl ToolSculpture {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub(super) const fn new(relic: HeldRelic) -> Self {
        Self { relic }
    }

    fn emit_flint_pickaxe(gx: f32, gz: f32, frame: &LimbFrame, target: &mut dyn MeshTarget) {
        let (bark_haft, worn_bark, rawhide, chert_core, flint_tip) = (
            Vec3::new(0.48, 0.31, 0.17),
            Vec3::new(0.38, 0.24, 0.13),
            Vec3::new(0.72, 0.57, 0.35),
            Vec3::new(0.22, 0.25, 0.32),
            Vec3::new(0.78, 0.84, 0.92),
        );
        TaperedPrism::hexagonal(Vec3::new(gx, -0.150, gz - 0.006), Vec3::new(gx, 0.040, gz), 0.013, 0.013, worn_bark).emit(frame, true, target);
        TaperedPrism::hexagonal(Vec3::new(gx, 0.040, gz), Vec3::new(gx, 0.248, gz + 0.003), 0.013, 0.015, bark_haft).emit(frame, true, target);
        TaperedPrism::new(6, Vec3::new(gx, 0.186, gz - 0.004), Vec3::new(gx, 0.236, gz + 0.006), 0.020, 0.026, 0.020, 0.026, rawhide).emit(frame, true, target);
        TaperedPrism::new(5, Vec3::new(gx, 0.212, gz - 0.092), Vec3::new(gx, 0.216, gz + 0.098), 0.015, 0.020, 0.015, 0.020, chert_core).emit(frame, true, target);
        TaperedPrism::new(4, Vec3::new(gx, 0.216, gz + 0.092), Vec3::new(gx, 0.176, gz + 0.156), 0.013, 0.018, 0.003, 0.005, flint_tip).emit(frame, true, target);
        TaperedPrism::new(4, Vec3::new(gx, 0.212, gz - 0.088), Vec3::new(gx, 0.178, gz - 0.146), 0.013, 0.018, 0.004, 0.006, flint_tip).emit(frame, true, target);
    }

    fn emit_iron_pickaxe(gx: f32, gz: f32, frame: &LimbFrame, target: &mut dyn MeshTarget) {
        let (ash_wood, worn_ash, socket_iron, pick_steel, bright_point) = (
            Vec3::new(0.62, 0.43, 0.24),
            Vec3::new(0.50, 0.34, 0.18),
            Vec3::new(0.30, 0.34, 0.40),
            Vec3::new(0.66, 0.71, 0.78),
            Vec3::new(0.92, 0.95, 0.99),
        );
        TaperedPrism::hexagonal(Vec3::new(gx, -0.155, gz - 0.006), Vec3::new(gx, -0.115, gz), 0.016, 0.012, worn_ash).emit(frame, true, target);
        TaperedPrism::hexagonal(Vec3::new(gx, -0.115, gz), Vec3::new(gx, 0.045, gz), 0.012, 0.013, worn_ash).emit(frame, false, target);
        TaperedPrism::hexagonal(Vec3::new(gx, 0.045, gz), Vec3::new(gx, 0.256, gz + 0.004), 0.013, 0.015, ash_wood).emit(frame, true, target);
        TaperedPrism::new(6, Vec3::new(gx, 0.192, gz - 0.004), Vec3::new(gx, 0.244, gz + 0.004), 0.019, 0.026, 0.019, 0.026, socket_iron).emit(frame, true, target);
        TaperedPrism::new(4, Vec3::new(gx, 0.218, gz + 0.012), Vec3::new(gx, 0.208, gz + 0.106), 0.013, 0.018, 0.009, 0.013, pick_steel).emit(frame, true, target);
        TaperedPrism::new(4, Vec3::new(gx, 0.208, gz + 0.106), Vec3::new(gx, 0.164, gz + 0.172), 0.009, 0.013, 0.002, 0.003, bright_point).emit(frame, true, target);
        TaperedPrism::new(4, Vec3::new(gx, 0.218, gz - 0.008), Vec3::new(gx, 0.208, gz - 0.098), 0.013, 0.018, 0.009, 0.013, pick_steel).emit(frame, true, target);
        TaperedPrism::new(4, Vec3::new(gx, 0.208, gz - 0.098), Vec3::new(gx, 0.168, gz - 0.158), 0.009, 0.013, 0.002, 0.004, bright_point).emit(frame, true, target);
    }

    pub(super) fn emit_in_palm(self, gx: f32, gz: f32, frame: &LimbFrame, target: &mut dyn MeshTarget) {
        match self.relic {
            HeldRelic::TimberTorch => {
                TaperedPrism::hexagonal(Vec3::new(gx, -0.10, gz), Vec3::new(gx, 0.18, gz + 0.01), 0.013, 0.017, Vec3::new(0.45, 0.28, 0.14)).emit(frame, false, target);
                TaperedPrism::hexagonal(Vec3::new(gx, 0.18, gz + 0.01), Vec3::new(gx, 0.25, gz + 0.015), 0.022, 0.008, Vec3::new(0.96, 0.52, 0.12)).emit(frame, true, target);
            }
            HeldRelic::WoodenMallet => {
                TaperedPrism::hexagonal(Vec3::new(gx, -0.10, gz), Vec3::new(gx, 0.18, gz), 0.013, 0.014, Vec3::new(0.48, 0.31, 0.16)).emit(frame, true, target);
                TaperedPrism::new(6, Vec3::new(gx, 0.14, gz - 0.04), Vec3::new(gx, 0.20, gz + 0.05), 0.032, 0.052, 0.032, 0.052, Vec3::new(0.36, 0.24, 0.13)).emit(frame, true, target);
            }
            HeldRelic::WoodmanAxe => {
                let (ash_wood, worn_ash, forged_iron, blade_steel, bright_edge) = (
                    Vec3::new(0.64, 0.45, 0.25),
                    Vec3::new(0.52, 0.35, 0.19),
                    Vec3::new(0.42, 0.46, 0.52),
                    Vec3::new(0.68, 0.73, 0.79),
                    Vec3::new(0.92, 0.95, 0.99),
                );
                TaperedPrism::hexagonal(Vec3::new(gx, -0.155, gz - 0.008), Vec3::new(gx, -0.115, gz), 0.016, 0.012, worn_ash).emit(frame, true, target);
                TaperedPrism::hexagonal(Vec3::new(gx, -0.115, gz), Vec3::new(gx, 0.045, gz), 0.012, 0.013, worn_ash).emit(frame, false, target);
                TaperedPrism::hexagonal(Vec3::new(gx, 0.045, gz), Vec3::new(gx, 0.252, gz + 0.004), 0.013, 0.015, ash_wood).emit(frame, true, target);
                TaperedPrism::new(6, Vec3::new(gx, 0.184, gz - 0.004), Vec3::new(gx, 0.244, gz - 0.002), 0.017, 0.025, 0.018, 0.026, forged_iron).emit(frame, true, target);
                TaperedPrism::new(4, Vec3::new(gx, 0.194, gz + 0.032), Vec3::new(gx, 0.242, gz + 0.034), 0.013, 0.024, 0.014, 0.026, forged_iron).emit(frame, true, target);
                TaperedPrism::new(4, Vec3::new(gx, 0.128, gz + 0.064), Vec3::new(gx, 0.194, gz + 0.062), 0.008, 0.020, 0.011, 0.026, blade_steel).emit(frame, true, target);
                TaperedPrism::new(4, Vec3::new(gx, 0.194, gz + 0.062), Vec3::new(gx, 0.254, gz + 0.064), 0.011, 0.026, 0.010, 0.024, blade_steel).emit(frame, true, target);
                TaperedPrism::new(4, Vec3::new(gx, 0.124, gz + 0.084), Vec3::new(gx, 0.172, gz + 0.094), 0.003, 0.010, 0.004, 0.014, bright_edge).emit(frame, true, target);
                TaperedPrism::new(4, Vec3::new(gx, 0.172, gz + 0.094), Vec3::new(gx, 0.218, gz + 0.096), 0.004, 0.014, 0.004, 0.014, bright_edge).emit(frame, true, target);
                TaperedPrism::new(4, Vec3::new(gx, 0.218, gz + 0.096), Vec3::new(gx, 0.258, gz + 0.086), 0.004, 0.014, 0.003, 0.010, bright_edge).emit(frame, true, target);
            }
            HeldRelic::KnightlySword => {
                let (grip_leather, guard_iron, fuller_steel, honed_edge) = (
                    Vec3::new(0.26, 0.15, 0.09),
                    Vec3::new(0.32, 0.35, 0.40),
                    Vec3::new(0.56, 0.60, 0.66),
                    Vec3::new(0.84, 0.88, 0.94),
                );
                TaperedPrism::new(8, Vec3::new(gx, -0.106, gz), Vec3::new(gx, -0.068, gz), 0.015, 0.024, 0.015, 0.024, guard_iron).emit(frame, true, target);
                TaperedPrism::new(6, Vec3::new(gx, -0.068, gz), Vec3::new(gx, 0.048, gz), 0.011, 0.015, 0.012, 0.016, grip_leather).emit(frame, false, target);
                TaperedPrism::new(6, Vec3::new(gx, 0.046, gz), Vec3::new(gx, 0.066, gz), 0.013, 0.082, 0.012, 0.076, guard_iron).emit(frame, true, target);
                TaperedPrism::new(4, Vec3::new(gx, 0.066, gz), Vec3::new(gx, 0.420, gz), 0.005, 0.025, 0.004, 0.020, honed_edge).emit(frame, false, target);
                TaperedPrism::new(4, Vec3::new(gx, 0.068, gz), Vec3::new(gx, 0.390, gz), 0.006, 0.011, 0.005, 0.008, fuller_steel).emit(frame, true, target);
                TaperedPrism::new(4, Vec3::new(gx, 0.420, gz), Vec3::new(gx, 0.680, gz), 0.004, 0.020, 0.001, 0.002, honed_edge).emit(frame, true, target);
            }
            HeldRelic::IronSpade => {
                let (ash_wood, worn_ash, socket_iron, spade_steel, honed_edge) = (
                    Vec3::new(0.54, 0.37, 0.20),
                    Vec3::new(0.42, 0.28, 0.15),
                    Vec3::new(0.30, 0.33, 0.38),
                    Vec3::new(0.54, 0.58, 0.64),
                    Vec3::new(0.84, 0.88, 0.93),
                );
                TaperedPrism::new(6, Vec3::new(gx, -0.164, gz), Vec3::new(gx, -0.144, gz), 0.012, 0.034, 0.012, 0.034, worn_ash).emit(frame, true, target);
                TaperedPrism::hexagonal(Vec3::new(gx, -0.150, gz), Vec3::new(gx, 0.190, gz), 0.012, 0.014, ash_wood).emit(frame, false, target);
                TaperedPrism::hexagonal(Vec3::new(gx, 0.185, gz), Vec3::new(gx, 0.235, gz + 0.004), 0.016, 0.020, socket_iron).emit(frame, true, target);
                TaperedPrism::new(4, Vec3::new(gx, 0.230, gz + 0.008), Vec3::new(gx, 0.345, gz + 0.016), 0.007, 0.052, 0.006, 0.046, spade_steel).emit(frame, true, target);
                TaperedPrism::new(4, Vec3::new(gx, 0.345, gz + 0.016), Vec3::new(gx, 0.395, gz + 0.022), 0.006, 0.046, 0.002, 0.018, honed_edge).emit(frame, true, target);
            }
            HeldRelic::FlintPickaxe => Self::emit_flint_pickaxe(gx, gz, frame, target),
            HeldRelic::IronPickaxe => Self::emit_iron_pickaxe(gx, gz, frame, target),
            _ => {}
        }
    }
}
