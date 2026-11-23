use glam::Vec3;
use super::hull_facet::HullFacet;
use super::hull_solid::HullSolid;
use super::wire_primitives::MAX_WIRE_EDGES;

#[cfg(test)]
#[path = "contour_tests.rs"]
mod tests;

/// 3D infinite line track accumulating collinear 1D intervals `(s_min, s_max)` for seam fusion.
struct CollinearTrack {
    dir: Vec3,
    anchor: Vec3,
    spans: Vec<(f32, f32)>,
}

impl CollinearTrack {
    /// Primary constructor doing nothing except assigning arguments to fields.
    const fn new(dir: Vec3, anchor: Vec3, spans: Vec<(f32, f32)>) -> Self {
        Self { dir, anchor, spans }
    }

    fn decompose(a: Vec3, b: Vec3) -> Option<(Vec3, Vec3, f32, f32)> {
        let raw = b - a;
        let len = raw.length();
        if len <= 1e-3 {
            return None;
        }
        let mut dir = raw / len;
        let flip = if dir.x.abs() > 1e-4 {
            dir.x < 0.0
        } else if dir.y.abs() > 1e-4 {
            dir.y < 0.0
        } else {
            dir.z < 0.0
        };
        if flip {
            dir = -dir;
        }
        let anchor = a - dir * a.dot(dir);
        let (s0, s1) = (a.dot(dir), b.dot(dir));
        Some((dir, anchor, s0.min(s1), s0.max(s1)))
    }

    fn matches(&self, dir: Vec3, anchor: Vec3) -> bool {
        self.dir.dot(dir) > 0.9998 && (self.anchor - anchor).length_squared() <= 1e-5
    }

    fn merged_segments(&mut self, out: &mut Vec<(Vec3, Vec3)>) {
        if self.spans.is_empty() {
            return;
        }
        self.spans.sort_by(|a, b| a.0.total_cmp(&b.0));
        let mut cur = self.spans[0];
        for &(lo, hi) in &self.spans[1..] {
            if lo <= cur.1 + 1e-3 {
                cur.1 = cur.1.max(hi);
            } else {
                out.push((self.anchor + self.dir * cur.0, self.anchor + self.dir * cur.1));
                cur = (lo, hi);
            }
        }
        out.push((self.anchor + self.dir * cur.0, self.anchor + self.dir * cur.1));
    }
}

/// Merged 3D outer-hull crease wireframe of a spacecraft without interior block seams or fills.
pub(super) struct HullContour {
    edges: Vec<(Vec3, Vec3)>,
}

impl HullContour {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub(super) const fn new(edges: Vec<(Vec3, Vec3)>) -> Self {
        Self { edges }
    }

    pub(super) fn from_solids(solids: &[HullSolid]) -> Self {
        if solids.is_empty() {
            return Self::new(Vec::new());
        }
        let mut facets = Vec::with_capacity(solids.len() * 8);
        let mut raw_edges = Vec::with_capacity(solids.len() * 16);
        for solid in solids {
            solid.emit_facets(&mut facets, &mut raw_edges);
        }

        let mut tracks: Vec<CollinearTrack> = Vec::new();
        for (a, b) in raw_edges {
            let Some((dir, anchor, lo, hi)) = CollinearTrack::decompose(a, b) else {
                continue;
            };
            if let Some(track) = tracks.iter_mut().find(|t| t.matches(dir, anchor)) {
                track.spans.push((lo, hi));
            } else {
                tracks.push(CollinearTrack::new(dir, anchor, vec![(lo, hi)]));
            }
        }

        let mut candidate_spans = Vec::with_capacity(tracks.len() * 2);
        for track in &mut tracks {
            track.merged_segments(&mut candidate_spans);
        }

        let mut kept = Vec::with_capacity(candidate_spans.len());
        let mut cuts = Vec::with_capacity(16);
        for (p, q) in candidate_spans {
            Self::extract_creases_along((p, q), &facets, solids, &mut cuts, &mut kept);
        }
        if kept.len() > MAX_WIRE_EDGES {
            kept.sort_by(|(a0, b0), (a1, b1)| {
                (b1 - a1).length_squared().total_cmp(&(b0 - a0).length_squared())
            });
            kept.truncate(MAX_WIRE_EDGES);
        }
        Self::new(kept)
    }

