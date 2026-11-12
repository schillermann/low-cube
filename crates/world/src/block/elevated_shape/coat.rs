use glam::Vec3;
use crate::MeshTarget;
use crate::frame::SurfaceFrame;
use super::super::cell::GridCell;
use super::super::shape::VoxelShape;
use super::super::timber_grain::BlockMedium;
use super::super::wedge::WedgeHeading;
use super::sod::TurfSod;

/// Decorator clothing a static world voxel shape in living or bare biological turf sod.
pub struct TurfCoat<S = Box<dyn VoxelShape>> {
    inner: S,
    sod: TurfSod,
}

impl<S: VoxelShape> TurfCoat<S> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(inner: S, sod: TurfSod) -> Self {
        Self { inner, sod }
    }

    /// Secondary constructor coating a freshly placed block in bare soil without grass.
    pub const fn bare(inner: S) -> Self {
        Self::new(inner, TurfSod::bare())
    }

    /// Secondary constructor coating a retained natural crown in established living grass.
    pub const fn verdant(inner: S) -> Self {
        Self::new(inner, TurfSod::verdant())
    }
}

impl<S: VoxelShape> VoxelShape for TurfCoat<S> {
    fn elevation_at(&self, local_u: f32, local_v: f32) -> f32 {
        self.inner.elevation_at(local_u, local_v)
    }

    fn shape_color(&self) -> Vec3 {
        self.inner.shape_color()
    }

    fn block_medium(&self) -> BlockMedium {
        self.inner.block_medium()
    }

    fn block_heading(&self) -> WedgeHeading {
        self.inner.block_heading()
    }

    fn wheel_radius(&self) -> f32 {
        self.inner.wheel_radius()
    }

    fn wheel_tread_drop(&self) -> Option<f32> {
        self.inner.wheel_tread_drop()
    }

    fn full_cube_envelope(&self) -> bool {
        self.inner.full_cube_envelope()
    }

    fn fertile_soil(&self) -> bool {
        self.inner.fertile_soil()
    }

    fn smothers_turf(&self) -> bool {
        self.inner.smothers_turf()
    }

    fn verdant_turf(&self) -> bool {
        self.inner.fertile_soil() && self.sod.verdant_coat()
    }

    fn bare_turf(&self) -> bool {
        self.inner.fertile_soil() && !self.sod.verdant_coat()
    }

    fn bare_surface(&self, ground_bare: bool) -> bool {
        if self.inner.fertile_soil() {
            !self.sod.verdant_coat()
        } else {
            self.inner.bare_surface(ground_bare)
        }
    }

    fn smother_turf(&mut self) {
        self.sod = TurfSod::bare();
    }

    fn colonize_turf(&mut self, cell: GridCell, dt: f32, grass_neighbors: u32) -> bool {
        if !self.inner.fertile_soil() {
            return false;
        }
        let (next_sod, grown) = self.sod.colonized(cell, dt, grass_neighbors);
        self.sod = next_sod;
        grown
    }

    fn step_camera_turn(&mut self, delta: i32) -> bool {
        self.inner.step_camera_turn(delta)
    }

    fn camera_tripod_hub(&self, cell: GridCell, floor_elevation: f32, frame: &SurfaceFrame) -> Option<(Vec3, Vec3, Vec3)> {
        self.inner.camera_tripod_hub(cell, floor_elevation, frame)
    }

    fn rotate_quarter(&mut self) {
        self.inner.rotate_quarter();
    }

    fn emit_shape(&self, cell: GridCell, floor_elevation: f32, frame: &SurfaceFrame, color: Vec3, target: &mut dyn MeshTarget) {
        self.inner.emit_shape(cell, floor_elevation, frame, color, target);
    }

    fn emit_driven_part(
        &self,
        cell: GridCell,
        floor_elevation: f32,
        spin_angle: f32,
        steer_angle: f32,
        frame: &SurfaceFrame,
        top_color: Vec3,
        wall_color: Vec3,
        target: &mut dyn MeshTarget,
    ) {
        self.inner.emit_driven_part(cell, floor_elevation, spin_angle, steer_angle, frame, top_color, wall_color, target);
    }

    fn emit_terraced_shape(
        &self,
        cell: GridCell,
        floor_elevation: f32,
        frame: &SurfaceFrame,
        top_color: Vec3,
        wall_color: Vec3,
        bare_top: bool,
        target: &mut dyn MeshTarget,
    ) {
        let bare = bare_top || !self.sod.verdant_coat();
        self.inner.emit_terraced_shape(cell, floor_elevation, frame, top_color, wall_color, bare, target);
    }
}
