use glam::Vec3;
use crate::frame::SurfaceFrame;
use crate::{MeshTarget, Quad};
use super::super::cell::GridCell;
use super::super::pedal_cluster::{FootwellPlinth, WalnutFloorPan};
use super::super::timber_grain::BlockMedium;
use super::super::wedge::WedgeHeading;

/// Sculpted spacecraft flight console with slanted avionics deck, glowing orbital screen, and twin throttle levers.
pub struct FlightConsoleMesh {
    heading: WedgeHeading,
    attitude_deflection: f32,
}

impl FlightConsoleMesh {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(heading: WedgeHeading, attitude_deflection: f32) -> Self {
        Self { heading, attitude_deflection }
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
        let screen_norm = (-fwd + up * 0.22).normalize_or_zero();
        out.push_quad(Quad::glowing(s0, s1, s2, s3, screen_norm, screen_emerald, 0.92));

        let needle = (self.attitude_deflection * 0.06).clamp(-0.08, 0.08);
        let [h0, h1, h2, h3] = [
            local_frame.point_at(needle - 0.04, 0.012, 0.395),
            local_frame.point_at(needle + 0.04, 0.012, 0.395),
            local_frame.point_at(needle + 0.04, 0.014, 0.408),
            local_frame.point_at(needle - 0.04, 0.014, 0.408),
        ];
        out.push_quad(Quad::glowing(h0, h1, h2, h3, screen_norm, horizon_gold, 0.98));

        for lx in [-0.11, 0.11] {
            BlockMedium::emit_box(lx - 0.016, lx + 0.016, -0.08, -0.04, 0.33, 0.42, &local_frame, brass_trim, brass_trim, false, &mut out);
            BlockMedium::emit_box(lx - 0.028, lx + 0.028, -0.09, -0.03, 0.41, 0.44, &local_frame, ruby_lamp, ruby_lamp, true, &mut out);
        }
    }
}
