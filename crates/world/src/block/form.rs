use glam::Vec3;
use crate::MeshTarget;
use crate::frame::SurfaceFrame;
use super::Block;
use super::cell::GridCell;
use super::corner::CornerHeading;
use super::cuboid::CuboidShape;
use super::pyramid::PyramidPeak;
use super::ridge::RidgeCorner;
use super::shape::VoxelShape;
use super::stair::StairFlight;
use super::valley::ValleyCorner;
use super::wedge::{RampWedge, WedgeHeading};

/// Form-Block encapsulating pure geometric shape and spatial cell coordinates.
pub struct FormBlock {
    cell: GridCell,
    base_elevation: f32,
    shape: Box<dyn VoxelShape>,
}

impl FormBlock {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(cell: GridCell, base_elevation: f32, shape: Box<dyn VoxelShape>) -> Self {
        Self {
            cell,
            base_elevation,
            shape,
        }
    }

    /// Secondary constructor for a 1.0m full cube form.
    pub fn cube(grid_u: i32, grid_v: i32, base: f32) -> Self {
        Self::new(GridCell::new(grid_u, grid_v), base, Box::new(CuboidShape::full_cube()))
    }

    /// Secondary constructor for a 0.5m shelf slab form.
    pub fn slab(grid_u: i32, grid_v: i32, base: f32) -> Self {
        Self::new(GridCell::new(grid_u, grid_v), base, Box::new(CuboidShape::half_slab()))
    }

    /// Secondary constructor for a 45° climbable ramp wedge form.
    pub fn wedge(grid_u: i32, grid_v: i32, base: f32, heading: WedgeHeading) -> Self {
        Self::new(GridCell::new(grid_u, grid_v), base, Box::new(RampWedge::new(heading)))
    }

    /// Secondary constructor for a 45° convex ridge corner wedge form.
    pub fn ridge_corner(grid_u: i32, grid_v: i32, base: f32, heading: CornerHeading) -> Self {
        Self::new(GridCell::new(grid_u, grid_v), base, Box::new(RidgeCorner::new(heading)))
    }

    /// Secondary constructor for a 45° concave valley corner wedge form.
    pub fn valley_corner(grid_u: i32, grid_v: i32, base: f32, heading: CornerHeading) -> Self {
        Self::new(GridCell::new(grid_u, grid_v), base, Box::new(ValleyCorner::new(heading)))
    }

    /// Secondary constructor for a 45° slope-aligned pyramidal apex peak form (0.5m height).
    pub fn peak(grid_u: i32, grid_v: i32, base: f32) -> Self {
        Self::new(GridCell::new(grid_u, grid_v), base, Box::new(PyramidPeak::slope_aligned()))
    }

    /// Secondary constructor for a two-tiered stepped staircase form.
    pub fn stair(grid_u: i32, grid_v: i32, base: f32, heading: WedgeHeading) -> Self {
        Self::new(GridCell::new(grid_u, grid_v), base, Box::new(StairFlight::new(heading)))
    }

    /// Secondary constructor for a gentle 2:1 lower ramp wedge (0.0m to 0.5m).
    pub fn gentle_lower_wedge(grid_u: i32, grid_v: i32, base: f32, heading: WedgeHeading) -> Self {
        Self::new(GridCell::new(grid_u, grid_v), base, Box::new(super::gentle_wedge::GentleWedge::lower(heading)))
    }

    /// Secondary constructor for a gentle 2:1 upper ramp wedge (0.5m to 1.0m).
    pub fn gentle_upper_wedge(grid_u: i32, grid_v: i32, base: f32, heading: WedgeHeading) -> Self {
        Self::new(GridCell::new(grid_u, grid_v), base, Box::new(super::gentle_wedge::GentleWedge::upper(heading)))
    }

    /// Secondary constructor for a gentle 2:1 lower convex ridge corner.
    pub fn gentle_lower_ridge(grid_u: i32, grid_v: i32, base: f32, heading: CornerHeading) -> Self {
        Self::new(GridCell::new(grid_u, grid_v), base, Box::new(super::gentle_ridge::GentleRidge::lower(heading)))
    }

    /// Secondary constructor for a gentle 2:1 upper convex ridge corner.
    pub fn gentle_upper_ridge(grid_u: i32, grid_v: i32, base: f32, heading: CornerHeading) -> Self {
        Self::new(GridCell::new(grid_u, grid_v), base, Box::new(super::gentle_ridge::GentleRidge::upper(heading)))
    }

    /// Secondary constructor for a gentle 2:1 lower concave valley corner.
    pub fn gentle_lower_valley(grid_u: i32, grid_v: i32, base: f32, heading: CornerHeading) -> Self {
        Self::new(GridCell::new(grid_u, grid_v), base, Box::new(super::gentle_valley::GentleValley::lower(heading)))
    }

    /// Secondary constructor for a gentle 2:1 upper concave valley corner.
    pub fn gentle_upper_valley(grid_u: i32, grid_v: i32, base: f32, heading: CornerHeading) -> Self {
        Self::new(GridCell::new(grid_u, grid_v), base, Box::new(super::gentle_valley::GentleValley::upper(heading)))
    }

    /// Secondary constructor for a stepped staircase corner.
    pub fn stair_corner(grid_u: i32, grid_v: i32, base: f32, heading: CornerHeading, concave: bool) -> Self {
        Self::new(GridCell::new(grid_u, grid_v), base, Box::new(super::stair_corner::StairCorner::new(heading, concave)))
    }

    /// Secondary constructor for an outer convex stepped staircase corner.
    pub fn stair_corner_convex(grid_u: i32, grid_v: i32, base: f32, heading: CornerHeading) -> Self {
        Self::stair_corner(grid_u, grid_v, base, heading, false)
    }

    /// Secondary constructor for an inner concave stepped staircase corner.
    pub fn stair_corner_concave(grid_u: i32, grid_v: i32, base: f32, heading: CornerHeading) -> Self {
        Self::stair_corner(grid_u, grid_v, base, heading, true)
    }


    /// Constrains a surface position if the tangential coordinates fall within this form.
    pub fn constrain_surface(
        &self,
        base: Vec3,
        u: f32,
        v: f32,
        frame: &SurfaceFrame,
        distance: f32,
    ) -> Vec3 {
        if self.cell.contains(u, v) {
            let (local_u, local_v) = self.cell.local_fraction(u, v);
            let elev = self.base_elevation + self.shape.elevation_at(local_u, local_v);
            let block_pos = frame.point_at(u, v, elev + distance);
            if frame.height_of(block_pos) > frame.height_of(base) {
                return block_pos;
            }
        }
        base
    }

}

impl Block for FormBlock {
    fn constrain_surface(&self, base: Vec3, u: f32, v: f32, frame: &SurfaceFrame, distance: f32) -> Vec3 {
        self.constrain_surface(base, u, v, frame, distance)
    }

    fn emit_surface(&self, frame: &SurfaceFrame, target: &mut dyn MeshTarget) {
        self.shape.emit_shape(
            self.cell,
            self.base_elevation,
            frame,
            self.shape.shape_color(),
            target,
        );
    }
}
