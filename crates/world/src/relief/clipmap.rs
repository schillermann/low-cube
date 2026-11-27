use std::collections::HashSet;
use glam::Vec3;
use crate::block::GridCell;
use crate::frame::SurfaceFrame;
use crate::{MeshTarget, Quad};
use super::elevation::ReliefElevation;
use super::palette::ReliefPalette;
use super::strata::ReliefStrata;

const MACRO_SUBDIVISIONS: i32 = 4;

/// Concentric height-clipmap LOD ring governing planetary surface quad density (`0 Bytes` untouched `Brick8` RAM).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClipmapLod {
    SteppedCell,
    HorizonRing,
    LimbRing,
    OrbitalMacro,
}

impl ClipmapLod {
    pub const fn for_sector_distance(sector_dist: i32, continental: bool) -> Self {
        if continental && sector_dist > 6 {
            Self::OrbitalMacro
        } else if sector_dist <= 3 {
            Self::SteppedCell
        } else if sector_dist <= 7 {
            Self::HorizonRing
        } else {
            Self::LimbRing
        }
    }

    pub const fn cell_stride(self) -> i32 {
        match self {
            Self::SteppedCell => 1,
            Self::HorizonRing => 2,
            Self::LimbRing => 4,
            Self::OrbitalMacro => 8,
        }
    }
}

/// Sparse-edit planetary height-clipmap streaming untouched terrain directly from seed heights (`0 Bytes` `Brick8` RAM, `4` fields).
pub struct PlanetHeightClipmap<'a> {
    frame: &'a SurfaceFrame,
    relief: &'a dyn ReliefElevation,
    palette: &'a ReliefPalette,
    touched: &'a HashSet<(i32, i32)>,
}

impl<'a> PlanetHeightClipmap<'a> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(
        frame: &'a SurfaceFrame,
        relief: &'a dyn ReliefElevation,
        palette: &'a ReliefPalette,
        touched: &'a HashSet<(i32, i32)>,
    ) -> Self {
        Self { frame, relief, palette, touched }
    }

    fn contains_edit(&self, u0: i32, u1: i32, v0: i32, v1: i32) -> bool {
        let sanctuary_inside = u0 <= -1 && -1 < u1 && v0 <= -1 && -1 < v1 && self.relief.layer_occupied_at(-1, -1, -0.25);
        if sanctuary_inside {
            return true;
        }
        if self.touched.is_empty() {
            return false;
        }
        self.touched
            .iter()
            .any(|&(cu, cv)| cu >= u0 && cu < u1 && cv >= v0 && cv < v1)
    }

    fn sample_height_vertex(&self, cell_u: i32, cell_v: i32, half: i32) -> Vec3 {
        let cu = cell_u.clamp(-half, half - 1);
        let cv = cell_v.clamp(-half, half - 1);
        let pu = (cell_u as f32) * GridCell::SPAN;
        let pv = (cell_v as f32) * GridCell::SPAN;
        let h = (self.relief.elevation_at(cu, cv) as f32) * GridCell::SPAN;
        self.frame.point_at(pu, pv, h)
    }

    fn emit_coarse_quad(&self, (tu0, tu1, tv0, tv1): (i32, i32, i32, i32), half: i32, target: &mut dyn MeshTarget) {
        let (surf_col, ramp_col) = self.palette.inspect_facets(|s, r, _, _, _, _| (s, r));
        let p00 = self.sample_height_vertex(tu0, tv0, half);
        let p10 = self.sample_height_vertex(tu1, tv0, half);
        let p11 = self.sample_height_vertex(tu1, tv1, half);
        let p01 = self.sample_height_vertex(tu0, tv1, half);
        let center_up = self.frame.up_at((p00 + p10 + p11 + p01) * 0.25);
        let raw_n = (p01 - p00).cross(p10 - p00);
        let n = if raw_n.dot(center_up) >= 0.0 {
            raw_n.normalize_or(center_up)
        } else {
            (-raw_n).normalize_or(center_up)
        };
        let col = if n.dot(center_up) < 0.94 { ramp_col } else { surf_col };
        target.push_quad(Quad::new(p00, p10, p11, p01, n, col));
    }

    pub fn emit_sector_bounds(
        &self,
        bounds: (i32, i32, i32, i32, i32),
        strata: &ReliefStrata,
        lod: ClipmapLod,
        target: &mut dyn MeshTarget,
    ) {
        let (u0, u1, v0, v1, half) = bounds;
        match lod {
            ClipmapLod::SteppedCell => {
                for cv in v0..v1 {
                    for cu in u0..u1 {
                        strata.emit_cell(GridCell::new(cu, cv), self.frame, self.relief, self.palette, half, target);
                    }
                }
            }
            ClipmapLod::HorizonRing | ClipmapLod::LimbRing => {
                let stride = lod.cell_stride();
                let mut cv = v0;
                while cv < v1 {
                    let tv1 = (cv + stride).min(v1);
                    let mut cu = u0;
                    while cu < u1 {
                        let tu1 = (cu + stride).min(u1);
                        if self.contains_edit(cu, tu1, cv, tv1) {
                            for fv in cv..tv1 {
                                for fu in cu..tu1 {
                                    strata.emit_cell(GridCell::new(fu, fv), self.frame, self.relief, self.palette, half, target);
                                }
                            }
                        } else {
                            self.emit_coarse_quad((cu, tu1, cv, tv1), half, target);
                        }
                        cu = tu1;
                    }
                    cv = tv1;
                }
            }
            ClipmapLod::OrbitalMacro => {
                let span_u = (u1 - u0).max(1);
                let span_v = (v1 - v0).max(1);
                let steps_u = MACRO_SUBDIVISIONS.min(span_u);
                let steps_v = MACRO_SUBDIVISIONS.min(span_v);
                for sv in 0..steps_v {
                    let tv0 = v0 + (sv * span_v) / steps_v;
                    let tv1 = v0 + ((sv + 1) * span_v) / steps_v;
                    for su in 0..steps_u {
                        let tu0 = u0 + (su * span_u) / steps_u;
                        let tu1 = u0 + ((su + 1) * span_u) / steps_u;
                        if self.contains_edit(tu0, tu1, tv0, tv1) {
                            for cv in tv0..tv1 {
                                for cu in tu0..tu1 {
                                    strata.emit_cell(GridCell::new(cu, cv), self.frame, self.relief, self.palette, half, target);
                                }
                            }
                        } else {
                            self.emit_coarse_quad((tu0, tu1, tv0, tv1), half, target);
                        }
                    }
                }
            }
        }
    }
}
