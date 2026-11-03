use glam::Vec3;
use crate::frame::SurfaceFrame;

/// Discrete integer coordinate cell in the planar voxel grid.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GridCell {
    grid_u: i32,
    grid_v: i32,
}

impl GridCell {
    pub const SPAN: f32 = 0.5;

    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(grid_u: i32, grid_v: i32) -> Self {
        Self { grid_u, grid_v }
    }

    /// Secondary constructor locating the discrete grid cell containing planar coordinates (u, v).
    pub fn at_planar(u: f32, v: f32) -> Self {
        Self::new((u / Self::SPAN).floor() as i32, (v / Self::SPAN).floor() as i32)
    }

    /// Delivers the discrete integer coordinates to a consumer closure ("Tell, Don't Ask").
    pub fn feed_coordinates<R>(&self, consumer: impl FnOnce(i32, i32) -> R) -> R {
        consumer(self.grid_u, self.grid_v)
    }

    pub fn u_bounds(&self) -> (f32, f32) {
        ((self.grid_u as f32) * Self::SPAN, ((self.grid_u + 1) as f32) * Self::SPAN)
    }

    pub fn v_bounds(&self) -> (f32, f32) {
        ((self.grid_v as f32) * Self::SPAN, ((self.grid_v + 1) as f32) * Self::SPAN)
    }

    pub fn corner_points(&self, frame: &SurfaceFrame, elevation: f32) -> [Vec3; 4] {
        let (u0, u1) = self.u_bounds();
        let (v0, v1) = self.v_bounds();
        [
            frame.point_at(u0, v0, elevation),
            frame.point_at(u1, v0, elevation),
            frame.point_at(u1, v1, elevation),
            frame.point_at(u0, v1, elevation),
        ]
    }

    pub fn contains(&self, u: f32, v: f32) -> bool {
        let (u0, u1) = self.u_bounds();
        let (v0, v1) = self.v_bounds();
        u >= u0 && u <= u1 && v >= v0 && v <= v1
    }

    pub fn local_fraction(&self, u: f32, v: f32) -> (f32, f32) {
        let (u0, u1) = self.u_bounds();
        let (v0, v1) = self.v_bounds();
        (
            ((u - u0) / (u1 - u0)).clamp(0.0, 1.0),
            ((v - v0) / (v1 - v0)).clamp(0.0, 1.0),
        )
    }

    pub fn center(&self) -> (f32, f32) {
        let (u0, u1) = self.u_bounds();
        let (v0, v1) = self.v_bounds();
        ((u0 + u1) * 0.5, (v0 + v1) * 0.5)
    }

    pub fn clamp_interior(&self, u: f32, v: f32, margin: f32) -> (f32, f32) {
        let (u0, u1) = self.u_bounds();
        let (v0, v1) = self.v_bounds();
        (u.clamp(u0 + margin, u1 - margin), v.clamp(v0 + margin, v1 - margin))
    }

    pub fn clamp_walled(&self, u: f32, v: f32, margin: f32, walls: (bool, bool, bool, bool)) -> (f32, f32) {
        let (u0, u1) = self.u_bounds();
        let (v0, v1) = self.v_bounds();
        let min_u = if walls.0 { u0 + margin } else { u0 };
        let max_u = if walls.1 { u1 - margin } else { u1 };
        let min_v = if walls.2 { v0 + margin } else { v0 };
        let max_v = if walls.3 { v1 - margin } else { v1 };
        (u.clamp(min_u, max_u), v.clamp(min_v, max_v))
    }
}
