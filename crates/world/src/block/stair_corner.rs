use glam::Vec3;
use crate::{MeshTarget, Quad};
use super::cell::GridCell;
use super::corner::CornerHeading;
use crate::frame::SurfaceFrame;
use super::shape::VoxelShape;

/// Stepped staircase corner block supporting both convex and concave turns.
pub struct StairCorner {
    heading: CornerHeading,
    concave: bool,
}

impl StairCorner {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(heading: CornerHeading, concave: bool) -> Self {
        Self { heading, concave }
    }

    /// Secondary constructor for an outer convex corner turn.
    pub fn convex(heading: CornerHeading) -> Self {
        Self::new(heading, false)
    }

    /// Secondary constructor for an inner concave corner turn.
    pub fn concave(heading: CornerHeading) -> Self {
        Self::new(heading, true)
    }
}

impl VoxelShape for StairCorner {
    fn elevation_at(&self, local_u: f32, local_v: f32) -> f32 {
        let t = if self.concave {
            self.heading.valley_elevation(local_u, local_v)
        } else {
            self.heading.ridge_elevation(local_u, local_v)
        };
        if t < 0.5 { 0.5 } else { 1.0 }
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
        let um = (u0 + u1) * 0.5;
        let vm = (v0 + v1) * 0.5;
        let h0 = base_elevation;
        let h1 = base_elevation + 0.5;
        let h2 = base_elevation + 1.0;
        let pt = |u: f32, v: f32, h: f32| frame.point_at(u, v, h);
        let n = frame.normal();

        // Lower step tread across the cell
        target.push_quad(Quad::new(pt(u0, v0, h1), pt(u1, v0, h1), pt(u1, v1, h1), pt(u0, v1, h1), n, color));

        // Upper step tread elevated in the corner
        let (cu0, cu1, cv0, cv1) = match self.heading {
            CornerHeading::NorthEast => (um, u1, vm, v1),
            CornerHeading::NorthWest => (u0, um, vm, v1),
            CornerHeading::SouthEast => (um, u1, v0, vm),
            CornerHeading::SouthWest => (u0, um, v0, vm),
        };
        target.push_quad(Quad::new(pt(cu0, cv0, h2), pt(cu1, cv0, h2), pt(cu1, cv1, h2), pt(cu0, cv1, h2), n, color));

        // Upper step risers
        target.push_quad(Quad::new(pt(cu0, cv0, h1), pt(cu1, cv0, h1), pt(cu1, cv0, h2), pt(cu0, cv0, h2), -frame.bitangent(), color));
        target.push_quad(Quad::new(pt(cu0, cv0, h1), pt(cu0, cv0, h2), pt(cu0, cv1, h2), pt(cu0, cv1, h1), -frame.tangent(), color));

        // Outer base walls
        let [b0, b1, b2, b3] = cell.corner_points(frame, h0);
        target.push_quad(Quad::new(b0, b1, pt(u1, v0, h1), pt(u0, v0, h1), -frame.bitangent(), color));
        target.push_quad(Quad::new(pt(u0, v1, h1), pt(u1, v1, h1), b2, b3, frame.bitangent(), color));
        target.push_quad(Quad::new(b3, b0, pt(u0, v0, h1), pt(u0, v1, h1), -frame.tangent(), color));
        target.push_quad(Quad::new(b1, b2, pt(u1, v1, h1), pt(u1, v0, h1), frame.tangent(), color));
    }
}
