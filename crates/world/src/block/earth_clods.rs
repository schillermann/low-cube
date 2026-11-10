use glam::Vec3;
use crate::frame::SurfaceFrame;
use crate::{MeshTarget, Quad, TurfMeshTarget};
use super::cell::GridCell;
use super::shape::VoxelShape;
use super::timber_grain::BlockMedium;

const LOAM_CROWN: Vec3 = Vec3::new(0.45, 0.30, 0.17);
const LOAM_WALL: Vec3 = Vec3::new(0.41, 0.27, 0.15);
const CLOD_A: Vec3 = Vec3::new(0.52, 0.36, 0.21);
const CLOD_B: Vec3 = Vec3::new(0.46, 0.31, 0.18);
const CLOD_C: Vec3 = Vec3::new(0.34, 0.22, 0.12);
const CLOD_D: Vec3 = Vec3::new(0.57, 0.40, 0.24);
const PEBBLE_LIGHT: Vec3 = Vec3::new(0.64, 0.61, 0.55);
const PEBBLE_DARK: Vec3 = Vec3::new(0.49, 0.46, 0.42);

const CROWN_PATTERN: [(f32, f32, f32, f32, f32, Vec3); 7] = [
    (0.03, 0.22, 0.03, 0.23, 0.018, CLOD_A),
    (0.25, 0.47, 0.04, 0.21, 0.013, CLOD_B),
    (0.05, 0.26, 0.26, 0.46, 0.015, CLOD_C),
    (0.28, 0.46, 0.24, 0.47, 0.020, CLOD_D),
    (0.14, 0.34, 0.15, 0.35, 0.026, CLOD_A),
    (0.35, 0.44, 0.07, 0.16, 0.028, PEBBLE_LIGHT),
    (0.07, 0.15, 0.34, 0.42, 0.024, PEBBLE_DARK),
];

const SOLE_PATTERN: [(f32, f32, f32, f32, f32, Vec3); 7] = [
    (0.04, 0.23, 0.04, 0.22, 0.017, CLOD_B),
    (0.26, 0.46, 0.03, 0.22, 0.014, CLOD_C),
    (0.04, 0.25, 0.25, 0.45, 0.016, CLOD_A),
    (0.27, 0.47, 0.25, 0.46, 0.019, CLOD_D),
    (0.15, 0.35, 0.14, 0.34, 0.024, CLOD_B),
    (0.08, 0.17, 0.07, 0.16, 0.026, PEBBLE_LIGHT),
    (0.33, 0.42, 0.33, 0.41, 0.023, PEBBLE_DARK),
];

