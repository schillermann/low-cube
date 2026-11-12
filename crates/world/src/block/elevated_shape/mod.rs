mod coat;
mod niche;
mod sod;

use glam::Vec3;
use crate::{CrateredMeshTarget, MeshTarget};
use crate::frame::SurfaceFrame;
use super::cell::GridCell;
use super::earth_clods::{LOAM_CROWN, LOAM_WALL};
use super::shape::VoxelShape;
use super::timber_grain::BlockMedium;
pub use self::coat::TurfCoat;
pub use self::niche::NicheColumn;
pub use self::sod::TurfSod;
use self::niche::{FlankMeshTarget, LoosenedPartTarget};

const BLOCK_HEIGHT: f32 = GridCell::SPAN;
const EYE_SPAN_SENTINEL: f32 = 1.0;
const UPRIGHT_STATURE_CLEARANCE: f32 = 1.80;
const HEADROOM_TOLERANCE: f32 = 0.03;
const FOOT_SUPPORT_TOLERANCE: f32 = 0.08;
const LAYER_FLOOR_MARGIN: f32 = 0.05;
const LAYER_CEILING_MARGIN: f32 = 0.48;
const RAY_INTERCEPT_MARGIN: f32 = 0.02;
pub(super) const NICHE_CONTACT_TOLERANCE: f32 = 0.05;

/// Decorator elevating a sculpted voxel shape above a planetary surface datum.
pub struct ElevatedShape {
    shape: Box<dyn VoxelShape>,
    elevation_offset: f32,
}

impl ElevatedShape {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(shape: Box<dyn VoxelShape>, elevation_offset: f32) -> Self {
        Self {
            shape,
            elevation_offset,
        }
    }

    /// Material medium of the underlying sculpted voxel shape.
    pub fn block_medium(&self) -> BlockMedium {
        self.shape.block_medium()
    }

    /// Evaluates whether the exposed surface at this block is bare dirt, falling back to `ground_bare` for non-earth.
    pub fn bare_surface(&self, ground_bare: bool) -> bool {
        self.shape.bare_surface(ground_bare)
    }

    /// Indicates whether this block carries bare earth eligible for grass colonization.
    pub fn bare_turf(&self) -> bool {
        self.shape.bare_turf()
    }

    /// Indicates whether this block carries established living grass sod.
    pub fn verdant_turf(&self) -> bool {
        self.shape.verdant_turf()
    }

    /// Indicates whether this block rests directly on `lower_ceiling` with a turf-smothering footprint.
    pub fn smothers_turf_below(&self, lower_ceiling: f32) -> bool {
        (self.elevation_offset - lower_ceiling).abs() < NICHE_CONTACT_TOLERANCE
            && self.shape.smothers_turf()
    }

    /// Strips any living grass coat when buried directly beneath an upper solid block.
    pub fn smother_turf(&mut self) {
        self.shape.smother_turf();
    }

    /// Advances biological grass colonization when touching `grass_neighbors` verdant cells.
    pub fn colonize_turf(&mut self, cell: GridCell, dt: f32, grass_neighbors: u32) -> bool {
        self.shape.colonize_turf(cell, dt, grass_neighbors)
    }

    /// Vertical ground lift required when this shape is a car wheel with lug tread extending below its cell floor.
    pub fn wheel_ground_lift(&self) -> f32 {
        self.shape.wheel_tread_drop().unwrap_or(0.0)
    }

    /// Indicates whether this elevated shape fills an entire 0.5m cube cell envelope.
    pub fn full_cube_envelope(&self) -> bool {
        self.shape.full_cube_envelope()
    }

    pub fn step_camera_turn(&mut self, delta: i32) -> bool {
        self.shape.step_camera_turn(delta)
    }

    pub fn camera_tripod_hub(&self, cell: GridCell, frame: &SurfaceFrame) -> Option<(Vec3, Vec3, Vec3)> {
        self.shape.camera_tripod_hub(cell, self.elevation_offset, frame)
    }

    /// Cardinal orientation of the underlying sculpted voxel shape.
    pub fn block_heading(&self) -> super::wedge::WedgeHeading {
        self.shape.block_heading()
    }

    /// Consumes this elevated wrapper and returns the inner sculpted voxel shape.
    pub fn into_shape(self) -> Box<dyn VoxelShape> {
        self.shape
    }

    /// Indicates whether this elevated block settles downward under gravity when unsupported below.
    pub fn settles_under_gravity(&self) -> bool {
        self.shape.block_medium().settles_under_gravity()
    }

    /// Settles this block downward by a single 0.5m layer step.
    pub fn settle_step(&mut self) {
        self.elevation_offset -= BLOCK_HEIGHT;
    }

    pub fn rotate_quarter(&mut self) {
        self.shape.rotate_quarter();
    }

