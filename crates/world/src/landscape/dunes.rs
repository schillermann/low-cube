use crate::block::{Block, CornerHeading, SandBlock, WedgeHeading};
use crate::cluster::VoxelCluster;
use crate::frame::SurfaceFrame;
use crate::topology::ClimateZone;
use crate::Face;

/// Desert dunes landscape factory assembling barchan sand ridges, apex peaks, and an incised wadi canyon valley.
pub struct DesertDunes;

impl DesertDunes {
    /// Factory method assembling desert dunes centered at local (center_u, center_v).
    pub fn at(face: Box<dyn Face>, frame: SurfaceFrame, center_u: i32, center_v: i32) -> VoxelCluster {
        let mut blocks: Vec<Box<dyn Block>> = Vec::new();

        // Northern Barchan Dune Ridge (slopes up to tier 1)
        for u in -1..=1 {
            blocks.push(Box::new(SandBlock::wedge(center_u + u, center_v + 2, 0.0, WedgeHeading::South)));
        }
        blocks.push(Box::new(SandBlock::ridge_corner(center_u + 2, center_v + 2, 0.0, CornerHeading::SouthWest)));
        blocks.push(Box::new(SandBlock::ridge_corner(center_u - 2, center_v + 2, 0.0, CornerHeading::SouthEast)));

        // Golden Dune Summit (tier 1 elevation 1.0)
        blocks.push(Box::new(SandBlock::peak(center_u, center_v + 1, 1.0)));
        blocks.push(Box::new(SandBlock::slab(center_u - 1, center_v + 1, 1.0)));
        blocks.push(Box::new(SandBlock::slab(center_u + 1, center_v + 1, 1.0)));

        // Incised Desert Wadi Valley (flanked by valley corners)
        blocks.push(Box::new(SandBlock::valley_corner(center_u + 1, center_v, 0.0, CornerHeading::SouthWest)));
        blocks.push(Box::new(SandBlock::valley_corner(center_u - 1, center_v, 0.0, CornerHeading::SouthEast)));
        blocks.push(Box::new(SandBlock::slab(center_u, center_v, 0.0)));
        blocks.push(Box::new(SandBlock::slab(center_u, center_v - 1, 0.0)));

        // Southern Dune Ridge
        for u in -1..=1 {
            blocks.push(Box::new(SandBlock::wedge(center_u + u, center_v - 2, 0.0, WedgeHeading::North)));
        }
        blocks.push(Box::new(SandBlock::ridge_corner(center_u + 2, center_v - 2, 0.0, CornerHeading::NorthWest)));
        blocks.push(Box::new(SandBlock::ridge_corner(center_u - 2, center_v - 2, 0.0, CornerHeading::NorthEast)));

        VoxelCluster::new(face, frame, blocks)
    }

    /// Factory method assembling desert dunes onto a canonical climate zone face.
    pub fn in_zone(face: Box<dyn Face>, zone: ClimateZone, half_size: f32) -> VoxelCluster {
        let frame = zone.surface_frame(half_size);
        Self::at(face, frame, 3, 3)
    }
}
