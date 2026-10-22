use glam::Vec3;
use crate::{MeshTarget, TurfMeshTarget};
use super::cell::GridCell;
use crate::frame::SurfaceFrame;

/// Interface for predefined multi-form voxel geometries.
pub trait VoxelShape: Send + Sync {
    fn elevation_at(&self, local_u: f32, local_v: f32) -> f32;
    fn shape_color(&self) -> Vec3;
    fn emit_shape(
        &self,
        cell: GridCell,
        base_elevation: f32,
        frame: &SurfaceFrame,
        color: Vec3,
        target: &mut dyn MeshTarget,
    );
    fn emit_terraced_shape(
        &self,
        cell: GridCell,
        base_elevation: f32,
        frame: &SurfaceFrame,
        top_color: Vec3,
        wall_color: Vec3,
        target: &mut dyn MeshTarget,
    ) {
        let corners = cell.corner_points(frame, base_elevation);
        let mut turf = TurfMeshTarget::new(target, frame.normal(), corners, top_color, wall_color, false);
        self.emit_shape(cell, base_elevation, frame, wall_color, &mut turf);
        turf.seal_underside();
    }
}
