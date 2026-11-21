use glam::Vec3;
use crate::frame::SurfaceFrame;
use crate::{MeshTarget, Quad};
use super::super::cell::GridCell;
use super::super::pedal_cluster::{FootwellPlinth, WalnutFloorPan};
use super::super::timber_grain::BlockMedium;
use super::super::wedge::WedgeHeading;

/// Sculpted spacecraft flight console with orbital screen, left fore/aft thrust lever, and right 4-way control stick.
pub struct FlightConsoleMesh {
    heading: WedgeHeading,
    motion: (f32, f32),
    guidance_engaged: bool,
}

impl FlightConsoleMesh {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn with_guidance(heading: WedgeHeading, motion: (f32, f32), guidance_engaged: bool) -> Self {
        Self { heading, motion, guidance_engaged }
    }

    /// Secondary constructor defaulting guidance indicator to engaged.
    pub const fn new(heading: WedgeHeading, motion: (f32, f32)) -> Self {
        Self::with_guidance(heading, motion, true)
    }

    pub fn emit(&self, cell: GridCell, floor_elev: f32, frame: &SurfaceFrame, target: &mut dyn MeshTarget) {
        WalnutFloorPan::new(self.heading, 0.07).emit(cell, floor_elev, frame, target);
        let (uc, vc) = cell.center();
        let fwd = self.heading.forward_vector(frame);
        let up = frame.normal();
        let right = fwd.cross(up).normalize_or_zero();
        let origin = frame.point_at(uc, vc, floor_elev);
        let local_frame = SurfaceFrame::new(right, fwd, up, 0.0);

        let hull_dark = Vec3::new(0.18, 0.21, 0.27);
        let brass_trim = Vec3::new(0.90, 0.72, 0.24);
        let bezel_slate = Vec3::new(0.12, 0.15, 0.20);
        let grip_leather = Vec3::new(0.15, 0.17, 0.22);
        let screen_emerald = Vec3::new(0.16, 0.94, 0.78);
        let horizon_gold = Vec3::new(0.98, 0.84, 0.28);
        let ruby_lamp = Vec3::new(0.98, 0.26, 0.20);

        struct ShiftedConsole<'a> {
            shift: Vec3,
            inner: &'a mut dyn MeshTarget,
        }
        impl MeshTarget for ShiftedConsole<'_> {
            fn push_quad(&mut self, quad: Quad) {
                let (s, glow) = (self.shift, quad.radiate(|g| g));
                self.inner.push_quad(quad.transfer(|v0, v1, v2, v3, n, col| {
                    Quad::glowing(v0 + s, v1 + s, v2 + s, v3 + s, n, col, glow)
                }));
            }
        }
        let mut out = ShiftedConsole { shift: origin, inner: target };

        BlockMedium::emit_box(-0.19, 0.19, -0.10, 0.16, 0.07, 0.30, &local_frame, hull_dark, bezel_slate, true, &mut out);
        BlockMedium::emit_box(-0.21, 0.21, -0.12, 0.18, 0.29, 0.33, &local_frame, brass_trim, bezel_slate, true, &mut out);
        BlockMedium::emit_box(-0.18, 0.18, 0.02, 0.17, 0.33, 0.47, &local_frame, hull_dark, brass_trim, false, &mut out);

        let [s0, s1, s2, s3] = [
            local_frame.point_at(-0.14, 0.015, 0.35),
            local_frame.point_at(0.14, 0.015, 0.35),
            local_frame.point_at(0.14, 0.035, 0.45),
            local_frame.point_at(-0.14, 0.035, 0.45),
        ];
        let (screen_col, screen_glow) = if self.guidance_engaged {
            (screen_emerald, 0.92)
        } else {
            (Vec3::new(0.86, 0.42, 0.14), 0.72)
        };
        let screen_norm = (-fwd + up * 0.22).normalize_or_zero();
        out.push_quad(Quad::glowing(s0, s1, s2, s3, screen_norm, screen_col, screen_glow));

        let (thrust, stick_defl) = (self.motion.0.clamp(-1.0, 1.0), self.motion.1.clamp(-1.0, 1.0));
        let needle = (stick_defl * 0.06).clamp(-0.08, 0.08);
        let [h0, h1, h2, h3] = [
            local_frame.point_at(needle - 0.04, 0.012, 0.395),
            local_frame.point_at(needle + 0.04, 0.012, 0.395),
            local_frame.point_at(needle + 0.04, 0.014, 0.408),
            local_frame.point_at(needle - 0.04, 0.014, 0.408),
        ];
        out.push_quad(Quad::glowing(h0, h1, h2, h3, screen_norm, horizon_gold, 0.98));

        let th_v = -0.062 + thrust * 0.036;
        BlockMedium::emit_box(-0.136, -0.084, -0.112, -0.008, 0.330, 0.346, &local_frame, hull_dark, bezel_slate, false, &mut out);
        BlockMedium::emit_box(-0.122, -0.098, th_v - 0.014, th_v + 0.014, 0.344, 0.412, &local_frame, brass_trim, brass_trim, false, &mut out);
        BlockMedium::emit_box(-0.152, -0.068, th_v - 0.020, th_v + 0.020, 0.408, 0.436, &local_frame, grip_leather, brass_trim, true, &mut out);

        let st_u = 0.110 + stick_defl * 0.024;
        let st_v = -0.062 + stick_defl * 0.018;
        BlockMedium::emit_box(0.080, 0.140, -0.092, -0.032, 0.330, 0.354, &local_frame, grip_leather, bezel_slate, false, &mut out);
        BlockMedium::emit_box(st_u - 0.012, st_u + 0.012, st_v - 0.012, st_v + 0.012, 0.352, 0.418, &local_frame, brass_trim, brass_trim, false, &mut out);
        BlockMedium::emit_box(st_u - 0.020, st_u + 0.020, st_v - 0.022, st_v + 0.022, 0.404, 0.450, &local_frame, grip_leather, ruby_lamp, true, &mut out);
    }
}
