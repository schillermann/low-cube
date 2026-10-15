use crate::block::{CornerHeading, FormBlock, WedgeHeading};
use super::material_block::MaterialBlock;

/// Factory assembling form blocks decorated with warm amber desert sand color.
pub struct SandBlock;

impl SandBlock {
    /// Secondary constructor decorating a form block with warm amber desert sand color.
    pub fn natural(form: FormBlock) -> MaterialBlock {
        MaterialBlock::sand(Box::new(form))
    }

    pub fn cube(grid_u: i32, grid_v: i32, base: f32) -> MaterialBlock {
        Self::natural(FormBlock::cube(grid_u, grid_v, base))
    }

    pub fn slab(grid_u: i32, grid_v: i32, base: f32) -> MaterialBlock {
        Self::natural(FormBlock::slab(grid_u, grid_v, base))
    }

    pub fn wedge(grid_u: i32, grid_v: i32, base: f32, heading: WedgeHeading) -> MaterialBlock {
        Self::natural(FormBlock::wedge(grid_u, grid_v, base, heading))
    }

    pub fn ridge_corner(grid_u: i32, grid_v: i32, base: f32, heading: CornerHeading) -> MaterialBlock {
        Self::natural(FormBlock::ridge_corner(grid_u, grid_v, base, heading))
    }

    pub fn valley_corner(grid_u: i32, grid_v: i32, base: f32, heading: CornerHeading) -> MaterialBlock {
        Self::natural(FormBlock::valley_corner(grid_u, grid_v, base, heading))
    }

    pub fn diagonal_slope(grid_u: i32, grid_v: i32, base: f32, heading: CornerHeading) -> MaterialBlock {
        Self::natural(FormBlock::diagonal_slope(grid_u, grid_v, base, heading))
    }

    pub fn peak(grid_u: i32, grid_v: i32, base: f32) -> MaterialBlock {
        Self::natural(FormBlock::peak(grid_u, grid_v, base))
    }

    pub fn stair(grid_u: i32, grid_v: i32, base: f32, heading: WedgeHeading) -> MaterialBlock {
        Self::natural(FormBlock::stair(grid_u, grid_v, base, heading))
    }

    pub fn gentle_lower_wedge(grid_u: i32, grid_v: i32, base: f32, heading: WedgeHeading) -> MaterialBlock {
        Self::natural(FormBlock::gentle_lower_wedge(grid_u, grid_v, base, heading))
    }

    pub fn gentle_upper_wedge(grid_u: i32, grid_v: i32, base: f32, heading: WedgeHeading) -> MaterialBlock {
        Self::natural(FormBlock::gentle_upper_wedge(grid_u, grid_v, base, heading))
    }

    pub fn gentle_lower_ridge(grid_u: i32, grid_v: i32, base: f32, heading: CornerHeading) -> MaterialBlock {
        Self::natural(FormBlock::gentle_lower_ridge(grid_u, grid_v, base, heading))
    }

    pub fn gentle_upper_ridge(grid_u: i32, grid_v: i32, base: f32, heading: CornerHeading) -> MaterialBlock {
        Self::natural(FormBlock::gentle_upper_ridge(grid_u, grid_v, base, heading))
    }

    pub fn gentle_lower_valley(grid_u: i32, grid_v: i32, base: f32, heading: CornerHeading) -> MaterialBlock {
        Self::natural(FormBlock::gentle_lower_valley(grid_u, grid_v, base, heading))
    }

    pub fn gentle_upper_valley(grid_u: i32, grid_v: i32, base: f32, heading: CornerHeading) -> MaterialBlock {
        Self::natural(FormBlock::gentle_upper_valley(grid_u, grid_v, base, heading))
    }

    pub fn stair_corner(grid_u: i32, grid_v: i32, base: f32, heading: CornerHeading, concave: bool) -> MaterialBlock {
        Self::natural(FormBlock::stair_corner(grid_u, grid_v, base, heading, concave))
    }

    pub fn stair_corner_convex(grid_u: i32, grid_v: i32, base: f32, heading: CornerHeading) -> MaterialBlock {
        Self::natural(FormBlock::stair_corner_convex(grid_u, grid_v, base, heading))
    }

    pub fn stair_corner_concave(grid_u: i32, grid_v: i32, base: f32, heading: CornerHeading) -> MaterialBlock {
        Self::natural(FormBlock::stair_corner_concave(grid_u, grid_v, base, heading))
    }
}
