use crate::block::{CornerHeading, FormBlock, WedgeHeading};
use super::material_block::MaterialBlock;

/// Factory assembling form blocks decorated with durable alpine stone granite or shale color.
pub struct StoneBlock;

impl StoneBlock {
    /// Secondary constructor with standard alpine gray granite stone.
    pub fn natural(form: FormBlock) -> MaterialBlock {
        MaterialBlock::stone(Box::new(form))
    }

    /// Secondary constructor for dark shale rock.
    pub fn dark(form: FormBlock) -> MaterialBlock {
        MaterialBlock::dark_stone(Box::new(form))
    }

    pub fn cube(grid_u: i32, grid_v: i32, base: f32) -> MaterialBlock {
        Self::natural(FormBlock::cube(grid_u, grid_v, base))
    }

    pub fn dark_cube(grid_u: i32, grid_v: i32, base: f32) -> MaterialBlock {
        Self::dark(FormBlock::cube(grid_u, grid_v, base))
    }

    pub fn slab(grid_u: i32, grid_v: i32, base: f32) -> MaterialBlock {
        Self::natural(FormBlock::slab(grid_u, grid_v, base))
    }

    pub fn dark_slab(grid_u: i32, grid_v: i32, base: f32) -> MaterialBlock {
        Self::dark(FormBlock::slab(grid_u, grid_v, base))
    }

    pub fn wedge(grid_u: i32, grid_v: i32, base: f32, heading: WedgeHeading) -> MaterialBlock {
        Self::natural(FormBlock::wedge(grid_u, grid_v, base, heading))
    }

    pub fn dark_wedge(grid_u: i32, grid_v: i32, base: f32, heading: WedgeHeading) -> MaterialBlock {
        Self::dark(FormBlock::wedge(grid_u, grid_v, base, heading))
    }

    pub fn ridge_corner(grid_u: i32, grid_v: i32, base: f32, heading: CornerHeading) -> MaterialBlock {
        Self::natural(FormBlock::ridge_corner(grid_u, grid_v, base, heading))
    }

    pub fn dark_ridge_corner(grid_u: i32, grid_v: i32, base: f32, heading: CornerHeading) -> MaterialBlock {
        Self::dark(FormBlock::ridge_corner(grid_u, grid_v, base, heading))
    }

    pub fn valley_corner(grid_u: i32, grid_v: i32, base: f32, heading: CornerHeading) -> MaterialBlock {
        Self::natural(FormBlock::valley_corner(grid_u, grid_v, base, heading))
    }

    pub fn dark_valley_corner(grid_u: i32, grid_v: i32, base: f32, heading: CornerHeading) -> MaterialBlock {
        Self::dark(FormBlock::valley_corner(grid_u, grid_v, base, heading))
    }

    pub fn diagonal_slope(grid_u: i32, grid_v: i32, base: f32, heading: CornerHeading) -> MaterialBlock {
        Self::natural(FormBlock::diagonal_slope(grid_u, grid_v, base, heading))
    }

    pub fn dark_diagonal_slope(grid_u: i32, grid_v: i32, base: f32, heading: CornerHeading) -> MaterialBlock {
        Self::dark(FormBlock::diagonal_slope(grid_u, grid_v, base, heading))
    }

    pub fn peak(grid_u: i32, grid_v: i32, base: f32) -> MaterialBlock {
        Self::natural(FormBlock::peak(grid_u, grid_v, base))
    }

    pub fn dark_peak(grid_u: i32, grid_v: i32, base: f32) -> MaterialBlock {
        Self::dark(FormBlock::peak(grid_u, grid_v, base))
    }

    pub fn stair(grid_u: i32, grid_v: i32, base: f32, heading: WedgeHeading) -> MaterialBlock {
        Self::natural(FormBlock::stair(grid_u, grid_v, base, heading))
    }

    pub fn dark_stair(grid_u: i32, grid_v: i32, base: f32, heading: WedgeHeading) -> MaterialBlock {
        Self::dark(FormBlock::stair(grid_u, grid_v, base, heading))
    }

    pub fn gentle_lower_wedge(grid_u: i32, grid_v: i32, base: f32, heading: WedgeHeading) -> MaterialBlock {
        Self::natural(FormBlock::gentle_lower_wedge(grid_u, grid_v, base, heading))
    }

