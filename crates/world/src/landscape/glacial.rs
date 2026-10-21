use crate::block::{Block, CornerHeading, SnowBlock, StoneBlock, WedgeHeading};
use crate::cluster::VoxelCluster;
use crate::frame::SurfaceFrame;
use crate::topology::ClimateZone;
use crate::Face;

/// Arctic glacial massif landscape factory assembling crystalline ice horn peaks and an incised cirque valley.
pub struct GlacialMassif;

impl GlacialMassif {
    /// Factory method assembling a glacial massif centered at local (center_u, center_v).
    pub fn at(face: Box<dyn Face>, frame: SurfaceFrame, center_u: i32, center_v: i32) -> VoxelCluster {
        let blocks: Vec<Box<dyn Block>> = vec![
            // Western Glacial Horn (granite base with crystalline snow peak)
            Box::new(StoneBlock::cube(center_u - 1, center_v + 1, 0.0)),
            Box::new(SnowBlock::peak(center_u - 1, center_v + 1, 0.5)),
            Box::new(SnowBlock::ridge_corner(center_u - 2, center_v + 1, 0.0, CornerHeading::NorthEast)),
            Box::new(SnowBlock::wedge(center_u - 1, center_v + 2, 0.0, WedgeHeading::South)),
            // Eastern Glacial Horn
            Box::new(StoneBlock::cube(center_u + 1, center_v + 1, 0.0)),
            Box::new(SnowBlock::peak(center_u + 1, center_v + 1, 0.5)),
            Box::new(SnowBlock::ridge_corner(center_u + 2, center_v + 1, 0.0, CornerHeading::NorthWest)),
            Box::new(SnowBlock::wedge(center_u + 1, center_v + 2, 0.0, WedgeHeading::South)),
            // Glacial Cirque Valley (sunken icy amphitheater trough)
            Box::new(SnowBlock::valley_corner(center_u, center_v + 1, 0.0, CornerHeading::NorthEast)),
            Box::new(SnowBlock::valley_corner(center_u, center_v - 1, 0.0, CornerHeading::SouthWest)),
            Box::new(SnowBlock::slab(center_u, center_v, 0.0)),
            Box::new(SnowBlock::slab(center_u - 1, center_v, 0.0)),
            Box::new(SnowBlock::slab(center_u + 1, center_v, 0.0)),
            // Terminal Moraine Approach Slopes
            Box::new(SnowBlock::wedge(center_u - 1, center_v - 2, 0.0, WedgeHeading::North)),
            Box::new(SnowBlock::wedge(center_u, center_v - 2, 0.0, WedgeHeading::North)),
            Box::new(SnowBlock::wedge(center_u + 1, center_v - 2, 0.0, WedgeHeading::North)),
        ];

        VoxelCluster::new(face, frame, blocks)
    }

    /// Factory method assembling glacial massif onto a canonical climate zone face.
    pub fn in_zone(face: Box<dyn Face>, zone: ClimateZone, half_size: f32) -> VoxelCluster {
        let frame = zone.surface_frame(half_size);
        Self::at(face, frame, 3, 3)
    }
}