    fn extract_creases_along(
        (p, q): (Vec3, Vec3),
        facets: &[HullFacet],
        solids: &[HullSolid],
        cuts: &mut Vec<f32>,
        out: &mut Vec<(Vec3, Vec3)>,
    ) {
        let span = q - p;
        let len_sq = span.length_squared();
        if len_sq <= 1e-6 {
            return;
        }
        let seg_lo = p.min(q);
        let seg_hi = p.max(q);
        cuts.clear();
        cuts.push(0.0);
        cuts.push(1.0);
        for facet in facets {
            if !facet.overlaps_box(seg_lo, seg_hi) {
                continue;
            }
            if let Some(t) = facet.plane_split(p, span) {
                cuts.push(t);
            }
            for v in facet.vertices() {
                let t = (v - p).dot(span) / len_sq;
                if t > 1e-4 && t < 1.0 - 1e-4 && ((p + span * t) - v).length_squared() <= 1e-6 {
                    cuts.push(t);
                }
            }
        }
        cuts.sort_by(|a, b| a.total_cmp(b));

        let mut active_start: Option<Vec3> = None;
        let mut active_end = p;
        for pair in cuts.windows(2) {
            let (t0, t1) = (pair[0], pair[1]);
            if t1 - t0 <= 1e-3 {
                continue;
            }
            let (sub_a, sub_b) = (p + span * t0, p + span * t1);
            if Self::crease_segment(sub_a, sub_b, facets, solids) {
                if active_start.is_none() {
                    active_start = Some(sub_a);
                }
                active_end = sub_b;
            } else if let Some(start) = active_start.take() {
                out.push((start, active_end));
            }
        }
        if let Some(start) = active_start {
            out.push((start, active_end));
        }
    }

    fn crease_segment(a: Vec3, b: Vec3, facets: &[HullFacet], solids: &[HullSolid]) -> bool {
        let span = b - a;
        let len = span.length();
        if len < 0.01 {
            return false;
        }
        let mid = (a + b) * 0.5;
        let tangent = span / len;
        let eps = (len * 0.16).clamp(0.002, 0.008);
        let out_step = eps * 0.10;
        let mut rays = [(Vec3::ZERO, Vec3::ZERO); 16];
        let mut count = 0usize;

        for facet in facets {
            if !facet.coplanar_with(mid, tangent) {
                continue;
            }
            let norm = facet.normal();
            let in_dir = tangent.cross(norm).normalize_or_zero();
            if in_dir.length_squared() < 0.5 {
                continue;
            }
            for sign in [-1.0_f32, 1.0_f32] {
                let ray = in_dir * sign;
                let q_plane = mid + ray * eps;
                if !facet.contains_strict(q_plane) {
                    continue;
                }
                let x_out = q_plane + norm * out_step;
                if solids.iter().any(|s| s.contains(x_out)) {
                    continue;
                }
                let duplicate = rays[..count]
                    .iter()
                    .any(|&(r, n)| r.dot(ray) > 0.99 && n.dot(norm) > 0.99);
                if !duplicate && count < rays.len() {
                    rays[count] = (ray, norm);
                    count += 1;
                }
            }
        }

        if count < 2 {
            return false;
        }
        if count > 2 {
            return true;
        }
        rays[0].1.dot(rays[1].1) <= 0.995 || rays[0].0.dot(rays[1].0) > -0.98
    }

    pub(super) fn visit_segments(&self, mut visitor: impl FnMut(Vec3, Vec3)) {
        for &(a, b) in &self.edges {
            visitor(a, b);
        }
    }
}
