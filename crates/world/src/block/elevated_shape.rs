use glam::Vec3;
use crate::{CrateredMeshTarget, MeshTarget};
use crate::frame::SurfaceFrame;
use super::cell::GridCell;
use super::shape::VoxelShape;
use super::timber_grain::BlockMedium;

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

    /// Indicates whether this elevated block sits above the primary ground contact layer.
    pub fn floating_above(&self, ground_top: f32) -> bool {
        self.elevation_offset >= ground_top + 0.1
    }

    /// Upper block boundary elevation above ground datum.
    pub fn ceiling_above(&self, ground_top: f32) -> f32 {
        (self.elevation_offset + 0.5).max(ground_top)
    }

    /// Elevates the supporting surface height when this block sits beneath the avatar or obstructs passage.
    pub fn constrain_support(&self, local_u: f32, local_v: f32, support_h: f32, foot_elev: f32, body_span: f32) -> f32 {
        let dh = self.shape.elevation_at(local_u, local_v);
        if dh <= 0.0 || (body_span > 1e-4 && self.elevation_offset >= support_h + 1.95 && self.elevation_offset >= foot_elev + 1.90) {
            support_h
        } else {
            support_h.max(self.elevation_offset + dh)
        }
    }

    /// Indicates whether the given vertical layer elevation falls within this block's 0.5m slot.
    pub fn occupies_layer(&self, layer_elev: f32) -> bool {
        layer_elev >= self.elevation_offset - 0.05 && layer_elev <= self.elevation_offset + 0.48
    }

    /// Indicates whether a 3D sight ray point intercepts the occupied volume of this shape.
    pub fn intercepts(&self, local_u: f32, local_v: f32, ray_elev: f32) -> bool {
        let dh = self.shape.elevation_at(local_u, local_v);
        dh > 0.0 && ray_elev >= self.elevation_offset - 0.02 && ray_elev <= self.elevation_offset + dh + 0.02
    }

    /// Emits this elevated shape through CrateredMeshTarget and polymorphic terraced shape rendering.
    pub fn emit_terraced(
        &self,
        cell: GridCell,
        frame: &SurfaceFrame,
        top_color: Vec3,
        wall_color: Vec3,
        stage: u8,
        target: &mut dyn MeshTarget,
    ) {
        let mut cratered = CrateredMeshTarget::new(target, stage);
        self.shape.emit_terraced_shape(cell, self.elevation_offset, frame, top_color, wall_color, &mut cratered);
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

