use crate::block::{Block, CornerHeading, SnowBlock, StoneBlock, WedgeHeading};
use crate::cluster::VoxelCluster;
use crate::frame::SurfaceFrame;
use crate::topology::ClimateZone;
use crate::{Face, MeshTarget};
use glam::Vec3;

/// Arctic glacial massif landscape featuring crystalline ice horn peaks and an incised cirque valley.
pub struct GlacialMassif {
    cluster: VoxelCluster,
}

impl GlacialMassif {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(cluster: VoxelCluster) -> Self {
        Self { cluster }
    }

    /// Secondary constructor assembling a glacial massif centered at local (center_u, center_v).
    pub fn at(face: Box<dyn Face>, frame: SurfaceFrame, center_u: i32, center_v: i32) -> Self {
        let mut blocks: Vec<Box<dyn Block>> = Vec::new();

        // Western Glacial Horn (granite base with crystalline snow peak)
        blocks.push(Box::new(StoneBlock::cube(center_u - 1, center_v + 1, 0.0)));
        blocks.push(Box::new(SnowBlock::peak(center_u - 1, center_v + 1, 1.0)));
        blocks.push(Box::new(SnowBlock::ridge_corner(center_u - 2, center_v + 1, 0.0, CornerHeading::NorthEast)));
        blocks.push(Box::new(SnowBlock::wedge(center_u - 1, center_v + 2, 0.0, WedgeHeading::South)));

        // Eastern Glacial Horn
        blocks.push(Box::new(StoneBlock::cube(center_u + 1, center_v + 1, 0.0)));
        blocks.push(Box::new(SnowBlock::peak(center_u + 1, center_v + 1, 1.0)));
        blocks.push(Box::new(SnowBlock::ridge_corner(center_u + 2, center_v + 1, 0.0, CornerHeading::NorthWest)));
        blocks.push(Box::new(SnowBlock::wedge(center_u + 1, center_v + 2, 0.0, WedgeHeading::South)));

        // Glacial Cirque Valley (sunken icy amphitheater trough)
        blocks.push(Box::new(SnowBlock::valley_corner(center_u, center_v + 1, 0.0, CornerHeading::NorthEast)));
        blocks.push(Box::new(SnowBlock::valley_corner(center_u, center_v - 1, 0.0, CornerHeading::SouthWest)));
        blocks.push(Box::new(SnowBlock::slab(center_u, center_v, 0.0)));
        blocks.push(Box::new(SnowBlock::slab(center_u - 1, center_v, 0.0)));
        blocks.push(Box::new(SnowBlock::slab(center_u + 1, center_v, 0.0)));

        // Terminal Moraine Approach Slopes
        blocks.push(Box::new(SnowBlock::wedge(center_u - 1, center_v - 2, 0.0, WedgeHeading::North)));
        blocks.push(Box::new(SnowBlock::wedge(center_u, center_v - 2, 0.0, WedgeHeading::North)));
        blocks.push(Box::new(SnowBlock::wedge(center_u + 1, center_v - 2, 0.0, WedgeHeading::North)));

        Self::new(VoxelCluster::new(face, frame, blocks))
    }

    /// Secondary constructor assembling glacial massif onto a canonical climate zone face.
    pub fn in_zone(face: Box<dyn Face>, zone: ClimateZone, half_size: f32) -> Self {
        let frame = zone.surface_frame(half_size);
        Self::at(face, frame, 3, 3)
    }
}

impl Face for GlacialMassif {
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
