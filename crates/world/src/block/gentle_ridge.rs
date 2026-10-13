use glam::Vec3;
use crate::{MeshTarget, Quad};
use super::cell::GridCell;
use super::corner::CornerHeading;
use crate::frame::SurfaceFrame;
use super::shape::VoxelShape;

/// Gentle 2:1 convex ridge corner wedge where two gentle outer slopes meet.
pub struct GentleRidge {
    heading: CornerHeading,
    start_elevation: f32,
    end_elevation: f32,
}

impl GentleRidge {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(heading: CornerHeading, start_elevation: f32, end_elevation: f32) -> Self {
        Self {
            heading,
            start_elevation,
            end_elevation,
        }
    }

    /// Secondary constructor for lower half of a 2-block run (0.0m to 0.5m).
    pub fn lower(heading: CornerHeading) -> Self {
        Self::new(heading, 0.0, 0.5)
    }

    /// Secondary constructor for upper half of a 2-block run (0.5m to 1.0m).
    pub fn upper(heading: CornerHeading) -> Self {
        Self::new(heading, 0.5, 1.0)
    }
}

impl VoxelShape for GentleRidge {
    fn elevation_at(&self, local_u: f32, local_v: f32) -> f32 {
        let frac = self.heading.ridge_elevation(local_u, local_v);
        self.start_elevation + frac * (self.end_elevation - self.start_elevation)
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
        let (s, e) = (self.start_elevation, self.end_elevation);

        let (h0, h1, h2, h3) = match self.heading {
            CornerHeading::NorthEast => (s, s, e, s),
            CornerHeading::NorthWest => (s, s, s, e),
            CornerHeading::SouthEast => (s, e, s, s),
            CornerHeading::SouthWest => (e, s, s, s),
        };

        let p0 = frame.point_at(u0, v0, base_elevation + h0);
        let p1 = frame.point_at(u1, v0, base_elevation + h1);
        let p2 = frame.point_at(u1, v1, base_elevation + h2);
        let p3 = frame.point_at(u0, v1, base_elevation + h3);

        let norm_a = (p1 - p0).cross(p2 - p0).normalize();
        let norm_b = (p2 - p0).cross(p3 - p0).normalize();
        target.push_quad(Quad::new(p0, p1, p2, p2, norm_a, color));
        target.push_quad(Quad::new(p0, p2, p3, p3, norm_b, color));

        let [b0, b1, b2, b3] = cell.corner_points(frame, base_elevation);

        if h1 > 0.0 || h2 > 0.0 {
            target.push_quad(Quad::new(b1, b2, p2, p1, frame.tangent(), color));
        }
        if h2 > 0.0 || h3 > 0.0 {
            target.push_quad(Quad::new(p3, p2, b2, b3, frame.bitangent(), color));
        }
        if h0 > 0.0 || h3 > 0.0 {
            target.push_quad(Quad::new(b3, b0, p0, p3, -frame.tangent(), color));
        }
        if h0 > 0.0 || h1 > 0.0 {
            target.push_quad(Quad::new(b0, b1, p1, p0, -frame.bitangent(), color));
        }
    }
}
