use glam::Vec3;
use crate::frame::SurfaceFrame;
use crate::{MeshTarget, Quad};
use super::cell::GridCell;

/// Sculpted stone arrival plinth with a luminous cyan-gold compass inlay marking an admin spawn point.
#[derive(Clone, Copy, Debug)]
pub struct SpawnPlinthMesh {
    cell: GridCell,
    floor_elevation: f32,
}

impl SpawnPlinthMesh {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(cell: GridCell, floor_elevation: f32) -> Self {
        Self { cell, floor_elevation }
    }

    #[allow(clippy::too_many_arguments)]
    fn emit_box(
        &self,
        frame: &SurfaceFrame,
        bounds: [f32; 6],
        top_col: Vec3,
        wall_col: Vec3,
        glow: f32,
        target: &mut dyn MeshTarget,
    ) {
        let [u0, u1, v0, v1, h0, h1] = bounds;
        let [t, b, n] = [frame.tangent(), frame.bitangent(), frame.normal()];
        let [b0, b1, b2, b3] = [
            frame.point_at(u0, v0, h0),
            frame.point_at(u1, v0, h0),
            frame.point_at(u1, v1, h0),
            frame.point_at(u0, v1, h0),
        ];
        let [t0, t1, t2, t3] = [
            frame.point_at(u0, v0, h1),
            frame.point_at(u1, v0, h1),
            frame.point_at(u1, v1, h1),
            frame.point_at(u0, v1, h1),
        ];
        target.push_quad(Quad::glowing(t0, t1, t2, t3, n, top_col, glow));
        target.push_quad(Quad::new(b0, b3, b2, b1, -n, wall_col * 0.72));
        target.push_quad(Quad::new(b0, b1, t1, t0, -b, wall_col));
        target.push_quad(Quad::new(b2, b3, t3, t2, b, wall_col));
        target.push_quad(Quad::new(b1, b2, t2, t1, t, wall_col * 0.92));
        target.push_quad(Quad::new(b3, b0, t0, t3, -t, wall_col * 0.92));
    }

    pub fn emit(&self, frame: &SurfaceFrame, target: &mut dyn MeshTarget) {
        let (cu, cv) = self.cell.center();
        let h = GridCell::SPAN * 0.5;
        let z0 = self.floor_elevation;
        let z1 = z0 + GridCell::SPAN;
        let basalt_top = Vec3::new(0.30, 0.34, 0.42);
        let basalt_wall = Vec3::new(0.22, 0.25, 0.32);
        let brass_rim = Vec3::new(0.92, 0.76, 0.28);
        let beacon_cyan = Vec3::new(0.22, 0.94, 0.98);
        self.emit_box(frame, [cu - h, cu + h, cv - h, cv + h, z0, z1 - 0.04], basalt_top, basalt_wall, 0.0, target);
        self.emit_box(
            frame,
            [cu - h * 0.82, cu + h * 0.82, cv - h * 0.82, cv + h * 0.82, z1 - 0.04, z1],
            brass_rim,
            brass_rim * 0.82,
            0.25,
            target,
        );
        self.emit_box(
            frame,
            [cu - h * 0.46, cu + h * 0.46, cv - h * 0.46, cv + h * 0.46, z1, z1 + 0.012],
            beacon_cyan,
            beacon_cyan,
            0.88,
            target,
        );
    }
}
