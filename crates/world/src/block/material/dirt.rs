use glam::Vec3;
use crate::MeshTarget;
use crate::frame::SurfaceFrame;
use crate::block::{Block, CornerHeading, FormBlock, WedgeHeading};

/// Material block composed of fertile earth and soil, derived from a FormBlock.
pub struct DirtBlock {
    form: FormBlock,
    color: Vec3,
}

impl DirtBlock {
    /// Primary constructor decorating a FormBlock with dirt material.
    pub fn new(form: FormBlock, color: Vec3) -> Self {
        Self { form, color }
    }

    /// Secondary constructor with standard brown soil color.
    pub fn natural(form: FormBlock) -> Self {
        Self::new(form, Vec3::new(0.48, 0.35, 0.22))
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

    pub fn peak(grid_u: i32, grid_v: i32, base: f32) -> Self {
        Self::natural(FormBlock::peak(grid_u, grid_v, base))
    }

    pub fn stair(grid_u: i32, grid_v: i32, base: f32, heading: WedgeHeading) -> Self {
        Self::natural(FormBlock::stair(grid_u, grid_v, base, heading))
    }
}

impl Block for DirtBlock {
    fn constrain_surface(&self, base: Vec3, u: f32, v: f32, frame: &SurfaceFrame, distance: f32) -> Vec3 {
        self.form.constrain_surface(base, u, v, frame, distance)
    }

    fn emit_surface(&self, frame: &SurfaceFrame, target: &mut dyn MeshTarget) {
        self.form.emit_form(frame, self.color, target);
    }
}
