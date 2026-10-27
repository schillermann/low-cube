use glam::Vec3;
use crate::{MeshTarget, Quad};
use super::cell::GridCell;
use super::corner::CornerHeading;
use crate::frame::SurfaceFrame;
use super::shape::VoxelShape;

/// Gentle 2:1 concave valley corner wedge where two gentle inner slopes meet.
pub struct GentleValley {
    heading: CornerHeading,
    start_elevation: f32,
    end_elevation: f32,
}

impl GentleValley {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(heading: CornerHeading, start_elevation: f32, end_elevation: f32) -> Self {
        Self {
            heading,
            start_elevation,
            end_elevation,
        }
    }

    /// Secondary constructor for lower half of a 2-block run (0.0m to 0.25m).
    pub fn lower(heading: CornerHeading) -> Self {
        Self::new(heading, 0.0, 0.25)
    }

    /// Secondary constructor for upper half of a 2-block run (0.25m to 0.5m).
    pub fn upper(heading: CornerHeading) -> Self {
        Self::new(heading, 0.25, 0.5)
    }
}

impl VoxelShape for GentleValley {
    fn elevation_at(&self, local_u: f32, local_v: f32) -> f32 {
        let frac = self.heading.valley_elevation(local_u, local_v);
        self.start_elevation + frac * (self.end_elevation - self.start_elevation)
    }

    fn shape_color(&self) -> Vec3 {
        if self.start_elevation < 0.125 {
            Vec3::new(1.0, 0.85, 0.2)
        } else {
            Vec3::new(1.0, 0.7, 0.0)
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
        let (u0, u1) = cell.u_bounds();
        let (v0, v1) = cell.v_bounds();
        let (s, e) = (self.start_elevation, self.end_elevation);

        let (h0, h1, h2, h3) = match self.heading {
            CornerHeading::NorthEast => (s, e, e, e),
            CornerHeading::NorthWest => (e, s, e, e),
            CornerHeading::SouthEast => (e, e, s, e),
            CornerHeading::SouthWest => (e, e, e, s),
        };

        let p0 = frame.point_at(u0, v0, base_elevation + h0);
        let p1 = frame.point_at(u1, v0, base_elevation + h1);
        let p2 = frame.point_at(u1, v1, base_elevation + h2);
        let p3 = frame.point_at(u0, v1, base_elevation + h3);

        let up_norm = |n: Vec3| if n.dot(frame.normal()) >= 0.0 { n } else { -n };
        match self.heading {
            CornerHeading::NorthEast | CornerHeading::SouthEast => {
                let norm_a = up_norm((p1 - p0).cross(p2 - p0).normalize());
                let norm_b = up_norm((p2 - p0).cross(p3 - p0).normalize());
                target.push_quad(Quad::new(p0, p1, p2, p2, norm_a, color));
                target.push_quad(Quad::new(p0, p2, p3, p3, norm_b, color));
            }
            CornerHeading::NorthWest | CornerHeading::SouthWest => {
                let norm_a = up_norm((p1 - p0).cross(p3 - p0).normalize());
                let norm_b = up_norm((p2 - p1).cross(p3 - p1).normalize());
                target.push_quad(Quad::new(p0, p1, p3, p3, norm_a, color));
                target.push_quad(Quad::new(p1, p2, p3, p3, norm_b, color));
            }
        }

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
