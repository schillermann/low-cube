use glam::Vec3;
use crate::{MeshTarget, Quad};
use super::cell::GridCell;
use crate::frame::SurfaceFrame;
use super::shape::VoxelShape;

/// Pyramidal apex peak form sloping upward from all four perimeter edges toward a central tip.
pub struct PyramidPeak {
    height: f32,
}

impl PyramidPeak {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(height: f32) -> Self {
        Self { height }
    }

    /// Secondary constructor with 45° slope inclination matching ramp wedges (height = 0.5m).
    pub fn slope_aligned() -> Self {
        Self::new(0.5)
    }

    /// Secondary constructor for a 1.0m steep apex peak.
    pub fn steep() -> Self {
        Self::new(1.0)
    }
}

impl VoxelShape for PyramidPeak {
    fn elevation_at(&self, local_u: f32, local_v: f32) -> f32 {
        let dist_u = local_u.min(1.0 - local_u);
        let dist_v = local_v.min(1.0 - local_v);
        let edge_dist = dist_u.min(dist_v);
        (edge_dist * 2.0).clamp(0.0, 1.0) * self.height
    }

    fn shape_color(&self) -> Vec3 {
        Vec3::new(0.75, 0.2, 1.0)
    }

    fn emit_shape(
        &self,
        cell: GridCell,
        base_elevation: f32,
        frame: &SurfaceFrame,
        color: Vec3,
        target: &mut dyn MeshTarget,
    ) {
        let (u0, u1) = cell.u_bounds();
        let (v0, v1) = cell.v_bounds();
        let u_mid = (u0 + u1) * 0.5;
        let v_mid = (v0 + v1) * 0.5;

        let [p0, p1, p2, p3] = cell.corner_points(frame, base_elevation);
        let apex = frame.point_at(u_mid, v_mid, base_elevation + self.height);

        // South triangular face
        let norm_s = (p1 - p0).cross(apex - p0).normalize();
        target.push_quad(Quad::new(p0, p1, apex, apex, norm_s, color));

        // East triangular face
        let norm_e = (p2 - p1).cross(apex - p1).normalize();
        target.push_quad(Quad::new(p1, p2, apex, apex, norm_e, color));

        // North triangular face
        let norm_n = (p3 - p2).cross(apex - p2).normalize();
        target.push_quad(Quad::new(p2, p3, apex, apex, norm_n, color));

        // West triangular face
        let norm_w = (p0 - p3).cross(apex - p3).normalize();
        target.push_quad(Quad::new(p3, p0, apex, apex, norm_w, color));
    }
}
