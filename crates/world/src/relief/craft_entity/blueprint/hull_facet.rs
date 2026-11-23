use glam::Vec3;
use crate::{MeshTarget, Quad};

/// Oriented planar boundary triangle of a `HullSolid` in canonical `(u, elev, v)` blueprint space.
#[derive(Clone, Copy, Debug)]
pub(super) struct HullFacet {
    verts: [Vec3; 3],
    normal: Vec3,
}

impl HullFacet {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub(super) const fn new(verts: [Vec3; 3], normal: Vec3) -> Self {
        Self { verts, normal }
    }

    pub(super) const fn normal(&self) -> Vec3 {
        self.normal
    }

    pub(super) const fn vertices(&self) -> [Vec3; 3] {
        self.verts
    }

    pub(super) fn overlaps_box(&self, lo: Vec3, hi: Vec3) -> bool {
        let f_lo = self.verts[0].min(self.verts[1]).min(self.verts[2]) - Vec3::splat(0.02);
        let f_hi = self.verts[0].max(self.verts[1]).max(self.verts[2]) + Vec3::splat(0.02);
        f_lo.x <= hi.x
            && f_hi.x >= lo.x
            && f_lo.y <= hi.y
            && f_hi.y >= lo.y
            && f_lo.z <= hi.z
            && f_hi.z >= lo.z
    }

    pub(super) fn plane_split(&self, origin: Vec3, span: Vec3) -> Option<f32> {
        let seg_lo = origin.min(origin + span);
        let seg_hi = origin.max(origin + span);
        if !self.overlaps_box(seg_lo, seg_hi) {
            return None;
        }
        let denom = span.dot(self.normal);
        if denom.abs() <= 1e-5 {
            return None;
        }
        let t = (self.verts[0] - origin).dot(self.normal) / denom;
        (t > 1e-4 && t < 1.0 - 1e-4).then_some(t)
    }

    pub(super) fn coplanar_with(&self, point: Vec3, tangent: Vec3) -> bool {
        let [a, b, c] = self.verts;
        let lo = a.min(b).min(c) - Vec3::splat(0.012);
        let hi = a.max(b).max(c) + Vec3::splat(0.012);
        if point.x < lo.x
            || point.x > hi.x
            || point.y < lo.y
            || point.y > hi.y
            || point.z < lo.z
            || point.z > hi.z
        {
            return false;
        }
        (point - a).dot(self.normal).abs() <= 1e-3 && tangent.dot(self.normal).abs() <= 1e-3
    }

    pub(super) fn contains_strict(&self, point: Vec3) -> bool {
        let [a, b, c] = self.verts;
        let area_vec = (b - a).cross(c - a);
        let area_len = area_vec.length();
        if area_len <= 1e-7 {
            return false;
        }
        let tol = 1e-6 * area_len;
        let c0 = (b - a).cross(point - a).dot(area_vec);
        let c1 = (c - b).cross(point - b).dot(area_vec);
        let c2 = (a - c).cross(point - c).dot(area_vec);
        c0 > tol && c1 > tol && c2 > tol
    }
}

/// `MeshTarget` sink decomposing watertight block quads into `HullFacet`s and perimeter segments.
pub(super) struct FacetSink<'a> {
    facets: &'a mut Vec<HullFacet>,
    edges: &'a mut Vec<(Vec3, Vec3)>,
    underside_sealed: bool,
}

impl<'a> FacetSink<'a> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub(super) fn new(
        facets: &'a mut Vec<HullFacet>,
        edges: &'a mut Vec<(Vec3, Vec3)>,
        underside_sealed: bool,
    ) -> Self {
        Self { facets, edges, underside_sealed }
    }

    pub(super) fn seal_underside(&mut self, corners: [Vec3; 4]) {
        if !self.underside_sealed {
            let [b0, b1, b2, b3] = corners;
            self.push_quad(Quad::new(b0, b3, b2, b1, Vec3::NEG_Y, Vec3::ONE));
            self.underside_sealed = true;
        }
    }
}

impl MeshTarget for FacetSink<'_> {
    fn push_quad(&mut self, quad: Quad) {
        quad.transfer(|v0, v1, v2, v3, raw_norm, _| {
            let norm = raw_norm.normalize_or_zero();
            if norm.length_squared() < 0.5 {
                return;
            }
            if norm.y < -0.9 {
                self.underside_sealed = true;
            }
            let mut ring = [Vec3::ZERO; 4];
            let mut count = 0usize;
            for v in [v0, v1, v2, v3] {
                if count == 0 || (v - ring[count - 1]).length_squared() > 1e-7 {
                    ring[count] = v;
                    count += 1;
                }
            }
            if count >= 2 && (ring[count - 1] - ring[0]).length_squared() <= 1e-7 {
                count -= 1;
            }
            if count < 3 {
                return;
            }
            for i in 0..count {
                let (a, b) = (ring[i], ring[(i + 1) % count]);
                if (b - a).length_squared() > 1e-6 {
                    self.edges.push((a, b));
                }
            }
            if (ring[1] - ring[0]).cross(ring[2] - ring[0]).length_squared() > 1e-8 {
                self.facets.push(HullFacet::new([ring[0], ring[1], ring[2]], norm));
            }
            if count == 4 && (ring[2] - ring[0]).cross(ring[3] - ring[0]).length_squared() > 1e-8 {
                self.facets.push(HullFacet::new([ring[0], ring[2], ring[3]], norm));
            }
        });
    }
}
