use crate::block::{Block, CornerHeading, DirtBlock, StoneBlock, WedgeHeading};
use crate::cluster::VoxelCluster;
use crate::frame::SurfaceFrame;
use crate::topology::ClimateZone;
use crate::Face;

/// Steppe terrace landscape factory assembling tiered mesa plateaus and an incised coulee canyon valley.
pub struct SteppeTerrace;

impl SteppeTerrace {
    /// Factory method assembling steppe terrace centered at local (center_u, center_v).
    pub fn at(face: Box<dyn Face>, frame: SurfaceFrame, center_u: i32, center_v: i32) -> VoxelCluster {
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

        VoxelCluster::new(face, frame, blocks)
    }

    /// Factory method assembling steppe terrace onto a canonical climate zone face.
    pub fn in_zone(face: Box<dyn Face>, zone: ClimateZone, half_size: f32) -> VoxelCluster {
        let frame = zone.surface_frame(half_size);
        Self::at(face, frame, 3, 3)
    }
}
