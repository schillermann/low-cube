use glam::Vec3;
use world::{BlockMold, MeshTarget, Quad};
use crate::menu::PixelStamp;
use super::super::limb::{AxialLimb, FacetedSolid, LimbFrame, TaperedPrism};
use super::HeldRelic;

struct LitSlotTarget<'a> {
    light_dir: Vec3,
    zenith: Vec3,
    inner: &'a mut dyn MeshTarget,
}

impl<'a> MeshTarget for LitSlotTarget<'a> {
    fn push_quad(&mut self, quad: Quad) {
        let lit = quad.transfer(|v0, v1, v2, v3, n, col| {
            let shade = 1.30 + 0.45 * n.dot(self.light_dir).max(0.0);
            Quad::new(v0, v1, v2, v3, self.zenith, col * shade + Vec3::splat(0.07))
        });
        self.inner.push_quad(lit);
    }
}

/// Decorator over `HeldRelic` and `BlockMold` emitting 3D palm-held and satchel-slot geometry.
#[derive(Clone, Copy)]
pub struct RelicFigure {
    relic: HeldRelic,
    mold: BlockMold,
}

impl RelicFigure {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(relic: HeldRelic, mold: BlockMold) -> Self {
        Self { relic, mold }
    }

    fn emit_deadwood(cradle: &LimbFrame, target: &mut dyn MeshTarget) {
        let (bark, dark, twig, heart, leaf) = (
            Vec3::new(0.46, 0.29, 0.16),
            Vec3::new(0.36, 0.23, 0.12),
            Vec3::new(0.54, 0.36, 0.20),
            Vec3::new(0.84, 0.68, 0.40),
            Vec3::new(0.34, 0.58, 0.22),
        );
        AxialLimb::new(Vec3::new(-0.082, 0.015, -0.024), Vec3::new(-0.070, 0.015, -0.019), 0.012, 0.013, heart).emit(cradle, true, target);
        AxialLimb::new(Vec3::new(-0.070, 0.015, -0.019), Vec3::new(-0.006, 0.014, 0.002), 0.014, 0.012, bark).emit(cradle, false, target);
        AxialLimb::new(Vec3::new(-0.006, 0.014, 0.002), Vec3::new(0.048, 0.019, 0.014), 0.012, 0.010, dark).emit(cradle, false, target);
        AxialLimb::new(Vec3::new(0.048, 0.019, 0.014), Vec3::new(0.094, 0.032, 0.035), 0.010, 0.005, bark).emit(cradle, true, target);
        AxialLimb::new(Vec3::new(-0.010, 0.015, 0.001), Vec3::new(-0.044, 0.052, 0.044), 0.008, 0.004, twig).emit(cradle, true, target);
        AxialLimb::new(Vec3::new(0.032, 0.018, 0.010), Vec3::new(0.068, 0.056, -0.024), 0.007, 0.003, dark).emit(cradle, true, target);
        AxialLimb::new(Vec3::new(-0.042, 0.050, 0.042), Vec3::new(-0.056, 0.064, 0.056), 0.006, 0.002, leaf).emit(cradle, true, target);
    }

    fn emit_flint(cradle: &LimbFrame, target: &mut dyn MeshTarget) {
        let (cortex, core, flake, edge) = (
            Vec3::new(0.76, 0.70, 0.58),
            Vec3::new(0.19, 0.22, 0.28),
            Vec3::new(0.52, 0.60, 0.70),
            Vec3::new(0.80, 0.86, 0.94),
        );
        TaperedPrism::new(7, Vec3::new(0.0, 0.003, 0.0), Vec3::new(0.0, 0.016, 0.0), 0.045, 0.036, 0.042, 0.034, cortex).emit(cradle, false, target);
        TaperedPrism::new(7, Vec3::new(0.0, 0.014, 0.0), Vec3::new(-0.003, 0.038, 0.003), 0.041, 0.033, 0.025, 0.020, core).emit(cradle, true, target);
        TaperedPrism::new(5, Vec3::new(-0.008, 0.018, 0.004), Vec3::new(-0.016, 0.054, 0.010), 0.023, 0.017, 0.003, 0.008, flake)
            .emit(&cradle.pitched(0.24).yawed(0.42), true, target);
        TaperedPrism::new(5, Vec3::new(0.010, 0.016, -0.004), Vec3::new(0.018, 0.048, -0.008), 0.019, 0.014, 0.002, 0.006, edge)
            .emit(&cradle.pitched(-0.20).yawed(-0.52), true, target);
    }

