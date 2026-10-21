use glam::Vec3;
use crate::{MeshTarget, Quad};
use crate::block::GridCell;
use crate::frame::SurfaceFrame;
use super::cavity::{CavitySpan, SubterraneanCavity};
use super::elevation::ReliefElevation;
use super::facet::ReliefFacet;
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

        self.facet.emit_facet(
            cell, frame, relief, half_blocks,
            palette, target,
        );
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
