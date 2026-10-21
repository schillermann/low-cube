use glam::Vec3;
use crate::{MeshTarget, Quad};
use super::cell::GridCell;
use super::corner::CornerHeading;
use crate::frame::SurfaceFrame;
use super::shape::VoxelShape;

/// 45° convex ridge corner wedge where two outer slopes meet.
pub struct RidgeCorner {
    heading: CornerHeading,
}

impl RidgeCorner {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(heading: CornerHeading) -> Self {
        Self { heading }
    }
}

impl VoxelShape for RidgeCorner {
    fn elevation_at(&self, local_u: f32, local_v: f32) -> f32 {
        self.heading.ridge_elevation(local_u, local_v) * 0.5
    }

    fn shape_color(&self) -> Vec3 {
        Vec3::new(1.0, 0.0, 0.8)
    }

    fn emit_shape(
        &self,
        cell: GridCell,
        base_elevation: f32,
        frame: &SurfaceFrame,
        color: Vec3,
        target: &mut dyn MeshTarget,
    ) {
        let [b0, b1, b2, b3] = cell.corner_points(frame, base_elevation);
        let (b_peak, b_1, b_2, b_opp, w1, w2) = match self.heading {
            CornerHeading::NorthEast => (b2, b1, b3, b0, frame.tangent(), frame.bitangent()),
            CornerHeading::NorthWest => (b3, b2, b0, b1, frame.bitangent(), -frame.tangent()),
            CornerHeading::SouthEast => (b1, b0, b2, b3, -frame.bitangent(), frame.tangent()),
            CornerHeading::SouthWest => (b0, b3, b1, b2, -frame.tangent(), -frame.bitangent()),
        };
        let p_peak = b_peak + frame.normal() * 0.5;
        let slope_norm = (p_peak - b_1).cross(b_2 - b_1).normalize();
        target.push_quad(Quad::new(b_1, p_peak, b_2, b_2, slope_norm, color));
        target.push_quad(Quad::new(b_opp, b_2, b_1, b_1, frame.normal(), color));
        target.push_quad(Quad::new(b_1, b_peak, p_peak, p_peak, w1, color));
        target.push_quad(Quad::new(p_peak, b_peak, b_2, b_2, w2, color));
        target.push_quad(Quad::new(b_1, b_2, b_peak, b_peak, -frame.normal(), color));
    }
}
