use glam::Vec3;
use crate::frame::SurfaceFrame;
use crate::{MeshTarget, Quad};
use super::cell::GridCell;

/// Obsidian-glassy fused basalt pad formed beneath sustained `FusionTorch` plasma exhaust (`<= 4` fields).
#[derive(Clone, Copy, Debug)]
pub struct VitrifiedBasaltMesh {
    cell: GridCell,
    floor_elevation: f32,
    height: f32,
}

impl VitrifiedBasaltMesh {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(cell: GridCell, floor_elevation: f32, height: f32) -> Self {
        Self { cell, floor_elevation, height }
    }

    pub fn emit(&self, frame: &SurfaceFrame, target: &mut dyn MeshTarget) {
        let (cu, cv) = self.cell.center();
        let h = GridCell::SPAN * 0.5;
        let z0 = self.floor_elevation;
        let z1 = z0 + self.height;
        let obsidian_top = Vec3::new(0.11, 0.12, 0.16);
        let obsidian_wall = Vec3::new(0.08, 0.09, 0.12);
        let glaze_sheen = Vec3::new(0.17, 0.20, 0.28);
        let ember_vein = Vec3::new(0.96, 0.34, 0.08);
        emit_glowing_prism(frame, [cu - h, cu + h, cv - h, cv + h, z0, z1], obsidian_top, obsidian_wall, 0.0, target);
        emit_glowing_prism(
            frame,
            [cu - h * 0.78, cu + h * 0.78, cv - h * 0.78, cv + h * 0.78, z1, z1 + 0.008],
            glaze_sheen,
            obsidian_wall,
            0.08,
            target,
        );
        emit_glowing_prism(
            frame,
            [cu - h * 0.62, cu + h * 0.62, cv - 0.024, cv + 0.024, z1 + 0.008, z1 + 0.014],
            ember_vein,
            ember_vein,
            0.48,
            target,
        );
        emit_glowing_prism(
            frame,
            [cu - 0.024, cu + 0.024, cv - h * 0.62, cv + h * 0.62, z1 + 0.008, z1 + 0.014],
            ember_vein,
            ember_vein,
            0.48,
            target,
        );
    }
}

/// Salvageable safety-orange flight black-box capsule with titanium crash ribs and locator beacon (`<= 4` fields).
#[derive(Clone, Copy, Debug)]
pub struct BlackBoxPodMesh {
    cell: GridCell,
    floor_elevation: f32,
}

impl BlackBoxPodMesh {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(cell: GridCell, floor_elevation: f32) -> Self {
        Self { cell, floor_elevation }
    }

    pub fn emit(&self, frame: &SurfaceFrame, target: &mut dyn MeshTarget) {
        let (cu, cv) = self.cell.center();
        let h = GridCell::SPAN * 0.44;
        let z0 = self.floor_elevation;
        let z1 = z0 + GridCell::SPAN * 0.86;
        let safety_orange = Vec3::new(0.98, 0.36, 0.05);
        let orange_flank = Vec3::new(0.88, 0.28, 0.04);
        let titanium_rib = Vec3::new(0.20, 0.23, 0.28);
        let hazard_gold = Vec3::new(0.96, 0.80, 0.18);
        let beacon_amber = Vec3::new(1.00, 0.68, 0.14);
        emit_glowing_prism(
            frame,
            [cu - h, cu + h, cv - h, cv + h, z0 + 0.03, z1],
            safety_orange,
            orange_flank,
            0.16,
            target,
        );
        for off in [-0.14, 0.14] {
            emit_glowing_prism(
                frame,
                [cu + off - 0.032, cu + off + 0.032, cv - h - 0.014, cv + h + 0.014, z0, z1 + 0.016],
                titanium_rib,
                titanium_rib,
                0.0,
                target,
            );
        }
        emit_glowing_prism(
            frame,
            [cu - 0.035, cu + 0.035, cv - h - 0.012, cv + h + 0.012, z0 + 0.06, z1 + 0.012],
            hazard_gold,
            hazard_gold,
            0.22,
            target,
        );
        emit_glowing_prism(
            frame,
            [cu - 0.055, cu + 0.055, cv - 0.055, cv + 0.055, z1 + 0.012, z1 + 0.052],
            beacon_amber,
            beacon_amber,
            0.92,
            target,
        );
    }
}

fn emit_glowing_prism(
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
    target.push_quad(Quad::new(b0, b3, b2, b1, -n, wall_col * 0.74));
    target.push_quad(Quad::new(b0, b1, t1, t0, -b, wall_col));
    target.push_quad(Quad::new(b2, b3, t3, t2, b, wall_col));
    target.push_quad(Quad::new(b1, b2, t2, t1, t, wall_col * 0.92));
    target.push_quad(Quad::new(b3, b0, t0, t3, -t, wall_col * 0.92));
}
