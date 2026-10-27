use glam::Vec3;
use crate::{MeshTarget, Quad};
use crate::block::GridCell;
use crate::frame::SurfaceFrame;

use super::{ReliefCorner, ReliefElevation, ReliefPalette};
use super::wall::ReliefWall;
use crate::block::EarthClods;

struct SuppressedTop;

impl MeshTarget for SuppressedTop {
    fn push_quad(&mut self, _quad: Quad) {}
}

/// Encapsulated geometric facet evaluating 1-block climbable ramps, corners, seam chamfers, and side walls.
pub struct ReliefFacet {
    corner: ReliefCorner,
    wall: ReliefWall,
}

impl ReliefFacet {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self { Self { corner: ReliefCorner::new(), wall: ReliefWall } }

    pub fn emit_facet(
        &self, cell: GridCell, frame: &SurfaceFrame, relief: &dyn ReliefElevation,
        half_blocks: i32, palette: &ReliefPalette, target: &mut dyn MeshTarget,
    ) {
        let (surf_col, ramp_col, ridge_col, valley_col, chamfer_col, cliff_col) = palette.inspect_facets(|s, r, rd, v, ch, cl| (s, r, rd, v, ch, cl));
        let (u, v) = cell.feed_coordinates(|u, v| (u, v));
        let (crust, h) = (if relief.breached_at(u, v) { i32::MIN } else { relief.crust_floor_at(u, v) }, relief.elevation_at(u, v));
        let (hn, hs, he, hw) = (relief.elevation_at(u, v + 1).max(crust), relief.elevation_at(u, v - 1).max(crust), relief.elevation_at(u + 1, v).max(crust), relief.elevation_at(u - 1, v).max(crust));
        let ((u0, u1), (v0, v1), top_h) = (cell.u_bounds(), cell.v_bounds(), (h as f32) * 0.5);
        relief.emit_molded_top(cell, top_h, frame, surf_col, Vec3::new(0.48, 0.35, 0.22), target);
        let stage = relief.dig_stage_at(u, v);
        let (tn, ts, te, tw) = (v + 1 >= half_blocks, v <= -half_blocks, u + 1 >= half_blocks, u <= -half_blocks);
        let mut suppressed = SuppressedTop;
        let top_out: &mut dyn MeshTarget = if stage == 0 { target } else { &mut suppressed };

        let (corners, top_col) = if h == 0 && (tn || ts || te || tw) {
            let cu0 = if tw { u0 + 0.25 } else { u0 };
            let cu1 = if te { u1 - 0.25 } else { u1 };
            let cv0 = if ts { v0 + 0.25 } else { v0 };
            let cv1 = if tn { v1 - 0.25 } else { v1 };
            let (h0, h1, h2, h3) = (if ts || tw { -0.25 } else { 0.0 }, if ts || te { -0.25 } else { 0.0 }, if tn || te { -0.25 } else { 0.0 }, if tn || tw { -0.25 } else { 0.0 });
            let (p0, p1, p2, p3) = (frame.point_at(cu0, cv0, h0), frame.point_at(cu1, cv0, h1), frame.point_at(cu1, cv1, h2), frame.point_at(cu0, cv1, h3));
            self.corner.push_corner_pair(p0, p1, p2, p3, chamfer_col, top_out);
            ([h0, h1, h2, h3], chamfer_col)
        } else if relief.reworked_at(u, v) {
            let (bare_col, sub_dirt) = (Vec3::new(0.45, 0.30, 0.17), relief.bare_dirt_at(u, v) || relief.layer_occupied_at(u, v, top_h + 0.75));
            let top_c = if sub_dirt { bare_col } else { surf_col };
            let (p0, p1, p2, p3) = (frame.point_at(u0, v0, top_h), frame.point_at(u1, v0, top_h), frame.point_at(u1, v1, top_h), frame.point_at(u0, v1, top_h));
            if !relief.breached_at(u, v) { top_out.push_quad(Quad::new(p0, p1, p2, p3, frame.normal(), top_c)); }
            if sub_dirt && !relief.breached_at(u, v) { EarthClods::new(cell, top_h - 0.5).emit_crown(frame, top_out); }
            ([top_h, top_h, top_h, top_h], top_c)
        } else {
            let (nhn, nhs, nhe, nhw) = (relief.natural_elevation_at(u, v + 1), relief.natural_elevation_at(u, v - 1), relief.natural_elevation_at(u + 1, v), relief.natural_elevation_at(u - 1, v));
            let (dn, ds, de, dw) = (h - nhn, h - nhs, h - nhe, h - nhw);
            if dn == 1 && ds <= 0 && de <= 0 && dw <= 0 {
                self.emit_ramp(u0, u1, v0, v1, top_h, 0, frame, ramp_col, top_out);
                ([top_h, top_h, top_h - 0.5, top_h - 0.5], ramp_col)
            } else if ds == 1 && dn <= 0 && de <= 0 && dw <= 0 {
                self.emit_ramp(u0, u1, v0, v1, top_h, 1, frame, ramp_col, top_out);
                ([top_h - 0.5, top_h - 0.5, top_h, top_h], ramp_col)
            } else if de == 1 && dn <= 0 && ds <= 0 && dw <= 0 {
                self.emit_ramp(u0, u1, v0, v1, top_h, 2, frame, ramp_col, top_out);
                ([top_h, top_h - 0.5, top_h - 0.5, top_h], ramp_col)
            } else if dw == 1 && dn <= 0 && ds <= 0 && de <= 0 {
                self.emit_ramp(u0, u1, v0, v1, top_h, 3, frame, ramp_col, top_out);
                ([top_h - 0.5, top_h, top_h, top_h - 0.5], ramp_col)
            } else if dn == 1 && de == 1 && ds <= 0 && dw <= 0 {
                self.corner.emit_ridge(u0, u1, v0, v1, top_h, 0, frame, ridge_col, top_out);
                ([top_h, top_h - 0.5, top_h - 0.5, top_h - 0.5], ridge_col)
            } else if dn == 1 && dw == 1 && ds <= 0 && de <= 0 {
                self.corner.emit_ridge(u0, u1, v0, v1, top_h, 1, frame, ridge_col, top_out);
                ([top_h - 0.5, top_h, top_h - 0.5, top_h - 0.5], ridge_col)
            } else if ds == 1 && de == 1 && dn <= 0 && dw <= 0 {
                self.corner.emit_ridge(u0, u1, v0, v1, top_h, 2, frame, ridge_col, top_out);
                ([top_h - 0.5, top_h - 0.5, top_h - 0.5, top_h], ridge_col)
            } else if ds == 1 && dw == 1 && dn <= 0 && de <= 0 {
                self.corner.emit_ridge(u0, u1, v0, v1, top_h, 3, frame, ridge_col, top_out);
                ([top_h - 0.5, top_h - 0.5, top_h, top_h - 0.5], ridge_col)
            } else if dn <= 0 && ds <= 0 && de <= 0 && dw <= 0 {
                let (hse, hsw, hne, hnw) = (relief.natural_elevation_at(u + 1, v - 1), relief.natural_elevation_at(u - 1, v - 1), relief.natural_elevation_at(u + 1, v + 1), relief.natural_elevation_at(u - 1, v + 1));
                if hse < h && nhs == h && nhe == h {
                    self.corner.emit_valley(u0, u1, v0, v1, top_h, 0, frame, valley_col, top_out);
                    ([top_h, top_h - 0.5, top_h, top_h], valley_col)
                } else if hsw < h && nhs == h && nhw == h {
                    self.corner.emit_valley(u0, u1, v0, v1, top_h, 1, frame, valley_col, top_out);
                    ([top_h - 0.5, top_h, top_h, top_h], valley_col)
                } else if hne < h && nhn == h && nhe == h {
                    self.corner.emit_valley(u0, u1, v0, v1, top_h, 2, frame, valley_col, top_out);
                    ([top_h, top_h, top_h - 0.5, top_h], valley_col)
                } else if hnw < h && nhn == h && nhw == h {
                    self.corner.emit_valley(u0, u1, v0, v1, top_h, 3, frame, valley_col, top_out);
                    ([top_h, top_h, top_h, top_h - 0.5], valley_col)
                } else {
                    let (p0, p1, p2, p3) = (frame.point_at(u0, v0, top_h), frame.point_at(u1, v0, top_h), frame.point_at(u1, v1, top_h), frame.point_at(u0, v1, top_h));
                    top_out.push_quad(Quad::new(p0, p1, p2, p3, frame.normal(), surf_col));
                    ([top_h, top_h, top_h, top_h], surf_col)
                }
            } else {
                let (p0, p1, p2, p3) = (frame.point_at(u0, v0, top_h), frame.point_at(u1, v0, top_h), frame.point_at(u1, v1, top_h), frame.point_at(u0, v1, top_h));
                top_out.push_quad(Quad::new(p0, p1, p2, p3, frame.normal(), surf_col));
                ([top_h, top_h, top_h, top_h], surf_col)
            }
        };

        if stage > 0 { self.emit_dig_crater(u0, u1, v0, v1, corners, stage, top_col, surf_col, palette, frame, target); }
        self.wall.emit_walls(u, v, u0, u1, v0, v1, h, hn, hs, he, hw, corners, relief, frame, cliff_col, target);
    }

