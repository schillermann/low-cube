use glam::Vec3;
use crate::{MeshTarget, Quad};
use crate::block::{EarthClods, GridCell};
use crate::frame::SurfaceFrame;
use super::{CrustVault, ReliefElevation};

/// Vertical cliff and excavated soil walls of a relief cell; dug soil walls carry the same clods as a hand-held earth block.
pub struct ReliefWall;

impl ReliefWall {
    /// Emits clod flanks for every full loam layer exposed between `bottom` and `top` on one cell side.
    fn emit_clods(&self, cell: GridCell, side: usize, bottom: f32, top: f32, frame: &SurfaceFrame, target: &mut dyn MeshTarget) {
        let layers = ((top - bottom) / 0.5 + 1e-3).floor().max(0.0) as i32;
        for layer in 0..layers { EarthClods::new(cell, bottom + (layer as f32) * 0.5).emit_flank(side, frame, target); }
    }

    #[allow(clippy::too_many_arguments)]
    fn emit_side(
        &self, cell: GridCell, side: usize, edge: ((f32, f32), (f32, f32)), b_l: f32, b_r: f32, t_l: f32, t_r: f32,
        dug: bool, norm: Vec3, col: Vec3, floor_step: i32, vault: &CrustVault, frame: &SurfaceFrame, target: &mut dyn MeshTarget,
    ) {
        if t_l <= b_l && t_r <= b_r { return; }
        let ((a0, b0), (a1, b1), crust_top) = (edge.0, edge.1, ((floor_step + 4) as f32) * 0.5);
        let low_step = ((b_l.min(b_r) * 2.0).round() as i32).max(floor_step);
        let high_step = ((t_l.max(t_r) * 2.0).round() as i32).min(floor_step + 4);
        for layer in low_step..high_step {
            vault.emit_crust_flank(edge, layer, floor_step, norm, frame, target);
        }
        let (ub_l, ub_r) = (b_l.max(crust_top).min(t_l), b_r.max(crust_top).min(t_r));
        if t_l > ub_l + 1e-4 || t_r > ub_r + 1e-4 {
            target.push_quad(Quad::new(frame.point_at(a0, b0, ub_l), frame.point_at(a1, b1, ub_r), frame.point_at(a1, b1, t_r), frame.point_at(a0, b0, t_l), norm, col));
            if dug { self.emit_clods(cell, side, ub_l.max(ub_r), t_l.min(t_r), frame, target); }
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn emit_walls(
        &self, u: i32, v: i32, u0: f32, u1: f32, v0: f32, v1: f32, h: i32, hn: i32, hs: i32, he: i32, hw: i32,
        corners: [f32; 4], vault: &CrustVault, relief: &dyn ReliefElevation, frame: &SurfaceFrame, cliff_col: Vec3, target: &mut dyn MeshTarget,
    ) {
        let (dug_self, dirt_col, cell, floor_step) = (relief.reworked_at(u, v), Vec3::new(0.41, 0.27, 0.15), GridCell::new(u, v), vault.floor_steps_at(u, v));
        if h >= hn {
            let fn_flat = relief.reworked_at(u, v + 1);
            let bn_l = ((if !fn_flat && (hw < hn || relief.elevation_at(u - 1, v + 1) < hn) { hn - 1 } else { hn }) as f32 * 0.5).min(corners[3]);
            let bn_r = ((if !fn_flat && (he < hn || relief.elevation_at(u + 1, v + 1) < hn) { hn - 1 } else { hn }) as f32 * 0.5).min(corners[2]);
            let dug = dug_self || fn_flat;
            self.emit_side(cell, 1, ((u0, v1), (u1, v1)), bn_l, bn_r, corners[3], corners[2], dug, frame.bitangent(), if dug { dirt_col } else { cliff_col }, floor_step, vault, frame, target);
        }
        if h >= hs {
            let fs_flat = relief.reworked_at(u, v - 1);
            let bs_l = ((if !fs_flat && (he < hs || relief.elevation_at(u + 1, v - 1) < hs) { hs - 1 } else { hs }) as f32 * 0.5).min(corners[1]);
            let bs_r = ((if !fs_flat && (hw < hs || relief.elevation_at(u - 1, v - 1) < hs) { hs - 1 } else { hs }) as f32 * 0.5).min(corners[0]);
            let dug = dug_self || fs_flat;
            self.emit_side(cell, 0, ((u1, v0), (u0, v0)), bs_l, bs_r, corners[1], corners[0], dug, -frame.bitangent(), if dug { dirt_col } else { cliff_col }, floor_step, vault, frame, target);
        }
        if h >= he {
            let fe_flat = relief.reworked_at(u + 1, v);
            let be_l = ((if !fe_flat && (hs < he || relief.elevation_at(u + 1, v - 1) < he) { he - 1 } else { he }) as f32 * 0.5).min(corners[1]);
            let be_r = ((if !fe_flat && (hn < he || relief.elevation_at(u + 1, v + 1) < he) { he - 1 } else { he }) as f32 * 0.5).min(corners[2]);
            let dug = dug_self || fe_flat;
            self.emit_side(cell, 3, ((u1, v0), (u1, v1)), be_l, be_r, corners[1], corners[2], dug, frame.tangent(), if dug { dirt_col } else { cliff_col }, floor_step, vault, frame, target);
        }
        if h >= hw {
            let fw_flat = relief.reworked_at(u - 1, v);
            let bw_l = ((if !fw_flat && (hn < hw || relief.elevation_at(u - 1, v + 1) < hw) { hw - 1 } else { hw }) as f32 * 0.5).min(corners[3]);
            let bw_r = ((if !fw_flat && (hs < hw || relief.elevation_at(u - 1, v - 1) < hw) { hw - 1 } else { hw }) as f32 * 0.5).min(corners[0]);
            let dug = dug_self || fw_flat;
            self.emit_side(cell, 2, ((u0, v1), (u0, v0)), bw_l, bw_r, corners[3], corners[0], dug, -frame.tangent(), if dug { dirt_col } else { cliff_col }, floor_step, vault, frame, target);
        }
    }
}
