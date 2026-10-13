use crate::block::{Block, CornerHeading, DirtBlock, StoneBlock, WedgeHeading};
use crate::cluster::VoxelCluster;
use crate::frame::SurfaceFrame;
use crate::topology::ClimateZone;
use crate::{Face, MeshTarget};
use glam::Vec3;

/// Steppe terrace landscape featuring tiered mesa plateaus and an incised coulee canyon valley.
pub struct SteppeTerrace {
    cluster: VoxelCluster,
}

impl SteppeTerrace {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(cluster: VoxelCluster) -> Self {
        Self { cluster }
    }

    /// Secondary constructor assembling steppe terrace centered at local (center_u, center_v).
    pub fn at(face: Box<dyn Face>, frame: SurfaceFrame, center_u: i32, center_v: i32) -> Self {
        let mut blocks: Vec<Box<dyn Block>> = Vec::new();

        // Tiered Mesa Plateau (tier 0 slabs and tier 1 crest)
        blocks.push(Box::new(DirtBlock::slab(center_u + 1, center_v + 1, 0.0)));
        blocks.push(Box::new(DirtBlock::slab(center_u + 2, center_v + 1, 0.0)));
        blocks.push(Box::new(StoneBlock::slab(center_u + 1, center_v + 2, 0.0)));
        blocks.push(Box::new(StoneBlock::dark_slab(center_u + 2, center_v + 2, 0.5)));
        blocks.push(Box::new(DirtBlock::wedge(center_u + 1, center_v + 3, 0.0, WedgeHeading::South)));
        blocks.push(Box::new(DirtBlock::ridge_corner(center_u + 2, center_v + 3, 0.0, CornerHeading::SouthWest)));

        // Sunken Coulee Canyon Valley (flanked by valley corners)
        blocks.push(Box::new(DirtBlock::valley_corner(center_u - 1, center_v, 0.0, CornerHeading::NorthWest)));
        blocks.push(Box::new(DirtBlock::valley_corner(center_u, center_v - 1, 0.0, CornerHeading::SouthEast)));
        blocks.push(Box::new(DirtBlock::slab(center_u - 1, center_v - 1, 0.0)));
        blocks.push(Box::new(DirtBlock::slab(center_u, center_v, 0.0)));

        // Valley Ramp and Ridge Corners
        blocks.push(Box::new(StoneBlock::wedge(center_u - 2, center_v - 1, 0.0, WedgeHeading::East)));
        blocks.push(Box::new(StoneBlock::ridge_corner(center_u - 2, center_v - 2, 0.0, CornerHeading::NorthEast)));

        Self::new(VoxelCluster::new(face, frame, blocks))
    }

    /// Secondary constructor assembling steppe terrace onto a canonical climate zone face.
    pub fn in_zone(face: Box<dyn Face>, zone: ClimateZone, half_size: f32) -> Self {
        let frame = zone.surface_frame(half_size);
        Self::at(face, frame, 3, 3)
    }
}

impl Face for SteppeTerrace {
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
