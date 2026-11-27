use glam::Vec3;
use world::{BlockMold, MeshTarget, Quad};
use super::super::compartment::{CompartmentBackplate, ShowcasedCompartment, SlotPlaque};
use super::super::limb::{AxialLimb, FacetedSolid, LimbFrame, TaperedPrism};
use super::plinth::{SlopedBeam, SquarePlinth, TitaniumIngot};
use super::temper::ToolCondition;
use super::tools::ToolSculpture;
use super::HeldRelic;

struct LitSlotTarget<'a> {
    light_dir: Vec3,
    zenith: Vec3,
    inner: &'a mut dyn MeshTarget,
}

impl<'a> MeshTarget for LitSlotTarget<'a> {
    fn push_quad(&mut self, quad: Quad) {
        if quad.radiate(|g| g > 1.5) {
            return;
        }
        let lit = quad.transfer(|v0, v1, v2, v3, n, col| {
            let shade = 1.30 + 0.45 * n.dot(self.light_dir).max(0.0);
            Quad::new(v0, v1, v2, v3, self.zenith, col * shade + Vec3::splat(0.07))
        });
        self.inner.push_quad(lit);
    }
}

/// Decorator over `HeldRelic`, `BlockMold`, and `ToolCondition` emitting 3D palm-held and satchel-slot geometry.
#[derive(Clone, Copy)]
pub struct RelicFigure {
    relic: HeldRelic,
    mold: BlockMold,
    condition: ToolCondition,
}

impl RelicFigure {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(relic: HeldRelic, mold: BlockMold, condition: ToolCondition) -> Self {
        Self { relic, mold, condition }
    }

    fn emit_cantilever(cradle: &LimbFrame, target: &mut dyn MeshTarget) {
        let (stone, iron_dark, iron_steel, beam_alloy, beam_trim, cyan_bolt, amber) = (
            Vec3::new(0.52, 0.54, 0.58),
            Vec3::new(0.24, 0.27, 0.32),
            Vec3::new(0.42, 0.46, 0.52),
            Vec3::new(0.66, 0.72, 0.80),
            Vec3::new(0.44, 0.50, 0.58),
            Vec3::new(0.28, 0.84, 0.96),
            Vec3::new(0.98, 0.68, 0.18),
        );
        SquarePlinth::new((Vec3::new(-0.037, 0.0, -0.027), Vec3::new(0.017, 0.009, 0.027)), (stone, stone)).emit(cradle, true, target);
        SquarePlinth::new((Vec3::new(-0.037, 0.009, -0.015), Vec3::new(-0.007, 0.020, 0.015)), (iron_dark, iron_steel)).emit(cradle, false, target);
        AxialLimb::new(Vec3::new(-0.022, 0.017, 0.0), Vec3::new(-0.022, 0.165, 0.0), 0.012, 0.011, beam_alloy).emit(cradle, true, target);
        TaperedPrism::new(6, Vec3::new(-0.022, 0.105, 0.0), Vec3::new(-0.022, 0.136, 0.0), 0.016, 0.016, 0.016, 0.016, iron_dark).emit(cradle, true, target);
        AxialLimb::new(Vec3::new(-0.022, 0.120, -0.020), Vec3::new(-0.022, 0.120, 0.020), 0.007, 0.007, cyan_bolt).emit(cradle, true, target);
        AxialLimb::new(Vec3::new(-0.026, 0.122, 0.0), Vec3::new(0.056, 0.122, 0.0), 0.010, 0.010, beam_alloy).emit(cradle, true, target);
        SquarePlinth::new((Vec3::new(-0.0355, 0.062, -0.0135), Vec3::new(-0.0085, 0.075, 0.0135)), (iron_dark, iron_steel)).emit(cradle, true, target);
        SquarePlinth::new((Vec3::new(0.015, 0.110, -0.0115), Vec3::new(0.029, 0.134, 0.0115)), (iron_dark, iron_steel)).emit(cradle, true, target);
        let brace_ends = (Vec3::new(-0.020, 0.069, 0.0), Vec3::new(0.022, 0.120, 0.0));
        SlopedBeam::new(brace_ends, (0.0052, 0.0056), (beam_trim, beam_alloy)).emit(cradle, false, target);
        SlopedBeam::new(brace_ends, (0.0025, 0.0066), (iron_dark, iron_steel)).emit(cradle, false, target);
        TaperedPrism::new(6, Vec3::new(0.054, 0.102, 0.0), Vec3::new(0.054, 0.142, 0.0), 0.012, 0.022, 0.012, 0.022, iron_dark).emit(cradle, true, target);
        TaperedPrism::new(6, Vec3::new(0.062, 0.108, 0.0), Vec3::new(0.062, 0.136, 0.0), 0.010, 0.016, 0.010, 0.016, amber).emit(cradle, true, target);
    }

