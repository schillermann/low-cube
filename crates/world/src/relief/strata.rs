use glam::Vec3;
use crate::{CrateredMeshTarget, MeshTarget, Quad};
use crate::block::{EarthClods, GridCell};
use crate::frame::SurfaceFrame;
use super::elevation::ReliefElevation;
use super::facet::ReliefFacet;
use super::crust_vault::CrustVault;
use super::palette::ReliefPalette;

/// Encapsulated facet assembler generating authentic watertight voxel columns, slopes, and caverns.
pub struct ReliefStrata {
    facet: ReliefFacet,
}

impl ReliefStrata {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new() -> Self {
        Self {
            facet: ReliefFacet::new(),
        }
    }

    /// Emits all surface and subterranean facets for a single discrete grid cell ("Tell, Don't Ask").
    pub fn emit_cell(
        &self,
        cell: GridCell,
        frame: &SurfaceFrame,
        relief: &dyn ReliefElevation,
        palette: &ReliefPalette,
        half_blocks: i32,
        target: &mut dyn MeshTarget,
    ) {
        let (u, v) = cell.feed_coordinates(|u, v| (u, v));
        let vault = CrustVault::beneath(half_blocks);
        if !relief.breached_at(u, v) {
            let (c_steps, f_steps) = (relief.crust_floor_at(u, v), vault.floor_steps_at(u, v));
            if c_steps > f_steps {
                self.emit_upward_shaft(cell, u, v, c_steps, &vault, relief, frame, target);
            } else {
                let geom = vault.geometry(frame);
                geom.emit_ceiling(cell, relief.layer_dig_stage_at(u, v, (c_steps as f32) * 0.5 + 0.25), target);
                for side in [(0, 1), (0, -1), (1, 0), (-1, 0)] {
                    let (nu, nv) = (u + side.0, v + side.1);
                    let (n_floor, n_crust) = (vault.floor_steps_at(nu, nv), relief.crust_floor_at(nu, nv));
                    if relief.breached_at(nu, nv) || n_crust > n_floor {
                        let collar_h = if relief.breached_at(nu, nv) { f32::INFINITY } else { (n_crust as f32) * 0.5 };
                        geom.emit_skirt(cell, side, collar_h, target);
                    }
                }
            }
        }
        vault.core().emit_patch(cell, frame, target);
        self.facet.emit_with_vault(
            cell, frame, relief, half_blocks,
            &vault, palette, target,
        );
    }

    #[allow(clippy::too_many_arguments)]
    fn emit_upward_shaft(
        &self,
        cell: GridCell,
        u: i32,
        v: i32,
        c_steps: i32,
        vault: &CrustVault,
        relief: &dyn ReliefElevation,
        frame: &SurfaceFrame,
        target: &mut dyn MeshTarget,
    ) {
        let ((u0, u1), (v0, v1), ceil_h) = (cell.u_bounds(), cell.v_bounds(), (c_steps as f32) * 0.5);
        let c_stage = relief.layer_dig_stage_at(u, v, ceil_h + 0.25);
        let tint = 1.0 - (c_stage.min(3) as f32) * 0.14;
        let dirt_col = Vec3::new(0.41, 0.27, 0.15);
        let geom = vault.geometry(frame);
        let mut ceil_crater = CrateredMeshTarget::new(target, c_stage);
        if vault.crust_layer_at(u, v, c_steps) {
            geom.emit_shaft_ceiling(cell, ceil_h, tint, &mut ceil_crater);
        } else {
            let (c0, c1, c2, c3) = (frame.point_at(u0, v0, ceil_h), frame.point_at(u1, v0, ceil_h), frame.point_at(u1, v1, ceil_h), frame.point_at(u0, v1, ceil_h));
            ceil_crater.push_quad(Quad::glowing(c3, c2, c1, c0, -frame.normal(), dirt_col * tint, 0.45));
            if c_stage == 0 { EarthClods::new(cell, ceil_h).emit_sole(frame, target); }
        }
        let sides = [
            ((0, 1), (u1, v1), (u0, v1), -frame.bitangent(), 0usize),
            ((0, -1), (u0, v0), (u1, v0), frame.bitangent(), 1usize),
            ((1, 0), (u1, v0), (u1, v1), -frame.tangent(), 2usize),
            ((-1, 0), (u0, v1), (u0, v0), frame.tangent(), 3usize),
        ];
        for ((du, dv), (a0, b0), (a1, b1), norm, flank) in sides {
            let (nu, nv) = (u + du, v + dv);
            if !relief.breached_at(nu, nv) {
                let n_floor = vault.floor_steps_at(nu, nv);
                let n_steps = relief.crust_floor_at(nu, nv).max(n_floor);
                if c_steps < n_steps && relief.crust_floor_at(nu, nv) <= n_floor {
                    geom.emit_skirt(GridCell::new(nu, nv), (-du, -dv), ceil_h, target);
                }
                let wall_top = c_steps.min(relief.elevation_at(nu, nv));
                for layer in n_steps..wall_top {
                    let (y0, y1) = ((layer as f32) * 0.5, ((layer + 1) as f32) * 0.5);
                    let l_stage = relief.layer_dig_stage_at(nu, nv, y0 + 0.25);
                    let mut cratered = CrateredMeshTarget::new(target, l_stage);
                    if vault.crust_layer_at(nu, nv, layer) {
                        geom.emit_crust_flank(((a0, b0), (a1, b1)), layer, n_floor, norm, &mut cratered);
                    } else {
                        cratered.push_quad(Quad::glowing(frame.point_at(a0, b0, y0), frame.point_at(a1, b1, y0), frame.point_at(a1, b1, y1), frame.point_at(a0, b0, y1), norm, dirt_col, 0.35));
                        if l_stage == 0 { EarthClods::new(GridCell::new(nu, nv), y0).emit_flank(flank, frame, target); }
                    }
                }
            }
        }
    }
}