    fn emit_cantilever(cradle: &LimbFrame, target: &mut dyn MeshTarget) {
        let (stone, iron_dark, iron_steel, timber, brass, amber) = (
            Vec3::new(0.54, 0.56, 0.60),
            Vec3::new(0.26, 0.30, 0.36),
            Vec3::new(0.44, 0.48, 0.54),
            Vec3::new(0.76, 0.56, 0.32),
            Vec3::new(0.88, 0.72, 0.28),
            Vec3::new(0.98, 0.68, 0.18),
        );
        TaperedPrism::new(8, Vec3::new(0.0, 0.0, 0.0), Vec3::new(0.0, 0.016, 0.0), 0.046, 0.046, 0.042, 0.042, stone).emit(cradle, true, target);
        TaperedPrism::new(6, Vec3::new(-0.022, 0.014, 0.0), Vec3::new(-0.022, 0.032, 0.0), 0.020, 0.020, 0.018, 0.018, iron_dark).emit(cradle, true, target);
        AxialLimb::new(Vec3::new(-0.022, 0.026, 0.0), Vec3::new(-0.022, 0.165, 0.0), 0.012, 0.011, timber).emit(cradle, true, target);
        TaperedPrism::new(6, Vec3::new(-0.022, 0.105, 0.0), Vec3::new(-0.022, 0.136, 0.0), 0.016, 0.016, 0.016, 0.016, iron_dark).emit(cradle, true, target);
        AxialLimb::new(Vec3::new(-0.022, 0.120, -0.020), Vec3::new(-0.022, 0.120, 0.020), 0.007, 0.007, brass).emit(cradle, true, target);
        AxialLimb::new(Vec3::new(-0.022, 0.068, 0.0), Vec3::new(0.024, 0.122, 0.0), 0.008, 0.008, iron_steel).emit(cradle, true, target);
        AxialLimb::new(Vec3::new(-0.026, 0.122, 0.0), Vec3::new(0.056, 0.122, 0.0), 0.010, 0.010, timber).emit(cradle, true, target);
        TaperedPrism::new(6, Vec3::new(0.054, 0.102, 0.0), Vec3::new(0.054, 0.142, 0.0), 0.012, 0.022, 0.012, 0.022, iron_dark).emit(cradle, true, target);
        TaperedPrism::new(6, Vec3::new(0.062, 0.108, 0.0), Vec3::new(0.062, 0.136, 0.0), 0.010, 0.016, 0.010, 0.016, amber).emit(cradle, true, target);
    }

    fn emit_camera(cradle: &LimbFrame, target: &mut dyn MeshTarget) {
        let (leather, mahogany, brass, glass, flash) = (
            Vec3::new(0.18, 0.19, 0.24),
            Vec3::new(0.46, 0.25, 0.14),
            Vec3::new(0.88, 0.72, 0.28),
            Vec3::new(0.32, 0.68, 0.88),
            Vec3::new(0.95, 0.92, 0.82),
        );
        TaperedPrism::new(4, Vec3::new(0.0, 0.004, 0.0), Vec3::new(0.0, 0.068, 0.0), 0.054, 0.036, 0.054, 0.036, leather).emit(cradle, true, target);
        TaperedPrism::new(4, Vec3::new(0.0, 0.008, 0.0), Vec3::new(0.0, 0.064, 0.0), 0.058, 0.028, 0.058, 0.028, mahogany).emit(cradle, true, target);
        AxialLimb::new(Vec3::new(0.0, 0.038, 0.020), Vec3::new(0.0, 0.038, 0.068), 0.026, 0.022, brass).emit(cradle, true, target);
        AxialLimb::new(Vec3::new(0.0, 0.038, 0.064), Vec3::new(0.0, 0.038, 0.074), 0.017, 0.017, glass).emit(cradle, true, target);
        TaperedPrism::new(6, Vec3::new(-0.024, 0.066, 0.0), Vec3::new(-0.024, 0.082, 0.0), 0.010, 0.010, 0.008, 0.008, brass).emit(cradle, true, target);
        TaperedPrism::new(4, Vec3::new(0.016, 0.066, 0.004), Vec3::new(0.016, 0.086, 0.004), 0.014, 0.012, 0.018, 0.014, flash).emit(cradle, true, target);
    }

    pub fn emit_in_palm(self, side: f32, frame: &LimbFrame, target: &mut dyn MeshTarget) {
        let (gx, gz) = (-side * 0.025, 0.054);
        match self.relic {
            HeldRelic::EmptyPalm => {}
            HeldRelic::QuarriedStone
            | HeldRelic::ExcavatedEarth
            | HeldRelic::TimberBlock
            | HeldRelic::PlankBlock
            | HeldRelic::CrustBlock
            | HeldRelic::SteeringWheel
            | HeldRelic::AircraftEngine
            | HeldRelic::RocketEngine
            | HeldRelic::CabinSeat
            | HeldRelic::CarWheel => {
                frame.emit_cradled_block(
                    side,
                    Vec3::new(-side * 0.015, 0.001, 0.048),
                    self.mold.in_medium(self.relic.block_medium()),
                    0.096,
                    Vec3::new(0.44, 0.31, 0.19),
                    Vec3::new(0.52, 0.38, 0.24),
                    target,
                );
            }
            HeldRelic::DeadwoodBough => Self::emit_deadwood(&frame.palm_cradle(side, Vec3::new(-side * 0.016, 0.001, 0.048)), target),
            HeldRelic::FlintStone => Self::emit_flint(&frame.palm_cradle(side, Vec3::new(-side * 0.016, 0.001, 0.048)), target),
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
            HeldRelic::CantileverCrane => Self::emit_cantilever(&frame.palm_cradle(side, Vec3::new(-side * 0.015, 0.002, 0.048)), target),
            HeldRelic::FieldCamera => {
                let cradle = frame
                    .palm_cradle(side, Vec3::new(-side * 0.015, 0.002, 0.048))
                    .yawed(self.mold.palm_camera_yaw(side));
                Self::emit_camera(&cradle, target);
            }
        }
    }

