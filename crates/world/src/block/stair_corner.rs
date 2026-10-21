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

    fn quadrant_heights(&self, h1: f32, h2: f32) -> [[f32; 2]; 2] {
        if self.concave {
            match self.heading {
                CornerHeading::NorthEast => [[h1, h2], [h2, h2]],
                CornerHeading::NorthWest => [[h2, h2], [h1, h2]],
                CornerHeading::SouthEast => [[h2, h1], [h2, h2]],
                CornerHeading::SouthWest => [[h2, h2], [h2, h1]],
            }
        } else {
            match self.heading {
                CornerHeading::NorthEast => [[h1, h1], [h1, h2]],
                CornerHeading::NorthWest => [[h1, h2], [h1, h1]],
                CornerHeading::SouthEast => [[h1, h1], [h2, h1]],
                CornerHeading::SouthWest => [[h2, h1], [h1, h1]],
            }
        }
    }
}

impl VoxelShape for StairCorner {
    fn elevation_at(&self, local_u: f32, local_v: f32) -> f32 {
        let qx = usize::from(local_u >= 0.5);
        let qy = usize::from(local_v >= 0.5);
        self.quadrant_heights(0.25, 0.5)[qx][qy]
    }

    fn shape_color(&self) -> Vec3 {
        if self.concave {
            Vec3::new(0.7, 0.95, 0.0)
        } else {
            Vec3::new(0.5, 0.95, 0.1)
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
        let um = (u0 + u1) * 0.5;
        let vm = (v0 + v1) * 0.5;
        let us = [u0, um, u1];
        let vs = [v0, vm, v1];
        let h0 = base_elevation;
        let hq = self.quadrant_heights(base_elevation + 0.25, base_elevation + 0.5);
        let pt = |u: f32, v: f32, h: f32| frame.point_at(u, v, h);
        let (n, t, b) = (frame.normal(), frame.tangent(), frame.bitangent());

        for qx in 0..2 {
            for qy in 0..2 {
                let (qu0, qu1, qv0, qv1, top) = (us[qx], us[qx + 1], vs[qy], vs[qy + 1], hq[qx][qy]);
                target.push_quad(Quad::new(pt(qu0, qv0, top), pt(qu1, qv0, top), pt(qu1, qv1, top), pt(qu0, qv1, top), n, color));

                let base_s = if qy == 0 { h0 } else { hq[qx][qy - 1] };
                if top > base_s {
                    target.push_quad(Quad::new(pt(qu0, qv0, base_s), pt(qu1, qv0, base_s), pt(qu1, qv0, top), pt(qu0, qv0, top), -b, color));
                }
                let base_n = if qy == 1 { h0 } else { hq[qx][qy + 1] };
                if top > base_n {
                    target.push_quad(Quad::new(pt(qu1, qv1, base_n), pt(qu0, qv1, base_n), pt(qu0, qv1, top), pt(qu1, qv1, top), b, color));
                }
                let base_w = if qx == 0 { h0 } else { hq[qx - 1][qy] };
                if top > base_w {
                    target.push_quad(Quad::new(pt(qu0, qv1, base_w), pt(qu0, qv0, base_w), pt(qu0, qv0, top), pt(qu0, qv1, top), -t, color));
                }
                let base_e = if qx == 1 { h0 } else { hq[qx + 1][qy] };
                if top > base_e {
                    target.push_quad(Quad::new(pt(qu1, qv0, base_e), pt(qu1, qv1, base_e), pt(qu1, qv1, top), pt(qu1, qv0, top), t, color));
                }
            }
        }
    }
}
