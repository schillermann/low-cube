use glam::Vec3;
use crate::frame::SurfaceFrame;

/// Discrete integer coordinate cell in the planar voxel grid.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GridCell {
    grid_u: i32,
    grid_v: i32,
}

impl GridCell {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(grid_u: i32, grid_v: i32) -> Self {
        Self { grid_u, grid_v }
    }

    /// Delivers the discrete integer coordinates to a consumer closure ("Tell, Don't Ask").
    pub fn feed_coordinates<R>(&self, consumer: impl FnOnce(i32, i32) -> R) -> R {
        consumer(self.grid_u, self.grid_v)
    }

    pub fn u_bounds(&self) -> (f32, f32) {
        (self.grid_u as f32, (self.grid_u + 1) as f32)
    }

    pub fn v_bounds(&self) -> (f32, f32) {
        (self.grid_v as f32, (self.grid_v + 1) as f32)
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
        let (u0, _) = self.u_bounds();
        let (v0, _) = self.v_bounds();
        ((u - u0).clamp(0.0, 1.0), (v - v0).clamp(0.0, 1.0))
    }
}
