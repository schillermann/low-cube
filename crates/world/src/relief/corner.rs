use glam::Vec3;
use crate::{MeshTarget, Quad};
use crate::block::{
    CornerHeading, CuboidShape, GridCell, RampWedge, RidgeCorner, ValleyCorner, VoxelShape,
    WedgeHeading,
};
use crate::frame::SurfaceFrame;

/// Natural top-layer crown topology of a discrete terrain cell derived from its 3x3 neighborhood.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NaturalCrown {
    Ramp(usize, WedgeHeading),
    Ridge(usize, CornerHeading),
    Valley(usize, CornerHeading),
    Flat,
}

impl NaturalCrown {
    /// Classifies the natural crown of `(u, v)` from an elevation evaluation closure.
    pub fn at(u: i32, v: i32, elev_at: impl Fn(i32, i32) -> i32) -> Self {
        let h = elev_at(u, v);
        let (nhn, nhs, nhe, nhw) = (elev_at(u, v + 1), elev_at(u, v - 1), elev_at(u + 1, v), elev_at(u - 1, v));
        let (dn, ds, de, dw) = (h - nhn, h - nhs, h - nhe, h - nhw);
        if dn == 1 && ds <= 0 && de <= 0 && dw <= 0 {
            Self::Ramp(0, WedgeHeading::South)
        } else if ds == 1 && dn <= 0 && de <= 0 && dw <= 0 {
            Self::Ramp(1, WedgeHeading::North)
        } else if de == 1 && dn <= 0 && ds <= 0 && dw <= 0 {
            Self::Ramp(2, WedgeHeading::West)
        } else if dw == 1 && dn <= 0 && ds <= 0 && de <= 0 {
            Self::Ramp(3, WedgeHeading::East)
        } else if dn == 1 && de == 1 && ds <= 0 && dw <= 0 {
            Self::Ridge(0, CornerHeading::SouthWest)
        } else if dn == 1 && dw == 1 && ds <= 0 && de <= 0 {
            Self::Ridge(1, CornerHeading::SouthEast)
        } else if ds == 1 && de == 1 && dn <= 0 && dw <= 0 {
            Self::Ridge(2, CornerHeading::NorthWest)
        } else if ds == 1 && dw == 1 && dn <= 0 && de <= 0 {
            Self::Ridge(3, CornerHeading::NorthEast)
        } else if dn <= 0 && ds <= 0 && de <= 0 && dw <= 0 {
            let (hse, hsw) = (elev_at(u + 1, v - 1), elev_at(u - 1, v - 1));
            let (hne, hnw) = (elev_at(u + 1, v + 1), elev_at(u - 1, v + 1));
            if hse < h && nhs == h && nhe == h {
                Self::Valley(0, CornerHeading::NorthWest)
            } else if hsw < h && nhs == h && nhw == h {
                Self::Valley(1, CornerHeading::NorthEast)
            } else if hne < h && nhn == h && nhe == h {
                Self::Valley(2, CornerHeading::SouthEast)
            } else if hnw < h && nhn == h && nhw == h {
                Self::Valley(3, CornerHeading::SouthWest)
            } else {
                Self::Flat
            }
        } else {
            Self::Flat
        }
    }

    /// Evaluates continuous surface height in meters at fractional cell coordinates `(frac_u, frac_v)`.
    pub fn surface_height(self, h: i32, frac_u: f32, frac_v: f32) -> f32 {
        let base = ((h - 1) as f32) * GridCell::SPAN;
        match self {
            Self::Ramp(_, heading) => base + heading.elevation_at(frac_u, frac_v) * GridCell::SPAN,
            Self::Ridge(_, heading) => base + heading.ridge_elevation(frac_u, frac_v) * GridCell::SPAN,
            Self::Valley(_, heading) => base + heading.valley_elevation(frac_u, frac_v) * GridCell::SPAN,
            Self::Flat => (h as f32) * GridCell::SPAN,
        }
    }

    /// Materializes this natural crown into a concrete 3D `VoxelShape`.
    pub fn into_shape(self) -> Box<dyn VoxelShape> {
        match self {
            Self::Ramp(_, heading) => Box::new(RampWedge::new(heading)),
            Self::Ridge(_, heading) => Box::new(RidgeCorner::new(heading)),
            Self::Valley(_, heading) => Box::new(ValleyCorner::new(heading)),
            Self::Flat => Box::new(CuboidShape::new(GridCell::SPAN)),
        }
    }

