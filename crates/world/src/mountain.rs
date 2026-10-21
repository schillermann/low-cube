use glam::Vec3;
use crate::Face;
use crate::block::{Block, CornerHeading, SnowBlock, StoneBlock, WedgeHeading};
use crate::cluster::VoxelCluster;
use crate::frame::SurfaceFrame;

/// Faceted alpine mountain massif factory assembling alpine peaks onto a planetary face.
pub struct VoxelMountain;

impl VoxelMountain {
    /// Factory method assembling an alpine mountain massif centered at (center_u, center_v).
    pub fn at(face: Box<dyn Face>, half_size: f32, center_u: i32, center_v: i32) -> VoxelCluster {
        let frame = SurfaceFrame::new(Vec3::X, Vec3::Z, Vec3::Y, half_size);
        let mut blocks: Vec<Box<dyn Block>> = Vec::new();

        // Tier 0 (base = 0.0, height = 1.0, 5x5 footprint from -2 to +2):
        for u in -1..=1 {
            blocks.push(Box::new(StoneBlock::dark_wedge(center_u + u, center_v + 2, 0.0, WedgeHeading::South)));
            if u == 0 {
                // Rock slab step with smooth vertical edges at the beginning of the mountain
                blocks.push(Box::new(StoneBlock::dark_slab(center_u, center_v - 2, 0.0)));
                blocks.push(Box::new(StoneBlock::dark_slab(center_u, center_v - 2, 0.25)));
            } else {
                blocks.push(Box::new(StoneBlock::dark_wedge(center_u + u, center_v - 2, 0.0, WedgeHeading::North)));
            }
        }
        for v in -1..=1 {
            blocks.push(Box::new(StoneBlock::dark_wedge(center_u + 2, center_v + v, 0.0, WedgeHeading::West)));
            blocks.push(Box::new(StoneBlock::dark_wedge(center_u - 2, center_v + v, 0.0, WedgeHeading::East)));
        }

        // Tier 0 Outer Corners:
        blocks.push(Box::new(StoneBlock::dark_ridge_corner(center_u + 2, center_v + 2, 0.0, CornerHeading::SouthWest)));
        blocks.push(Box::new(StoneBlock::dark_ridge_corner(center_u - 2, center_v + 2, 0.0, CornerHeading::SouthEast)));
        blocks.push(Box::new(StoneBlock::dark_ridge_corner(center_u + 2, center_v - 2, 0.0, CornerHeading::NorthWest)));
        blocks.push(Box::new(StoneBlock::dark_ridge_corner(center_u - 2, center_v - 2, 0.0, CornerHeading::NorthEast)));

        // Tier 0 Interior Core (3x3 cubes at base 0.0):
        for u in -1..=1 {
            for v in -1..=1 {
                blocks.push(Box::new(StoneBlock::cube(center_u + u, center_v + v, 0.0)));
            }
        }

        // Tier 1 (base = 0.5, 3x3 footprint from -1 to +1):
        blocks.push(Box::new(StoneBlock::wedge(center_u, center_v + 1, 0.5, WedgeHeading::South)));
        blocks.push(Box::new(StoneBlock::wedge(center_u, center_v - 1, 0.5, WedgeHeading::North)));
        blocks.push(Box::new(StoneBlock::wedge(center_u + 1, center_v, 0.5, WedgeHeading::West)));
        blocks.push(Box::new(StoneBlock::wedge(center_u - 1, center_v, 0.5, WedgeHeading::East)));

        // Tier 1 Corners:
        blocks.push(Box::new(StoneBlock::ridge_corner(center_u + 1, center_v + 1, 0.5, CornerHeading::SouthWest)));
        blocks.push(Box::new(StoneBlock::ridge_corner(center_u - 1, center_v + 1, 0.5, CornerHeading::SouthEast)));
        blocks.push(Box::new(StoneBlock::ridge_corner(center_u + 1, center_v - 1, 0.5, CornerHeading::NorthWest)));
        blocks.push(Box::new(StoneBlock::ridge_corner(center_u - 1, center_v - 1, 0.5, CornerHeading::NorthEast)));

        // Tier 1 Interior Core:
        blocks.push(Box::new(StoneBlock::cube(center_u, center_v, 0.5)));

        // Tier 2 Summit Peak (base = 1.0, crystalline snow pyramid apex):
        blocks.push(Box::new(SnowBlock::peak(center_u, center_v, 1.0)));

        VoxelCluster::new(face, frame, blocks)
    }

    /// Factory method assembling the alpine mountain on top face at (0, 4).
    pub fn alpine(face: Box<dyn Face>, half_size: f32) -> VoxelCluster {
        Self::at(face, half_size, 0, 4)
    }
}