    fn emit_camera(cradle: &LimbFrame, target: &mut dyn MeshTarget) {
        let (chassis, grip, alloy, glass, flash) = (
            Vec3::new(0.18, 0.21, 0.26),
            Vec3::new(0.28, 0.33, 0.40),
            Vec3::new(0.76, 0.82, 0.88),
            Vec3::new(0.32, 0.68, 0.88),
            Vec3::new(0.95, 0.92, 0.82),
        );
        TaperedPrism::new(4, Vec3::new(0.0, 0.004, 0.0), Vec3::new(0.0, 0.068, 0.0), 0.054, 0.036, 0.054, 0.036, chassis).emit(cradle, true, target);
        TaperedPrism::new(4, Vec3::new(0.0, 0.008, 0.0), Vec3::new(0.0, 0.064, 0.0), 0.058, 0.028, 0.058, 0.028, grip).emit(cradle, true, target);
        AxialLimb::new(Vec3::new(0.0, 0.038, 0.020), Vec3::new(0.0, 0.038, 0.068), 0.026, 0.022, alloy).emit(cradle, true, target);
        AxialLimb::new(Vec3::new(0.0, 0.038, 0.064), Vec3::new(0.0, 0.038, 0.074), 0.017, 0.017, glass).emit(cradle, true, target);
        TaperedPrism::new(6, Vec3::new(-0.024, 0.066, 0.0), Vec3::new(-0.024, 0.082, 0.0), 0.010, 0.010, 0.008, 0.008, alloy).emit(cradle, true, target);
        TaperedPrism::new(4, Vec3::new(0.016, 0.066, 0.004), Vec3::new(0.016, 0.086, 0.004), 0.014, 0.012, 0.018, 0.014, flash).emit(cradle, true, target);
    }

    pub fn emit_in_palm(self, side: f32, frame: &LimbFrame, target: &mut dyn MeshTarget) {
        let (gx, gz) = (-side * 0.025, 0.054);
        if self.relic.free_palm() {
            return;
        }
        if self.relic == HeldRelic::CantileverCrane {
            Self::emit_cantilever(&frame.palm_cradle(side, Vec3::new(-side * 0.015, 0.002, 0.048)), target);
        } else if self.relic == HeldRelic::FieldCamera {
            let cradle = frame
                .palm_cradle(side, Vec3::new(-side * 0.015, 0.002, 0.048))
                .yawed(self.mold.palm_camera_yaw(side));
            Self::emit_camera(&cradle, target);
        } else if self.relic.placeable_block() {
            frame.emit_cradled_block(
                side,
                Vec3::new(-side * 0.015, 0.001, 0.048),
                self.mold.in_medium(self.relic.block_medium()),
                0.096,
                Vec3::new(0.44, 0.31, 0.19),
                Vec3::new(0.28, 0.65, 0.25),
                target,
            );
        } else if self.relic == HeldRelic::TitaniumBillet {
            TitaniumIngot::vacuum_cast().emit(&frame.palm_cradle(side, Vec3::new(-side * 0.015, 0.002, 0.048)), true, target);
        } else {
            ToolSculpture::new(self.relic, self.condition).emit_in_palm(gx, gz, frame, target);
        }
    }

    pub fn emit_showcase(self, slot_frame: &LimbFrame, target: &mut dyn MeshTarget) {
        if self.relic.free_palm() {
            return;
        }
        let light_dir = (slot_frame.point(0.4, 0.6, -0.7) - slot_frame.point(0.0, 0.0, 0.0)).normalize_or_zero();
        let zenith = (slot_frame.point(0.0, 1.0, -1.0) - slot_frame.point(0.0, 0.0, 0.0)).normalize_or_zero();
        let mut lit = LitSlotTarget { light_dir, zenith, inner: target };
        let showcase = slot_frame.shifted(0.0, 0.0, -0.048).pitched(-0.46).yawed(0.64);
        if self.relic == HeldRelic::CantileverCrane {
            Self::emit_cantilever(&showcase.shifted(0.0, -0.032, 0.0).scaled(0.44), &mut lit);
        } else if self.relic == HeldRelic::FieldCamera {
            Self::emit_camera(&showcase.shifted(0.0, -0.022, 0.0).scaled(0.58), &mut lit);
        } else if self.relic.placeable_block() {
            showcase.emit_upright_block(
                Vec3::new(0.0, -0.025, 0.0),
                self.mold.in_medium(self.relic.block_medium()),
                0.082,
                Vec3::new(0.44, 0.31, 0.19),
                Vec3::new(0.28, 0.65, 0.25),
                &mut lit,
            );
        } else if self.relic == HeldRelic::TitaniumBillet {
            TitaniumIngot::vacuum_cast().emit(&showcase.shifted(0.0, -0.018, 0.0).scaled(0.68), true, &mut lit);
        } else {
            let (scale, yaw, mid_y, mid_z) = if self.relic == HeldRelic::RigWrench {
                (0.185, 1.18, 0.068, 0.056)
            } else {
                (0.215, 0.55, 0.056, 0.054)
            };
            let tool_frame = slot_frame
                .shifted(0.0, 0.0, -0.035)
                .pitched(-0.22)
                .yawed(yaw)
                .scaled(scale)
                .shifted(0.025, -mid_y, -mid_z);
            self.emit_in_palm(1.0, &tool_frame, &mut lit);
        }
    }

    pub fn emit_in_slot(self, selected: bool, rim: Vec3, slot_frame: &LimbFrame, target: &mut dyn MeshTarget) {
        ShowcasedCompartment::new(CompartmentBackplate::new(selected, self.relic.gripped(), rim), self)
            .emit_slot(slot_frame, target);
    }
}
