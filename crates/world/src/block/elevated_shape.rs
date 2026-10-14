use glam::Vec3;
use crate::MeshTarget;
use crate::frame::SurfaceFrame;
use super::cell::GridCell;
use super::shape::VoxelShape;

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
}

impl VoxelShape for ElevatedShape {
    fn elevation_at(&self, local_u: f32, local_v: f32) -> f32 {
        self.shape.elevation_at(local_u, local_v) + self.elevation_offset
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
