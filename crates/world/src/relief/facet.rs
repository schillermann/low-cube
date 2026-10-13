use glam::Vec3;
use crate::{MeshTarget, Quad};
use crate::block::GridCell;
use crate::frame::SurfaceFrame;

use super::{ReliefCorner, ReliefElevation};

/// Encapsulated geometric facet evaluating 1-block climbable ramps, corners, seam chamfers, and side walls.
pub struct ReliefFacet {
    corner: ReliefCorner,
}

impl ReliefFacet {
    pub fn new() -> Self {
        Self {
            corner: ReliefCorner::new(),
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn emit_facet(
        &self,
        cell: GridCell,
        frame: &SurfaceFrame,
        relief: &dyn ReliefElevation,
        half_blocks: i32,
        surf_col: Vec3,
        wall_col: Vec3,
        target: &mut dyn MeshTarget,
    ) {
        let (u, v) = cell.feed_coordinates(|u, v| (u, v));
        let h = relief.elevation_at(u, v);
        let hn = relief.elevation_at(u, v + 1);
        let hs = relief.elevation_at(u, v - 1);
        let he = relief.elevation_at(u + 1, v);
        let hw = relief.elevation_at(u - 1, v);
        let (u0, u1) = cell.u_bounds();
        let (v0, v1) = cell.v_bounds();
        let top_h = h as f32;

        let tn = v1 as i32 >= half_blocks;
        let ts = v0 as i32 <= -half_blocks;
        let te = u1 as i32 >= half_blocks;
        let tw = u0 as i32 <= -half_blocks;

        if h == 0 && (tn || ts || te || tw) {
            self.emit_chamfer(u0, u1, v0, v1, tn, ts, te, tw, frame, surf_col, target);
            return;
        }

        let (dn, ds, de, dw) = (h - hn, h - hs, h - he, h - hw);

        if dn == 1 && ds <= 0 && de <= 0 && dw <= 0 { self.emit_ramp(u0, u1, v0, v1, top_h, 0, frame, surf_col, target); return; }
        if ds == 1 && dn <= 0 && de <= 0 && dw <= 0 { self.emit_ramp(u0, u1, v0, v1, top_h, 1, frame, surf_col, target); return; }
        if de == 1 && dn <= 0 && ds <= 0 && dw <= 0 { self.emit_ramp(u0, u1, v0, v1, top_h, 2, frame, surf_col, target); return; }
        if dw == 1 && dn <= 0 && ds <= 0 && de <= 0 { self.emit_ramp(u0, u1, v0, v1, top_h, 3, frame, surf_col, target); return; }

        // Convex ridge corners (outward slopes)
        if dn == 1 && de == 1 && ds <= 0 && dw <= 0 { self.corner.emit_ridge(u0, u1, v0, v1, top_h, 0, frame, surf_col, target); return; }
        if dn == 1 && dw == 1 && ds <= 0 && de <= 0 { self.corner.emit_ridge(u0, u1, v0, v1, top_h, 1, frame, surf_col, target); return; }
        if ds == 1 && de == 1 && dn <= 0 && dw <= 0 { self.corner.emit_ridge(u0, u1, v0, v1, top_h, 2, frame, surf_col, target); return; }
        if ds == 1 && dw == 1 && dn <= 0 && de <= 0 { self.corner.emit_ridge(u0, u1, v0, v1, top_h, 3, frame, surf_col, target); return; }

        // Concave valley corners (two inward slopes replacing flat blocks between sloped blocks)
        if dn <= 0 && ds <= 0 && de <= 0 && dw <= 0 {
            let hse = relief.elevation_at(u + 1, v - 1);
            let hsw = relief.elevation_at(u - 1, v - 1);
            let hne = relief.elevation_at(u + 1, v + 1);
            let hnw = relief.elevation_at(u - 1, v + 1);
            if hse < h && hs == h && he == h { self.corner.emit_valley(u0, u1, v0, v1, top_h, 0, frame, surf_col, target); return; }
            if hsw < h && hs == h && hw == h { self.corner.emit_valley(u0, u1, v0, v1, top_h, 1, frame, surf_col, target); return; }
            if hne < h && hn == h && he == h { self.corner.emit_valley(u0, u1, v0, v1, top_h, 2, frame, surf_col, target); return; }
            if hnw < h && hn == h && hw == h { self.corner.emit_valley(u0, u1, v0, v1, top_h, 3, frame, surf_col, target); return; }
        }

        self.emit_flat(u0, u1, v0, v1, top_h, h, hn, hs, he, hw, frame, surf_col, wall_col, target);
    }

    fn emit_ramp(
        &self, u0: f32, u1: f32, v0: f32, v1: f32, top_h: f32, mode: usize,
        frame: &SurfaceFrame, surf_col: Vec3, target: &mut dyn MeshTarget,
    ) {
        let (d0, d1, d2, d3, norm) = match mode {
            0 => (0.0, 0.0, -1.0, -1.0, (frame.normal() - frame.bitangent()).normalize()),
            1 => (-1.0, -1.0, 0.0, 0.0, (frame.normal() + frame.bitangent()).normalize()),
            2 => (0.0, -1.0, -1.0, 0.0, (frame.normal() - frame.tangent()).normalize()),
            _ => (-1.0, 0.0, 0.0, -1.0, (frame.normal() + frame.tangent()).normalize()),
        };
        let (p0, p1) = (frame.point_at(u0, v0, top_h + d0), frame.point_at(u1, v0, top_h + d1));
        let (p2, p3) = (frame.point_at(u1, v1, top_h + d2), frame.point_at(u0, v1, top_h + d3));
        target.push_quad(Quad::new(p0, p1, p2, p3, norm, surf_col));
    }

    #[allow(clippy::too_many_arguments)]
    fn emit_flat(
        &self, u0: f32, u1: f32, v0: f32, v1: f32, top_h: f32,
        h: i32, hn: i32, hs: i32, he: i32, hw: i32,
        frame: &SurfaceFrame, surf_col: Vec3, wall_col: Vec3, target: &mut dyn MeshTarget,
    ) {
        let (p0, p1) = (frame.point_at(u0, v0, top_h), frame.point_at(u1, v0, top_h));
        let (p2, p3) = (frame.point_at(u1, v1, top_h), frame.point_at(u0, v1, top_h));
        target.push_quad(Quad::new(p0, p1, p2, p3, frame.normal(), surf_col));

        if h > hn { target.push_quad(Quad::new(frame.point_at(u0, v1, hn as f32), frame.point_at(u1, v1, hn as f32), p2, p3, frame.bitangent(), wall_col)); }
        if h > hs { target.push_quad(Quad::new(frame.point_at(u1, v0, hs as f32), frame.point_at(u0, v0, hs as f32), p0, p1, -frame.bitangent(), wall_col)); }
        if h > he { target.push_quad(Quad::new(frame.point_at(u1, v0, he as f32), frame.point_at(u1, v1, he as f32), p2, p1, frame.tangent(), wall_col)); }
        if h > hw { target.push_quad(Quad::new(frame.point_at(u0, v1, hw as f32), frame.point_at(u0, v0, hw as f32), p0, p3, -frame.tangent(), wall_col)); }
    }

    #[allow(clippy::too_many_arguments)]
    fn emit_chamfer(
        &self, u0: f32, u1: f32, v0: f32, v1: f32,
        tn: bool, ts: bool, te: bool, tw: bool,
        frame: &SurfaceFrame, surf_col: Vec3, target: &mut dyn MeshTarget,
    ) {
        let h0 = if ts || tw { -0.5 } else { 0.0 };
        let h1 = if ts || te { -0.5 } else { 0.0 };
        let h2 = if tn || te { -0.5 } else { 0.0 };
        let h3 = if tn || tw { -0.5 } else { 0.0 };
        let (p0, p1) = (frame.point_at(u0, v0, h0), frame.point_at(u1, v0, h1));
        let (p2, p3) = (frame.point_at(u1, v1, h2), frame.point_at(u0, v1, h3));
        self.corner.push_corner_pair(p0, p1, p2, p3, surf_col, target);
    }
}
