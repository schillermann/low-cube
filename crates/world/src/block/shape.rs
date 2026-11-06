use glam::Vec3;
use crate::{MeshTarget, TurfMeshTarget};
use super::cell::GridCell;
use super::timber_grain::BlockMedium;
use super::wedge::WedgeHeading;
use crate::frame::SurfaceFrame;

/// Interface for predefined multi-form voxel geometries.
pub trait VoxelShape: Send + Sync {
    fn elevation_at(&self, local_u: f32, local_v: f32) -> f32;
    fn shape_color(&self) -> Vec3;
    fn block_medium(&self) -> BlockMedium {
        BlockMedium::TurfEarth
    }
    fn block_heading(&self) -> WedgeHeading {
        WedgeHeading::South
    }
    fn wheel_radius(&self) -> f32 {
        0.25
    }
    fn rotate_quarter(&mut self) {}
    fn emit_shape(
        &self,
        cell: GridCell,
        base_elevation: f32,
        frame: &SurfaceFrame,
        color: Vec3,
        target: &mut dyn MeshTarget,
    );
    #[allow(clippy::too_many_arguments)]
    fn emit_driven_part(
        &self,
        cell: GridCell,
        base_elevation: f32,
        _spin_angle: f32,
        _steer_angle: f32,
        frame: &SurfaceFrame,
        top_color: Vec3,
        wall_color: Vec3,
        target: &mut dyn MeshTarget,
    ) {
        self.emit_terraced_shape(cell, base_elevation, frame, top_color, wall_color, true, target);
    }
    #[allow(clippy::too_many_arguments)]
    fn emit_terraced_shape(
        &self,
        cell: GridCell,
        base_elevation: f32,
        frame: &SurfaceFrame,
        top_color: Vec3,
        wall_color: Vec3,
        bare_top: bool,
        target: &mut dyn MeshTarget,
    ) {
        let cap = if bare_top { wall_color } else { top_color };
        let corners = cell.corner_points(frame, base_elevation);
        let mut turf = TurfMeshTarget::new(target, frame.normal(), corners, cap, wall_color, false);
        self.emit_shape(cell, base_elevation, frame, wall_color, &mut turf);
        turf.seal_underside();
    }
}
