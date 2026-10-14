use glam::Vec3;
use crate::{MeshTarget, Quad};
use super::cell::GridCell;
use crate::frame::SurfaceFrame;
use super::shape::VoxelShape;
use super::wedge::WedgeHeading;

/// Gentle 2:1 slope wedge block with customizable vertical range.
pub struct GentleWedge {
    heading: WedgeHeading,
    start_elevation: f32,
    end_elevation: f32,
}

impl GentleWedge {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(heading: WedgeHeading, start_elevation: f32, end_elevation: f32) -> Self {
        Self {
            heading,
            start_elevation,
            end_elevation,
        }
    }

    /// Secondary constructor for lower half of a 2-block run (0.0m to 0.5m).
    pub fn lower(heading: WedgeHeading) -> Self {
        Self::new(heading, 0.0, 0.5)
    }

    /// Secondary constructor for upper half of a 2-block run (0.5m to 1.0m).
    pub fn upper(heading: WedgeHeading) -> Self {
        Self::new(heading, 0.5, 1.0)
    }

    fn slope_normal(&self, frame: &SurfaceFrame) -> Vec3 {
        let rise = self.end_elevation - self.start_elevation;
        match self.heading {
            WedgeHeading::North => (frame.normal() - frame.bitangent() * rise).normalize(),
            WedgeHeading::South => (frame.normal() + frame.bitangent() * rise).normalize(),
            WedgeHeading::East => (frame.normal() - frame.tangent() * rise).normalize(),
            WedgeHeading::West => (frame.normal() + frame.tangent() * rise).normalize(),
        }
    }
}

impl VoxelShape for GentleWedge {
    fn elevation_at(&self, local_u: f32, local_v: f32) -> f32 {
        let frac = self.heading.elevation_at(local_u, local_v);
        self.start_elevation + frac * (self.end_elevation - self.start_elevation)
    }

    fn shape_color(&self) -> Vec3 {
        if self.start_elevation < 0.25 {
            Vec3::new(0.25, 0.75, 1.0)
        } else {
            Vec3::new(0.1, 0.45, 1.0)
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
        let rise = self.end_elevation - self.start_elevation;

        let (h0, h1, h2, h3) = match self.heading {
            WedgeHeading::North => (self.start_elevation, self.start_elevation, self.end_elevation, self.end_elevation),
            WedgeHeading::South => (self.end_elevation, self.end_elevation, self.start_elevation, self.start_elevation),
            WedgeHeading::East => (self.start_elevation, self.end_elevation, self.end_elevation, self.start_elevation),
            WedgeHeading::West => (self.end_elevation, self.start_elevation, self.start_elevation, self.end_elevation),
        };

        let p0 = frame.point_at(u0, v0, base_elevation + h0);
        let p1 = frame.point_at(u1, v0, base_elevation + h1);
        let p2 = frame.point_at(u1, v1, base_elevation + h2);
        let p3 = frame.point_at(u0, v1, base_elevation + h3);

        target.push_quad(Quad::new(p0, p1, p2, p3, self.slope_normal(frame), color));

        let [b0, b1, b2, b3] = cell.corner_points(frame, base_elevation);

        if self.start_elevation > 0.0 || rise > 0.0 {
            if h0 > 0.0 || h1 > 0.0 {
                target.push_quad(Quad::new(b0, b1, p1, p0, -frame.bitangent(), color));
            }
            if h2 > 0.0 || h3 > 0.0 {
                target.push_quad(Quad::new(p3, p2, b2, b3, frame.bitangent(), color));
            }
            target.push_quad(Quad::new(b3, b0, p0, p3, -frame.tangent(), color));
            target.push_quad(Quad::new(b1, b2, p2, p1, frame.tangent(), color));
        }
    }
}
