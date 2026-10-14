use glam::Vec3;
use crate::{MeshTarget, Quad};
use super::cell::GridCell;
use crate::frame::SurfaceFrame;
use super::shape::VoxelShape;

/// Rectangular cuboid voxel shape with flat top and 90° vertical walls.
pub struct CuboidShape {
    height: f32,
}

impl CuboidShape {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(height: f32) -> Self {
        Self { height }
    }

    /// Secondary constructor for a 1.0m full cube block.
    pub fn full_cube() -> Self {
        Self::new(1.0)
    }

    /// Secondary constructor for a 0.5m rock shelf slab.
    pub fn half_slab() -> Self {
        Self::new(0.5)
    }
}

impl VoxelShape for CuboidShape {
    fn elevation_at(&self, _local_u: f32, _local_v: f32) -> f32 {
        self.height
    }

    fn shape_color(&self) -> Vec3 {
        if self.height >= 0.9 {
            Vec3::new(0.0, 0.9, 1.0)
        } else {
            Vec3::new(0.1, 0.95, 0.7)
        }
    }

    fn emit_shape(
        &self,
        cell: GridCell,
        base_elevation: f32,
        frame: &SurfaceFrame,
        color: Vec3,
        target: &mut dyn MeshTarget,
    ) {
        let [p0, p1, p2, p3] = cell.corner_points(frame, base_elevation + self.height);
        let [b0, b1, b2, b3] = cell.corner_points(frame, base_elevation);

        target.push_quad(Quad::new(p0, p1, p2, p3, frame.normal(), color));
        target.push_quad(Quad::new(b0, b1, p1, p0, -frame.bitangent(), color));
        target.push_quad(Quad::new(p3, p2, b2, b3, frame.bitangent(), color));
        target.push_quad(Quad::new(b3, b0, p0, p3, -frame.tangent(), color));
        target.push_quad(Quad::new(b1, b2, p2, p1, frame.tangent(), color));
    }
}
