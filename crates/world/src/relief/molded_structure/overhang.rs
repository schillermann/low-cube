use glam::Vec3;
use crate::MeshTarget;
use crate::block::{GridCell, VoxelShape};
use crate::frame::SurfaceFrame;

/// Decorator stamping an undercut natural slope crown with `NATURAL_OVERHANG_STAMP` (`0xF0`).
pub struct NaturalOverhang {
    inner: Box<dyn VoxelShape>,
}

impl NaturalOverhang {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(inner: Box<dyn VoxelShape>) -> Self {
        Self { inner }
    }
}

impl VoxelShape for NaturalOverhang {
    fn elevation_at(&self, local_u: f32, local_v: f32) -> f32 {
        self.inner.elevation_at(local_u, local_v)
    }

    fn shape_color(&self) -> Vec3 {
        self.inner.shape_color()
    }

    fn shape_stamp(&self) -> [u8; 3] {
        [crate::NATURAL_OVERHANG_STAMP, 0, 0]
    }

    fn full_cube_envelope(&self) -> bool {
        self.inner.full_cube_envelope()
    }

    fn emit_shape(
        &self,
        cell: GridCell,
        base_elevation: f32,
        frame: &SurfaceFrame,
        color: Vec3,
        target: &mut dyn MeshTarget,
    ) {
        self.inner.emit_shape(cell, base_elevation, frame, color, target);
    }

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
        self.inner
            .emit_terraced_shape(cell, base_elevation, frame, top_color, wall_color, bare_top, target);
    }
}