    pub fn dark_gentle_lower_wedge(grid_u: i32, grid_v: i32, base: f32, heading: WedgeHeading) -> MaterialBlock {
        Self::dark(FormBlock::gentle_lower_wedge(grid_u, grid_v, base, heading))
    }

    pub fn gentle_upper_wedge(grid_u: i32, grid_v: i32, base: f32, heading: WedgeHeading) -> MaterialBlock {
        Self::natural(FormBlock::gentle_upper_wedge(grid_u, grid_v, base, heading))
    }

    pub fn dark_gentle_upper_wedge(grid_u: i32, grid_v: i32, base: f32, heading: WedgeHeading) -> MaterialBlock {
        Self::dark(FormBlock::gentle_upper_wedge(grid_u, grid_v, base, heading))
    }

    pub fn gentle_lower_ridge(grid_u: i32, grid_v: i32, base: f32, heading: CornerHeading) -> MaterialBlock {
        Self::natural(FormBlock::gentle_lower_ridge(grid_u, grid_v, base, heading))
    }

    pub fn dark_gentle_lower_ridge(grid_u: i32, grid_v: i32, base: f32, heading: CornerHeading) -> MaterialBlock {
        Self::dark(FormBlock::gentle_lower_ridge(grid_u, grid_v, base, heading))
    }

    pub fn gentle_upper_ridge(grid_u: i32, grid_v: i32, base: f32, heading: CornerHeading) -> MaterialBlock {
        Self::natural(FormBlock::gentle_upper_ridge(grid_u, grid_v, base, heading))
    }

    pub fn dark_gentle_upper_ridge(grid_u: i32, grid_v: i32, base: f32, heading: CornerHeading) -> MaterialBlock {
        Self::dark(FormBlock::gentle_upper_ridge(grid_u, grid_v, base, heading))
    }

    pub fn gentle_lower_valley(grid_u: i32, grid_v: i32, base: f32, heading: CornerHeading) -> MaterialBlock {
        Self::natural(FormBlock::gentle_lower_valley(grid_u, grid_v, base, heading))
    }

    pub fn dark_gentle_lower_valley(grid_u: i32, grid_v: i32, base: f32, heading: CornerHeading) -> MaterialBlock {
        Self::dark(FormBlock::gentle_lower_valley(grid_u, grid_v, base, heading))
    }

    pub fn gentle_upper_valley(grid_u: i32, grid_v: i32, base: f32, heading: CornerHeading) -> MaterialBlock {
        Self::natural(FormBlock::gentle_upper_valley(grid_u, grid_v, base, heading))
    }

    pub fn dark_gentle_upper_valley(grid_u: i32, grid_v: i32, base: f32, heading: CornerHeading) -> MaterialBlock {
        Self::dark(FormBlock::gentle_upper_valley(grid_u, grid_v, base, heading))
    }

    pub fn stair_corner(grid_u: i32, grid_v: i32, base: f32, heading: CornerHeading, concave: bool) -> MaterialBlock {
        Self::natural(FormBlock::stair_corner(grid_u, grid_v, base, heading, concave))
    }

    pub fn dark_stair_corner(grid_u: i32, grid_v: i32, base: f32, heading: CornerHeading, concave: bool) -> MaterialBlock {
        Self::dark(FormBlock::stair_corner(grid_u, grid_v, base, heading, concave))
    }

    pub fn stair_corner_convex(grid_u: i32, grid_v: i32, base: f32, heading: CornerHeading) -> MaterialBlock {
        Self::natural(FormBlock::stair_corner_convex(grid_u, grid_v, base, heading))
    }

    pub fn dark_stair_corner_convex(grid_u: i32, grid_v: i32, base: f32, heading: CornerHeading) -> MaterialBlock {
        Self::dark(FormBlock::stair_corner_convex(grid_u, grid_v, base, heading))
    }

    pub fn stair_corner_concave(grid_u: i32, grid_v: i32, base: f32, heading: CornerHeading) -> MaterialBlock {
        Self::natural(FormBlock::stair_corner_concave(grid_u, grid_v, base, heading))
    }

    pub fn dark_stair_corner_concave(grid_u: i32, grid_v: i32, base: f32, heading: CornerHeading) -> MaterialBlock {
        Self::dark(FormBlock::stair_corner_concave(grid_u, grid_v, base, heading))
    }
}
