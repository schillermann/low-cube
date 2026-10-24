use glam::Vec3;
use crate::{MeshTarget, Quad};
use crate::block::{EarthClods, GridCell};
use crate::frame::SurfaceFrame;
use super::ReliefElevation;

/// Vertical cliff and excavated soil walls of a relief cell; dug soil walls carry the same clods as a hand-held earth block.
pub struct ReliefWall;

impl ReliefWall {
    /// Emits clod flanks for every full loam layer exposed between `bottom` and `top` on one cell side.
    fn emit_clods(&self, cell: GridCell, side: usize, bottom: f32, top: f32, frame: &SurfaceFrame, target: &mut dyn MeshTarget) {
        let layers = ((top - bottom) / 0.5 + 1e-3).floor().max(0.0) as i32;
        for layer in 0..layers { EarthClods::new(cell, bottom + (layer as f32) * 0.5).emit_flank(side, frame, target); }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn emit_walls(
        &self, u: i32, v: i32, u0: f32, u1: f32, v0: f32, v1: f32, h: i32, hn: i32, hs: i32, he: i32, hw: i32,
        corners: [f32; 4], relief: &dyn ReliefElevation, frame: &SurfaceFrame, cliff_col: Vec3, target: &mut dyn MeshTarget,
    ) {
        let (dug_self, dirt_col, cell) = (relief.reworked_at(u, v), Vec3::new(0.41, 0.27, 0.15), GridCell::new(u, v));
        if h >= hn {
            let fn_flat = relief.reworked_at(u, v + 1);
            let bn_l = ((if !fn_flat && (hw < hn || relief.elevation_at(u - 1, v + 1) < hn) { hn - 1 } else { hn }) as f32 * 0.5).min(corners[3]);
            let bn_r = ((if !fn_flat && (he < hn || relief.elevation_at(u + 1, v + 1) < hn) { hn - 1 } else { hn }) as f32 * 0.5).min(corners[2]);
            if corners[3] > bn_l || corners[2] > bn_r {
                target.push_quad(Quad::new(frame.point_at(u0, v1, bn_l), frame.point_at(u1, v1, bn_r), frame.point_at(u1, v1, corners[2]), frame.point_at(u0, v1, corners[3]), frame.bitangent(), if dug_self || fn_flat { dirt_col } else { cliff_col }));
                if dug_self || fn_flat { self.emit_clods(cell, 1, bn_l.max(bn_r), corners[3].min(corners[2]), frame, target); }
            }
        }
        if h >= hs {
            let fs_flat = relief.reworked_at(u, v - 1);
            let bs_l = ((if !fs_flat && (he < hs || relief.elevation_at(u + 1, v - 1) < hs) { hs - 1 } else { hs }) as f32 * 0.5).min(corners[1]);
            let bs_r = ((if !fs_flat && (hw < hs || relief.elevation_at(u - 1, v - 1) < hs) { hs - 1 } else { hs }) as f32 * 0.5).min(corners[0]);
            if corners[1] > bs_l || corners[0] > bs_r {
                target.push_quad(Quad::new(frame.point_at(u1, v0, bs_l), frame.point_at(u0, v0, bs_r), frame.point_at(u0, v0, corners[0]), frame.point_at(u1, v0, corners[1]), -frame.bitangent(), if dug_self || fs_flat { dirt_col } else { cliff_col }));
                if dug_self || fs_flat { self.emit_clods(cell, 0, bs_l.max(bs_r), corners[1].min(corners[0]), frame, target); }
            }
        }
        if h >= he {
            let fe_flat = relief.reworked_at(u + 1, v);
            let be_l = ((if !fe_flat && (hs < he || relief.elevation_at(u + 1, v - 1) < he) { he - 1 } else { he }) as f32 * 0.5).min(corners[1]);
            let be_r = ((if !fe_flat && (hn < he || relief.elevation_at(u + 1, v + 1) < he) { he - 1 } else { he }) as f32 * 0.5).min(corners[2]);
            if corners[1] > be_l || corners[2] > be_r {
                target.push_quad(Quad::new(frame.point_at(u1, v0, be_l), frame.point_at(u1, v1, be_r), frame.point_at(u1, v1, corners[2]), frame.point_at(u1, v0, corners[1]), frame.tangent(), if dug_self || fe_flat { dirt_col } else { cliff_col }));
                if dug_self || fe_flat { self.emit_clods(cell, 3, be_l.max(be_r), corners[1].min(corners[2]), frame, target); }
            }
        }
        if h >= hw {
            let fw_flat = relief.reworked_at(u - 1, v);
            let bw_l = ((if !fw_flat && (hn < hw || relief.elevation_at(u - 1, v + 1) < hw) { hw - 1 } else { hw }) as f32 * 0.5).min(corners[3]);
            let bw_r = ((if !fw_flat && (hs < hw || relief.elevation_at(u - 1, v - 1) < hw) { hw - 1 } else { hw }) as f32 * 0.5).min(corners[0]);
            if corners[3] > bw_l || corners[0] > bw_r {
                target.push_quad(Quad::new(frame.point_at(u0, v1, bw_l), frame.point_at(u0, v0, bw_r), frame.point_at(u0, v0, corners[0]), frame.point_at(u0, v1, corners[3]), -frame.tangent(), if dug_self || fw_flat { dirt_col } else { cliff_col }));
                if dug_self || fw_flat { self.emit_clods(cell, 2, bw_l.max(bw_r), corners[3].min(corners[0]), frame, target); }
            }
        }
    }
}
