use glam::Vec3;
use crate::MeshTarget;
use super::cell::GridCell;
use crate::frame::SurfaceFrame;

/// Interface for predefined multi-form voxel geometries.
pub trait VoxelShape: Send + Sync {
    fn elevation_at(&self, local_u: f32, local_v: f32) -> f32;
    fn emit_shape(
        &self,
        cell: GridCell,
        base_elevation: f32,
        frame: &SurfaceFrame,
        color: Vec3,
        target: &mut dyn MeshTarget,
    );
}
