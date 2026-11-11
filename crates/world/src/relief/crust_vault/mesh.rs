use glam::Vec3;
use crate::{MeshTarget, Quad};
use crate::block::GridCell;
use crate::frame::SurfaceFrame;

/// Decorator over `SurfaceFrame` emitting mineral-veined 3D crust wall and floor geometry.
#[derive(Clone, Copy)]
pub struct VaultGeometry<'a> {
    frame: &'a SurfaceFrame,
}

impl<'a> VaultGeometry<'a> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(frame: &'a SurfaceFrame) -> Self {
        Self { frame }
    }

    fn emit_crag_relief(q0: Vec3, q1: Vec3, q3: Vec3, out_n: Vec3, tint: f32, target: &mut dyn MeshTarget) {
        let (va, vb) = ((q1 - q0) * 2.0, (q3 - q0) * 2.0);
        let (na, nb) = (va.normalize_or_zero(), vb.normalize_or_zero());
        for (a0, a1, b0, b1, d, raw_col, glow) in [
            (0.03, 0.21, 0.04, 0.22, 0.022, Vec3::new(0.43, 0.38, 0.35), 0.18),
            (0.25, 0.46, 0.05, 0.20, 0.018, Vec3::new(0.25, 0.21, 0.20), 0.18),
            (0.06, 0.27, 0.26, 0.45, 0.025, Vec3::new(0.38, 0.33, 0.30), 0.18),
            (0.30, 0.47, 0.24, 0.46, 0.020, Vec3::new(0.47, 0.41, 0.37), 0.18),
            (0.15, 0.35, 0.15, 0.29, 0.030, Vec3::new(0.84, 0.58, 0.26), 0.62),
            (0.34, 0.44, 0.08, 0.17, 0.028, Vec3::new(0.88, 0.68, 0.36), 0.70),
            (0.07, 0.17, 0.32, 0.41, 0.026, Vec3::new(0.76, 0.48, 0.20), 0.55),
        ] {
            let (col, side_col) = (raw_col * tint, raw_col * (tint * 0.86));
            let [r0, r1, r2, r3] = [
                q0 + va * a0 + vb * b0,
                q0 + va * a1 + vb * b0,
                q0 + va * a1 + vb * b1,
                q0 + va * a0 + vb * b1,
            ];
            let [t0, t1, t2, t3] = [r0 + out_n * d, r1 + out_n * d, r2 + out_n * d, r3 + out_n * d];
            target.push_quad(Quad::glowing(t0, t1, t2, t3, out_n, col, glow));
            target.push_quad(Quad::glowing(r0, r1, t1, t0, -nb, side_col, glow));
            target.push_quad(Quad::glowing(t3, t2, r2, r3, nb, side_col, glow));
            target.push_quad(Quad::glowing(r3, r0, t0, t3, -na, side_col, glow));
            target.push_quad(Quad::glowing(r1, r2, t2, t1, na, side_col, glow));
        }
    }

    pub fn emit_crust_sole(&self, cell: GridCell, floor_h: f32, target: &mut dyn MeshTarget) {
        let ((u0, u1), (v0, v1)) = (cell.u_bounds(), cell.v_bounds());
        let (c0, c1, c2, c3) = (
            self.frame.point_at(u0, v0, floor_h),
            self.frame.point_at(u1, v0, floor_h),
            self.frame.point_at(u1, v1, floor_h),
            self.frame.point_at(u0, v1, floor_h),
        );
        target.push_quad(Quad::glowing(c3, c2, c1, c0, -self.frame.normal(), Vec3::new(0.31, 0.27, 0.25), 0.35));
        Self::emit_crag_relief(c3, c2, c0, -self.frame.normal(), 1.0, target);
    }

    pub fn emit_crust_flank(&self, edge: ((f32, f32), (f32, f32)), layer: i32, floor_step: i32, norm: Vec3, target: &mut dyn MeshTarget) {
        let (((a0, b0), (a1, b1)), y0, y1) = (edge, (layer as f32) * 0.5, ((layer + 1) as f32) * 0.5);
        let shade = 0.92 + ((layer - floor_step) as f32) * 0.04;
        let ((seam, face), (ym0, ym1)) = ((Vec3::new(0.22, 0.19, 0.18), Vec3::new(0.31, 0.27, 0.25) * shade), (y0 + 0.035, y1 - 0.035));
        let (q0, q1, q3) = (self.frame.point_at(a0, b0, y0), self.frame.point_at(a1, b1, y0), self.frame.point_at(a0, b0, y1));
        target.push_quad(Quad::glowing(q0, q1, self.frame.point_at(a1, b1, ym0), self.frame.point_at(a0, b0, ym0), norm, seam, 0.24));
        target.push_quad(Quad::glowing(self.frame.point_at(a0, b0, ym0), self.frame.point_at(a1, b1, ym0), self.frame.point_at(a1, b1, ym1), self.frame.point_at(a0, b0, ym1), norm, face, 0.30));
        target.push_quad(Quad::glowing(self.frame.point_at(a0, b0, ym1), self.frame.point_at(a1, b1, ym1), self.frame.point_at(a1, b1, y1), q3, norm, seam, 0.24));
        Self::emit_crag_relief(q0, q1, q3, norm, shade, target);
    }

    pub fn emit_crust_floor(&self, cell: GridCell, top_h: f32, core_exposed: bool, target: &mut dyn MeshTarget) -> Vec3 {
        let ((u0, u1), (v0, v1)) = (cell.u_bounds(), cell.v_bounds());
        let (p0, p1, p2, p3) = (
            self.frame.point_at(u0, v0, top_h),
            self.frame.point_at(u1, v0, top_h),
            self.frame.point_at(u1, v1, top_h),
            self.frame.point_at(u0, v1, top_h),
        );
        if core_exposed {
            let col = Vec3::new(0.99, 0.36, 0.06);
            target.push_quad(Quad::glowing(p0, p1, p2, p3, self.frame.normal(), col, 0.95));
            Self::emit_crag_relief(p0, p1, p3, self.frame.normal(), 1.35, target);
            col
        } else {
            let col = Vec3::new(0.35, 0.31, 0.28);
            target.push_quad(Quad::glowing(p0, p1, p2, p3, self.frame.normal(), col, 0.26));
            Self::emit_crag_relief(p0, p1, p3, self.frame.normal(), 1.0, target);
            col
        }
    }
}
