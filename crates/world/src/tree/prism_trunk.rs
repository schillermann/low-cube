use std::f32::consts::PI;
use glam::Vec3;
use crate::{MeshTarget, Quad, SurfaceFrame};
use super::facet::ArborealFacet;

/// Faceted tapered prismatic trunk or branch column.
pub struct PrismTrunk {
    segments: usize,
    radius_base: f32,
    radius_top: f32,
    offset_top_u: f32,
    offset_top_v: f32,
    height: f32,
    color: Vec3,
}

impl PrismTrunk {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(
        segments: usize,
        radius_base: f32,
        radius_top: f32,
        offset_top_u: f32,
        offset_top_v: f32,
        height: f32,
        color: Vec3,
    ) -> Self {
        Self {
            segments,
            radius_base,
            radius_top,
            offset_top_u,
            offset_top_v,
            height,
            color,
        }
    }

    /// Secondary constructor for vertical straight columns without lateral drift.
    pub fn vertical(
        segments: usize,
        radius_base: f32,
        radius_top: f32,
        height: f32,
        color: Vec3,
    ) -> Self {
        Self::new(segments, radius_base, radius_top, 0.0, 0.0, height, color)
    }
}

impl ArborealFacet for PrismTrunk {
    fn emit(
        &self,
        frame: &SurfaceFrame,
        u: f32,
        v: f32,
        elevation: f32,
        target: &mut dyn MeshTarget,
    ) {
        let segs = self.segments.max(3);
        let step = (2.0 * PI) / (segs as f32);

        for i in 0..segs {
            let a0 = (i as f32) * step;
            let a1 = ((i + 1) as f32) * step;

            let b0_u = u + self.radius_base * a0.cos();
            let b0_v = v + self.radius_base * a0.sin();
            let b1_u = u + self.radius_base * a1.cos();
            let b1_v = v + self.radius_base * a1.sin();

            let t0_u = u + self.offset_top_u + self.radius_top * a0.cos();
            let t0_v = v + self.offset_top_v + self.radius_top * a0.sin();
            let t1_u = u + self.offset_top_u + self.radius_top * a1.cos();
            let t1_v = v + self.offset_top_v + self.radius_top * a1.sin();

            let p0 = frame.point_at(b0_u, b0_v, elevation);
            let p1 = frame.point_at(b1_u, b1_v, elevation);
            let p2 = frame.point_at(t1_u, t1_v, elevation + self.height);
            let p3 = frame.point_at(t0_u, t0_v, elevation + self.height);

            let cross = (p1 - p0).cross(p3 - p0);
            let normal = if cross.length_squared() > 1e-6 {
                cross.normalize()
            } else {
                frame.normal()
            };
            target.push_quad(Quad::new(p0, p1, p2, p3, normal, self.color));
        }
    }
}
