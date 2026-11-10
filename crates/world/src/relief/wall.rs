use glam::Vec3;
use crate::{CrateredMeshTarget, MeshTarget, Quad};
use crate::block::{EarthClods, GridCell};
use crate::frame::SurfaceFrame;
use super::{CrustVault, ReliefElevation};

/// Vertical cliff and excavated soil walls of a relief cell; dug soil walls carry the same clods as a hand-held earth block.
pub struct ReliefWall;

impl ReliefWall {
    /// Emits clod flanks for every full loam layer exposed between `bottom` and `top` on one cell side.
    fn emit_clods(&self, cell: GridCell, side: usize, bottom: f32, top: f32, top_h: f32, turf_rim: Option<Vec3>, frame: &SurfaceFrame, target: &mut dyn MeshTarget) {
        let layers = ((top - bottom) / 0.5 + 1e-3).floor().max(0.0) as i32;
        for layer in 0..layers {
            let y0 = bottom + (layer as f32) * 0.5;
            if let Some(t_col) = turf_rim.filter(|_| (y0 + 0.5 - top_h).abs() < 1e-3) {
                EarthClods::new(cell, y0).emit_turf_flank(side, t_col, frame, target);
            } else {
                EarthClods::new(cell, y0).emit_flank(side, frame, target);
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn emit_side(
        &self, cell: GridCell, side: usize, edge: ((f32, f32), (f32, f32)), b_l: f32, b_r: f32, t_l: f32, t_r: f32,
        dug: bool, norm: Vec3, col: Vec3, floor_step: i32, top_h: f32, turf_rim: Option<Vec3>, vault: &CrustVault, relief: &dyn ReliefElevation, frame: &SurfaceFrame, target: &mut dyn MeshTarget,
    ) {
        let floor_h = (floor_step as f32) * 0.5;
        let (b_l, b_r) = (b_l.max(floor_h), b_r.max(floor_h));
        if t_l <= b_l && t_r <= b_r { return; }
        let (u, v) = cell.feed_coordinates(|u, v| (u, v));
        let ((a0, b0), (a1, b1), top_step) = (edge.0, edge.1, vault.crust_top_step_at(u, v));
        let crust_top = (top_step as f32) * 0.5;
        let low_step = ((b_l.min(b_r) * 2.0).round() as i32).max(floor_step);
        let high_step = ((t_l.max(t_r) * 2.0).round() as i32).min(top_step);
        for layer in low_step..high_step {
            let l_stage = relief.layer_dig_stage_at(u, v, (layer as f32) * 0.5 + 0.25);
            let mut cratered = CrateredMeshTarget::new(target, l_stage);
            vault.geometry(frame).emit_crust_flank(edge, layer, floor_step, norm, &mut cratered);
        }
        let (ub_l, ub_r) = (b_l.max(crust_top).min(t_l), b_r.max(crust_top).min(t_r));
        let side_turf = turf_rim.filter(|_| (t_l - top_h).abs() < 1e-3 && (t_r - top_h).abs() < 1e-3);
        if t_l > ub_l + 1e-4 || t_r > ub_r + 1e-4 {
            let (s_low, s_high) = ((ub_l.min(ub_r) * 2.0).floor() as i32, (t_l.max(t_r) * 2.0).ceil() as i32);
            if (s_low..s_high).all(|layer| relief.layer_dig_stage_at(u, v, (layer as f32) * 0.5 + 0.25) == 0) {
                target.push_quad(Quad::new(frame.point_at(a0, b0, ub_l), frame.point_at(a1, b1, ub_r), frame.point_at(a1, b1, t_r), frame.point_at(a0, b0, t_l), norm, col));
                if dug { self.emit_clods(cell, side, ub_l.max(ub_r), t_l.min(t_r), top_h, side_turf, frame, target); }
            } else {
                for layer in s_low..s_high {
                    let (y0, y1) = ((layer as f32) * 0.5, ((layer + 1) as f32) * 0.5);
                    let (ly0_l, ly0_r, ly1_l, ly1_r) = (y0.clamp(ub_l, t_l), y0.clamp(ub_r, t_r), y1.clamp(ub_l, t_l), y1.clamp(ub_r, t_r));
                    if ly1_l > ly0_l + 1e-4 || ly1_r > ly0_r + 1e-4 {
                        let l_stage = relief.layer_dig_stage_at(u, v, y0 + 0.25);
                        let mut cratered = CrateredMeshTarget::new(target, l_stage);
                        cratered.push_quad(Quad::new(frame.point_at(a0, b0, ly0_l), frame.point_at(a1, b1, ly0_r), frame.point_at(a1, b1, ly1_r), frame.point_at(a0, b0, ly1_l), norm, col));
                        if dug && l_stage == 0 && ly0_l >= ub_l - 1e-4 && ly0_r >= ub_r - 1e-4 && ly1_l <= t_l + 1e-4 && ly1_r <= t_r + 1e-4 {
                            if let Some(t_col) = side_turf.filter(|_| (y1 - top_h).abs() < 1e-3) {
                                EarthClods::new(cell, y0).emit_turf_flank(side, t_col, frame, target);
                            } else {
                                EarthClods::new(cell, y0).emit_flank(side, frame, target);
                            }
                        }
                    }
                }
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn emit_walls(
        &self, u: i32, v: i32, u0: f32, u1: f32, v0: f32, v1: f32, h: i32, hn: i32, hs: i32, he: i32, hw: i32,
        corners: [f32; 4], vault: &CrustVault, relief: &dyn ReliefElevation, frame: &SurfaceFrame, cliff_col: Vec3, turf_rim: Option<Vec3>, target: &mut dyn MeshTarget,
    ) {
        let (dug_self, dirt_col, cell, floor_step, top_h) = (relief.reworked_at(u, v), Vec3::new(0.41, 0.27, 0.15), GridCell::new(u, v), relief.crust_floor_at(u, v), (h as f32) * 0.5);
        let (nhn, nhs, nhe, nhw) = (relief.natural_elevation_at(u, v + 1), relief.natural_elevation_at(u, v - 1), relief.natural_elevation_at(u + 1, v), relief.natural_elevation_at(u - 1, v));
        if h >= hn {
            let fn_flat = relief.reworked_at(u, v + 1);
            let bn_l = ((if !fn_flat && (nhw < nhn || relief.natural_elevation_at(u - 1, v + 1) < nhn) { hn - 1 } else { hn }) as f32 * 0.5).min(corners[3]);
            let bn_r = ((if !fn_flat && (nhe < nhn || relief.natural_elevation_at(u + 1, v + 1) < nhn) { hn - 1 } else { hn }) as f32 * 0.5).min(corners[2]);
            let dug = dug_self || fn_flat;
            self.emit_side(cell, 1, ((u0, v1), (u1, v1)), bn_l, bn_r, corners[3], corners[2], dug, frame.bitangent(), if dug { dirt_col } else { cliff_col }, floor_step, top_h, turf_rim, vault, relief, frame, target);
        }
        if h >= hs {
            let fs_flat = relief.reworked_at(u, v - 1);
            let bs_l = ((if !fs_flat && (nhe < nhs || relief.natural_elevation_at(u + 1, v - 1) < nhs) { hs - 1 } else { hs }) as f32 * 0.5).min(corners[1]);
            let bs_r = ((if !fs_flat && (nhw < nhs || relief.natural_elevation_at(u - 1, v - 1) < nhs) { hs - 1 } else { hs }) as f32 * 0.5).min(corners[0]);
            let dug = dug_self || fs_flat;
            self.emit_side(cell, 0, ((u1, v0), (u0, v0)), bs_l, bs_r, corners[1], corners[0], dug, -frame.bitangent(), if dug { dirt_col } else { cliff_col }, floor_step, top_h, turf_rim, vault, relief, frame, target);
        }
        if h >= he {
            let fe_flat = relief.reworked_at(u + 1, v);
            let be_l = ((if !fe_flat && (nhs < nhe || relief.natural_elevation_at(u + 1, v - 1) < nhe) { he - 1 } else { he }) as f32 * 0.5).min(corners[1]);
            let be_r = ((if !fe_flat && (nhn < nhe || relief.natural_elevation_at(u + 1, v + 1) < nhe) { he - 1 } else { he }) as f32 * 0.5).min(corners[2]);
            let dug = dug_self || fe_flat;
            self.emit_side(cell, 3, ((u1, v0), (u1, v1)), be_l, be_r, corners[1], corners[2], dug, frame.tangent(), if dug { dirt_col } else { cliff_col }, floor_step, top_h, turf_rim, vault, relief, frame, target);
        }
        if h >= hw {
            let fw_flat = relief.reworked_at(u - 1, v);
            let bw_l = ((if !fw_flat && (nhn < nhw || relief.natural_elevation_at(u - 1, v + 1) < nhw) { hw - 1 } else { hw }) as f32 * 0.5).min(corners[3]);
            let bw_r = ((if !fw_flat && (nhs < nhw || relief.natural_elevation_at(u - 1, v - 1) < nhw) { hw - 1 } else { hw }) as f32 * 0.5).min(corners[0]);
            let dug = dug_self || fw_flat;
            self.emit_side(cell, 2, ((u0, v1), (u0, v0)), bw_l, bw_r, corners[3], corners[0], dug, -frame.tangent(), if dug { dirt_col } else { cliff_col }, floor_step, top_h, turf_rim, vault, relief, frame, target);
        }
    }
}