    /// Emits the top surface facet for this natural crown and returns its four corner heights and color.
    pub fn emit_top(
        self,
        corner: &ReliefCorner,
        cell: GridCell,
        top_h: f32,
        frame: &SurfaceFrame,
        colors: (Vec3, Vec3, Vec3, Vec3),
        target: &mut dyn MeshTarget,
    ) -> ([f32; 4], Vec3) {
        let ((u0, u1), (v0, v1)) = (cell.u_bounds(), cell.v_bounds());
        let (surf_col, ramp_col, ridge_col, valley_col) = colors;
        match self {
            Self::Ramp(mode, _) => {
                corner.emit_ramp(u0, u1, v0, v1, top_h, mode, frame, ramp_col, target);
                let c = match mode {
                    0 => [top_h, top_h, top_h - 0.5, top_h - 0.5],
                    1 => [top_h - 0.5, top_h - 0.5, top_h, top_h],
                    2 => [top_h, top_h - 0.5, top_h - 0.5, top_h],
                    _ => [top_h - 0.5, top_h, top_h, top_h - 0.5],
                };
                (c, ramp_col)
            }
            Self::Ridge(mode, _) => {
                corner.emit_ridge(u0, u1, v0, v1, top_h, mode, frame, ridge_col, target);
                let c = match mode {
                    0 => [top_h, top_h - 0.5, top_h - 0.5, top_h - 0.5],
                    1 => [top_h - 0.5, top_h, top_h - 0.5, top_h - 0.5],
                    2 => [top_h - 0.5, top_h - 0.5, top_h - 0.5, top_h],
                    _ => [top_h - 0.5, top_h - 0.5, top_h, top_h - 0.5],
                };
                (c, ridge_col)
            }
            Self::Valley(mode, _) => {
                corner.emit_valley(u0, u1, v0, v1, top_h, mode, frame, valley_col, target);
                let c = match mode {
                    0 => [top_h, top_h - 0.5, top_h, top_h],
                    1 => [top_h - 0.5, top_h, top_h, top_h],
                    2 => [top_h, top_h, top_h - 0.5, top_h],
                    _ => [top_h, top_h, top_h, top_h - 0.5],
                };
                (c, valley_col)
            }
            Self::Flat => {
                let (p0, p1) = (frame.point_at(u0, v0, top_h), frame.point_at(u1, v0, top_h));
                let (p2, p3) = (frame.point_at(u1, v1, top_h), frame.point_at(u0, v1, top_h));
                target.push_quad(Quad::new(p0, p1, p2, p3, frame.normal(), surf_col));
                ([top_h, top_h, top_h, top_h], surf_col)
            }
        }
    }
}

/// Encapsulated facet evaluating convex ridge, concave valley, and ramp terrain corners.
pub struct ReliefCorner;

