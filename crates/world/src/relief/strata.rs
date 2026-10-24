use glam::Vec3;
use crate::{MeshTarget, Quad};
use crate::block::{EarthClods, GridCell};
use crate::frame::SurfaceFrame;
use super::cavity::{CavitySpan, SubterraneanCavity};
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
    #[allow(clippy::too_many_arguments)]
    pub fn emit_cell(
        &self,
        cell: GridCell,
        frame: &SurfaceFrame,
        relief: &dyn ReliefElevation,
        cavity: &dyn SubterraneanCavity,
        palette: &ReliefPalette,
        half_blocks: i32,
        target: &mut dyn MeshTarget,
    ) {
        let (u, v) = cell.feed_coordinates(|u, v| (u, v));
        let top_h = (relief.elevation_at(u, v) as f32) * 0.5;
        let mut cavern_span: Option<CavitySpan> = None;
        cavity.inspect_cavity(u, v, &mut |span| cavern_span = Some(span));

        let (cave_floor_col, cave_wall_col, cave_ceil_col) =
            palette.inspect_subterranean(|f, w, c| (f, w, c));

        if let Some(span) = cavern_span {
            let (floor, ceil) = span.inspect(|f, c| (f, c));
            let effective_ceil = ceil.min(top_h - 0.5);
            if effective_ceil > floor {
                self.emit_cavern(cell, frame, floor, effective_ceil, cave_floor_col, cave_wall_col, cave_ceil_col, target);
            }
        }

        let vault = CrustVault::beneath(half_blocks);
        if !relief.breached_at(u, v) {
            let (c_steps, f_steps) = (relief.crust_floor_at(u, v), vault.floor_steps_at(u, v));
            if c_steps > f_steps {
                self.emit_upward_shaft(cell, u, v, c_steps, &vault, relief, frame, target);
            } else {
                vault.emit_ceiling(cell, relief.dig_stage_at(u, v), frame, target);
                for side in [(0, 1), (0, -1), (1, 0), (-1, 0)] {
                    let (nu, nv) = (u + side.0, v + side.1);
                    if relief.breached_at(nu, nv) || relief.crust_floor_at(nu, nv) > vault.floor_steps_at(nu, nv) {
                        vault.emit_skirt(cell, side, frame, target);
                    }
                }
            }
        }
        vault.core().emit_patch(cell, frame, target);

        self.facet.emit_facet(
            cell, frame, relief, half_blocks,
            palette, target,
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
        let tint = 1.0 - (relief.dig_stage_at(u, v).min(3) as f32) * 0.14;
        let (ceil_col, dirt_col) = (Vec3::new(0.38, 0.25, 0.16) * tint, Vec3::new(0.41, 0.27, 0.15));
        let (c0, c1, c2, c3) = (frame.point_at(u0, v0, ceil_h), frame.point_at(u1, v0, ceil_h), frame.point_at(u1, v1, ceil_h), frame.point_at(u0, v1, ceil_h));
        target.push_quad(Quad::glowing(c3, c2, c1, c0, -frame.normal(), ceil_col, 0.55));
        let sides = [
            ((0, 1), (u1, v1), (u0, v1), -frame.bitangent(), 0usize),
            ((0, -1), (u0, v0), (u1, v0), frame.bitangent(), 1usize),
            ((1, 0), (u1, v0), (u1, v1), -frame.tangent(), 2usize),
            ((-1, 0), (u0, v1), (u0, v0), frame.tangent(), 3usize),
        ];
        for ((du, dv), (a0, b0), (a1, b1), norm, flank) in sides {
            let (nu, nv) = (u + du, v + dv);
            if !relief.breached_at(nu, nv) {
                let n_steps = relief.crust_floor_at(nu, nv).max(vault.floor_steps_at(nu, nv));
                if c_steps > n_steps {
                    let low_h = (n_steps as f32) * 0.5;
                    target.push_quad(Quad::glowing(frame.point_at(a0, b0, low_h), frame.point_at(a1, b1, low_h), frame.point_at(a1, b1, ceil_h), frame.point_at(a0, b0, ceil_h), norm, dirt_col, 0.35));
                    for layer in n_steps..c_steps {
                        EarthClods::new(GridCell::new(nu, nv), (layer as f32) * 0.5).emit_flank(flank, frame, target);
                    }
                }
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn emit_cavern(
        &self,
        cell: GridCell,
        frame: &SurfaceFrame,
        floor: f32,
        ceil: f32,
        floor_col: Vec3,
        wall_col: Vec3,
        ceil_col: Vec3,
        target: &mut dyn MeshTarget,
    ) {
        let (u0, u1) = cell.u_bounds();
        let (v0, v1) = cell.v_bounds();

        let f0 = frame.point_at(u0, v0, floor);
        let f1 = frame.point_at(u1, v0, floor);
        let f2 = frame.point_at(u1, v1, floor);
        let f3 = frame.point_at(u0, v1, floor);
        target.push_quad(Quad::new(f0, f1, f2, f3, frame.normal(), floor_col));

        let c0 = frame.point_at(u0, v0, ceil);
        let c1 = frame.point_at(u1, v0, ceil);
        let c2 = frame.point_at(u1, v1, ceil);
        let c3 = frame.point_at(u0, v1, ceil);
        target.push_quad(Quad::new(c3, c2, c1, c0, -frame.normal(), ceil_col));

        target.push_quad(Quad::new(f0, f1, c1, c0, -frame.bitangent(), wall_col));
        target.push_quad(Quad::new(f3, c3, c2, f2, frame.bitangent(), wall_col));
        target.push_quad(Quad::new(f0, c0, c3, f3, -frame.tangent(), wall_col));
        target.push_quad(Quad::new(f1, f2, c2, c1, frame.tangent(), wall_col));
    }
}
