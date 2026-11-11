use glam::Vec3;
use crate::{CrateredMeshTarget, MeshTarget, Quad};
use crate::frame::SurfaceFrame;
use super::cell::GridCell;
use super::shape::VoxelShape;
use super::timber_grain::BlockMedium;

struct FlankMeshTarget<'a> {
    inner: &'a mut dyn MeshTarget,
    up: Vec3,
}

impl<'a> MeshTarget for FlankMeshTarget<'a> {
    fn push_quad(&mut self, quad: Quad) {
        let horizontal = quad.transfer(|_, _, _, _, norm, _| norm.dot(self.up).abs() > 0.9);
        if !horizontal {
            self.inner.push_quad(quad);
        }
    }
}

struct LoosenedPartTarget<'a> {
    inner: &'a mut dyn MeshTarget,
    pivot: Vec3,
    stage: u8,
}

impl<'a> MeshTarget for LoosenedPartTarget<'a> {
    fn push_quad(&mut self, quad: Quad) {
        let s = self.stage.min(3) as f32;
        let (scale, shade, glow) = (1.0 - s * 0.14, 1.0 - s * 0.16, quad.radiate(|g| g));
        let p = self.pivot;
        let contracted = quad.transfer(|v0, v1, v2, v3, norm, col| {
            Quad::glowing(
                p + (v0 - p) * scale,
                p + (v1 - p) * scale,
                p + (v2 - p) * scale,
                p + (v3 - p) * scale,
                norm,
                (col * shade).clamp(Vec3::ZERO, Vec3::ONE),
                glow,
            )
        });
        self.inner.push_quad(contracted);
    }
}

const BLOCK_HEIGHT: f32 = GridCell::SPAN;
const EYE_SPAN_SENTINEL: f32 = 1.0;
const UPRIGHT_STATURE_CLEARANCE: f32 = 1.80;
const HEADROOM_TOLERANCE: f32 = 0.03;
const FOOT_SUPPORT_TOLERANCE: f32 = 0.08;
const LAYER_FLOOR_MARGIN: f32 = 0.05;
const LAYER_CEILING_MARGIN: f32 = 0.48;
const RAY_INTERCEPT_MARGIN: f32 = 0.02;
const NICHE_CONTACT_TOLERANCE: f32 = 0.05;
const NICHE_MIN_GAP: f32 = 0.45;
const NICHE_WALL_MARGIN: f32 = 0.10;
const NICHE_HEAD_CLEARANCE: f32 = 0.15;
const SUB_METER_NICHE_SPAN: f32 = 0.75;

/// Vertical column slice of elevated blocks evaluating sub-meter niche cavities.
pub struct NicheColumn<'a> {
    blocks: &'a [ElevatedShape],
    ground_h: f32,
}

impl<'a> NicheColumn<'a> {
    pub const fn new(blocks: &'a [ElevatedShape], ground_h: f32) -> Self {
        Self { blocks, ground_h }
    }

    pub fn confine(&self, elev: f32) -> Option<f32> {
        let (mut idx, mut floor) = (0, self.ground_h);
        while idx < self.blocks.len() {
            let base = self.blocks[idx].floor();
            if base <= floor + NICHE_CONTACT_TOLERANCE {
                floor = floor.max(self.blocks[idx].ceiling());
                idx += 1;
            } else if base >= floor + NICHE_MIN_GAP {
                let (cav_ceil, mut roof_top) = (base, self.blocks[idx].ceiling());
                idx += 1;
                while idx < self.blocks.len() && self.blocks[idx].floor() <= roof_top + NICHE_CONTACT_TOLERANCE {
                    roof_top = roof_top.max(self.blocks[idx].ceiling());
                    idx += 1;
                }
                let limit = (cav_ceil + SUB_METER_NICHE_SPAN).max(roof_top - NICHE_WALL_MARGIN);
                if elev >= floor - NICHE_WALL_MARGIN && elev <= limit {
                    let min_z = floor + NICHE_HEAD_CLEARANCE;
                    return Some(elev.clamp(min_z, (cav_ceil - NICHE_HEAD_CLEARANCE).max(min_z)));
                }
                floor = roof_top;
            } else {
                floor = floor.max(self.blocks[idx].ceiling());
                idx += 1;
            }
        }
        None
    }
}

/// Decorator elevating a voxel shape's base height by an encapsulated offset.
pub struct ElevatedShape {
    shape: Box<dyn VoxelShape>,
    elevation_offset: f32,
}

impl ElevatedShape {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(shape: Box<dyn VoxelShape>, elevation_offset: f32) -> Self {
        Self {
            shape,
            elevation_offset,
        }
    }

    /// Material medium of the underlying sculpted voxel shape.
    pub fn block_medium(&self) -> BlockMedium {
        self.shape.block_medium()
    }

    /// Vertical ground lift required when this shape is a car wheel with lug tread extending below its cell floor.
    pub fn wheel_ground_lift(&self) -> f32 {
        if self.shape.block_medium() == BlockMedium::CarWheel {
            (self.shape.wheel_radius() * 1.07 - 0.11).max(0.05)
        } else {
            0.0
        }
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
        bare_top: bool,
        flank_only: bool,
        stage: u8,
        target: &mut dyn MeshTarget,
    ) {
        if stage > 0 && self.shape.block_medium().mechanical_part() {
            let (uc, vc) = cell.center();
            let mut loosened = LoosenedPartTarget {
                inner: target,
                pivot: frame.point_at(uc, vc, self.elevation_offset + BLOCK_HEIGHT * 0.5),
                stage,
            };
            self.shape.emit_terraced_shape(cell, self.elevation_offset, frame, top_color, wall_color, bare_top, &mut loosened);
            return;
        }
        let mut cratered = CrateredMeshTarget::new(target, stage);
        if stage > 0 && self.shape.block_medium() == BlockMedium::TurfEarth && self.shape.full_cube_envelope() {
            let ((u0, u1), (v0, v1)) = (cell.u_bounds(), cell.v_bounds());
            let cap = if bare_top { Vec3::new(0.45, 0.30, 0.17) } else { top_color };
            BlockMedium::emit_box(u0, u1, v0, v1, self.elevation_offset, self.elevation_offset + BLOCK_HEIGHT, frame, super::earth_clods::LOAM_WALL, cap, true, &mut cratered);
        } else if flank_only && stage == 0 {
            let mut flank = FlankMeshTarget { inner: &mut cratered, up: frame.normal() };
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
        self.shape.emit_shape(
            cell,
            base_elevation + self.elevation_offset,
            frame,
            color,
            target,
        );
    }
}

