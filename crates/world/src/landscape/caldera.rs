use glam::Vec3;
use crate::block::{
    Block, CornerHeading, FormBlock, MaterialBlock, StoneBlock, WedgeHeading,
};
use crate::cluster::VoxelCluster;
use crate::frame::SurfaceFrame;
use crate::topology::ClimateZone;
use crate::Face;

/// Volcanic caldera landscape factory assembling an elevated crater rim and sunken magma basin valley.
pub struct VolcanicCaldera;

impl VolcanicCaldera {
    /// Factory method assembling a volcanic caldera centered at local (center_u, center_v).
    pub fn at(face: Box<dyn Face>, frame: SurfaceFrame, center_u: i32, center_v: i32) -> VoxelCluster {
        let mut blocks: Vec<Box<dyn Block>> = Vec::new();

        // Tier 0 Outer Rim slopes (base = 0.0, height = 1.0, 5x5 footprint)
        for u in -1..=1 {
            blocks.push(Box::new(StoneBlock::dark_wedge(center_u + u, center_v + 2, 0.0, WedgeHeading::South)));
            blocks.push(Box::new(StoneBlock::dark_wedge(center_u + u, center_v - 2, 0.0, WedgeHeading::North)));
        }
        for v in -1..=1 {
            blocks.push(Box::new(StoneBlock::dark_wedge(center_u + 2, center_v + v, 0.0, WedgeHeading::West)));
            blocks.push(Box::new(StoneBlock::dark_wedge(center_u - 2, center_v + v, 0.0, WedgeHeading::East)));
        }

        // Tier 0 Outer Ridge Corners
        blocks.push(Box::new(StoneBlock::dark_ridge_corner(center_u + 2, center_v + 2, 0.0, CornerHeading::SouthWest)));
        blocks.push(Box::new(StoneBlock::dark_ridge_corner(center_u - 2, center_v + 2, 0.0, CornerHeading::SouthEast)));
        blocks.push(Box::new(StoneBlock::dark_ridge_corner(center_u + 2, center_v - 2, 0.0, CornerHeading::NorthWest)));
        blocks.push(Box::new(StoneBlock::dark_ridge_corner(center_u - 2, center_v - 2, 0.0, CornerHeading::NorthEast)));

        // Tier 1 Crater Rim Crest (elevation 0.5)
        blocks.push(Box::new(StoneBlock::dark_slab(center_u, center_v + 1, 0.5)));
        blocks.push(Box::new(StoneBlock::dark_slab(center_u, center_v - 1, 0.5)));
        blocks.push(Box::new(StoneBlock::dark_slab(center_u + 1, center_v, 0.5)));
        blocks.push(Box::new(StoneBlock::dark_slab(center_u - 1, center_v, 0.5)));

        // Inner Caldera Valley Corners descending to the magma basin
        blocks.push(Box::new(StoneBlock::dark_valley_corner(center_u + 1, center_v + 1, 0.0, CornerHeading::SouthWest)));
        blocks.push(Box::new(StoneBlock::dark_valley_corner(center_u - 1, center_v + 1, 0.0, CornerHeading::SouthEast)));
        blocks.push(Box::new(StoneBlock::dark_valley_corner(center_u + 1, center_v - 1, 0.0, CornerHeading::NorthWest)));
        blocks.push(Box::new(StoneBlock::dark_valley_corner(center_u - 1, center_v - 1, 0.0, CornerHeading::NorthEast)));

        // Sunken Magma Basin Floor (incandescent lava rock slab)
        let lava_color = Vec3::new(0.95, 0.28, 0.05);
        let lava_form = Box::new(FormBlock::slab(center_u, center_v, 0.0));
        blocks.push(Box::new(MaterialBlock::new(lava_form, lava_color)));

        VoxelCluster::new(face, frame, blocks)
    }

    /// Factory method assembling volcanic caldera onto a canonical climate zone face.
    pub fn in_zone(face: Box<dyn Face>, zone: ClimateZone, half_size: f32) -> VoxelCluster {
        let frame = zone.surface_frame(half_size);
        Self::at(face, frame, 3, 3)
    }
}