const FLANK_PATTERN: [(f32, f32, f32, f32, f32, Vec3); 7] = [
    (0.04, 0.24, 0.04, 0.21, 0.016, CLOD_B),
    (0.27, 0.46, 0.04, 0.19, 0.013, CLOD_C),
    (0.05, 0.28, 0.25, 0.44, 0.018, CLOD_A),
    (0.31, 0.46, 0.23, 0.45, 0.015, CLOD_D),
    (0.15, 0.33, 0.14, 0.31, 0.022, CLOD_B),
    (0.34, 0.43, 0.09, 0.18, 0.025, PEBBLE_LIGHT),
    (0.07, 0.16, 0.30, 0.39, 0.021, PEBBLE_DARK),
];

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

    /// Emits a terraced earth block preserving all four clodded flanks, clodded underside, and either a grass cap or clodded bare crown.
    pub fn emit_terraced(&self, top_col: Vec3, bare_top: bool, frame: &SurfaceFrame, target: &mut dyn MeshTarget) {
        let ((u0, u1), (v0, v1)) = (self.cell.u_bounds(), self.cell.v_bounds());
        let crown_col = if bare_top { LOAM_CROWN } else { top_col };
        BlockMedium::emit_box(u0, u1, v0, v1, self.floor_elev, self.floor_elev + 0.5, frame, LOAM_WALL, crown_col, true, target);
        let crown = crate::MeadowCrown::for_cell(self.cell, frame.normal());
        if bare_top {
            self.emit_crown(frame, target);
        } else if crown.verdant(top_col) {
            crown.emit_hybrid(self.cell.corner_points(frame, self.floor_elev + 0.5), frame.normal(), top_col, target);
        }
        self.emit_sole(frame, target);
        for side in 0..4 {
            self.emit_flank(side, frame, target);
        }
    }

    /// Emits any sculpted voxel form with the full-cube earth block's loam tones, crown clods, sole clods, and flank clods.
    pub fn emit_shaped(&self, shape: &dyn VoxelShape, top_col: Vec3, bare_top: bool, frame: &SurfaceFrame, target: &mut dyn MeshTarget) {
        let crown_col = if bare_top { LOAM_CROWN } else { top_col };
        let mut turf = TurfMeshTarget::new(target, frame.normal(), self.cell.corner_points(frame, self.floor_elev), crown_col, LOAM_WALL, false);
        shape.emit_shape(self.cell, self.floor_elev, frame, LOAM_WALL, &mut turf);
        turf.seal_underside();
        if bare_top {
            self.emit_shaped_crown(shape, frame, target);
        }
        self.emit_shaped_sole(shape, frame, target);
        for side in 0..4 {
            self.emit_shaped_flank(shape, side, frame, target);
        }
    }

    /// Emits soil clods and pebbles protruding from the top face of this loam layer.
    pub fn emit_crown(&self, frame: &SurfaceFrame, target: &mut dyn MeshTarget) {
        let ((u0, _), (v0, _), top_elev) = (self.cell.u_bounds(), self.cell.v_bounds(), self.floor_elev + 0.5);
        for (du0, du1, dv0, dv1, dh, col) in CROWN_PATTERN {
            BlockMedium::emit_box(u0 + du0, u0 + du1, v0 + dv0, v0 + dv1, top_elev, top_elev + dh, frame, col, col, false, target);
        }
    }

    fn emit_shaped_crown(&self, shape: &dyn VoxelShape, frame: &SurfaceFrame, target: &mut dyn MeshTarget) {
        let ((u0, _), (v0, _)) = (self.cell.u_bounds(), self.cell.v_bounds());
        let (n, t, b) = (frame.normal(), frame.tangent(), frame.bitangent());
        for (du0, du1, dv0, dv1, dh, col) in CROWN_PATTERN {
            let (fu0, fu1, fv0, fv1) = (du0 * 2.0, du1 * 2.0, dv0 * 2.0, dv1 * 2.0);
            let (e00, e10, e11, e01, ec) = (
                shape.elevation_at(fu0, fv0),
                shape.elevation_at(fu1, fv0),
                shape.elevation_at(fu1, fv1),
                shape.elevation_at(fu0, fv1),
                shape.elevation_at(du0 + du1, dv0 + dv1),
            );
            let (e_min, e_max) = (e00.min(e10).min(e11).min(e01), e00.max(e10).max(e11).max(e01));
            if ec <= 0.0 || e_min <= 0.0 {
                continue;
            }
            let [h0, h1, h2, h3] = if e_max - e_min > 0.23 {
                [self.floor_elev + ec; 4]
            } else {
                [self.floor_elev + e00, self.floor_elev + e10, self.floor_elev + e11, self.floor_elev + e01]
            };
            let (cu0, cu1, cv0, cv1) = (u0 + du0, u0 + du1, v0 + dv0, v0 + dv1);
            let [b0, b1, b2, b3] = [
                frame.point_at(cu0, cv0, h0), frame.point_at(cu1, cv0, h1),
                frame.point_at(cu1, cv1, h2), frame.point_at(cu0, cv1, h3),
            ];
            let [p0, p1, p2, p3] = [
                frame.point_at(cu0, cv0, h0 + dh), frame.point_at(cu1, cv0, h1 + dh),
                frame.point_at(cu1, cv1, h2 + dh), frame.point_at(cu0, cv1, h3 + dh),
            ];
            let raw_n = (p1 - p0).cross(p3 - p0);
            let top_n = if raw_n.dot(n) >= 0.0 { raw_n.normalize_or_zero() } else { -raw_n.normalize_or_zero() };
            target.push_quad(Quad::new(p0, p1, p2, p3, if top_n.length_squared() > 0.5 { top_n } else { n }, col));
            target.push_quad(Quad::new(b0, b1, p1, p0, -b, col));
            target.push_quad(Quad::new(p3, p2, b2, b3, b, col));
            target.push_quad(Quad::new(b3, b0, p0, p3, -t, col));
            target.push_quad(Quad::new(b1, b2, p2, p1, t, col));
        }
    }

    /// Emits soil clods and pebbles protruding downward from the underside of this loam layer.
    pub fn emit_sole(&self, frame: &SurfaceFrame, target: &mut dyn MeshTarget) {
        let ((u0, _), (v0, _)) = (self.cell.u_bounds(), self.cell.v_bounds());
        for (du0, du1, dv0, dv1, dh, col) in SOLE_PATTERN {
            BlockMedium::emit_box(u0 + du0, u0 + du1, v0 + dv0, v0 + dv1, self.floor_elev - dh, self.floor_elev, frame, col, col, true, target);
        }
    }

    fn emit_shaped_sole(&self, shape: &dyn VoxelShape, frame: &SurfaceFrame, target: &mut dyn MeshTarget) {
        let ((u0, _), (v0, _)) = (self.cell.u_bounds(), self.cell.v_bounds());
        for (du0, du1, dv0, dv1, dh, col) in SOLE_PATTERN {
            if shape.elevation_at(du0 + du1, dv0 + dv1) > 0.0 {
                BlockMedium::emit_box(u0 + du0, u0 + du1, v0 + dv0, v0 + dv1, self.floor_elev - dh, self.floor_elev, frame, col, col, true, target);
            }
        }
    }

    /// Emits soil clods protruding from one flank (0 = -v, 1 = +v, 2 = -u, 3 = +u) of this loam layer.
    pub fn emit_flank(&self, side: usize, frame: &SurfaceFrame, target: &mut dyn MeshTarget) {
        let ((u0, u1), (v0, v1)) = (self.cell.u_bounds(), self.cell.v_bounds());
        for (a0, a1, h0, h1, d, col) in FLANK_PATTERN {
            self.emit_flank_box(side, u0, u1, v0, v1, a0, a1, self.floor_elev + h0, self.floor_elev + h1, d, col, frame, target);
        }
    }

    fn emit_shaped_flank(&self, shape: &dyn VoxelShape, side: usize, frame: &SurfaceFrame, target: &mut dyn MeshTarget) {
        let ((u0, u1), (v0, v1)) = (self.cell.u_bounds(), self.cell.v_bounds());
        for (a0, a1, h0, h1, d, col) in FLANK_PATTERN {
            let wall_h = match side {
                0 => shape.elevation_at(a0 * 2.0, 0.0).min(shape.elevation_at(a1 * 2.0, 0.0)),
                1 => shape.elevation_at(1.0 - a1 * 2.0, 1.0).min(shape.elevation_at(1.0 - a0 * 2.0, 1.0)),
                2 => shape.elevation_at(0.0, a0 * 2.0).min(shape.elevation_at(0.0, a1 * 2.0)),
                _ => shape.elevation_at(1.0, 1.0 - a1 * 2.0).min(shape.elevation_at(1.0, 1.0 - a0 * 2.0)),
            };
            if wall_h >= 0.08 {
                let scale = (wall_h * 2.0).min(1.0);
                let (y0, y1) = (self.floor_elev + h0 * scale, self.floor_elev + h1 * scale);
                self.emit_flank_box(side, u0, u1, v0, v1, a0, a1, y0, y1, d, col, frame, target);
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn emit_flank_box(
        &self, side: usize, u0: f32, u1: f32, v0: f32, v1: f32, a0: f32, a1: f32, y0: f32, y1: f32, d: f32, col: Vec3, frame: &SurfaceFrame, target: &mut dyn MeshTarget,
    ) {
        match side {
            0 => BlockMedium::emit_box(u0 + a0, u0 + a1, v0 - d, v0, y0, y1, frame, col, col, true, target),
            1 => BlockMedium::emit_box(u1 - a1, u1 - a0, v1, v1 + d, y0, y1, frame, col, col, true, target),
            2 => BlockMedium::emit_box(u0 - d, u0, v0 + a0, v0 + a1, y0, y1, frame, col, col, true, target),
            _ => BlockMedium::emit_box(u1, u1 + d, v1 - a1, v1 - a0, y0, y1, frame, col, col, true, target),
        }
    }
}
