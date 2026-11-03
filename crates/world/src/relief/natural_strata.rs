use std::sync::OnceLock;
use crate::block::{
    CornerHeading, CuboidShape, GridCell, RampWedge, RidgeCorner, ValleyCorner, VoxelShape,
    WedgeHeading,
};
use super::crust_vault::CrustVault;
use super::elevation::ReliefElevation;

/// Memoized natural planetary strata and inner crust vault geometry for a relief face.
pub struct NaturalStrata {
    origin: Box<dyn ReliefElevation>,
    grid: OnceLock<Vec<i16>>,
    half_span: i32,
}

impl NaturalStrata {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(origin: Box<dyn ReliefElevation>, grid: OnceLock<Vec<i16>>, half_span: i32) -> Self {
        Self { origin, grid, half_span }
    }

    /// Secondary constructor sizing the memoized natural grid for a face with `half_blocks` radius.
    pub fn in_span(origin: Box<dyn ReliefElevation>, half_blocks: i32) -> Self {
        Self::new(origin, OnceLock::new(), half_blocks + 2)
    }

    /// Inner planetary crust vault beneath this relief face.
    pub fn vault(&self) -> CrustVault {
        CrustVault::beneath(self.half_span - 2)
    }

    /// Pristine discrete natural block elevation at (u, v) prior to any excavation.
    pub fn elevation_at(&self, u: i32, v: i32) -> i32 {
        if u.abs() <= self.half_span && v.abs() <= self.half_span {
            let side = (self.half_span * 2 + 1) as usize;
            let table = self.grid.get_or_init(|| {
                let mut cells = Vec::with_capacity(side * side);
                for gu in -self.half_span..=self.half_span {
                    for gv in -self.half_span..=self.half_span {
                        cells.push(self.origin.elevation_at(gu, gv) as i16);
                    }
                }
                cells
            });
            table[((u + self.half_span) as usize) * side + ((v + self.half_span) as usize)] as i32
        } else {
            self.origin.elevation_at(u, v)
        }
    }

    /// Continuous natural surface elevation in meters at tangential coordinates (u, v).
    pub fn surface_height_at(&self, u: f32, v: f32) -> f32 {
        self.origin.surface_height_at(u, v)
    }

    /// Pristine voxel shape of the natural top layer at (u, v) so undercut slopes never expand into full cubes.
    pub fn crown_shape_at(&self, u: i32, v: i32) -> Box<dyn VoxelShape> {
        let h = self.elevation_at(u, v);
        let (nhn, nhs) = (self.elevation_at(u, v + 1), self.elevation_at(u, v - 1));
        let (nhe, nhw) = (self.elevation_at(u + 1, v), self.elevation_at(u - 1, v));
        let (dn, ds, de, dw) = (h - nhn, h - nhs, h - nhe, h - nhw);
        if dn == 1 && ds <= 0 && de <= 0 && dw <= 0 {
            Box::new(RampWedge::new(WedgeHeading::South))
        } else if ds == 1 && dn <= 0 && de <= 0 && dw <= 0 {
            Box::new(RampWedge::new(WedgeHeading::North))
        } else if de == 1 && dn <= 0 && ds <= 0 && dw <= 0 {
            Box::new(RampWedge::new(WedgeHeading::West))
        } else if dw == 1 && dn <= 0 && ds <= 0 && de <= 0 {
            Box::new(RampWedge::new(WedgeHeading::East))
        } else if dn == 1 && de == 1 && ds <= 0 && dw <= 0 {
            Box::new(RidgeCorner::new(CornerHeading::SouthWest))
        } else if dn == 1 && dw == 1 && ds <= 0 && de <= 0 {
            Box::new(RidgeCorner::new(CornerHeading::SouthEast))
        } else if ds == 1 && de == 1 && dn <= 0 && dw <= 0 {
            Box::new(RidgeCorner::new(CornerHeading::NorthWest))
        } else if ds == 1 && dw == 1 && dn <= 0 && de <= 0 {
            Box::new(RidgeCorner::new(CornerHeading::NorthEast))
        } else if dn <= 0 && ds <= 0 && de <= 0 && dw <= 0 {
            let (hse, hsw) = (self.elevation_at(u + 1, v - 1), self.elevation_at(u - 1, v - 1));
            let (hne, hnw) = (self.elevation_at(u + 1, v + 1), self.elevation_at(u - 1, v + 1));
            if hse < h && nhs == h && nhe == h {
                Box::new(ValleyCorner::new(CornerHeading::NorthWest))
            } else if hsw < h && nhs == h && nhw == h {
                Box::new(ValleyCorner::new(CornerHeading::NorthEast))
            } else if hne < h && nhn == h && nhe == h {
                Box::new(ValleyCorner::new(CornerHeading::SouthEast))
            } else if hnw < h && nhn == h && nhw == h {
                Box::new(ValleyCorner::new(CornerHeading::SouthWest))
            } else {
                Box::new(CuboidShape::new(GridCell::SPAN))
            }
        } else {
            Box::new(CuboidShape::new(GridCell::SPAN))
        }
    }
}
