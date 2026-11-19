use glam::Vec3;
use crate::frame::SurfaceFrame;
use crate::{MeshTarget, Quad};
use super::super::cell::GridCell;
use super::NozzleAxis;

/// Sculpted bell-shaped rocket thruster nozzle with cooling ribs and live glowing exhaust plume.
pub struct RocketNozzleMesh {
    axis: NozzleAxis,
    plume_glow: f32,
}

impl RocketNozzleMesh {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(axis: NozzleAxis, plume_glow: f32) -> Self {
        Self { axis, plume_glow }
    }

    fn axis_frame(&self, cell: GridCell, floor_elev: f32, frame: &SurfaceFrame) -> (Vec3, Vec3, Vec3, Vec3) {
        let (uc, vc) = cell.center();
        let center = frame.point_at(uc, vc, floor_elev + 0.25);
        let exhaust = self.axis.exhaust_vector(frame).normalize_or_zero();
        let ref_vec = if exhaust.dot(frame.normal()).abs() > 0.9 {
            frame.tangent()
        } else {
            frame.normal()
        };
        let side_a = exhaust.cross(ref_vec).normalize_or_zero();
        let side_b = side_a.cross(exhaust).normalize_or_zero();
        (center, side_a, side_b, exhaust)
    }

    #[allow(clippy::too_many_arguments)]
    fn emit_frustum(
        center: Vec3,
        sa: Vec3,
        sb: Vec3,
        ex: Vec3,
        z0: f32,
        r0: f32,
        z1: f32,
        r1: f32,
        col: Vec3,
        glow: f32,
        caps: (bool, bool),
        target: &mut dyn MeshTarget,
    ) {
        let ring = |z: f32, r: f32| -> [Vec3; 4] {
            let o = center + ex * z;
            [o - sa * r - sb * r, o + sa * r - sb * r, o + sa * r + sb * r, o - sa * r + sb * r]
        };
        let ([a0, a1, a2, a3], [b0, b1, b2, b3]) = (ring(z0, r0), ring(z1, r1));
        for (p0, p1, q1, q0) in [(a0, a1, b1, b0), (a1, a2, b2, b1), (a2, a3, b3, b2), (a3, a0, b0, b3)] {
            let norm = (p1 - p0).cross(q0 - p0).normalize_or_zero();
            target.push_quad(Quad::glowing(p0, p1, q1, q0, norm, col, glow));
        }
        if caps.0 {
            target.push_quad(Quad::glowing(a0, a3, a2, a1, -ex, col, glow));
        }
        if caps.1 {
            target.push_quad(Quad::glowing(b0, b1, b2, b3, ex, col, glow));
        }
    }

    pub fn emit(&self, cell: GridCell, floor_elev: f32, frame: &SurfaceFrame, target: &mut dyn MeshTarget) {
        let (center, sa, sb, ex) = self.axis_frame(cell, floor_elev, frame);
        let mount_steel = Vec3::new(0.24, 0.26, 0.32);
        let bell_graphite = Vec3::new(0.15, 0.16, 0.20);
        let brass_ring = Vec3::new(0.88, 0.68, 0.22);
        let throat_ember = if self.plume_glow > 0.02 {
            Vec3::new(1.00, 0.86, 0.42)
        } else {
            Vec3::new(0.86, 0.28, 0.12)
        };

        Self::emit_frustum(center, sa, sb, ex, -0.25, 0.22, -0.05, 0.19, mount_steel, 0.0, (true, false), target);
        Self::emit_frustum(center, sa, sb, ex, -0.06, 0.205, 0.01, 0.145, brass_ring, 0.0, (true, true), target);
        Self::emit_frustum(center, sa, sb, ex, 0.01, 0.13, 0.24, 0.22, bell_graphite, 0.0, (false, false), target);
        Self::emit_frustum(center, sa, sb, ex, 0.12, 0.185, 0.15, 0.195, brass_ring, 0.0, (true, true), target);

        let throat_glow = if self.plume_glow > 0.02 { 0.98 } else { 0.55 };
        Self::emit_frustum(center, sa, sb, ex, 0.22, 0.18, 0.245, 0.18, throat_ember, throat_glow, (false, true), target);

        if self.plume_glow > 0.02 {
            let reach = 0.52 + 0.34 * self.plume_glow;
            let core_col = Vec3::new(0.98, 0.94, 0.68);
            let outer_col = Vec3::new(0.99, 0.38, 0.10);
            Self::emit_frustum(center, sa, sb, ex, 0.24, 0.15, 0.24 + reach * 0.55, 0.06, core_col, 0.99, (false, true), target);
            Self::emit_frustum(center, sa, sb, ex, 0.25, 0.19, 0.24 + reach, 0.02, outer_col, 0.94, (false, true), target);
        }
    }
}
