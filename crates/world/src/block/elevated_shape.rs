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

const BLOCK_HEIGHT: f32 = GridCell::SPAN;
const SURFACE_CONTACT_TOLERANCE: f32 = 0.1;
const EYE_SPAN_SENTINEL: f32 = 1.0;
const UPRIGHT_STATURE_CLEARANCE: f32 = 1.80;
const HEADROOM_TOLERANCE: f32 = 0.03;
const FOOT_SUPPORT_TOLERANCE: f32 = 0.08;
const LAYER_FLOOR_MARGIN: f32 = 0.05;
const LAYER_CEILING_MARGIN: f32 = 0.48;
const RAY_INTERCEPT_MARGIN: f32 = 0.02;

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

    /// Indicates whether this elevated block sits away from the primary surface contact layer.
    pub fn floating_above(&self, ground_top: f32) -> bool {
        (self.elevation_offset - ground_top).abs() >= SURFACE_CONTACT_TOLERANCE
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
        let mut cratered = CrateredMeshTarget::new(target, stage);
        if flank_only && stage == 0 {
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

