use glam::Vec3;
use crate::{MeshTarget, Quad};
use crate::frame::SurfaceFrame;
use super::cell::GridCell;
use super::corner::CornerHeading;
use super::cuboid::CuboidShape;
use super::diagonal_slope::DiagonalSlope;
use super::gentle_ridge::GentleRidge;
use super::gentle_valley::GentleValley;
use super::gentle_wedge::GentleWedge;
use super::ridge::RidgeCorner;
use super::shape::VoxelShape;
use super::timber_grain::BlockMedium;
use super::valley::ValleyCorner;
use super::wedge::{RampWedge, WedgeHeading};

struct MirroredMeshTarget<'a> {
    frame: &'a SurfaceFrame,
    floor_elevation: f32,
    target: &'a mut dyn MeshTarget,
}

impl MeshTarget for MirroredMeshTarget<'_> {
    fn push_quad(&mut self, quad: Quad) {
        let up = self.frame.normal();
        let mirror_pt = |p: Vec3| {
            let dh = self.frame.elevation_of(p) - self.floor_elevation;
            p + up * (0.5 - 2.0 * dh)
        };
        let mirrored = quad.transfer(|v0, v1, v2, v3, normal, color| {
            let at_floor = (self.frame.elevation_of(v0) - self.floor_elevation).abs() < 1e-4
                && (self.frame.elevation_of(v1) - self.floor_elevation).abs() < 1e-4
                && (self.frame.elevation_of(v2) - self.floor_elevation).abs() < 1e-4;
            if at_floor && normal.dot(up).abs() > 0.9 {
                let (t1, t2, t_peak) = (mirror_pt(v0), mirror_pt(v1), mirror_pt(v2));
                let t_opp = t1 + t2 - t_peak;
                return Quad::new(t2, t1, t_opp, t_opp, -up, color);
            }
            let (m0, m1, m2, m3) = (mirror_pt(v0), mirror_pt(v1), mirror_pt(v2), mirror_pt(v3));
            let up_norm = if normal.dot(up).abs() > 0.05 && normal.dot(up) < 0.0 { -normal } else { normal };
            let m_norm = (up_norm - up * (2.0 * up_norm.dot(up))).normalize_or_zero();
            if (v2 - v3).length_squared() < 1e-6 {
                Quad::new(m2, m1, m0, m0, m_norm, color)
            } else {
                Quad::new(m3, m2, m1, m0, m_norm, color)
            }
        });
        self.target.push_quad(mirrored);
    }
}

/// Decorator vertically mirroring a VoxelShape within its 0.5m layer to form an overhead beveled ceiling block.
pub struct InvertedShape {
    inner: Box<dyn VoxelShape>,
}

impl InvertedShape {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(inner: Box<dyn VoxelShape>) -> Self {
        Self { inner }
    }

