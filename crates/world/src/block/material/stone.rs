use glam::Vec3;
use crate::block::{Block, CornerHeading, FormBlock, WedgeHeading};
use crate::frame::SurfaceFrame;
use crate::MeshTarget;
use super::material_block::MaterialBlock;

/// Material entity decorating form blocks with durable alpine stone granite or shale color.
pub struct StoneBlock {
    decorated: MaterialBlock,
}

impl StoneBlock {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(decorated: MaterialBlock) -> Self {
        Self { decorated }
    }

    /// Secondary constructor with standard alpine gray granite stone.
    pub fn natural(form: FormBlock) -> Self {
        Self::new(MaterialBlock::new(Box::new(form), Vec3::new(0.48, 0.50, 0.46)))
    }

    /// Secondary constructor for dark shale rock.
    pub fn dark(form: FormBlock) -> Self {
        Self::new(MaterialBlock::new(Box::new(form), Vec3::new(0.35, 0.42, 0.35)))
    }

    pub fn cube(grid_u: i32, grid_v: i32, base: f32) -> Self {
        Self::natural(FormBlock::cube(grid_u, grid_v, base))
    }

    pub fn dark_cube(grid_u: i32, grid_v: i32, base: f32) -> Self {
        Self::dark(FormBlock::cube(grid_u, grid_v, base))
    }

    pub fn slab(grid_u: i32, grid_v: i32, base: f32) -> Self {
        Self::natural(FormBlock::slab(grid_u, grid_v, base))
    }

    pub fn dark_slab(grid_u: i32, grid_v: i32, base: f32) -> Self {
        Self::dark(FormBlock::slab(grid_u, grid_v, base))
    }

    pub fn wedge(grid_u: i32, grid_v: i32, base: f32, heading: WedgeHeading) -> Self {
        Self::natural(FormBlock::wedge(grid_u, grid_v, base, heading))
    }

    pub fn dark_wedge(grid_u: i32, grid_v: i32, base: f32, heading: WedgeHeading) -> Self {
        Self::dark(FormBlock::wedge(grid_u, grid_v, base, heading))
    }

    pub fn ridge_corner(grid_u: i32, grid_v: i32, base: f32, heading: CornerHeading) -> Self {
        Self::natural(FormBlock::ridge_corner(grid_u, grid_v, base, heading))
    }

    pub fn dark_ridge_corner(grid_u: i32, grid_v: i32, base: f32, heading: CornerHeading) -> Self {
        Self::dark(FormBlock::ridge_corner(grid_u, grid_v, base, heading))
    }

    pub fn valley_corner(grid_u: i32, grid_v: i32, base: f32, heading: CornerHeading) -> Self {
        Self::natural(FormBlock::valley_corner(grid_u, grid_v, base, heading))
    }

    pub fn dark_valley_corner(grid_u: i32, grid_v: i32, base: f32, heading: CornerHeading) -> Self {
        Self::dark(FormBlock::valley_corner(grid_u, grid_v, base, heading))
    }

    pub fn peak(grid_u: i32, grid_v: i32, base: f32) -> Self {
        Self::natural(FormBlock::peak(grid_u, grid_v, base))
    }

    pub fn dark_peak(grid_u: i32, grid_v: i32, base: f32) -> Self {
        Self::dark(FormBlock::peak(grid_u, grid_v, base))
    }

    pub fn stair(grid_u: i32, grid_v: i32, base: f32, heading: WedgeHeading) -> Self {
        Self::natural(FormBlock::stair(grid_u, grid_v, base, heading))
    }

    pub fn dark_stair(grid_u: i32, grid_v: i32, base: f32, heading: WedgeHeading) -> Self {
        Self::dark(FormBlock::stair(grid_u, grid_v, base, heading))
    }

    pub fn gentle_lower_wedge(grid_u: i32, grid_v: i32, base: f32, heading: WedgeHeading) -> Self {
        Self::natural(FormBlock::gentle_lower_wedge(grid_u, grid_v, base, heading))
    }

