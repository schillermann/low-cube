use glam::Vec3;
use crate::{MeshTarget, Quad};
use super::cell::GridCell;
use crate::frame::SurfaceFrame;
use super::shape::VoxelShape;

/// Cardinal heading along which a directional slope rises.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WedgeHeading {
    North,
    South,
    East,
    West,
}

impl WedgeHeading {
    pub fn elevation_at(&self, u: f32, v: f32) -> f32 {
        match self {
            Self::North => v,
            Self::South => 1.0 - v,
            Self::East => u,
            Self::West => 1.0 - u,
        }
    }

    pub fn corner_heights(&self) -> (f32, f32, f32, f32) {
        match self {
            Self::North => (0.0, 0.0, 1.0, 1.0),
            Self::South => (1.0, 1.0, 0.0, 0.0),
            Self::East => (0.0, 1.0, 1.0, 0.0),
            Self::West => (1.0, 0.0, 0.0, 1.0),
        }
    }

    pub fn slope_normal(&self, frame: &SurfaceFrame) -> Vec3 {
        match self {
            Self::North => (frame.normal() - frame.bitangent()).normalize(),
            Self::South => (frame.normal() + frame.bitangent()).normalize(),
            Self::East => (frame.normal() - frame.tangent()).normalize(),
            Self::West => (frame.normal() + frame.tangent()).normalize(),
        }
    }
}

/// 45° faceted climbable ramp wedge rising along an encapsulated heading.
pub struct RampWedge {
    heading: WedgeHeading,
}

impl RampWedge {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(heading: WedgeHeading) -> Self {
        Self { heading }
    }
}

impl VoxelShape for RampWedge {
    fn elevation_at(&self, local_u: f32, local_v: f32) -> f32 {
        self.heading.elevation_at(local_u, local_v)
    }

    fn shape_color(&self) -> Vec3 {
        Vec3::new(1.0, 0.55, 0.0)
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

        let (h0, h1, h2, h3) = self.heading.corner_heights();
        let slope_normal = self.heading.slope_normal(frame);

        let p0 = frame.point_at(u0, v0, base_elevation + h0);
        let p1 = frame.point_at(u1, v0, base_elevation + h1);
        let p2 = frame.point_at(u1, v1, base_elevation + h2);
        let p3 = frame.point_at(u0, v1, base_elevation + h3);

        target.push_quad(Quad::new(p0, p1, p2, p3, slope_normal, color));

        let [b0, b1, b2, b3] = cell.corner_points(frame, base_elevation);

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
