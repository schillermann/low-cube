use glam::Vec3;
use crate::{MeshTarget, Quad};
use crate::frame::SurfaceFrame;

/// Encapsulated facet evaluating convex ridge and concave valley terrain corners.
pub struct ReliefCorner;

impl ReliefCorner {
    /// Primary constructor doing nothing except instantiating state.
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self
    }

    #[allow(clippy::too_many_arguments)]
    pub fn emit_ridge(
        &self, u0: f32, u1: f32, v0: f32, v1: f32, top_h: f32, mode: usize,
        frame: &SurfaceFrame, surf_col: Vec3, target: &mut dyn MeshTarget,
    ) {
        let (p0, p1, p2, p3) = match mode {
            0 => (
                frame.point_at(u0, v0, top_h),
                frame.point_at(u1, v0, top_h - 1.0),
                frame.point_at(u1, v1, top_h - 1.0),
                frame.point_at(u0, v1, top_h - 1.0),
            ),
            1 => (
                frame.point_at(u0, v0, top_h - 1.0),
                frame.point_at(u1, v0, top_h),
                frame.point_at(u1, v1, top_h - 1.0),
                frame.point_at(u0, v1, top_h - 1.0),
            ),
            2 => (
                frame.point_at(u0, v0, top_h - 1.0),
                frame.point_at(u1, v0, top_h - 1.0),
                frame.point_at(u1, v1, top_h - 1.0),
                frame.point_at(u0, v1, top_h),
            ),
            _ => (
                frame.point_at(u0, v0, top_h - 1.0),
                frame.point_at(u1, v0, top_h - 1.0),
                frame.point_at(u1, v1, top_h),
                frame.point_at(u0, v1, top_h - 1.0),
            ),
        };
        match mode {
            0 => {
                let norm = (p1 - p0).cross(p3 - p0).normalize();
                self.push_tri(p0, p1, p3, norm, surf_col, target);
                self.push_tri(p1, p2, p3, frame.normal(), surf_col, target);
            }
            1 => {
                let norm = (p2 - p1).cross(p0 - p1).normalize();
                self.push_tri(p1, p2, p0, norm, surf_col, target);
                self.push_tri(p2, p3, p0, frame.normal(), surf_col, target);
            }
            2 => {
                let norm = (p0 - p3).cross(p2 - p3).normalize();
                self.push_tri(p3, p0, p2, norm, surf_col, target);
                self.push_tri(p0, p1, p2, frame.normal(), surf_col, target);
            }
            _ => {
                let norm = (p3 - p2).cross(p1 - p2).normalize();
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
            0 => (top_h, top_h - 1.0, top_h, top_h),
            1 => (top_h - 1.0, top_h, top_h, top_h),
            2 => (top_h, top_h, top_h - 1.0, top_h),
            _ => (top_h, top_h, top_h, top_h - 1.0),
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
