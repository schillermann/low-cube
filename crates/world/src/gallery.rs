use glam::Vec3;
use crate::{Face, MeshTarget};
use crate::block::{
    Block, CornerHeading, DirtBlock, SandBlock, SnowBlock, StoneBlock, WedgeHeading,
};
use crate::cluster::VoxelCluster;
use crate::frame::SurfaceFrame;

/// Exhibition gallery showcasing all available materials in parallel lines across all block shapes.
pub struct BlockGallery {
    cluster: VoxelCluster,
}

impl BlockGallery {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(cluster: VoxelCluster) -> Self {
        Self { cluster }
    }

    /// Secondary constructor assembling all materials in parallel lines with 1 block spacing.
    pub fn parallel_lines(face: Box<dyn Face>, half_size: f32) -> Self {
        let frame = SurfaceFrame::new(Vec3::X, Vec3::Z, Vec3::Y, half_size);
        let mut blocks: Vec<Box<dyn Block>> = Vec::new();

        // 4 Parallel lines along V (-8..=-2) with 1 block spacing along U:
        // Stone (u = -3), Dirt (u = -1), Sand (u = 1), Snow (u = 3)
        // Forms in each line: Cube, Slab, Stair, Wedge, Ridge, Valley, Peak

        // Line 1: Stone (u = -3)
        blocks.push(Box::new(StoneBlock::cube(-3, -8, 0.0)));
        blocks.push(Box::new(StoneBlock::slab(-3, -7, 0.0)));
        blocks.push(Box::new(StoneBlock::stair(-3, -6, 0.0, WedgeHeading::North)));
        blocks.push(Box::new(StoneBlock::wedge(-3, -5, 0.0, WedgeHeading::North)));
        blocks.push(Box::new(StoneBlock::ridge_corner(-3, -4, 0.0, CornerHeading::NorthEast)));
        blocks.push(Box::new(StoneBlock::valley_corner(-3, -3, 0.0, CornerHeading::NorthEast)));
        blocks.push(Box::new(StoneBlock::peak(-3, -2, 0.0)));

        // Line 2: Dirt (u = -1)
        blocks.push(Box::new(DirtBlock::cube(-1, -8, 0.0)));
        blocks.push(Box::new(DirtBlock::slab(-1, -7, 0.0)));
        blocks.push(Box::new(DirtBlock::stair(-1, -6, 0.0, WedgeHeading::North)));
        blocks.push(Box::new(DirtBlock::wedge(-1, -5, 0.0, WedgeHeading::North)));
        blocks.push(Box::new(DirtBlock::ridge_corner(-1, -4, 0.0, CornerHeading::NorthEast)));
        blocks.push(Box::new(DirtBlock::valley_corner(-1, -3, 0.0, CornerHeading::NorthEast)));
        blocks.push(Box::new(DirtBlock::peak(-1, -2, 0.0)));

        // Central walkway at u = 0

        // Line 3: Sand (u = 1)
        blocks.push(Box::new(SandBlock::cube(1, -8, 0.0)));
        blocks.push(Box::new(SandBlock::slab(1, -7, 0.0)));
        blocks.push(Box::new(SandBlock::stair(1, -6, 0.0, WedgeHeading::North)));
        blocks.push(Box::new(SandBlock::wedge(1, -5, 0.0, WedgeHeading::North)));
        blocks.push(Box::new(SandBlock::ridge_corner(1, -4, 0.0, CornerHeading::NorthEast)));
        blocks.push(Box::new(SandBlock::valley_corner(1, -3, 0.0, CornerHeading::NorthEast)));
        blocks.push(Box::new(SandBlock::peak(1, -2, 0.0)));

        // Line 4: Snow (u = 3)
        blocks.push(Box::new(SnowBlock::cube(3, -8, 0.0)));
        blocks.push(Box::new(SnowBlock::slab(3, -7, 0.0)));
        blocks.push(Box::new(SnowBlock::stair(3, -6, 0.0, WedgeHeading::North)));
        blocks.push(Box::new(SnowBlock::wedge(3, -5, 0.0, WedgeHeading::North)));
        blocks.push(Box::new(SnowBlock::ridge_corner(3, -4, 0.0, CornerHeading::NorthEast)));
        blocks.push(Box::new(SnowBlock::valley_corner(3, -3, 0.0, CornerHeading::NorthEast)));
        blocks.push(Box::new(SnowBlock::peak(3, -2, 0.0)));

        Self::new(VoxelCluster::new(face, frame, blocks))
    }
}

impl Face for BlockGallery {
    fn matches(&self, point: Vec3) -> bool {
        self.cluster.matches(point)
    }

    fn surface_up(&self) -> Vec3 {
        self.cluster.surface_up()
    }

    fn pull_direction(&self) -> Vec3 {
        self.cluster.pull_direction()
    }

    fn constrain(&self, point: Vec3, distance: f32) -> Vec3 {
        self.cluster.constrain(point, distance)
    }

    fn emit_surface(&self, target: &mut dyn MeshTarget) {
        self.cluster.emit_surface(target);
    }
}
