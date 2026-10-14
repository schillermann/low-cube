use glam::Vec3;
use crate::{MeshTarget, Quad};
use super::cell::GridCell;
use crate::frame::SurfaceFrame;
use super::shape::VoxelShape;
use super::wedge::WedgeHeading;

/// Two-tiered stepped staircase ascending along a cardinal heading.
pub struct StairFlight {
    heading: WedgeHeading,
}

impl StairFlight {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(heading: WedgeHeading) -> Self {
        Self { heading }
    }
}

impl VoxelShape for StairFlight {
    fn elevation_at(&self, local_u: f32, local_v: f32) -> f32 {
        let t = match self.heading {
            WedgeHeading::North => local_v,
            WedgeHeading::South => 1.0 - local_v,
            WedgeHeading::East => local_u,
            WedgeHeading::West => 1.0 - local_u,
        };
        if t < 0.5 {
            0.5
        } else {
            1.0
        }
    }

    fn shape_color(&self) -> Vec3 {
        Vec3::new(0.1, 0.95, 0.2)
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
        let t = frame.tangent();
        let b = frame.bitangent();

        match self.heading {
            WedgeHeading::North => {
                // Treads
                target.push_quad(Quad::new(pt(u0, v0, h1), pt(u1, v0, h1), pt(u1, vm, h1), pt(u0, vm, h1), n, color));
                target.push_quad(Quad::new(pt(u0, vm, h2), pt(u1, vm, h2), pt(u1, v1, h2), pt(u0, v1, h2), n, color));
                // Risers
                target.push_quad(Quad::new(pt(u0, v0, h0), pt(u1, v0, h0), pt(u1, v0, h1), pt(u0, v0, h1), -b, color));
                target.push_quad(Quad::new(pt(u0, vm, h1), pt(u1, vm, h1), pt(u1, vm, h2), pt(u0, vm, h2), -b, color));
                // Back
                target.push_quad(Quad::new(pt(u1, v1, h0), pt(u0, v1, h0), pt(u0, v1, h2), pt(u1, v1, h2), b, color));
                // Sides (West & East)
                target.push_quad(Quad::new(pt(u0, vm, h0), pt(u0, v0, h0), pt(u0, v0, h1), pt(u0, vm, h1), -t, color));
                target.push_quad(Quad::new(pt(u0, v1, h0), pt(u0, vm, h0), pt(u0, vm, h2), pt(u0, v1, h2), -t, color));
                target.push_quad(Quad::new(pt(u1, v0, h0), pt(u1, vm, h0), pt(u1, vm, h1), pt(u1, v0, h1), t, color));
                target.push_quad(Quad::new(pt(u1, vm, h0), pt(u1, v1, h0), pt(u1, v1, h2), pt(u1, vm, h2), t, color));
            }
            WedgeHeading::South => {
                // Treads
                target.push_quad(Quad::new(pt(u0, vm, h1), pt(u1, vm, h1), pt(u1, v1, h1), pt(u0, v1, h1), n, color));
                target.push_quad(Quad::new(pt(u0, v0, h2), pt(u1, v0, h2), pt(u1, vm, h2), pt(u0, vm, h2), n, color));
                // Risers
                target.push_quad(Quad::new(pt(u1, v1, h0), pt(u0, v1, h0), pt(u0, v1, h1), pt(u1, v1, h1), b, color));
                target.push_quad(Quad::new(pt(u1, vm, h1), pt(u0, vm, h1), pt(u0, vm, h2), pt(u1, vm, h2), b, color));
                // Back
                target.push_quad(Quad::new(pt(u0, v0, h0), pt(u1, v0, h0), pt(u1, v0, h2), pt(u0, v0, h2), -b, color));
                // Sides (West & East)
                target.push_quad(Quad::new(pt(u0, v1, h0), pt(u0, vm, h0), pt(u0, vm, h1), pt(u0, v1, h1), -t, color));
                target.push_quad(Quad::new(pt(u0, vm, h0), pt(u0, v0, h0), pt(u0, v0, h2), pt(u0, vm, h2), -t, color));
                target.push_quad(Quad::new(pt(u1, vm, h0), pt(u1, v1, h0), pt(u1, v1, h1), pt(u1, vm, h1), t, color));
                target.push_quad(Quad::new(pt(u1, v0, h0), pt(u1, vm, h0), pt(u1, vm, h2), pt(u1, v0, h2), t, color));
            }
            WedgeHeading::East => {
                // Treads
                target.push_quad(Quad::new(pt(u0, v0, h1), pt(um, v0, h1), pt(um, v1, h1), pt(u0, v1, h1), n, color));
                target.push_quad(Quad::new(pt(um, v0, h2), pt(u1, v0, h2), pt(u1, v1, h2), pt(um, v1, h2), n, color));
                // Risers
                target.push_quad(Quad::new(pt(u0, v1, h0), pt(u0, v0, h0), pt(u0, v0, h1), pt(u0, v1, h1), -t, color));
                target.push_quad(Quad::new(pt(um, v1, h1), pt(um, v0, h1), pt(um, v0, h2), pt(um, v1, h2), -t, color));
                // Back
                target.push_quad(Quad::new(pt(u1, v0, h0), pt(u1, v1, h0), pt(u1, v1, h2), pt(u1, v0, h2), t, color));
                // Sides (South & North)
                target.push_quad(Quad::new(pt(u0, v0, h0), pt(um, v0, h0), pt(um, v0, h1), pt(u0, v0, h1), -b, color));
                target.push_quad(Quad::new(pt(um, v0, h0), pt(u1, v0, h0), pt(u1, v0, h2), pt(um, v0, h2), -b, color));
                target.push_quad(Quad::new(pt(um, v1, h0), pt(u0, v1, h0), pt(u0, v1, h1), pt(um, v1, h1), b, color));
                target.push_quad(Quad::new(pt(u1, v1, h0), pt(um, v1, h0), pt(um, v1, h2), pt(u1, v1, h2), b, color));
            }
            WedgeHeading::West => {
                // Treads
                target.push_quad(Quad::new(pt(um, v0, h1), pt(u1, v0, h1), pt(u1, v1, h1), pt(um, v1, h1), n, color));
                target.push_quad(Quad::new(pt(u0, v0, h2), pt(um, v0, h2), pt(um, v1, h2), pt(u0, v1, h2), n, color));
                // Risers
                target.push_quad(Quad::new(pt(u1, v0, h0), pt(u1, v1, h0), pt(u1, v1, h1), pt(u1, v0, h1), t, color));
                target.push_quad(Quad::new(pt(um, v0, h1), pt(um, v1, h1), pt(um, v1, h2), pt(um, v0, h2), t, color));
                // Back
                target.push_quad(Quad::new(pt(u0, v1, h0), pt(u0, v0, h0), pt(u0, v0, h2), pt(u0, v1, h2), -t, color));
                // Sides (South & North)
                target.push_quad(Quad::new(pt(um, v0, h0), pt(u1, v0, h0), pt(u1, v0, h1), pt(um, v0, h1), -b, color));
                target.push_quad(Quad::new(pt(u0, v0, h0), pt(um, v0, h0), pt(um, v0, h2), pt(u0, v0, h2), -b, color));
                target.push_quad(Quad::new(pt(u1, v1, h0), pt(um, v1, h0), pt(um, v1, h1), pt(u1, v1, h1), b, color));
                target.push_quad(Quad::new(pt(um, v1, h0), pt(u0, v1, h0), pt(u0, v1, h2), pt(um, v1, h2), b, color));
            }
        }
    }
}
