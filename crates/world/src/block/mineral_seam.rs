use glam::Vec3;
use crate::{MeshTarget, Quad};
use crate::frame::SurfaceFrame;
use super::cell::GridCell;
use super::timber_grain::BlockMedium;

/// Sculpted 3D mineral-ore matrix and gleaming metallic seam geometry for walls, floors, and held blocks.
#[derive(Clone, Copy, Debug)]
pub struct MineralSeam {
    medium: BlockMedium,
}

impl MineralSeam {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(medium: BlockMedium) -> Self {
        Self { medium }
    }

    const fn palette(self) -> (Vec3, Vec3, Vec3, Vec3, f32) {
        match self.medium {
            BlockMedium::CopperOre => (
                Vec3::new(0.36, 0.35, 0.33),
                Vec3::new(0.26, 0.25, 0.24),
                Vec3::new(0.86, 0.46, 0.22),
                Vec3::new(0.18, 0.68, 0.52),
                0.42,
            ),
            BlockMedium::IronOre => (
                Vec3::new(0.30, 0.30, 0.33),
                Vec3::new(0.21, 0.21, 0.24),
                Vec3::new(0.72, 0.75, 0.80),
                Vec3::new(0.64, 0.30, 0.20),
                0.36,
            ),
            _ => (
                Vec3::new(0.25, 0.23, 0.27),
                Vec3::new(0.17, 0.16, 0.19),
                Vec3::new(0.98, 0.80, 0.22),
                Vec3::new(0.92, 0.62, 0.14),
                0.72,
            ),
        }
    }

    fn emit_nuggets(&self, q0: Vec3, q1: Vec3, q3: Vec3, out_n: Vec3, target: &mut dyn MeshTarget) {
        let (rock, crag, primary_ore, accent_ore, ore_glow) = self.palette();
        let (va, vb) = ((q1 - q0) * 2.0, (q3 - q0) * 2.0);
        let (na, nb) = (va.normalize_or_zero(), vb.normalize_or_zero());
        for (a0, a1, b0, b1, d, col, glow) in [
            (0.04, 0.20, 0.05, 0.21, 0.018, rock * 1.12, 0.08),
            (0.28, 0.45, 0.26, 0.44, 0.018, crag, 0.08),
            (0.12, 0.34, 0.16, 0.30, 0.026, primary_ore, ore_glow),
            (0.32, 0.45, 0.07, 0.19, 0.024, accent_ore, ore_glow * 0.85),
            (0.06, 0.19, 0.30, 0.43, 0.024, primary_ore, ore_glow),
        ] {
            let side_col = col * 0.84;
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

    pub fn emit_wall_flank(
        &self,
        edge: ((f32, f32), (f32, f32)),
        layer: i32,
        norm: Vec3,
        frame: &SurfaceFrame,
        target: &mut dyn MeshTarget,
    ) {
        let (((a0, b0), (a1, b1)), y0, y1) = (edge, (layer as f32) * 0.5, ((layer + 1) as f32) * 0.5);
        let (rock, seam, _, _, _) = self.palette();
        let (ym0, ym1) = (y0 + 0.035, y1 - 0.035);
        let (q0, q1, q3) = (frame.point_at(a0, b0, y0), frame.point_at(a1, b1, y0), frame.point_at(a0, b0, y1));
        target.push_quad(Quad::glowing(q0, q1, frame.point_at(a1, b1, ym0), frame.point_at(a0, b0, ym0), norm, seam, 0.12));
        target.push_quad(Quad::glowing(frame.point_at(a0, b0, ym0), frame.point_at(a1, b1, ym0), frame.point_at(a1, b1, ym1), frame.point_at(a0, b0, ym1), norm, rock, 0.16));
        target.push_quad(Quad::glowing(frame.point_at(a0, b0, ym1), frame.point_at(a1, b1, ym1), frame.point_at(a1, b1, y1), q3, norm, seam, 0.12));
        self.emit_nuggets(q0, q1, q3, norm, target);
    }

    pub fn emit_floor_face(&self, cell: GridCell, top_h: f32, frame: &SurfaceFrame, target: &mut dyn MeshTarget) -> Vec3 {
        let ((u0, u1), (v0, v1)) = (cell.u_bounds(), cell.v_bounds());
        let (rock, _, _, _, _) = self.palette();
        let [p0, p1, p2, p3] = [
            frame.point_at(u0, v0, top_h),
            frame.point_at(u1, v0, top_h),
            frame.point_at(u1, v1, top_h),
            frame.point_at(u0, v1, top_h),
        ];
        target.push_quad(Quad::glowing(p0, p1, p2, p3, frame.normal(), rock, 0.16));
        self.emit_nuggets(p0, p1, p3, frame.normal(), target);
        rock
    }

    pub fn emit_ore_cube(&self, cell: GridCell, floor_elev: f32, frame: &SurfaceFrame, target: &mut dyn MeshTarget) {
        let ((u0, u1), (v0, v1), y0, y1) = (cell.u_bounds(), cell.v_bounds(), floor_elev, floor_elev + 0.5);
        let (_, seam, _, _, _) = self.palette();
        let _ = self.emit_floor_face(cell, y1, frame, target);
        let [c0, c1, c2, c3] = [
            frame.point_at(u0, v0, y0),
            frame.point_at(u1, v0, y0),
            frame.point_at(u1, v1, y0),
            frame.point_at(u0, v1, y0),
        ];
        target.push_quad(Quad::glowing(c3, c2, c1, c0, -frame.normal(), seam, 0.14));
        self.emit_nuggets(c3, c2, c0, -frame.normal(), target);
        let layer = (floor_elev * 2.0).round() as i32;
        for (edge, norm) in [
            (((u0, v1), (u1, v1)), frame.bitangent()),
            (((u1, v0), (u0, v0)), -frame.bitangent()),
            (((u1, v0), (u1, v1)), frame.tangent()),
            (((u0, v1), (u0, v0)), -frame.tangent()),
        ] {
            self.emit_wall_flank(edge, layer, norm, frame, target);
        }
    }
}
