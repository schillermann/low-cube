use glam::Vec3;
use crate::{MeshTarget, Quad};
use crate::block::GridCell;
use crate::frame::SurfaceFrame;

use super::{ReliefCorner, ReliefElevation, ReliefPalette};

/// Encapsulated geometric facet evaluating 1-block climbable ramps, corners, seam chamfers, and side walls.
pub struct ReliefFacet {
    corner: ReliefCorner,
}

impl ReliefFacet {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            corner: ReliefCorner::new(),
        }
    }

    pub fn emit_facet(
        &self,
        cell: GridCell,
        frame: &SurfaceFrame,
        relief: &dyn ReliefElevation,
        half_blocks: i32,
        palette: &ReliefPalette,
        target: &mut dyn MeshTarget,
    ) {
        let (surf_col, ramp_col, ridge_col, valley_col, chamfer_col, cliff_col) =
            palette.inspect_facets(|s, r, rd, v, ch, cl| (s, r, rd, v, ch, cl));
        let (u, v) = cell.feed_coordinates(|u, v| (u, v));
        let h = relief.elevation_at(u, v);
        let hn = relief.elevation_at(u, v + 1);
        let hs = relief.elevation_at(u, v - 1);
        let he = relief.elevation_at(u + 1, v);
        let hw = relief.elevation_at(u - 1, v);
        let (u0, u1) = cell.u_bounds();
        let (v0, v1) = cell.v_bounds();
        let top_h = (h as f32) * 0.5;

        let tn = v + 1 >= half_blocks;
        let ts = v <= -half_blocks;
        let te = u + 1 >= half_blocks;
        let tw = u <= -half_blocks;

        let corners = if h == 0 && (tn || ts || te || tw) {
            let cu0 = if tw { u0 + 0.25 } else { u0 };
            let cu1 = if te { u1 - 0.25 } else { u1 };
            let cv0 = if ts { v0 + 0.25 } else { v0 };
            let cv1 = if tn { v1 - 0.25 } else { v1 };
            let h0 = if ts || tw { -0.25 } else { 0.0 };
            let h1 = if ts || te { -0.25 } else { 0.0 };
            let h2 = if tn || te { -0.25 } else { 0.0 };
            let h3 = if tn || tw { -0.25 } else { 0.0 };
            let (p0, p1) = (frame.point_at(cu0, cv0, h0), frame.point_at(cu1, cv0, h1));
            let (p2, p3) = (frame.point_at(cu1, cv1, h2), frame.point_at(cu0, cv1, h3));
            self.corner.push_corner_pair(p0, p1, p2, p3, chamfer_col, target);
            [h0, h1, h2, h3]
        } else if relief.reworked_at(u, v) {
            let col = Vec3::new(0.48, 0.35, 0.22);
            let (p0, p1) = (frame.point_at(u0, v0, top_h), frame.point_at(u1, v0, top_h));
            let (p2, p3) = (frame.point_at(u1, v1, top_h), frame.point_at(u0, v1, top_h));
            target.push_quad(Quad::new(p0, p1, p2, p3, frame.normal(), col));
            [top_h, top_h, top_h, top_h]
        } else {
            let nhn = relief.natural_elevation_at(u, v + 1);
            let nhs = relief.natural_elevation_at(u, v - 1);
            let nhe = relief.natural_elevation_at(u + 1, v);
            let nhw = relief.natural_elevation_at(u - 1, v);
            let (dn, ds, de, dw) = (h - nhn, h - nhs, h - nhe, h - nhw);
            if dn == 1 && ds <= 0 && de <= 0 && dw <= 0 {
                self.emit_ramp(u0, u1, v0, v1, top_h, 0, frame, ramp_col, target);
                [top_h, top_h, top_h - 0.5, top_h - 0.5]
            } else if ds == 1 && dn <= 0 && de <= 0 && dw <= 0 {
                self.emit_ramp(u0, u1, v0, v1, top_h, 1, frame, ramp_col, target);
                [top_h - 0.5, top_h - 0.5, top_h, top_h]
            } else if de == 1 && dn <= 0 && ds <= 0 && dw <= 0 {
                self.emit_ramp(u0, u1, v0, v1, top_h, 2, frame, ramp_col, target);
                [top_h, top_h - 0.5, top_h - 0.5, top_h]
            } else if dw == 1 && dn <= 0 && ds <= 0 && de <= 0 {
                self.emit_ramp(u0, u1, v0, v1, top_h, 3, frame, ramp_col, target);
                [top_h - 0.5, top_h, top_h, top_h - 0.5]
            } else if dn == 1 && de == 1 && ds <= 0 && dw <= 0 {
                self.corner.emit_ridge(u0, u1, v0, v1, top_h, 0, frame, ridge_col, target);
                [top_h, top_h - 0.5, top_h - 0.5, top_h - 0.5]
            } else if dn == 1 && dw == 1 && ds <= 0 && de <= 0 {
                self.corner.emit_ridge(u0, u1, v0, v1, top_h, 1, frame, ridge_col, target);
                [top_h - 0.5, top_h, top_h - 0.5, top_h - 0.5]
            } else if ds == 1 && de == 1 && dn <= 0 && dw <= 0 {
                self.corner.emit_ridge(u0, u1, v0, v1, top_h, 2, frame, ridge_col, target);
                [top_h - 0.5, top_h - 0.5, top_h - 0.5, top_h]
            } else if ds == 1 && dw == 1 && dn <= 0 && de <= 0 {
                self.corner.emit_ridge(u0, u1, v0, v1, top_h, 3, frame, ridge_col, target);
                [top_h - 0.5, top_h - 0.5, top_h, top_h - 0.5]
            } else if dn <= 0 && ds <= 0 && de <= 0 && dw <= 0 {
                let hse = relief.natural_elevation_at(u + 1, v - 1);
                let hsw = relief.natural_elevation_at(u - 1, v - 1);
                let hne = relief.natural_elevation_at(u + 1, v + 1);
                let hnw = relief.natural_elevation_at(u - 1, v + 1);
                if hse < h && nhs == h && nhe == h {
                    self.corner.emit_valley(u0, u1, v0, v1, top_h, 0, frame, valley_col, target);
                    [top_h, top_h - 0.5, top_h, top_h]
                } else if hsw < h && nhs == h && nhw == h {
                    self.corner.emit_valley(u0, u1, v0, v1, top_h, 1, frame, valley_col, target);
                    [top_h - 0.5, top_h, top_h, top_h]
                } else if hne < h && nhn == h && nhe == h {
                    self.corner.emit_valley(u0, u1, v0, v1, top_h, 2, frame, valley_col, target);
                    [top_h, top_h, top_h - 0.5, top_h]
                } else if hnw < h && nhn == h && nhw == h {
                    self.corner.emit_valley(u0, u1, v0, v1, top_h, 3, frame, valley_col, target);
                    [top_h, top_h, top_h, top_h - 0.5]
                } else {
                    let (p0, p1) = (frame.point_at(u0, v0, top_h), frame.point_at(u1, v0, top_h));
                    let (p2, p3) = (frame.point_at(u1, v1, top_h), frame.point_at(u0, v1, top_h));
                    target.push_quad(Quad::new(p0, p1, p2, p3, frame.normal(), surf_col));
                    if h == hn && h == hs && h == he && h == hw && nhn == hn && nhs == hs && nhe == he && nhw == hw {
                        return;
                    }
                    [top_h, top_h, top_h, top_h]
                }
            } else {
                let (p0, p1) = (frame.point_at(u0, v0, top_h), frame.point_at(u1, v0, top_h));
                let (p2, p3) = (frame.point_at(u1, v1, top_h), frame.point_at(u0, v1, top_h));
                target.push_quad(Quad::new(p0, p1, p2, p3, frame.normal(), surf_col));
                [top_h, top_h, top_h, top_h]
            }
        };

        self.emit_walls(u, v, u0, u1, v0, v1, h, hn, hs, he, hw, corners, relief, frame, cliff_col, target);
    }

    #[allow(clippy::too_many_arguments)]
    fn emit_walls(
        &self, u: i32, v: i32, u0: f32, u1: f32, v0: f32, v1: f32,
        h: i32, hn: i32, hs: i32, he: i32, hw: i32,
        corners: [f32; 4], relief: &dyn ReliefElevation,
        frame: &SurfaceFrame, cliff_col: Vec3, target: &mut dyn MeshTarget,
    ) {
        if h >= hn {
            let fn_flat = relief.reworked_at(u, v + 1);
            let bn_l = ((if !fn_flat && (hw < hn || relief.elevation_at(u - 1, v + 1) < hn) { hn - 1 } else { hn }) as f32 * 0.5).min(corners[3]);
            let bn_r = ((if !fn_flat && (he < hn || relief.elevation_at(u + 1, v + 1) < hn) { hn - 1 } else { hn }) as f32 * 0.5).min(corners[2]);
            if corners[3] > bn_l || corners[2] > bn_r {
                target.push_quad(Quad::new(frame.point_at(u0, v1, bn_l), frame.point_at(u1, v1, bn_r), frame.point_at(u1, v1, corners[2]), frame.point_at(u0, v1, corners[3]), frame.bitangent(), cliff_col));
            }
        }
        if h >= hs {
            let fs_flat = relief.reworked_at(u, v - 1);
            let bs_l = ((if !fs_flat && (he < hs || relief.elevation_at(u + 1, v - 1) < hs) { hs - 1 } else { hs }) as f32 * 0.5).min(corners[1]);
            let bs_r = ((if !fs_flat && (hw < hs || relief.elevation_at(u - 1, v - 1) < hs) { hs - 1 } else { hs }) as f32 * 0.5).min(corners[0]);
            if corners[1] > bs_l || corners[0] > bs_r {
                target.push_quad(Quad::new(frame.point_at(u1, v0, bs_l), frame.point_at(u0, v0, bs_r), frame.point_at(u0, v0, corners[0]), frame.point_at(u1, v0, corners[1]), -frame.bitangent(), cliff_col));
            }
        }
        if h >= he {
            let fe_flat = relief.reworked_at(u + 1, v);
            let be_l = ((if !fe_flat && (hs < he || relief.elevation_at(u + 1, v - 1) < he) { he - 1 } else { he }) as f32 * 0.5).min(corners[1]);
            let be_r = ((if !fe_flat && (hn < he || relief.elevation_at(u + 1, v + 1) < he) { he - 1 } else { he }) as f32 * 0.5).min(corners[2]);
            if corners[1] > be_l || corners[2] > be_r {
                target.push_quad(Quad::new(frame.point_at(u1, v0, be_l), frame.point_at(u1, v1, be_r), frame.point_at(u1, v1, corners[2]), frame.point_at(u1, v0, corners[1]), frame.tangent(), cliff_col));
            }
        }
        if h >= hw {
            let fw_flat = relief.reworked_at(u - 1, v);
            let bw_l = ((if !fw_flat && (hn < hw || relief.elevation_at(u - 1, v + 1) < hw) { hw - 1 } else { hw }) as f32 * 0.5).min(corners[3]);
            let bw_r = ((if !fw_flat && (hs < hw || relief.elevation_at(u - 1, v - 1) < hw) { hw - 1 } else { hw }) as f32 * 0.5).min(corners[0]);
            if corners[3] > bw_l || corners[0] > bw_r {
                target.push_quad(Quad::new(frame.point_at(u0, v1, bw_l), frame.point_at(u0, v0, bw_r), frame.point_at(u0, v0, corners[0]), frame.point_at(u0, v1, corners[3]), -frame.tangent(), cliff_col));
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn emit_ramp(
        &self, u0: f32, u1: f32, v0: f32, v1: f32, top_h: f32, mode: usize,
        frame: &SurfaceFrame, surf_col: Vec3, target: &mut dyn MeshTarget,
    ) {
        let (d0, d1, d2, d3, norm) = match mode {
            0 => (0.0, 0.0, -0.5, -0.5, (frame.normal() - frame.bitangent()).normalize()),
            1 => (-0.5, -0.5, 0.0, 0.0, (frame.normal() + frame.bitangent()).normalize()),
            2 => (0.0, -0.5, -0.5, 0.0, (frame.normal() - frame.tangent()).normalize()),
            _ => (-0.5, 0.0, 0.0, -0.5, (frame.normal() + frame.tangent()).normalize()),
        };
        let (p0, p1) = (frame.point_at(u0, v0, top_h + d0), frame.point_at(u1, v0, top_h + d1));
        let (p2, p3) = (frame.point_at(u1, v1, top_h + d2), frame.point_at(u0, v1, top_h + d3));
        target.push_quad(Quad::new(p0, p1, p2, p3, norm, surf_col));
    }
}
