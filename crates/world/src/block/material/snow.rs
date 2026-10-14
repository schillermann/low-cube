use glam::Vec3;
use crate::block::{Block, CornerHeading, FormBlock, WedgeHeading};
use crate::frame::SurfaceFrame;
use crate::MeshTarget;
use super::material_block::MaterialBlock;

/// Material entity decorating form blocks with crystalline alpine snow color.
pub struct SnowBlock {
    decorated: MaterialBlock,
}

impl SnowBlock {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(decorated: MaterialBlock) -> Self {
        Self { decorated }
    }

    /// Secondary constructor decorating a form block with crystalline alpine snow color.
    pub fn natural(form: FormBlock) -> Self {
        Self::new(MaterialBlock::new(Box::new(form), Vec3::new(0.92, 0.95, 0.98)))
    }

    pub fn cube(grid_u: i32, grid_v: i32, base: f32) -> Self {
        Self::natural(FormBlock::cube(grid_u, grid_v, base))
    }

    pub fn slab(grid_u: i32, grid_v: i32, base: f32) -> Self {
        Self::natural(FormBlock::slab(grid_u, grid_v, base))
    }

    pub fn wedge(grid_u: i32, grid_v: i32, base: f32, heading: WedgeHeading) -> Self {
        Self::natural(FormBlock::wedge(grid_u, grid_v, base, heading))
    }

    pub fn ridge_corner(grid_u: i32, grid_v: i32, base: f32, heading: CornerHeading) -> Self {
        Self::natural(FormBlock::ridge_corner(grid_u, grid_v, base, heading))
    }

    pub fn valley_corner(grid_u: i32, grid_v: i32, base: f32, heading: CornerHeading) -> Self {
        Self::natural(FormBlock::valley_corner(grid_u, grid_v, base, heading))
    }

    pub fn diagonal_slope(grid_u: i32, grid_v: i32, base: f32, heading: CornerHeading) -> Self {
        Self::natural(FormBlock::diagonal_slope(grid_u, grid_v, base, heading))
    }

    pub fn peak(grid_u: i32, grid_v: i32, base: f32) -> Self {
        Self::natural(FormBlock::peak(grid_u, grid_v, base))
    }

    pub fn stair(grid_u: i32, grid_v: i32, base: f32, heading: WedgeHeading) -> Self {
        Self::natural(FormBlock::stair(grid_u, grid_v, base, heading))
    }

    pub fn gentle_lower_wedge(grid_u: i32, grid_v: i32, base: f32, heading: WedgeHeading) -> Self {
        Self::natural(FormBlock::gentle_lower_wedge(grid_u, grid_v, base, heading))
    }

    pub fn gentle_upper_wedge(grid_u: i32, grid_v: i32, base: f32, heading: WedgeHeading) -> Self {
        Self::natural(FormBlock::gentle_upper_wedge(grid_u, grid_v, base, heading))
    }

    pub fn gentle_lower_ridge(grid_u: i32, grid_v: i32, base: f32, heading: CornerHeading) -> Self {
        Self::natural(FormBlock::gentle_lower_ridge(grid_u, grid_v, base, heading))
    }

    pub fn gentle_upper_ridge(grid_u: i32, grid_v: i32, base: f32, heading: CornerHeading) -> Self {
        Self::natural(FormBlock::gentle_upper_ridge(grid_u, grid_v, base, heading))
    }

    pub fn gentle_lower_valley(grid_u: i32, grid_v: i32, base: f32, heading: CornerHeading) -> Self {
        Self::natural(FormBlock::gentle_lower_valley(grid_u, grid_v, base, heading))
    }

    pub fn gentle_upper_valley(grid_u: i32, grid_v: i32, base: f32, heading: CornerHeading) -> Self {
        Self::natural(FormBlock::gentle_upper_valley(grid_u, grid_v, base, heading))
    }

    pub fn stair_corner(grid_u: i32, grid_v: i32, base: f32, heading: CornerHeading, concave: bool) -> Self {
        Self::natural(FormBlock::stair_corner(grid_u, grid_v, base, heading, concave))
    }

    pub fn stair_corner_convex(grid_u: i32, grid_v: i32, base: f32, heading: CornerHeading) -> Self {
        Self::natural(FormBlock::stair_corner_convex(grid_u, grid_v, base, heading))
    }

    pub fn stair_corner_concave(grid_u: i32, grid_v: i32, base: f32, heading: CornerHeading) -> Self {
        Self::natural(FormBlock::stair_corner_concave(grid_u, grid_v, base, heading))
    }
}

impl Block for SnowBlock {
    fn constrain_surface(
        &self,
        base: Vec3,
        u: f32,
        v: f32,
        frame: &SurfaceFrame,
        distance: f32,
    ) -> Vec3 {
        self.decorated.constrain_surface(base, u, v, frame, distance)
    }

    fn emit_surface(&self, frame: &SurfaceFrame, target: &mut dyn MeshTarget) {
        self.decorated.emit_surface(frame, target);
    }
}
