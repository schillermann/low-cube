use glam::Vec3;
use crate::{MeshTarget, Quad, SurfaceFrame};
use super::facet::ArborealFacet;

/// Faceted icosahedral foliage clump for deciduous trees.
pub struct PolyhedralCrown {
    offset_u: f32,
    offset_v: f32,
    elevation_offset: f32,
    radius: f32,
    color: Vec3,
}

impl PolyhedralCrown {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(
        offset_u: f32,
        offset_v: f32,
        elevation_offset: f32,
        radius: f32,
        color: Vec3,
    ) -> Self {
        Self {
            offset_u,
            offset_v,
            elevation_offset,
            radius,
            color,
        }
    }
}

impl ArborealFacet for PolyhedralCrown {
    fn emit(
        &self,
        frame: &SurfaceFrame,
        u: f32,
        v: f32,
        elevation: f32,
        target: &mut dyn MeshTarget,
    ) {
        let phi = (1.0 + 5.0_f32.sqrt()) * 0.5;
        let base_raw = [
            Vec3::new(-1.0, phi, 0.0),
            Vec3::new(1.0, phi, 0.0),
            Vec3::new(-1.0, -phi, 0.0),
            Vec3::new(1.0, -phi, 0.0),
            Vec3::new(0.0, -1.0, phi),
            Vec3::new(0.0, 1.0, phi),
            Vec3::new(0.0, -1.0, -phi),
            Vec3::new(0.0, 1.0, -phi),
            Vec3::new(phi, 0.0, -1.0),
            Vec3::new(phi, 0.0, 1.0),
            Vec3::new(-phi, 0.0, -1.0),
            Vec3::new(-phi, 0.0, 1.0),
        ];

        let center_u = u + self.offset_u;
        let center_v = v + self.offset_v;
        let center_elev = elevation + self.elevation_offset;

        let verts: Vec<Vec3> = base_raw
            .iter()
            .map(|r| {
                let unit = r.normalize();
                frame.point_at(
                    center_u + unit.x * self.radius,
                    center_v + unit.z * self.radius,
                    center_elev + unit.y * self.radius,
                )
            })
            .collect();

        let indices: [(usize, usize, usize); 20] = [
            (0, 11, 5), (0, 5, 1), (0, 1, 7), (0, 7, 10), (0, 10, 11),
            (1, 5, 9), (5, 11, 4), (11, 10, 2), (10, 7, 6), (7, 1, 8),
            (3, 9, 4), (3, 4, 2), (3, 2, 6), (3, 6, 8), (3, 8, 9),
            (4, 9, 5), (2, 4, 11), (6, 2, 10), (8, 6, 7), (9, 8, 1),
        ];

        for (i0, i1, i2) in indices {
            let p0 = verts[i0];
            let p1 = verts[i1];
            let p2 = verts[i2];
            let cross = (p1 - p0).cross(p2 - p0);
            let normal = if cross.length_squared() > 1e-6 {
                cross.normalize()
            } else {
                frame.normal()
            };
            target.push_quad(Quad::new(p0, p1, p2, p2, normal, self.color));
        }
    }
}
