use glam::Vec3;
use crate::frame::SurfaceFrame;
use crate::{MeshTarget, Quad, TurfMeshTarget};
use super::cell::GridCell;
use super::shape::VoxelShape;
use super::timber_grain::BlockMedium;

const LOAM_CROWN: Vec3 = Vec3::new(0.45, 0.30, 0.17);
pub(crate) const LOAM_WALL: Vec3 = Vec3::new(0.41, 0.27, 0.15);
const CLOD_A: Vec3 = Vec3::new(0.52, 0.36, 0.21);
const CLOD_B: Vec3 = Vec3::new(0.46, 0.31, 0.18);
const CLOD_C: Vec3 = Vec3::new(0.34, 0.22, 0.12);
const CLOD_D: Vec3 = Vec3::new(0.57, 0.40, 0.24);
const PEBBLE_LIGHT: Vec3 = Vec3::new(0.64, 0.61, 0.55);
const PEBBLE_DARK: Vec3 = Vec3::new(0.49, 0.46, 0.42);
const TURF_BASE: Vec3 = Vec3::new(0.28, 0.62, 0.24);
pub(crate) const SOD_A: Vec3 = Vec3::new(0.35, 0.72, 0.29);
pub(crate) const SOD_B: Vec3 = Vec3::new(0.30, 0.66, 0.25);
pub(crate) const SOD_C: Vec3 = Vec3::new(0.24, 0.56, 0.21);

pub(crate) const CROWN_PATTERN: [(f32, f32, f32, f32, f32, Vec3); 7] = [
    (0.03, 0.22, 0.03, 0.23, 0.018, CLOD_A),
    (0.25, 0.47, 0.04, 0.21, 0.013, CLOD_B),
    (0.05, 0.26, 0.26, 0.46, 0.015, CLOD_C),
    (0.28, 0.46, 0.24, 0.47, 0.020, CLOD_D),
    (0.14, 0.34, 0.15, 0.35, 0.026, CLOD_A),
    (0.35, 0.44, 0.07, 0.16, 0.028, PEBBLE_LIGHT),
    (0.07, 0.15, 0.34, 0.42, 0.024, PEBBLE_DARK),
];

