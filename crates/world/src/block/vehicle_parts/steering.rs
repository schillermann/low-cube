use std::f32::consts::PI;
use glam::Vec3;
use crate::frame::SurfaceFrame;
use crate::{MeshTarget, Quad};
use super::super::cell::GridCell;
use super::super::wedge::WedgeHeading;

/// Sculpted walnut dashboard cowl, instrument dials, and rotating steering wheel column.
pub(super) struct SteeringColumn {
    heading: WedgeHeading,
    steer_angle: f32,
}

impl SteeringColumn {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub(super) const fn new(heading: WedgeHeading, steer_angle: f32) -> Self {
        Self { heading, steer_angle }
    }

    pub(super) fn emit(&self, cell: GridCell, floor_elev: f32, frame: &SurfaceFrame, target: &mut dyn MeshTarget) {
        let (uc, vc) = cell.center();
        let (t, b, up) = (frame.tangent(), frame.bitangent(), frame.normal());
        let (fwd, right) = match self.heading {
            WedgeHeading::East => (t, b),
            WedgeHeading::West => (-t, -b),
            WedgeHeading::North => (b, -t),
            WedgeHeading::South => (-b, t),
        };
        let origin = frame.point_at(uc, vc, floor_elev);
        let (walnut, iron_base, column_steel, spoke_chrome, grip_leather, grip_seam, brass, marker_red) = (
            Vec3::new(0.34, 0.19, 0.11),
            Vec3::new(0.20, 0.22, 0.27),
            Vec3::new(0.32, 0.36, 0.42),
            Vec3::new(0.66, 0.70, 0.76),
            Vec3::new(0.14, 0.16, 0.20),
            Vec3::new(0.22, 0.24, 0.30),
            Vec3::new(0.94, 0.76, 0.24),
            Vec3::new(0.86, 0.22, 0.16),
        );
        let emit_prism = |a0: Vec3, a1: Vec3, a2: Vec3, a3: Vec3, b0: Vec3, b1: Vec3, b2: Vec3, b3: Vec3, w_col: Vec3, t_col: Vec3, out: &mut dyn MeshTarget| {
            let n_top = (b1 - b0).cross(b3 - b0).normalize_or_zero();
            out.push_quad(Quad::new(b0, b1, b2, b3, n_top, t_col));
            out.push_quad(Quad::new(a0, a3, a2, a1, -n_top, w_col));
            for [p0, p1, p2, p3] in [[a0, a1, b1, b0], [a1, a2, b2, b1], [a2, a3, b3, b2], [a3, a0, b0, b3]] {
                out.push_quad(Quad::new(p0, p1, p2, p3, (p1 - p0).cross(p3 - p0).normalize_or_zero(), w_col));
            }
        };
        let base_pt = |x: f32, y: f32, z: f32| origin + right * x + up * y + fwd * z;
        emit_prism(
            base_pt(-0.25, 0.0, -0.06), base_pt(0.25, 0.0, -0.06), base_pt(0.25, 0.0, 0.25), base_pt(-0.25, 0.0, 0.25),
            base_pt(-0.23, 0.25, -0.02), base_pt(0.23, 0.25, -0.02), base_pt(0.25, 0.29, 0.25), base_pt(-0.25, 0.29, 0.25),
            walnut, grip_leather, target,
        );
        emit_prism(
            base_pt(-0.245, 0.24, -0.04), base_pt(0.245, 0.24, -0.04), base_pt(0.245, 0.25, 0.06), base_pt(-0.245, 0.25, 0.06),
            base_pt(-0.245, 0.27, -0.04), base_pt(0.245, 0.27, -0.04), base_pt(0.245, 0.28, 0.06), base_pt(-0.245, 0.28, 0.06),
            brass, grip_leather, target,
        );
        for (gx, dial_col) in [(-0.14f32, Vec3::new(0.26, 0.90, 0.82)), (0.0f32, brass), (0.14f32, marker_red)] {
            target.push_quad(Quad::glowing(
                base_pt(gx - 0.036, 0.13, -0.052), base_pt(gx + 0.036, 0.13, -0.052),
                base_pt(gx + 0.036, 0.21, -0.032), base_pt(gx - 0.036, 0.21, -0.032),
                -fwd, dial_col, 0.88,
            ));
        }
        target.push_quad(Quad::glowing(
            base_pt(-0.045, 0.285, 0.07), base_pt(0.045, 0.285, 0.07), base_pt(0.0, 0.292, 0.18), base_pt(0.0, 0.292, 0.18),
            up, brass, 0.85,
        ));
        emit_prism(
            base_pt(-0.045, 0.16, -0.02), base_pt(0.045, 0.16, -0.02), base_pt(0.045, 0.18, 0.06), base_pt(-0.045, 0.18, 0.06),
            base_pt(-0.032, 0.30, -0.09), base_pt(0.032, 0.30, -0.09), base_pt(0.032, 0.33, -0.03), base_pt(-0.032, 0.33, -0.03),
            iron_base, column_steel, target,
        );

        let hub = base_pt(0.0, 0.33, -0.08);
        let col_axis = (-fwd * 0.84 + up * 0.54).normalize_or_zero();
        let w_up = col_axis.cross(right).normalize_or_zero();
        let steer_phi = self.steer_angle * 0.85;
        let wheel_pt = |angle: f32, r: f32, z: f32| {
            let (s, c) = (angle + steer_phi).sin_cos();
            hub + (right * s + w_up * c) * r + col_axis * z
        };
        let (segs, r_in, r_out, r_hub, hz) = (8usize, 0.125f32, 0.165f32, 0.042f32, 0.018f32);
        let step = (2.0 * PI) / (segs as f32);
        for i in 0..segs {
            let (a0, a1) = ((i as f32) * step, ((i + 1) as f32) * step);
            let rim_col = if i == 0 { marker_red } else if i % 2 == 0 { grip_leather } else { grip_seam };
            emit_prism(
                wheel_pt(a0, r_in, -hz), wheel_pt(a1, r_in, -hz), wheel_pt(a1, r_out, -hz), wheel_pt(a0, r_out, -hz),
                wheel_pt(a0, r_in, hz), wheel_pt(a1, r_in, hz), wheel_pt(a1, r_out, hz), wheel_pt(a0, r_out, hz),
                rim_col, rim_col, target,
            );
            let [h0, h1, h2] = [hub + col_axis * (hz + 0.008), wheel_pt(a0, r_hub, hz + 0.008), wheel_pt(a1, r_hub, hz + 0.008)];
            target.push_quad(Quad::glowing(h0, h2, h1, h1, col_axis, brass, 0.72));
        }
        for spoke_ang in [0.52 * PI, PI, 1.48 * PI] {
            emit_prism(
                wheel_pt(spoke_ang - 0.12, r_hub * 0.8, -0.010),
                wheel_pt(spoke_ang + 0.12, r_hub * 0.8, -0.010),
                wheel_pt(spoke_ang + 0.06, r_in + 0.006, -0.010),
                wheel_pt(spoke_ang - 0.06, r_in + 0.006, -0.010),
                wheel_pt(spoke_ang - 0.12, r_hub * 0.8, 0.010),
                wheel_pt(spoke_ang + 0.12, r_hub * 0.8, 0.010),
                wheel_pt(spoke_ang + 0.06, r_in + 0.006, 0.010),
                wheel_pt(spoke_ang - 0.06, r_in + 0.006, 0.010),
                column_steel, spoke_chrome, target,
            );
        }
    }
}
