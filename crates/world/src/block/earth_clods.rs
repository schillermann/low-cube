use glam::Vec3;
use crate::frame::SurfaceFrame;
use crate::MeshTarget;
use super::cell::GridCell;
use super::timber_grain::BlockMedium;

const CLOD_A: Vec3 = Vec3::new(0.52, 0.36, 0.21);
const CLOD_B: Vec3 = Vec3::new(0.46, 0.31, 0.18);
const CLOD_C: Vec3 = Vec3::new(0.34, 0.22, 0.12);
const CLOD_D: Vec3 = Vec3::new(0.57, 0.40, 0.24);
const PEBBLE_LIGHT: Vec3 = Vec3::new(0.64, 0.61, 0.55);
const PEBBLE_DARK: Vec3 = Vec3::new(0.49, 0.46, 0.42);

/// Sculpted loam layer of protruding soil clods and river pebbles, shared by earth blocks in hand and exposed terrain soil.
#[derive(Clone, Copy, Debug)]
pub struct EarthClods {
    cell: GridCell,
    floor_elev: f32,
}

impl EarthClods {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(cell: GridCell, floor_elev: f32) -> Self {
        Self { cell, floor_elev }
    }

    /// Emits the complete clodded loam earth block: body, crown clods, and all four flanks.
    pub fn emit_block(&self, frame: &SurfaceFrame, target: &mut dyn MeshTarget) {
        let ((u0, u1), (v0, v1)) = (self.cell.u_bounds(), self.cell.v_bounds());
        BlockMedium::emit_box(u0, u1, v0, v1, self.floor_elev, self.floor_elev + 0.5, frame, Vec3::new(0.41, 0.27, 0.15), Vec3::new(0.45, 0.30, 0.17), true, target);
        self.emit_crown(frame, target);
        for side in 0..4 { self.emit_flank(side, frame, target); }
    }

    /// Emits soil clods and pebbles protruding from the top face of this loam layer.
    pub fn emit_crown(&self, frame: &SurfaceFrame, target: &mut dyn MeshTarget) {
        let ((u0, _), (v0, _), top_elev) = (self.cell.u_bounds(), self.cell.v_bounds(), self.floor_elev + 0.5);
        for (du0, du1, dv0, dv1, dh, col) in [
            (0.03, 0.22, 0.03, 0.23, 0.018, CLOD_A), (0.25, 0.47, 0.04, 0.21, 0.013, CLOD_B),
            (0.05, 0.26, 0.26, 0.46, 0.015, CLOD_C), (0.28, 0.46, 0.24, 0.47, 0.020, CLOD_D),
            (0.14, 0.34, 0.15, 0.35, 0.026, CLOD_A), (0.35, 0.44, 0.07, 0.16, 0.028, PEBBLE_LIGHT),
            (0.07, 0.15, 0.34, 0.42, 0.024, PEBBLE_DARK),
        ] {
            BlockMedium::emit_box(u0 + du0, u0 + du1, v0 + dv0, v0 + dv1, top_elev, top_elev + dh, frame, col, col, false, target);
        }
    }

    /// Emits soil clods protruding from one flank (0 = -v, 1 = +v, 2 = -u, 3 = +u) of this loam layer.
    pub fn emit_flank(&self, side: usize, frame: &SurfaceFrame, target: &mut dyn MeshTarget) {
        let ((u0, u1), (v0, v1)) = (self.cell.u_bounds(), self.cell.v_bounds());
        for (a0, a1, h0, h1, d, col) in [
            (0.04, 0.24, 0.04, 0.21, 0.016, CLOD_B), (0.27, 0.46, 0.04, 0.19, 0.013, CLOD_C),
            (0.05, 0.28, 0.25, 0.44, 0.018, CLOD_A), (0.31, 0.46, 0.23, 0.45, 0.015, CLOD_D),
            (0.15, 0.33, 0.14, 0.31, 0.022, CLOD_B), (0.34, 0.43, 0.09, 0.18, 0.025, PEBBLE_LIGHT),
            (0.07, 0.16, 0.30, 0.39, 0.021, PEBBLE_DARK),
        ] {
            let (y0, y1) = (self.floor_elev + h0, self.floor_elev + h1);
            match side {
                0 => BlockMedium::emit_box(u0 + a0, u0 + a1, v0 - d, v0, y0, y1, frame, col, col, true, target),
                1 => BlockMedium::emit_box(u1 - a1, u1 - a0, v1, v1 + d, y0, y1, frame, col, col, true, target),
                2 => BlockMedium::emit_box(u0 - d, u0, v0 + a0, v0 + a1, y0, y1, frame, col, col, true, target),
                _ => BlockMedium::emit_box(u1, u1 + d, v1 - a1, v1 - a0, y0, y1, frame, col, col, true, target),
            }
        }
    }
}