    /// Secondary constructor selecting the beveled ceiling block matching four 0.25m corner drops (SW, SE, NE, NW).
    pub fn from_corner_drops(s0: i32, s1: i32, s2: i32, s3: i32) -> Self {
        let inner: Box<dyn VoxelShape> = match (s0, s1, s2, s3) {
            (0, 0, 0, 0) | (2, 2, 2, 2) => Box::new(CuboidShape::full_cube()),
            (1, 1, 1, 1) => Box::new(CuboidShape::half_slab()),
            (0, 0, 1, 1) => Box::new(GentleWedge::lower(WedgeHeading::North)),
            (1, 1, 0, 0) => Box::new(GentleWedge::lower(WedgeHeading::South)),
            (0, 1, 1, 0) => Box::new(GentleWedge::lower(WedgeHeading::East)),
            (1, 0, 0, 1) => Box::new(GentleWedge::lower(WedgeHeading::West)),
            (1, 1, 2, 2) => Box::new(GentleWedge::upper(WedgeHeading::North)),
            (2, 2, 1, 1) => Box::new(GentleWedge::upper(WedgeHeading::South)),
            (1, 2, 2, 1) => Box::new(GentleWedge::upper(WedgeHeading::East)),
            (2, 1, 1, 2) => Box::new(GentleWedge::upper(WedgeHeading::West)),
            (0, 0, 2, 2) => Box::new(RampWedge::new(WedgeHeading::North)),
            (2, 2, 0, 0) => Box::new(RampWedge::new(WedgeHeading::South)),
            (0, 2, 2, 0) => Box::new(RampWedge::new(WedgeHeading::East)),
            (2, 0, 0, 2) => Box::new(RampWedge::new(WedgeHeading::West)),
            (0, 0, 1, 0) => Box::new(GentleRidge::lower(CornerHeading::NorthEast)),
            (0, 0, 0, 1) => Box::new(GentleRidge::lower(CornerHeading::NorthWest)),
            (0, 1, 0, 0) => Box::new(GentleRidge::lower(CornerHeading::SouthEast)),
            (1, 0, 0, 0) => Box::new(GentleRidge::lower(CornerHeading::SouthWest)),
            (1, 1, 2, 1) => Box::new(GentleRidge::upper(CornerHeading::NorthEast)),
            (1, 1, 1, 2) => Box::new(GentleRidge::upper(CornerHeading::NorthWest)),
            (1, 2, 1, 1) => Box::new(GentleRidge::upper(CornerHeading::SouthEast)),
            (2, 1, 1, 1) => Box::new(GentleRidge::upper(CornerHeading::SouthWest)),
            (0, 0, 2, 0) => Box::new(RidgeCorner::new(CornerHeading::NorthEast)),
            (0, 0, 0, 2) => Box::new(RidgeCorner::new(CornerHeading::NorthWest)),
            (0, 2, 0, 0) => Box::new(RidgeCorner::new(CornerHeading::SouthEast)),
            (2, 0, 0, 0) => Box::new(RidgeCorner::new(CornerHeading::SouthWest)),
            (0, 1, 1, 1) => Box::new(GentleValley::lower(CornerHeading::NorthEast)),
            (1, 0, 1, 1) => Box::new(GentleValley::lower(CornerHeading::NorthWest)),
            (1, 1, 0, 1) => Box::new(GentleValley::lower(CornerHeading::SouthEast)),
            (1, 1, 1, 0) => Box::new(GentleValley::lower(CornerHeading::SouthWest)),
            (1, 2, 2, 2) => Box::new(GentleValley::upper(CornerHeading::NorthEast)),
            (2, 1, 2, 2) => Box::new(GentleValley::upper(CornerHeading::NorthWest)),
            (2, 2, 1, 2) => Box::new(GentleValley::upper(CornerHeading::SouthEast)),
            (2, 2, 2, 1) => Box::new(GentleValley::upper(CornerHeading::SouthWest)),
            (0, 2, 2, 2) => Box::new(ValleyCorner::new(CornerHeading::NorthEast)),
            (2, 0, 2, 2) => Box::new(ValleyCorner::new(CornerHeading::NorthWest)),
            (2, 2, 0, 2) => Box::new(ValleyCorner::new(CornerHeading::SouthEast)),
            (2, 2, 2, 0) => Box::new(ValleyCorner::new(CornerHeading::SouthWest)),
            _ => {
                let (ne, nw) = (s2 - s0, s3 - s1);
                if ne.abs() >= nw.abs() {
                    if ne >= 0 { Box::new(DiagonalSlope::new(CornerHeading::NorthEast)) } else { Box::new(DiagonalSlope::new(CornerHeading::SouthWest)) }
                } else if nw >= 0 {
                    Box::new(DiagonalSlope::new(CornerHeading::NorthWest))
                } else {
                    Box::new(DiagonalSlope::new(CornerHeading::SouthEast))
                }
            }
        };
        Self::new(inner)
    }

    /// Ceiling elevations at the four cell corners (SW, SE, NE, NW) for `floor_elevation`.
    pub fn ceiling_corners(&self, floor_elevation: f32) -> [f32; 4] {
        [
            floor_elevation + 0.5 - self.inner.elevation_at(0.0, 0.0),
            floor_elevation + 0.5 - self.inner.elevation_at(1.0, 0.0),
            floor_elevation + 0.5 - self.inner.elevation_at(1.0, 1.0),
            floor_elevation + 0.5 - self.inner.elevation_at(0.0, 1.0),
        ]
    }
}

impl VoxelShape for InvertedShape {
    fn elevation_at(&self, local_u: f32, local_v: f32) -> f32 {
        if self.inner.elevation_at(local_u, local_v) > 0.0 { 0.5 } else { 0.0 }
    }

    fn shape_color(&self) -> Vec3 {
        self.inner.shape_color()
    }

    fn block_medium(&self) -> BlockMedium {
        self.inner.block_medium()
    }

    fn emit_shape(
        &self,
        cell: GridCell,
        floor_elevation: f32,
        frame: &SurfaceFrame,
        color: Vec3,
        target: &mut dyn MeshTarget,
    ) {
        let mut mirrored = MirroredMeshTarget { frame, floor_elevation, target };
        self.inner.emit_shape(cell, floor_elevation, frame, color, &mut mirrored);
    }
}