impl ReliefCorner {
    /// Primary constructor doing nothing except instantiating state.
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self
    }

    #[allow(clippy::too_many_arguments)]
    pub fn emit_ramp(
        &self, u0: f32, u1: f32, v0: f32, v1: f32, top_h: f32, mode: usize,
        frame: &SurfaceFrame, surf_col: Vec3, target: &mut dyn MeshTarget,
    ) {
        let (d0, d1, d2, d3, norm) = match mode {
            0 => (0.0, 0.0, -0.5, -0.5, (frame.normal() - frame.bitangent()).normalize()),
            1 => (-0.5, -0.5, 0.0, 0.0, (frame.normal() + frame.bitangent()).normalize()),
            2 => (0.0, -0.5, -0.5, 0.0, (frame.normal() - frame.tangent()).normalize()),
            _ => (-0.5, 0.0, 0.0, -0.5, (frame.normal() + frame.tangent()).normalize()),
        };
        let (p0, p1) = (frame.point_at(u0, v0, top_h + d0), frame.point_at(u1, v0, top_h + d1));
        let (p2, p3) = (frame.point_at(u1, v1, top_h + d2), frame.point_at(u0, v1, top_h + d3));
        target.push_quad(Quad::new(p0, p1, p2, p3, norm, surf_col));
    }

    #[allow(clippy::too_many_arguments)]
    pub fn emit_ridge(
        &self, u0: f32, u1: f32, v0: f32, v1: f32, top_h: f32, mode: usize,
        frame: &SurfaceFrame, surf_col: Vec3, target: &mut dyn MeshTarget,
    ) {
        let (p0, p1, p2, p3) = match mode {
            0 => (
                frame.point_at(u0, v0, top_h),
                frame.point_at(u1, v0, top_h - 0.5),
                frame.point_at(u1, v1, top_h - 0.5),
                frame.point_at(u0, v1, top_h - 0.5),
            ),
            1 => (
                frame.point_at(u0, v0, top_h - 0.5),
                frame.point_at(u1, v0, top_h),
                frame.point_at(u1, v1, top_h - 0.5),
                frame.point_at(u0, v1, top_h - 0.5),
            ),
            2 => (
                frame.point_at(u0, v0, top_h - 0.5),
                frame.point_at(u1, v0, top_h - 0.5),
                frame.point_at(u1, v1, top_h - 0.5),
                frame.point_at(u0, v1, top_h),
            ),
            _ => (
                frame.point_at(u0, v0, top_h - 0.5),
                frame.point_at(u1, v0, top_h - 0.5),
                frame.point_at(u1, v1, top_h),
                frame.point_at(u0, v1, top_h - 0.5),
            ),
        };
        match mode {
            0 => {
                let norm = (p3 - p0).cross(p1 - p0).normalize();
                self.push_tri(p0, p1, p3, norm, surf_col, target);
                self.push_tri(p1, p2, p3, frame.normal(), surf_col, target);
            }
            1 => {
                let norm = (p0 - p1).cross(p2 - p1).normalize();
                self.push_tri(p1, p2, p0, norm, surf_col, target);
                self.push_tri(p2, p3, p0, frame.normal(), surf_col, target);
            }
            2 => {
                let norm = (p2 - p3).cross(p0 - p3).normalize();
                self.push_tri(p3, p0, p2, norm, surf_col, target);
                self.push_tri(p0, p1, p2, frame.normal(), surf_col, target);
            }
            _ => {
                let norm = (p1 - p2).cross(p3 - p2).normalize();
                self.push_tri(p2, p3, p1, norm, surf_col, target);
                self.push_tri(p3, p0, p1, frame.normal(), surf_col, target);
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn emit_valley(
        &self, u0: f32, u1: f32, v0: f32, v1: f32, top_h: f32, mode: usize,
        frame: &SurfaceFrame, surf_col: Vec3, target: &mut dyn MeshTarget,
    ) {
        let (h0, h1, h2, h3) = match mode {
            0 => (top_h, top_h - 0.5, top_h, top_h),
            1 => (top_h - 0.5, top_h, top_h, top_h),
            2 => (top_h, top_h, top_h - 0.5, top_h),
            _ => (top_h, top_h, top_h, top_h - 0.5),
        };
        let (p0, p1) = (frame.point_at(u0, v0, h0), frame.point_at(u1, v0, h1));
        let (p2, p3) = (frame.point_at(u1, v1, h2), frame.point_at(u0, v1, h3));
        match mode {
            0 => self.push_corner_pair(p3, p0, p1, p2, surf_col, target),
            1 => self.push_corner_pair(p2, p1, p0, p3, surf_col, target),
            2 => self.push_corner_pair(p0, p1, p2, p3, surf_col, target),
            _ => self.push_corner_pair(p1, p2, p3, p0, surf_col, target),
        }
    }

    pub fn push_corner_pair(&self, p0: Vec3, p1: Vec3, p2: Vec3, p3: Vec3, color: Vec3, target: &mut dyn MeshTarget) {
        let n1 = (p1 - p0).cross(p2 - p0).normalize();
        let n2 = (p2 - p0).cross(p3 - p0).normalize();
        self.push_tri(p0, p1, p2, n1, color, target);
        self.push_tri(p0, p2, p3, n2, color, target);
    }

    fn push_tri(&self, a: Vec3, b: Vec3, c: Vec3, norm: Vec3, col: Vec3, target: &mut dyn MeshTarget) {
        target.push_quad(Quad::new(a, b, c, c, norm, col));
    }
}