    pub fn dark_gentle_lower_wedge(grid_u: i32, grid_v: i32, base: f32, heading: WedgeHeading) -> Self {
        Self::dark(FormBlock::gentle_lower_wedge(grid_u, grid_v, base, heading))
    }

    pub fn gentle_upper_wedge(grid_u: i32, grid_v: i32, base: f32, heading: WedgeHeading) -> Self {
        Self::natural(FormBlock::gentle_upper_wedge(grid_u, grid_v, base, heading))
    }

    pub fn dark_gentle_upper_wedge(grid_u: i32, grid_v: i32, base: f32, heading: WedgeHeading) -> Self {
        Self::dark(FormBlock::gentle_upper_wedge(grid_u, grid_v, base, heading))
    }

    pub fn gentle_lower_ridge(grid_u: i32, grid_v: i32, base: f32, heading: CornerHeading) -> Self {
        Self::natural(FormBlock::gentle_lower_ridge(grid_u, grid_v, base, heading))
    }

    pub fn dark_gentle_lower_ridge(grid_u: i32, grid_v: i32, base: f32, heading: CornerHeading) -> Self {
        Self::dark(FormBlock::gentle_lower_ridge(grid_u, grid_v, base, heading))
    }

    pub fn gentle_upper_ridge(grid_u: i32, grid_v: i32, base: f32, heading: CornerHeading) -> Self {
        Self::natural(FormBlock::gentle_upper_ridge(grid_u, grid_v, base, heading))
    }

    pub fn dark_gentle_upper_ridge(grid_u: i32, grid_v: i32, base: f32, heading: CornerHeading) -> Self {
        Self::dark(FormBlock::gentle_upper_ridge(grid_u, grid_v, base, heading))
    }

    pub fn gentle_lower_valley(grid_u: i32, grid_v: i32, base: f32, heading: CornerHeading) -> Self {
        Self::natural(FormBlock::gentle_lower_valley(grid_u, grid_v, base, heading))
    }

    pub fn dark_gentle_lower_valley(grid_u: i32, grid_v: i32, base: f32, heading: CornerHeading) -> Self {
        Self::dark(FormBlock::gentle_lower_valley(grid_u, grid_v, base, heading))
    }

    pub fn gentle_upper_valley(grid_u: i32, grid_v: i32, base: f32, heading: CornerHeading) -> Self {
        Self::natural(FormBlock::gentle_upper_valley(grid_u, grid_v, base, heading))
    }

    pub fn dark_gentle_upper_valley(grid_u: i32, grid_v: i32, base: f32, heading: CornerHeading) -> Self {
        Self::dark(FormBlock::gentle_upper_valley(grid_u, grid_v, base, heading))
    }

    pub fn stair_corner(grid_u: i32, grid_v: i32, base: f32, heading: CornerHeading, concave: bool) -> Self {
        Self::natural(FormBlock::stair_corner(grid_u, grid_v, base, heading, concave))
    }

    pub fn dark_stair_corner(grid_u: i32, grid_v: i32, base: f32, heading: CornerHeading, concave: bool) -> Self {
        Self::dark(FormBlock::stair_corner(grid_u, grid_v, base, heading, concave))
    }

    pub fn stair_corner_convex(grid_u: i32, grid_v: i32, base: f32, heading: CornerHeading) -> Self {
        Self::natural(FormBlock::stair_corner_convex(grid_u, grid_v, base, heading))
    }

    pub fn dark_stair_corner_convex(grid_u: i32, grid_v: i32, base: f32, heading: CornerHeading) -> Self {
        Self::dark(FormBlock::stair_corner_convex(grid_u, grid_v, base, heading))
    }

    pub fn stair_corner_concave(grid_u: i32, grid_v: i32, base: f32, heading: CornerHeading) -> Self {
        Self::natural(FormBlock::stair_corner_concave(grid_u, grid_v, base, heading))
    }

    pub fn dark_stair_corner_concave(grid_u: i32, grid_v: i32, base: f32, heading: CornerHeading) -> Self {
        Self::dark(FormBlock::stair_corner_concave(grid_u, grid_v, base, heading))
    }
}

impl Block for StoneBlock {
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
