use crate::block::{Block, CornerHeading, DirtBlock, StoneBlock, WedgeHeading};
use crate::cluster::VoxelCluster;
use crate::frame::SurfaceFrame;
use crate::topology::ClimateZone;
use crate::{Face, MeshTarget};
use glam::Vec3;

/// Jungle karst landscape featuring tower karst limestone pillars and a winding river gorge valley.
pub struct JungleKarst {
    cluster: VoxelCluster,
}

impl JungleKarst {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(cluster: VoxelCluster) -> Self {
        Self { cluster }
    }

    /// Secondary constructor assembling jungle karst centered at local (center_u, center_v).
    pub fn at(face: Box<dyn Face>, frame: SurfaceFrame, center_u: i32, center_v: i32) -> Self {
        let mut blocks: Vec<Box<dyn Block>> = Vec::new();

        // Northwestern Karst Tower (granite base, loam summit, ascending stairs)
        blocks.push(Box::new(StoneBlock::cube(center_u - 1, center_v + 1, 0.0)));
        blocks.push(Box::new(DirtBlock::cube(center_u - 1, center_v + 1, 1.0)));
        blocks.push(Box::new(DirtBlock::stair(center_u - 1, center_v + 2, 0.0, WedgeHeading::South)));
        blocks.push(Box::new(DirtBlock::ridge_corner(center_u - 2, center_v + 1, 0.0, CornerHeading::NorthEast)));

        // Southeastern Karst Tower
        blocks.push(Box::new(StoneBlock::cube(center_u + 1, center_v - 1, 0.0)));
        blocks.push(Box::new(DirtBlock::cube(center_u + 1, center_v - 1, 1.0)));
        blocks.push(Box::new(DirtBlock::stair(center_u + 1, center_v - 2, 0.0, WedgeHeading::North)));
        blocks.push(Box::new(DirtBlock::ridge_corner(center_u + 2, center_v - 1, 0.0, CornerHeading::SouthWest)));

        // Karst Gorge Valley (incised tropical canyon flanked by valley corners)
        blocks.push(Box::new(DirtBlock::valley_corner(center_u, center_v + 1, 0.0, CornerHeading::SouthWest)));
        blocks.push(Box::new(DirtBlock::valley_corner(center_u, center_v - 1, 0.0, CornerHeading::NorthEast)));
        blocks.push(Box::new(DirtBlock::slab(center_u, center_v, 0.0)));
        blocks.push(Box::new(DirtBlock::slab(center_u + 1, center_v, 0.0)));
        blocks.push(Box::new(DirtBlock::slab(center_u - 1, center_v, 0.0)));

        // Approach Slopes
        blocks.push(Box::new(DirtBlock::wedge(center_u, center_v + 2, 0.0, WedgeHeading::South)));
        blocks.push(Box::new(DirtBlock::wedge(center_u, center_v - 2, 0.0, WedgeHeading::North)));

        Self::new(VoxelCluster::new(face, frame, blocks))
    }

    /// Secondary constructor assembling jungle karst onto a canonical climate zone face.
    pub fn in_zone(face: Box<dyn Face>, zone: ClimateZone, half_size: f32) -> Self {
        let frame = zone.surface_frame(half_size);
        Self::at(face, frame, 3, 3)
    }
}

impl Face for JungleKarst {
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
