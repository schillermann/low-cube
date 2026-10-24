use glam::Vec3;
use crate::frame::SurfaceFrame;
use crate::{MeshTarget, Quad, TurfMeshTarget};
use super::cell::GridCell;
use super::earth_clods::EarthClods;
use super::shape::VoxelShape;

/// Material medium governing the sculpted 3D surface of a molded block in hand and world.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BlockMedium {
    TurfEarth,
    QuarriedStone,
    RawTimber,
    PlanedPlanks,
}

impl BlockMedium {
    /// Indicates whether this medium carries its own self-colored architectural geometry across all forms.
    pub const fn self_pigmented(self) -> bool {
        !matches!(self, Self::TurfEarth)
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn emit_box(
        u0: f32, u1: f32, v0: f32, v1: f32, h0: f32, h1: f32,
        frame: &SurfaceFrame, wall_col: Vec3, top_col: Vec3, bottom_cap: bool, target: &mut dyn MeshTarget,
    ) {
        let (n, t, b) = (frame.normal(), frame.tangent(), frame.bitangent());
        let [b0, b1, b2, b3] = [
            frame.point_at(u0, v0, h0), frame.point_at(u1, v0, h0),
            frame.point_at(u1, v1, h0), frame.point_at(u0, v1, h0),
        ];
        let [p0, p1, p2, p3] = [
            frame.point_at(u0, v0, h1), frame.point_at(u1, v0, h1),
            frame.point_at(u1, v1, h1), frame.point_at(u0, v1, h1),
        ];
        target.push_quad(Quad::new(p0, p1, p2, p3, n, top_col));
        if bottom_cap {
            target.push_quad(Quad::new(b0, b3, b2, b1, -n, wall_col));
        }
        target.push_quad(Quad::new(b0, b1, p1, p0, -b, wall_col));
        target.push_quad(Quad::new(p3, p2, b2, b3, b, wall_col));
        target.push_quad(Quad::new(b3, b0, p0, p3, -t, wall_col));
        target.push_quad(Quad::new(b1, b2, p2, p1, t, wall_col));
    }

    /// Emits Variant 1A Timber Block: Bark-ridged timber block with bright sapwood and concentric annual rings.
    fn emit_raw_timber_cube(cell: GridCell, floor_elev: f32, frame: &SurfaceFrame, target: &mut dyn MeshTarget) {
        let ((u0, u1), (v0, v1), top_elev) = (cell.u_bounds(), cell.v_bounds(), floor_elev + 0.5);
        let (bark_dark, bark_ridge, sapwood, ring_dark, heartwood) = (
            Vec3::new(0.32, 0.20, 0.11), Vec3::new(0.44, 0.28, 0.16),
            Vec3::new(0.85, 0.70, 0.46), Vec3::new(0.64, 0.47, 0.27), Vec3::new(0.48, 0.32, 0.17),
        );
        Self::emit_box(u0, u1, v0, v1, floor_elev, top_elev, frame, bark_dark, sapwood, true, target);
        let (r_h0, r_h1, d, w) = (floor_elev + 0.01, top_elev - 0.01, 0.014, 0.040);
        for step in 0..3 {
            let center_u = u0 + (0.20 + (step as f32) * 0.30) * (u1 - u0);
            let center_v = v0 + (0.20 + (step as f32) * 0.30) * (v1 - v0);
            Self::emit_box(center_u - w, center_u + w, v1, v1 + d, r_h0, r_h1, frame, bark_ridge, bark_ridge, true, target);
            Self::emit_box(center_u - w, center_u + w, v0 - d, v0, r_h0, r_h1, frame, bark_ridge, bark_ridge, true, target);
            Self::emit_box(u1, u1 + d, center_v - w, center_v + w, r_h0, r_h1, frame, bark_ridge, bark_ridge, true, target);
            Self::emit_box(u0 - d, u0, center_v - w, center_v + w, r_h0, r_h1, frame, bark_ridge, bark_ridge, true, target);
        }
        for (inset, h_off, col) in [(0.035, 0.000, sapwood), (0.105, 0.006, ring_dark), (0.165, 0.012, sapwood), (0.212, 0.018, heartwood)] {
            Self::emit_box(u0 + inset, u1 - inset, v0 + inset, v1 - inset, top_elev + h_off, top_elev + h_off + 0.006, frame, col, col, false, target);
        }
    }

    /// Emits Variant 2B Plank Block: Planed honey-oak floorboards separated by deep dark shadow grooves.
    fn emit_planed_planks_cube(cell: GridCell, floor_elev: f32, cross_grain: bool, frame: &SurfaceFrame, target: &mut dyn MeshTarget) {
        let ((u0, u1), (v0, v1), top_elev) = (cell.u_bounds(), cell.v_bounds(), floor_elev + 0.5);
        let groove_core = Vec3::new(0.22, 0.14, 0.08);
        let board_tones = [
            Vec3::new(0.86, 0.69, 0.43), Vec3::new(0.76, 0.58, 0.34),
            Vec3::new(0.82, 0.65, 0.39), Vec3::new(0.73, 0.55, 0.32),
        ];
        Self::emit_box(u0 + 0.016, u1 - 0.016, v0 + 0.016, v1 - 0.016, floor_elev + 0.008, top_elev - 0.002, frame, groove_core, groove_core, true, target);
        let (bh, v_pad, bw, h_pad) = (0.125, 0.009, 0.125, 0.009);
        for (i, &tone) in board_tones.iter().enumerate() {
            let (y0, y1) = (floor_elev + (i as f32) * bh + v_pad, floor_elev + ((i + 1) as f32) * bh - v_pad);
            Self::emit_box(u0, u1, v0, v1, y0, y1, frame, tone, tone, true, target);
        }
        for i in 0..4 {
            let tone = board_tones[(i + 1) % 4];
            if cross_grain {
                let (pv0, pv1) = (v0 + (i as f32) * bw + h_pad, v0 + ((i + 1) as f32) * bw - h_pad);
                Self::emit_box(u0, u1, pv0, pv1, top_elev - 0.002, top_elev + 0.010, frame, tone, tone, false, target);
            } else {
                let (pu0, pu1) = (u0 + (i as f32) * bw + h_pad, u0 + ((i + 1) as f32) * bw - h_pad);
                Self::emit_box(pu0, pu1, v0, v1, top_elev - 0.002, top_elev + 0.010, frame, tone, tone, false, target);
            }
        }
    }
}

/// Decorator binding a material medium to a VoxelShape.
pub struct MediumShape {
    inner: Box<dyn VoxelShape>,
    medium: BlockMedium,
    full_cube: bool,
    cross_grain: bool,
}

impl MediumShape {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(inner: Box<dyn VoxelShape>, medium: BlockMedium, full_cube: bool, cross_grain: bool) -> Self {
        Self { inner, medium, full_cube, cross_grain }
    }
}

impl VoxelShape for MediumShape {
    fn elevation_at(&self, local_u: f32, local_v: f32) -> f32 { self.inner.elevation_at(local_u, local_v) }
    fn shape_color(&self) -> Vec3 { self.inner.shape_color() }
    fn block_medium(&self) -> BlockMedium { self.medium }