pub(crate) const TURF_FRINGE_PATTERN: [(f32, f32, f32, f32, f32, Vec3); 3] = [
    (0.04, 0.17, 0.35, 0.44, 0.016, SOD_B),
    (0.20, 0.34, 0.37, 0.44, 0.018, SOD_A),
    (0.37, 0.47, 0.34, 0.44, 0.016, SOD_C),
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

pub(crate) const FLANK_PATTERN: [(f32, f32, f32, f32, f32, Vec3); 7] = [
    (0.04, 0.24, 0.04, 0.21, 0.016, CLOD_B),
    (0.27, 0.46, 0.04, 0.19, 0.013, CLOD_C),
    (0.05, 0.28, 0.25, 0.44, 0.018, CLOD_A),
    (0.31, 0.46, 0.23, 0.45, 0.015, CLOD_D),
    (0.15, 0.33, 0.14, 0.31, 0.022, CLOD_B),
    (0.34, 0.43, 0.09, 0.18, 0.025, PEBBLE_LIGHT),
    (0.07, 0.16, 0.30, 0.39, 0.021, PEBBLE_DARK),
];

/// Sculpted loam layer of protruding soil clods, river pebbles, and grass sod pads.
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

    pub(crate) fn turf_tone(&self, top_col: Vec3, swatch: Vec3) -> Vec3 {
        if top_col.y > top_col.x * 1.15 && top_col.y > top_col.z * 1.15 {
            (swatch * (top_col / TURF_BASE)).clamp(Vec3::ZERO, Vec3::ONE)
        } else {
            (top_col * (swatch.y / TURF_BASE.y)).clamp(Vec3::ZERO, Vec3::ONE)
        }
    }

    /// Emits a terraced earth block with clodded flanks, clodded underside, and either a grass-sod crown with overhanging turf rim or a bare loam crown.
    pub fn emit_terraced(&self, top_col: Vec3, bare_top: bool, frame: &SurfaceFrame, target: &mut dyn MeshTarget) {
        let ((u0, u1), (v0, v1)) = (self.cell.u_bounds(), self.cell.v_bounds());
        let crown_col = if bare_top { LOAM_CROWN } else { top_col };
        BlockMedium::emit_box(u0, u1, v0, v1, self.floor_elev, self.floor_elev + 0.5, frame, LOAM_WALL, crown_col, true, target);
        if bare_top {
            self.emit_crown(frame, target);
            for side in 0..4 {
                self.emit_flank(side, frame, target);
            }
        } else {
            let corners = self.cell.corner_points(frame, self.floor_elev + 0.5);
            let crown = crate::MeadowCrown::for_cell(self.cell, frame.normal());
            crown.emit_contour_terraces(corners, frame.normal(), top_col, true, target);
            crown.emit_hybrid(corners, frame.normal(), top_col, target);
            for side in 0..4 {
                self.emit_turf_flank(side, top_col, frame, target);
            }
        }
        self.emit_sole(frame, target);
    }

    /// Emits any sculpted voxel form with the full-cube earth block's loam tones, crown clods or turf sods, sole clods, and flank clods.
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
            self.emit_shaped_flank(shape, side, bare_top, top_col, frame, target);
        }
    }

    /// Emits bare soil clods and pebbles protruding from the top face of this loam layer.
    pub fn emit_crown(&self, frame: &SurfaceFrame, target: &mut dyn MeshTarget) {
        let ((u0, _), (v0, _), top_elev) = (self.cell.u_bounds(), self.cell.v_bounds(), self.floor_elev + 0.5);
        for (du0, du1, dv0, dv1, dh, col) in CROWN_PATTERN {
            BlockMedium::emit_box(u0 + du0, u0 + du1, v0 + dv0, v0 + dv1, top_elev, top_elev + dh, frame, col, col, false, target);
        }
    }

    fn emit_shaped_crown(&self, shape: &dyn VoxelShape, frame: &SurfaceFrame, target: &mut dyn MeshTarget) {
        let ((u0, _), (v0, _), n) = (self.cell.u_bounds(), self.cell.v_bounds(), frame.normal());
        for (du0, du1, dv0, dv1, dh, col) in CROWN_PATTERN {
            let (fu0, fu1, fv0, fv1) = (du0 * 2.0, du1 * 2.0, dv0 * 2.0, dv1 * 2.0);
            let (e00, e10, e11, e01, ec) = (
                shape.elevation_at(fu0, fv0), shape.elevation_at(fu1, fv0),
                shape.elevation_at(fu1, fv1), shape.elevation_at(fu0, fv1),
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
            let lift = n * dh;
            let [p0, p1, p2, p3] = [b0 + lift, b1 + lift, b2 + lift, b3 + lift];
            let raw_n = (b1 - b0).cross(b3 - b0);
            let top_n = if raw_n.dot(n) >= 0.0 { raw_n.normalize_or_zero() } else { -raw_n.normalize_or_zero() };
            let (nu, nv) = ((b1 - b0).normalize_or_zero(), (b3 - b0).normalize_or_zero());
            target.push_quad(Quad::new(p0, p1, p2, p3, if top_n.length_squared() > 0.5 { top_n } else { n }, col));
            target.push_quad(Quad::new(b0, b1, p1, p0, -nv, col));
            target.push_quad(Quad::new(p3, p2, b2, b3, nv, col));
            target.push_quad(Quad::new(b3, b0, p0, p3, -nu, col));
            target.push_quad(Quad::new(b1, b2, p2, p1, nu, col));
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

    /// Emits soil clods protruding from one bare flank (0 = -v, 1 = +v, 2 = -u, 3 = +u) of this loam layer.
    pub fn emit_flank(&self, side: usize, frame: &SurfaceFrame, target: &mut dyn MeshTarget) {
        let ((u0, u1), (v0, v1)) = (self.cell.u_bounds(), self.cell.v_bounds());
        for (a0, a1, h0, h1, d, col) in FLANK_PATTERN {
            self.emit_flank_box(side, u0, u1, v0, v1, a0, a1, self.floor_elev + h0, self.floor_elev + h1, d, col, frame, target);
        }
    }

    /// Emits overhanging green turf rim and stepped sod fringes above lower flank soil clods on `side`.
    pub fn emit_turf_flank(&self, side: usize, top_col: Vec3, frame: &SurfaceFrame, target: &mut dyn MeshTarget) {
        let ((u0, u1), (v0, v1)) = (self.cell.u_bounds(), self.cell.v_bounds());
        let rim_col = self.turf_tone(top_col, SOD_C);
        self.emit_flank_box(side, u0, u1, v0, v1, 0.0, 0.5, self.floor_elev + 0.43, self.floor_elev + 0.499, 0.008, rim_col, frame, target);
        for (a0, a1, h0, h1, d, swatch) in TURF_FRINGE_PATTERN {
            let col = self.turf_tone(top_col, swatch);
            self.emit_flank_box(side, u0, u1, v0, v1, a0, a1, self.floor_elev + h0, self.floor_elev + h1, d, col, frame, target);
        }
        for (a0, a1, h0, h1, d, col) in FLANK_PATTERN {
            let capped_h1 = h1.min(0.37);
            self.emit_flank_box(side, u0, u1, v0, v1, a0, a1, self.floor_elev + h0, self.floor_elev + capped_h1, d, col, frame, target);
        }
    }

    fn emit_shaped_flank(&self, shape: &dyn VoxelShape, side: usize, bare_top: bool, top_col: Vec3, frame: &SurfaceFrame, target: &mut dyn MeshTarget) {
        let ((u0, u1), (v0, v1)) = (self.cell.u_bounds(), self.cell.v_bounds());
        let wall_height = |a0: f32, a1: f32| match side {
            0 => shape.elevation_at(a0 * 2.0, 0.0).min(shape.elevation_at(a1 * 2.0, 0.0)),
            1 => shape.elevation_at(1.0 - a1 * 2.0, 1.0).min(shape.elevation_at(1.0 - a0 * 2.0, 1.0)),
            2 => shape.elevation_at(0.0, a0 * 2.0).min(shape.elevation_at(0.0, a1 * 2.0)),
            _ => shape.elevation_at(1.0, 1.0 - a1 * 2.0).min(shape.elevation_at(1.0, 1.0 - a0 * 2.0)),
        };
        if !bare_top {
            for (a0, a1, h0, h1, d, swatch) in TURF_FRINGE_PATTERN {
                let wh = wall_height(a0, a1);
                if wh >= 0.12 {
                    let scale = (wh * 2.0).min(1.0);
                    let col = self.turf_tone(top_col, swatch);
                    self.emit_flank_box(side, u0, u1, v0, v1, a0, a1, self.floor_elev + h0 * scale, self.floor_elev + h1 * scale, d, col, frame, target);
                }
            }
        }
        for (a0, a1, h0, h1, d, col) in FLANK_PATTERN {
            let wall_h = wall_height(a0, a1);
            if wall_h >= 0.08 {
                let scale = (wall_h * 2.0).min(1.0);
                let top_h1 = if bare_top { h1 } else { h1.min(0.37) };
                let (y0, y1) = (self.floor_elev + h0 * scale, self.floor_elev + top_h1 * scale);
                self.emit_flank_box(side, u0, u1, v0, v1, a0, a1, y0, y1, d, col, frame, target);
            }
        }
    }

    /// Emits overhanging 3D green turf rim (`Grasnarben-Rand`) and soil clods along the top edge of a world-cube step wall quad.
    pub fn emit_wall_turf(&self, corners: [Vec3; 4], up: Vec3, normal: Vec3, top_col: Vec3, full_boxes: bool, target: &mut dyn MeshTarget) {
        let [v0, v1, v2, v3] = corners;
        let wall_h = (v3 - v0).dot(up).min((v2 - v1).dot(up));
        if (v2 - v3).length_squared() < 0.12 || (v2 - v3).dot(up).abs() > 0.02 || wall_h < 0.20 {
            return;
        }
        let edge = |a: f32, drop: f32, out: f32| v3 + (v2 - v3) * (a * 2.0) - up * drop + normal * out;
        let push_strip = |a0: f32, a1: f32, d_top: f32, d_bot: f32, out: f32, col: Vec3, tgt: &mut dyn MeshTarget| {
            let [f0, f1, f2, f3] = [edge(a0, d_bot, out), edge(a1, d_bot, out), edge(a1, d_top, out), edge(a0, d_top, out)];
            tgt.push_quad(Quad::new(f0, f1, f2, f3, normal, col));
            if full_boxes {
                let [w0, w1, w2, w3] = [edge(a0, d_bot, 0.0), edge(a1, d_bot, 0.0), edge(a1, d_top, 0.0), edge(a0, d_top, 0.0)];
                tgt.push_quad(Quad::new(w3, w2, f2, f3, up, col));
                tgt.push_quad(Quad::new(f0, f1, w1, w0, -up, col));
            }
        };
        let layer_drop = wall_h.min(0.5);
        push_strip(0.0, 0.5, 0.0, layer_drop, 0.002, LOAM_WALL, target);
        push_strip(0.0, 0.5, 0.001, 0.07, 0.008, self.turf_tone(top_col, SOD_C), target);
        for (a0, a1, h0, h1, d, swatch) in TURF_FRINGE_PATTERN {
            push_strip(a0, a1, 0.5 - h1, 0.5 - h0, d, self.turf_tone(top_col, swatch), target);
        }
        if full_boxes {
            for (a0, a1, h0, h1, d, col) in FLANK_PATTERN {
                let (d_top, d_bot) = (0.5 - h1.min(0.37), 0.5 - h0);
                if d_bot <= layer_drop {
                    push_strip(a0, a1, d_top, d_bot, d, col, target);
                }
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
