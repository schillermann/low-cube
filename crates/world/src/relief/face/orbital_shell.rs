use std::collections::HashSet;
use glam::Vec3;
use crate::block::GridCell;
use crate::frame::SurfaceFrame;
use crate::{MeshTarget, Quad};
use super::super::elevation::ReliefElevation;
use super::super::palette::ReliefPalette;

const MACRO_SUBDIVISIONS: i32 = 4;

/// Coarse macro-relief emitter rendering distant/orbital sectors of continental cubes (`> 256` blocks) without full-cell allocation.
pub(super) struct OrbitalShell<'a> {
    frame: &'a SurfaceFrame,
    relief: &'a dyn ReliefElevation,
    palette: &'a ReliefPalette,
    touched: &'a HashSet<(i32, i32)>,
}

impl<'a> OrbitalShell<'a> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub(super) const fn new(
        frame: &'a SurfaceFrame,
        relief: &'a dyn ReliefElevation,
        palette: &'a ReliefPalette,
        touched: &'a HashSet<(i32, i32)>,
    ) -> Self {
        Self { frame, relief, palette, touched }
    }

    fn contains_touched(&self, u0: i32, u1: i32, v0: i32, v1: i32) -> bool {
        if self.touched.is_empty() {
            return false;
        }
        self.touched
            .iter()
            .any(|&(cu, cv)| cu >= u0 && cu < u1 && cv >= v0 && cv < v1)
    }

    fn sample_point(&self, cell_u: i32, cell_v: i32, half: i32) -> Vec3 {
        let cu = cell_u.clamp(-half, half - 1);
        let cv = cell_v.clamp(-half, half - 1);
        let pu = (cell_u as f32) * GridCell::SPAN;
        let pv = (cell_v as f32) * GridCell::SPAN;
        let h = (self.relief.elevation_at(cu, cv) as f32) * GridCell::SPAN;
        self.frame.point_at(pu, pv, h)
    }

    pub(super) fn emit_macro_patch(
        &self,
        bounds: (i32, i32, i32, i32, i32),
        target: &mut dyn MeshTarget,
        mut emit_fine: impl FnMut(i32, i32, &mut dyn MeshTarget),
    ) {
        let (u0, u1, v0, v1, half) = bounds;
        let span_u = (u1 - u0).max(1);
        let span_v = (v1 - v0).max(1);
        let steps_u = MACRO_SUBDIVISIONS.min(span_u);
        let steps_v = MACRO_SUBDIVISIONS.min(span_v);
        let (surf_col, ramp_col) = self.palette.inspect_facets(|s, r, _, _, _, _| (s, r));
        let face_up = self.frame.normal();
        for sv in 0..steps_v {
            let tv0 = v0 + (sv * span_v) / steps_v;
            let tv1 = v0 + ((sv + 1) * span_v) / steps_v;
            for su in 0..steps_u {
                let tu0 = u0 + (su * span_u) / steps_u;
                let tu1 = u0 + ((su + 1) * span_u) / steps_u;
                if self.contains_touched(tu0, tu1, tv0, tv1) {
                    for cv in tv0..tv1 {
                        for cu in tu0..tu1 {
                            emit_fine(cu, cv, target);
                        }
                    }
                    continue;
                }
                let p00 = self.sample_point(tu0, tv0, half);
                let p10 = self.sample_point(tu1, tv0, half);
                let p11 = self.sample_point(tu1, tv1, half);
                let p01 = self.sample_point(tu0, tv1, half);
                let raw_n = (p10 - p00).cross(p01 - p00);
                let n = if raw_n.dot(face_up) >= 0.0 {
                    raw_n.normalize_or(face_up)
                } else {
                    (-raw_n).normalize_or(face_up)
                };
                let col = if n.dot(face_up) < 0.94 { ramp_col } else { surf_col };
                target.push_quad(Quad::new(p00, p10, p11, p01, n, col));
            }
        }
    }
}