    fn emit_shape(&self, cell: GridCell, floor_elevation: f32, frame: &SurfaceFrame, color: Vec3, target: &mut dyn MeshTarget) {
        self.emit_terraced_shape(cell, floor_elevation, frame, color, color, target);
    }

    fn emit_terraced_shape(&self, cell: GridCell, floor_elevation: f32, frame: &SurfaceFrame, top_color: Vec3, wall_color: Vec3, target: &mut dyn MeshTarget) {
        match (self.medium, self.full_cube) {
            (BlockMedium::TurfEarth, true) => EarthClods::new(cell, floor_elevation).emit_block(frame, target),
            (BlockMedium::RawTimber, true) => BlockMedium::emit_raw_timber_cube(cell, floor_elevation, frame, target),
            (BlockMedium::PlanedPlanks, true) => BlockMedium::emit_planed_planks_cube(cell, floor_elevation, self.cross_grain, frame, target),
            (BlockMedium::RawTimber, false) => {
                let mut turf = TurfMeshTarget::new(target, frame.normal(), cell.corner_points(frame, floor_elevation), Vec3::new(0.85, 0.70, 0.46), Vec3::new(0.32, 0.20, 0.11), false);
                self.inner.emit_shape(cell, floor_elevation, frame, Vec3::new(0.32, 0.20, 0.11), &mut turf);
                turf.seal_underside();
            }
            (BlockMedium::PlanedPlanks, false) => {
                let mut turf = TurfMeshTarget::new(target, frame.normal(), cell.corner_points(frame, floor_elevation), Vec3::new(0.84, 0.67, 0.41), Vec3::new(0.74, 0.56, 0.33), false);
                self.inner.emit_shape(cell, floor_elevation, frame, Vec3::new(0.74, 0.56, 0.33), &mut turf);
                turf.seal_underside();
            }
            (BlockMedium::QuarriedStone, _) => {
                let mut turf = TurfMeshTarget::new(target, frame.normal(), cell.corner_points(frame, floor_elevation), Vec3::new(0.62, 0.64, 0.66), Vec3::new(0.50, 0.51, 0.53), false);
                self.inner.emit_shape(cell, floor_elevation, frame, Vec3::new(0.50, 0.51, 0.53), &mut turf);
                turf.seal_underside();
            }
            (BlockMedium::TurfEarth, false) => {
                let mut turf = TurfMeshTarget::new(target, frame.normal(), cell.corner_points(frame, floor_elevation), top_color, wall_color, false);
                self.inner.emit_shape(cell, floor_elevation, frame, wall_color, &mut turf);
                turf.seal_underside();
            }
        }
    }
}
