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
    fn wheel_tread_drop(&self) -> Option<f32> {
        (self.block_medium() == BlockMedium::CarWheel)
            .then(|| (self.wheel_radius() * 1.07 - 0.11).max(0.05))
    }
    fn full_cube_envelope(&self) -> bool {
        false
    }
    fn fertile_soil(&self) -> bool {
        matches!(self.block_medium(), BlockMedium::TurfEarth)
    }
    fn smothers_turf(&self) -> bool {
        self.fertile_soil() || self.full_cube_envelope()
    }
    fn verdant_turf(&self) -> bool {
        false
    }
    fn bare_turf(&self) -> bool {
        false
    }
    fn bare_surface(&self, ground_bare: bool) -> bool {
        self.fertile_soil() || ground_bare
    }
    fn smother_turf(&mut self) {}
    fn colonize_turf(&mut self, _cell: GridCell, _dt: f32, _grass_neighbors: u32) -> bool {
        false
    }
    fn step_camera_turn(&mut self, _delta: i32) -> bool {
        false
    }
    fn camera_tripod_hub(&self, _cell: GridCell, _floor_elevation: f32, _frame: &SurfaceFrame) -> Option<(Vec3, Vec3, Vec3)> {
        None
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

impl<S: VoxelShape + ?Sized> VoxelShape for Box<S> {
    fn elevation_at(&self, local_u: f32, local_v: f32) -> f32 { (**self).elevation_at(local_u, local_v) }
    fn shape_color(&self) -> Vec3 { (**self).shape_color() }
    fn block_medium(&self) -> BlockMedium { (**self).block_medium() }
    fn block_heading(&self) -> WedgeHeading { (**self).block_heading() }
    fn wheel_radius(&self) -> f32 { (**self).wheel_radius() }
    fn wheel_tread_drop(&self) -> Option<f32> {
        (**self).wheel_tread_drop()
    }
    fn full_cube_envelope(&self) -> bool { (**self).full_cube_envelope() }
    fn fertile_soil(&self) -> bool {
        (**self).fertile_soil()
    }
    fn smothers_turf(&self) -> bool {
        (**self).smothers_turf()
    }
    fn verdant_turf(&self) -> bool {
        (**self).verdant_turf()
    }
    fn bare_turf(&self) -> bool {
        (**self).bare_turf()
    }
    fn bare_surface(&self, ground_bare: bool) -> bool {
        (**self).bare_surface(ground_bare)
    }
    fn smother_turf(&mut self) {
        (**self).smother_turf();
    }
    fn colonize_turf(&mut self, cell: GridCell, dt: f32, grass_neighbors: u32) -> bool {
        (**self).colonize_turf(cell, dt, grass_neighbors)
    }
    fn step_camera_turn(&mut self, delta: i32) -> bool { (**self).step_camera_turn(delta) }
    fn camera_tripod_hub(&self, cell: GridCell, floor_elevation: f32, frame: &SurfaceFrame) -> Option<(Vec3, Vec3, Vec3)> {
        (**self).camera_tripod_hub(cell, floor_elevation, frame)
    }
    fn rotate_quarter(&mut self) { (**self).rotate_quarter(); }
    fn emit_shape(&self, cell: GridCell, base_elevation: f32, frame: &SurfaceFrame, color: Vec3, target: &mut dyn MeshTarget) {
        (**self).emit_shape(cell, base_elevation, frame, color, target);
    }
    fn emit_driven_part(&self, cell: GridCell, base_elevation: f32, spin_angle: f32, steer_angle: f32, frame: &SurfaceFrame, top_color: Vec3, wall_color: Vec3, target: &mut dyn MeshTarget) {
        (**self).emit_driven_part(cell, base_elevation, spin_angle, steer_angle, frame, top_color, wall_color, target);
    }
    fn emit_terraced_shape(&self, cell: GridCell, base_elevation: f32, frame: &SurfaceFrame, top_color: Vec3, wall_color: Vec3, bare_top: bool, target: &mut dyn MeshTarget) {
        (**self).emit_terraced_shape(cell, base_elevation, frame, top_color, wall_color, bare_top, target);
    }
}