    #[allow(clippy::too_many_arguments)]
    fn emit_dig_crater(
        &self, u0: f32, u1: f32, v0: f32, v1: f32, c: [f32; 4], stage: u8,
        top_col: Vec3, surf_col: Vec3, palette: &ReliefPalette, frame: &SurfaceFrame, target: &mut dyn MeshTarget,
    ) {
        let pt = |fu: f32, fv: f32, drop: f32| {
            let h = c[0] * (1.0 - fu) * (1.0 - fv) + c[1] * fu * (1.0 - fv) + c[2] * fu * fv + c[3] * (1.0 - fu) * fv - drop;
            frame.point_at(u0 + (u1 - u0) * fu, v0 + (v1 - v0) * fv, h)
        };
        let (r0, depth) = (0.40 - (stage.min(3) as f32) * 0.10, (stage.min(3) as f32) * 0.125);
        let (r1, f0, f1) = (1.0 - r0, r0 + 0.08, 0.92 - r0);
        let o = [pt(0.0, 0.0, 0.0), pt(1.0, 0.0, 0.0), pt(1.0, 1.0, 0.0), pt(0.0, 1.0, 0.0)];
        let m = [pt(r0, r0, 0.0), pt(r1, r0, 0.0), pt(r1, r1, 0.0), pt(r0, r1, 0.0)];
        let b = [pt(f0, f0, depth), pt(f1, f0, depth), pt(f1, f1, depth), pt(f0, f1, depth)];
        let raw_n = (o[3] - o[0]).cross(o[1] - o[0]);
        let surf_n = if raw_n.dot(frame.normal()) >= 0.0 { raw_n.normalize_or_zero() } else { -raw_n.normalize_or_zero() };
        let (floor_col, wall_col) = if surf_col.y > surf_col.x && surf_col.y > surf_col.z {
            (Vec3::new(0.28, 0.19, 0.11), Vec3::new(0.42, 0.29, 0.17))
        } else { palette.inspect_subterranean(|f, w, _| (f, w)) };
        let (up, tan, bitan) = (frame.normal() * 0.6, frame.tangent(), frame.bitangent());
        let wall_n = [(up + bitan).normalize(), (up - tan).normalize(), (up - bitan).normalize(), (up + tan).normalize()];
        for i in 0..4 {
            let j = (i + 1) & 3;
            target.push_quad(Quad::new(o[i], o[j], m[j], m[i], surf_n, top_col));
            target.push_quad(Quad::new(b[i], b[j], m[j], m[i], wall_n[i], wall_col));
        }
        target.push_quad(Quad::new(b[0], b[1], b[2], b[3], surf_n, floor_col));
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
