mod cell;
mod corner;
mod cuboid;
mod form;
mod material;
mod pyramid;
mod ridge;
mod shape;
mod stair;
mod valley;
mod wedge;

#[cfg(test)]
mod tests;

pub use cell::GridCell;
pub use corner::CornerHeading;
pub use cuboid::CuboidShape;
pub use form::FormBlock;
pub use material::{DirtBlock, SandBlock, SnowBlock, StoneBlock};
pub use pyramid::PyramidPeak;
pub use ridge::RidgeCorner;
pub use shape::VoxelShape;
pub use stair::StairFlight;
pub use valley::ValleyCorner;
pub use wedge::{RampWedge, WedgeHeading};

use glam::Vec3;
use crate::MeshTarget;
use crate::frame::SurfaceFrame;

/// Core interface for an individual placed block on a planetary face ("Tell, Don't Ask").
pub trait Block: Send + Sync {
    fn constrain_surface(
        &self,
        base: Vec3,
        u: f32,
        v: f32,
        frame: &SurfaceFrame,
        distance: f32,
    ) -> Vec3;

    fn emit_surface(
        &self,
        frame: &SurfaceFrame,
        target: &mut dyn MeshTarget,
    );
}
