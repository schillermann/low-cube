mod cell;
mod corner;
mod cuboid;
mod diagonal_slope;
mod earth_clods;
mod elevated_shape;
mod form;
mod gentle_ridge;
mod gentle_valley;
mod gentle_wedge;
mod inverted_shape;
mod material;
mod mold;
mod pyramid;
mod ridge;
mod shape;
mod stair;
mod stair_corner;
mod timber_grain;
mod valley;
mod wedge;

#[cfg(test)]
mod tests;

pub use cell::GridCell;
pub use corner::CornerHeading;
pub use cuboid::CuboidShape;
pub use diagonal_slope::DiagonalSlope;
pub use earth_clods::EarthClods;
pub use elevated_shape::ElevatedShape;
pub use form::FormBlock;
pub use gentle_ridge::GentleRidge;
pub use gentle_valley::GentleValley;
pub use gentle_wedge::GentleWedge;
pub use inverted_shape::InvertedShape;
pub use material::{DirtBlock, MaterialBlock, SandBlock, SnowBlock, StoneBlock};
pub use mold::{BlockMold, CardinalTurn, MoldArchetype};
pub use pyramid::PyramidPeak;
pub use ridge::RidgeCorner;
pub use shape::VoxelShape;
pub use stair::StairFlight;
pub use stair_corner::StairCorner;
pub use timber_grain::{BlockMedium, MediumShape};
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
