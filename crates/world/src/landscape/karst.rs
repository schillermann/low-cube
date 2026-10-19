use crate::block::{Block, CornerHeading, DirtBlock, StoneBlock, WedgeHeading};
use crate::cluster::VoxelCluster;
use crate::frame::SurfaceFrame;
use crate::topology::ClimateZone;
use crate::Face;

/// Jungle karst landscape factory assembling tower karst limestone pillars and a winding river gorge valley.
pub struct JungleKarst;

impl JungleKarst {
    /// Factory method assembling jungle karst centered at local (center_u, center_v).
    pub fn at(face: Box<dyn Face>, frame: SurfaceFrame, center_u: i32, center_v: i32) -> VoxelCluster {
        let blocks: Vec<Box<dyn Block>> = vec![
            // Northwestern Karst Tower (granite base, loam summit, ascending stairs)
            Box::new(StoneBlock::cube(center_u - 1, center_v + 1, 0.0)),
            Box::new(DirtBlock::cube(center_u - 1, center_v + 1, 1.0)),
            Box::new(DirtBlock::stair(center_u - 1, center_v + 2, 0.0, WedgeHeading::South)),
            Box::new(DirtBlock::ridge_corner(center_u - 2, center_v + 1, 0.0, CornerHeading::NorthEast)),
            // Southeastern Karst Tower
            Box::new(StoneBlock::cube(center_u + 1, center_v - 1, 0.0)),
            Box::new(DirtBlock::cube(center_u + 1, center_v - 1, 1.0)),
            Box::new(DirtBlock::stair(center_u + 1, center_v - 2, 0.0, WedgeHeading::North)),
            Box::new(DirtBlock::ridge_corner(center_u + 2, center_v - 1, 0.0, CornerHeading::SouthWest)),
            // Karst Gorge Valley (incised tropical canyon flanked by valley corners)
            Box::new(DirtBlock::valley_corner(center_u, center_v + 1, 0.0, CornerHeading::SouthWest)),
            Box::new(DirtBlock::valley_corner(center_u, center_v - 1, 0.0, CornerHeading::NorthEast)),
            Box::new(DirtBlock::slab(center_u, center_v, 0.0)),
            Box::new(DirtBlock::slab(center_u + 1, center_v, 0.0)),
            Box::new(DirtBlock::slab(center_u - 1, center_v, 0.0)),
            // Approach Slopes
            Box::new(DirtBlock::wedge(center_u, center_v + 2, 0.0, WedgeHeading::South)),
            Box::new(DirtBlock::wedge(center_u, center_v - 2, 0.0, WedgeHeading::North)),
        ];

        VoxelCluster::new(face, frame, blocks)
    }

    /// Factory method assembling jungle karst onto a canonical climate zone face.
    pub fn in_zone(face: Box<dyn Face>, zone: ClimateZone, half_size: f32) -> VoxelCluster {
        let frame = zone.surface_frame(half_size);
        Self::at(face, frame, 3, 3)
    }
}
