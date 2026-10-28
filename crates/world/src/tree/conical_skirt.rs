use std::f32::consts::PI;
use glam::Vec3;
use crate::{MeshTarget, Quad, SurfaceFrame};
use super::facet::ArborealFacet;

/// Faceted conical skirt layer for coniferous or umbrella foliage.
pub struct ConicalSkirt {
    segments: usize,
    radii: (f32, f32),
    span: (f32, f32),
    color: Vec3,
}

impl ConicalSkirt {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(
        segments: usize,
        radius_base: f32,
        radius_top: f32,
        height: f32,
        elevation_offset: f32,
        color: Vec3,
    ) -> Self {
        Self {
            segments,
            radii: (radius_base, radius_top),
            span: (height, elevation_offset),
            color,
        }
    }
}

impl ArborealFacet for ConicalSkirt {
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
        let (radius_base, radius_top) = self.radii;
        let (height, elevation_offset) = self.span;
        let base_y = elevation + elevation_offset;
        let top_y = base_y + height;

        for i in 0..segs {
            let a0 = (i as f32) * step;
            let a1 = ((i + 1) as f32) * step;

            let b0_u = u + radius_base * a0.cos();
            let b0_v = v + radius_base * a0.sin();
            let b1_u = u + radius_base * a1.cos();
            let b1_v = v + radius_base * a1.sin();

            let t0_u = u + radius_top * a0.cos();
            let t0_v = v + radius_top * a0.sin();
            let t1_u = u + radius_top * a1.cos();
            let t1_v = v + radius_top * a1.sin();

            let p0 = frame.point_at(b0_u, b0_v, base_y);
            let p1 = frame.point_at(b1_u, b1_v, base_y);
            let p2 = frame.point_at(t1_u, t1_v, top_y);
            let p3 = frame.point_at(t0_u, t0_v, top_y);

            let cross = (p1 - p0).cross(p3 - p0);
            let normal = if cross.length_squared() > 1e-6 {
                cross.normalize()
            } else {
                frame.normal()
            };
            target.push_quad(Quad::new(p0, p1, p2, p3, normal, self.color));

            let center = frame.point_at(u, v, base_y);
            let under_norm = -frame.normal();
            target.push_quad(Quad::new(center, p1, p0, center, under_norm, self.color * 0.7));
        }
    }
}