    pub fn emit_in_slot(self, selected: bool, rim: Vec3, slot_frame: &LimbFrame, target: &mut dyn MeshTarget) {
        let bg = if selected { Vec3::new(0.28, 0.33, 0.44) } else { Vec3::new(0.16, 0.19, 0.27) };
        slot_frame.emit_framed_plaque(0.044, 0.044, 0.0032, bg, rim * 1.25, target);
        let light_dir = (slot_frame.point(0.4, 0.6, -0.7) - slot_frame.point(0.0, 0.0, 0.0)).normalize_or_zero();
        let zenith = (slot_frame.point(0.0, 1.0, -1.0) - slot_frame.point(0.0, 0.0, 0.0)).normalize_or_zero();
        let mut lit = LitSlotTarget { light_dir, zenith, inner: target };
        let showcase = slot_frame.shifted(0.0, 0.0, -0.048).pitched(-0.46).yawed(0.64);
        match self.relic {
            HeldRelic::EmptyPalm => {}
            HeldRelic::ExcavatedEarth
            | HeldRelic::QuarriedStone
            | HeldRelic::TimberBlock
            | HeldRelic::PlankBlock
            | HeldRelic::CrustBlock
            | HeldRelic::SteeringWheel
            | HeldRelic::AircraftEngine
            | HeldRelic::RocketEngine
            | HeldRelic::CabinSeat
            | HeldRelic::CarWheel => {
                showcase.emit_upright_block(Vec3::new(0.0, -0.025, 0.0), self.mold.in_medium(self.relic.block_medium()), 0.082, Vec3::new(0.44, 0.31, 0.19), Vec3::new(0.56, 0.41, 0.25), &mut lit);
            }
            HeldRelic::DeadwoodBough => Self::emit_deadwood(&showcase.shifted(0.0, -0.012, 0.0).scaled(0.38), &mut lit),
            HeldRelic::FlintStone => Self::emit_flint(&showcase.shifted(0.0, -0.016, 0.0).scaled(0.62), &mut lit),
            HeldRelic::CantileverCrane => Self::emit_cantilever(&showcase.shifted(0.0, -0.032, 0.0).scaled(0.44), &mut lit),
            HeldRelic::FieldCamera => Self::emit_camera(&showcase.shifted(0.0, -0.022, 0.0).scaled(0.58), &mut lit),
            _ => {
                let (scale, yaw, mid_y, mid_z) = match self.relic {
                    HeldRelic::KnightlySword => (0.090, 0.45, 0.285, 0.054),
                    HeldRelic::IronSpade => (0.125, 0.45, 0.115, 0.062),
                    HeldRelic::WoodmanAxe => (0.165, 1.32, 0.052, 0.092),
                    HeldRelic::TimberTorch => (0.185, 0.45, 0.075, 0.058),
                    _ => (0.205, 0.85, 0.050, 0.054),
                };
                let tool_frame = slot_frame.shifted(0.0, 0.0, -0.035).pitched(-0.22).yawed(yaw).scaled(scale).shifted(0.025, -mid_y, -mid_z);
                self.emit_in_palm(1.0, &tool_frame, &mut lit);
            }
        }
    }

    pub fn emit_legend(relic: HeldRelic, legend_frame: &LimbFrame, target: &mut dyn MeshTarget) {
        legend_frame.emit_framed_plaque(0.168, 0.032, 0.0024, Vec3::new(0.08, 0.10, 0.15), Vec3::new(0.42, 0.27, 0.16), target);
        PixelStamp::new(0.0024, 0.0024, Vec3::new(1.55, 1.30, 0.48)).emit_on_frame(relic.title(), 0.0, 0.017, 0.016, legend_frame, target);
        let hint = PixelStamp::new(0.0022, 0.0022, Vec3::new(1.42, 1.46, 1.54));
        hint.emit_on_frame("WASD: NAVIGATION  ESC: CLOSE", 0.0, 0.0, 0.016, legend_frame, target);
        hint.emit_on_frame("Q: LEFT HAND  E: RIGHT HAND", 0.0, -0.017, 0.016, legend_frame, target);
    }
}
