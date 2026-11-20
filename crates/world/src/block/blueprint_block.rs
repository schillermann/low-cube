use glam::Vec3;
use crate::frame::SurfaceFrame;
use crate::{MeshTarget, Quad};
use super::cell::GridCell;
use super::timber_grain::BlockMedium;
use super::wedge::WedgeHeading;

/// Sculpted 3D engineer's vehicle blueprint block with cobalt vellum, brass clasps, and glowing schematic lines.
#[derive(Clone, Copy, Debug)]
pub struct BlueprintBlockMesh {
    heading: WedgeHeading,
}

impl BlueprintBlockMesh {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(heading: WedgeHeading) -> Self {
        Self { heading }
    }

    pub fn emit(&self, cell: GridCell, floor_elev: f32, frame: &SurfaceFrame, target: &mut dyn MeshTarget) {
        let ((u0, u1), (v0, v1)) = (cell.u_bounds(), cell.v_bounds());
        let top_elev = floor_elev + 0.44;
        let case_wood = Vec3::new(0.26, 0.17, 0.11);
        let cobalt_vellum = Vec3::new(0.08, 0.25, 0.56);
        let cobalt_deep = Vec3::new(0.05, 0.16, 0.38);
        let brass = Vec3::new(0.88, 0.72, 0.26);
        let cyan_line = Vec3::new(0.68, 0.95, 1.0);
        let scroll_parchment = Vec3::new(0.92, 0.87, 0.74);

        BlockMedium::emit_box(u0 + 0.02, u1 - 0.02, v0 + 0.02, v1 - 0.02, floor_elev, top_elev, frame, case_wood, cobalt_deep, true, target);
        BlockMedium::emit_box(u0 + 0.045, u1 - 0.045, v0 + 0.045, v1 - 0.045, top_elev, top_elev + 0.024, frame, cobalt_deep, cobalt_vellum, false, target);

        let clasp = 0.065;
        for (cu0, cu1) in [(u0 + 0.015, u0 + 0.015 + clasp), (u1 - 0.015 - clasp, u1 - 0.015)] {
            for (cv0, cv1) in [(v0 + 0.015, v0 + 0.015 + clasp), (v1 - 0.015 - clasp, v1 - 0.015)] {
                BlockMedium::emit_box(cu0, cu1, cv0, cv1, floor_elev + 0.01, top_elev + 0.032, frame, brass, brass, true, target);
            }
        }

        let line_h = top_elev + 0.027;
        let n = frame.normal();
        let emit_line = |lu0: f32, lu1: f32, lv0: f32, lv1: f32, out: &mut dyn MeshTarget| {
            let [p0, p1, p2, p3] = [
                frame.point_at(lu0, lv0, line_h),
                frame.point_at(lu1, lv0, line_h),
                frame.point_at(lu1, lv1, line_h),
                frame.point_at(lu0, lv1, line_h),
            ];
            out.push_quad(Quad::glowing(p0, p1, p2, p3, n, cyan_line, 0.55));
        };

        for step in 1..=3 {
            let frac = (step as f32) * 0.25;
            let gu = u0 + 0.06 + (u1 - u0 - 0.12) * frac;
            let gv = v0 + 0.06 + (v1 - v0 - 0.12) * frac;
            emit_line(gu - 0.005, gu + 0.005, v0 + 0.06, v1 - 0.06, target);
            emit_line(u0 + 0.06, u1 - 0.06, gv - 0.005, gv + 0.005, target);
        }

        let (cu, cv) = ((u0 + u1) * 0.5, (v0 + v1) * 0.5);
        emit_line(cu - 0.11, cu + 0.11, cv - 0.07, cv - 0.052, target);
        emit_line(cu - 0.11, cu + 0.11, cv + 0.052, cv + 0.07, target);
        emit_line(cu - 0.11, cu - 0.092, cv - 0.07, cv + 0.07, target);
        emit_line(cu + 0.092, cu + 0.11, cv - 0.07, cv + 0.07, target);
        emit_line(cu - 0.04, cu + 0.04, cv - 0.025, cv + 0.025, target);

        let (su0, su1, sv0, sv1) = match self.heading {
            WedgeHeading::North => (u0 + 0.08, u1 - 0.08, v1 - 0.10, v1 - 0.03),
            WedgeHeading::South => (u0 + 0.08, u1 - 0.08, v0 + 0.03, v0 + 0.10),
            WedgeHeading::East => (u1 - 0.10, u1 - 0.03, v0 + 0.08, v1 - 0.08),
            WedgeHeading::West => (u0 + 0.03, u0 + 0.10, v0 + 0.08, v1 - 0.08),
        };
        BlockMedium::emit_box(su0, su1, sv0, sv1, top_elev + 0.022, top_elev + 0.072, frame, scroll_parchment, scroll_parchment, false, target);
        BlockMedium::emit_box(
            (su0 + su1) * 0.5 - 0.022,
            (su0 + su1) * 0.5 + 0.022,
            (sv0 + sv1) * 0.5 - 0.022,
            (sv0 + sv1) * 0.5 + 0.022,
            top_elev + 0.020,
            top_elev + 0.076,
            frame,
            brass,
            brass,
            false,
            target,
        );
    }
}