    /// Shifts this block's vertical elevation by `delta` meters.
    pub fn shift_elevation(&mut self, delta: f32) {
        self.elevation_offset += delta;
    }

    /// Lower block boundary elevation above face datum.
    pub fn floor(&self) -> f32 {
        self.elevation_offset
    }

    /// Upper block boundary elevation above face datum.
    pub fn ceiling(&self) -> f32 {
        self.elevation_offset + BLOCK_HEIGHT
    }

    /// Elevates the supporting surface height when this block sits beneath the avatar or obstructs passage.
    pub fn constrain_support(&self, local_u: f32, local_v: f32, support_h: f32, foot_elev: f32, body_span: f32) -> f32 {
        let dh = self.shape.elevation_at(local_u, local_v);
        let req_clearance = if (body_span - EYE_SPAN_SENTINEL).abs() < 1e-3 {
            UPRIGHT_STATURE_CLEARANCE
        } else {
            body_span
        };
        if dh <= 0.0
            || (body_span > 1e-4
                && self.elevation_offset >= support_h + req_clearance - HEADROOM_TOLERANCE
                && foot_elev + FOOT_SUPPORT_TOLERANCE <= self.elevation_offset)
        {
            support_h
        } else {
            support_h.max(self.elevation_offset + dh)
        }
    }

    /// Indicates whether the given vertical layer elevation falls within this block's 0.5m slot.
    pub fn occupies_layer(&self, layer_elev: f32) -> bool {
        layer_elev >= self.elevation_offset - LAYER_FLOOR_MARGIN
            && layer_elev <= self.elevation_offset + LAYER_CEILING_MARGIN
    }

    /// Indicates whether a 3D sight ray point intercepts the occupied volume of this shape.
    pub fn intercepts(&self, local_u: f32, local_v: f32, ray_elev: f32) -> bool {
        let dh = self.shape.elevation_at(local_u, local_v);
        dh > 0.0
            && ray_elev >= self.elevation_offset - RAY_INTERCEPT_MARGIN
            && ray_elev <= self.elevation_offset + dh + RAY_INTERCEPT_MARGIN
    }

    /// Emits this elevated shape through CrateredMeshTarget and polymorphic terraced shape rendering.
    #[allow(clippy::too_many_arguments)]
    pub fn emit_terraced(
        &self,
        cell: GridCell,
        frame: &SurfaceFrame,
        top_color: Vec3,
        wall_color: Vec3,
        covered_above: bool,
        flank_only: bool,
        stage: u8,
        target: &mut dyn MeshTarget,
    ) {
        let bare_top = covered_above || !self.shape.verdant_turf();
        if stage > 0 && self.shape.block_medium().mechanical_part() {
            let (uc, vc) = cell.center();
            let pivot = frame.point_at(uc, vc, self.elevation_offset + BLOCK_HEIGHT * 0.5);
            let mut loosened = LoosenedPartTarget::new(target, pivot, stage);
            self.shape.emit_terraced_shape(cell, self.elevation_offset, frame, top_color, wall_color, bare_top, &mut loosened);
            return;
        }
        let mut cratered = CrateredMeshTarget::new(target, stage);
        if stage > 0 && self.shape.fertile_soil() && self.shape.full_cube_envelope() {
            let ((u0, u1), (v0, v1)) = (cell.u_bounds(), cell.v_bounds());
            let cap = if bare_top { LOAM_CROWN } else { top_color };
            BlockMedium::emit_box(u0, u1, v0, v1, self.elevation_offset, self.elevation_offset + BLOCK_HEIGHT, frame, LOAM_WALL, cap, true, &mut cratered);
        } else if flank_only && stage == 0 {
            let mut flank = FlankMeshTarget::new(&mut cratered, frame.normal());
            self.shape.emit_terraced_shape(cell, self.elevation_offset, frame, top_color, wall_color, bare_top, &mut flank);
        } else {
            self.shape.emit_terraced_shape(cell, self.elevation_offset, frame, top_color, wall_color, bare_top, &mut cratered);
        }
    }
}

impl VoxelShape for ElevatedShape {
    fn elevation_at(&self, local_u: f32, local_v: f32) -> f32 {
        let dh = self.shape.elevation_at(local_u, local_v);
        if dh > 0.0 { dh + self.elevation_offset } else { 0.0 }
    }

    fn shape_color(&self) -> Vec3 {
        self.shape.shape_color()
    }

    fn emit_shape(
        &self,
        cell: GridCell,
        base_elevation: f32,
        frame: &SurfaceFrame,
        color: Vec3,
        target: &mut dyn MeshTarget,
    ) {
        self.shape.emit_shape(cell, base_elevation + self.elevation_offset, frame, color, target);
    }
}
