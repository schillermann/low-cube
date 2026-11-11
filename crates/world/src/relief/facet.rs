use glam::Vec3;
use crate::{CrateredMeshTarget, MeshTarget, Quad};
use crate::block::GridCell;
use crate::frame::SurfaceFrame;

use super::corner::{NaturalCrown, ReliefCorner};
use super::{ReliefElevation, ReliefPalette};
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
        self.emit_with_vault(cell, frame, relief, half_blocks, &super::CrustVault::beneath(half_blocks), palette, target);
    }

    #[allow(clippy::too_many_arguments)]
    pub fn emit_with_vault(
        &self, cell: GridCell, frame: &SurfaceFrame, relief: &dyn ReliefElevation,
        half_blocks: i32, vault: &super::CrustVault, palette: &ReliefPalette, target: &mut dyn MeshTarget,
    ) {
        let (surf_col, ramp_col, ridge_col, valley_col, chamfer_col, cliff_col) = palette.inspect_facets(|s, r, rd, v, ch, cl| (s, r, rd, v, ch, cl));
        let (u, v) = cell.feed_coordinates(|u, v| (u, v));
        let (crust, h) = (relief.crust_floor_at(u, v), relief.elevation_at(u, v));
        let (hn, hs, he, hw) = (relief.elevation_at(u, v + 1).max(crust), relief.elevation_at(u, v - 1).max(crust), relief.elevation_at(u + 1, v).max(crust), relief.elevation_at(u - 1, v).max(crust));
        let ((u0, u1), (v0, v1), top_h) = (cell.u_bounds(), cell.v_bounds(), (h as f32) * 0.5);
        let molded = relief.emit_molded_top(cell, frame, surf_col, Vec3::new(0.48, 0.35, 0.22), target);
        let stage = relief.layer_dig_stage_at(u, v, top_h - 0.25);
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
            let top_c = if vault.crust_surface_at(u, v, h) {
                let mut cratered = CrateredMeshTarget::new(target, stage);
                let core_exposed = h <= vault.floor_steps_at(u, v);
                vault.geometry(frame).emit_crust_floor(cell, top_h, core_exposed, &mut cratered)
            } else {
                let (p0, p1, p2, p3) = (frame.point_at(u0, v0, top_h), frame.point_at(u1, v0, top_h), frame.point_at(u1, v1, top_h), frame.point_at(u0, v1, top_h));
                let sub_dirt = relief.bare_dirt_at(u, v) || molded;
                let col = if sub_dirt { Vec3::new(0.45, 0.30, 0.17) } else { surf_col };
                let glow = if !sub_dirt && col.y > col.x * 1.15 && col.y > col.z * 1.15 { 0.02 } else { 0.0 };
                top_out.push_quad(Quad::glowing(p0, p1, p2, p3, frame.normal(), col, glow));
                if sub_dirt {
                    EarthClods::new(cell, top_h - 0.5).emit_crown(frame, top_out);
                }
                col
            };
            ([top_h, top_h, top_h, top_h], top_c)
        } else {
            let colors = (surf_col, ramp_col, ridge_col, valley_col);
            NaturalCrown::at(u, v, |gu, gv| relief.natural_elevation_at(gu, gv))
                .emit_top(&self.corner, cell, top_h, frame, colors, top_out)
        };

        let crust_top = relief.reworked_at(u, v) && vault.crust_surface_at(u, v, h);
        if stage > 0 && !crust_top { self.emit_dig_crater(u0, u1, v0, v1, corners, stage, top_col, palette, frame, target); }
        let turf_rim = (!relief.bare_dirt_at(u, v) && !molded && !crust_top).then_some(surf_col);
        self.wall.emit_walls(u, v, u0, u1, v0, v1, h, hn, hs, he, hw, corners, vault, relief, frame, cliff_col, turf_rim, target);
    }

    #[allow(clippy::too_many_arguments)]
    fn emit_dig_crater(
        &self, u0: f32, u1: f32, v0: f32, v1: f32, c: [f32; 4], stage: u8,
        top_col: Vec3, palette: &ReliefPalette, frame: &SurfaceFrame, target: &mut dyn MeshTarget,
    ) {
        let pt = |fu: f32, fv: f32, drop: f32| {
            let h_a = if fu >= fv { c[1] + (1.0 - fu) * (c[0] - c[1]) + fv * (c[2] - c[1]) } else { c[3] + fu * (c[2] - c[3]) + (1.0 - fv) * (c[0] - c[3]) };
            let h_b = if fu + fv <= 1.0 { c[0] + fu * (c[1] - c[0]) + fv * (c[3] - c[0]) } else { c[2] + (1.0 - fu) * (c[3] - c[2]) + (1.0 - fv) * (c[1] - c[2]) };
            let h = if (c[0] - c[2]).abs() < (c[1] - c[3]).abs() { h_a } else { h_b } - drop;
            frame.point_at(u0 + (u1 - u0) * fu, v0 + (v1 - v0) * fv, h)
        };
        let (r0, depth) = (0.40 - (stage.min(3) as f32) * 0.10, (stage.min(3) as f32) * 0.125);
        let (r1, f0, f1) = (1.0 - r0, r0 + 0.08, 0.92 - r0);
        let o = [pt(0.0, 0.0, 0.0), pt(1.0, 0.0, 0.0), pt(1.0, 1.0, 0.0), pt(0.0, 1.0, 0.0)];
        let m = [pt(r0, r0, 0.0), pt(r1, r0, 0.0), pt(r1, r1, 0.0), pt(r0, r1, 0.0)];
        let b = [pt(f0, f0, depth), pt(f1, f0, depth), pt(f1, f1, depth), pt(f0, f1, depth)];
        let raw_n = (o[3] - o[0]).cross(o[1] - o[0]);
        let surf_n = if raw_n.dot(frame.normal()) >= 0.0 { raw_n.normalize_or_zero() } else { -raw_n.normalize_or_zero() };
        let (floor_col, wall_col) = palette.inspect_subterranean(|f, w| (f, w));
        let (up, tan, bitan) = (frame.normal() * 0.6, frame.tangent(), frame.bitangent());
        let wall_n = [(up + bitan).normalize(), (up - tan).normalize(), (up - bitan).normalize(), (up + tan).normalize()];
        for i in 0..4 {
            let j = (i + 1) & 3;
            target.push_quad(Quad::new(o[i], o[j], m[j], m[i], surf_n, top_col));
            target.push_quad(Quad::new(b[i], b[j], m[j], m[i], wall_n[i], wall_col));
        }
        target.push_quad(Quad::new(b[0], b[1], b[2], b[3], surf_n, floor_col));
    }
}
