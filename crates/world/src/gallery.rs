use glam::Vec3;
use crate::Face;
use crate::block::{
    Block, CornerHeading, DirtBlock, SandBlock, SnowBlock, StoneBlock, WedgeHeading,
};
use crate::cluster::VoxelCluster;
use crate::frame::SurfaceFrame;

/// Exhibition gallery factory showcasing all available materials in parallel lines across all block shapes.
pub struct BlockGallery;

impl BlockGallery {
    /// Factory method assembling all materials in parallel lines with 1 block spacing.
    pub fn parallel_lines(face: Box<dyn Face>, half_size: f32) -> VoxelCluster {
        let frame = SurfaceFrame::new(Vec3::X, Vec3::Z, Vec3::Y, half_size);
        let blocks: Vec<Box<dyn Block>> = vec![
            // Line 1: Stone (u = -3)
            Box::new(StoneBlock::cube(-3, -8, 0.0)),
            Box::new(StoneBlock::slab(-3, -7, 0.0)),
            Box::new(StoneBlock::stair(-3, -6, 0.0, WedgeHeading::North)),
            Box::new(StoneBlock::wedge(-3, -5, 0.0, WedgeHeading::North)),
            Box::new(StoneBlock::ridge_corner(-3, -4, 0.0, CornerHeading::NorthEast)),
            Box::new(StoneBlock::valley_corner(-3, -3, 0.0, CornerHeading::NorthEast)),
            Box::new(StoneBlock::peak(-3, -2, 0.0)),
            // Line 2: Dirt (u = -1)
            Box::new(DirtBlock::cube(-1, -8, 0.0)),
            Box::new(DirtBlock::slab(-1, -7, 0.0)),
            Box::new(DirtBlock::stair(-1, -6, 0.0, WedgeHeading::North)),
            Box::new(DirtBlock::wedge(-1, -5, 0.0, WedgeHeading::North)),
            Box::new(DirtBlock::ridge_corner(-1, -4, 0.0, CornerHeading::NorthEast)),
            Box::new(DirtBlock::valley_corner(-1, -3, 0.0, CornerHeading::NorthEast)),
            Box::new(DirtBlock::peak(-1, -2, 0.0)),
            // Line 3: Sand (u = 1)
            Box::new(SandBlock::cube(1, -8, 0.0)),
            Box::new(SandBlock::slab(1, -7, 0.0)),
            Box::new(SandBlock::stair(1, -6, 0.0, WedgeHeading::North)),
            Box::new(SandBlock::wedge(1, -5, 0.0, WedgeHeading::North)),
            Box::new(SandBlock::ridge_corner(1, -4, 0.0, CornerHeading::NorthEast)),
            Box::new(SandBlock::valley_corner(1, -3, 0.0, CornerHeading::NorthEast)),
            Box::new(SandBlock::peak(1, -2, 0.0)),
            // Line 4: Snow (u = 3)
            Box::new(SnowBlock::cube(3, -8, 0.0)),
            Box::new(SnowBlock::slab(3, -7, 0.0)),
            Box::new(SnowBlock::stair(3, -6, 0.0, WedgeHeading::North)),
            Box::new(SnowBlock::wedge(3, -5, 0.0, WedgeHeading::North)),
            Box::new(SnowBlock::ridge_corner(3, -4, 0.0, CornerHeading::NorthEast)),
            Box::new(SnowBlock::valley_corner(3, -3, 0.0, CornerHeading::NorthEast)),
            Box::new(SnowBlock::peak(3, -2, 0.0)),
        ];

        VoxelCluster::new(face, frame, blocks)
    }
}
