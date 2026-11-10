use glam::Vec3;
use crate::{MeshTarget, Quad};

/// Sculpted 3D multi-depth contour terrace and original 30°-splayed grass-tuft composer atop verdant quads.
#[derive(Clone, Copy, Debug)]
pub struct MeadowCrown {
    up: Vec3,
    cell: crate::block::GridCell,
    seed: i32,
}

impl MeadowCrown {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn with_cell(up: Vec3, cell: crate::block::GridCell, seed: i32) -> Self {
        Self { up, cell, seed }
    }

    /// Secondary constructor initializing at the origin cell with an explicit seed.
    pub const fn new(up: Vec3, seed: i32) -> Self {
        Self::with_cell(up, crate::block::GridCell::new(0, 0), seed)
    }

    /// Secondary constructor deriving a deterministic meadow seed from a discrete grid cell.
    pub fn for_cell(cell: crate::block::GridCell, up: Vec3) -> Self {
        let (u, v) = cell.feed_coordinates(|u, v| (u, v));
        let hash = (u.wrapping_mul(73_856_093) ^ v.wrapping_mul(19_349_663)).abs();
        Self::with_cell(up, cell, hash)
    }

    /// Indicates whether a surface pigment represents living green turf rather than rock, sand, snow, or soil.
    pub fn verdant(&self, color: Vec3) -> bool {
        color.y > color.x * 1.15 && color.y > color.z * 1.15
    }

    /// Indicates whether a quad with `normal` and `color` is an upward-facing living turf surface.
    pub fn upward_verdant(&self, normal: Vec3, color: Vec3) -> bool {
        normal.dot(self.up) > 0.25 && self.verdant(color)
    }

    /// Indicates whether a quad with `normal` is a vertical step wall beneath a living turf crown.
    pub fn vertical_flank(&self, normal: Vec3, top_col: Vec3) -> bool {
        self.verdant(top_col) && normal.dot(self.up).abs() < 0.1
    }

    fn bilerp(&self, corners: [Vec3; 4], u: f32, v: f32) -> Vec3 {
        let [v0, v1, v2, v3] = corners;
        let (cu, cv) = (u.clamp(0.0, 1.0), v.clamp(0.0, 1.0));
        v0 * ((1.0 - cu) * (1.0 - cv))
            + v1 * (cu * (1.0 - cv))
            + v2 * (cu * cv)
            + v3 * ((1.0 - cu) * cv)
    }

    fn depth_tier_at(&self, p: Vec3) -> usize {
        let an = self.up.abs();
        let (wx, wz) = if an.y >= an.x.max(an.z) {
            (p.x, p.z)
        } else if an.x >= an.z {
            (p.y, p.z)
        } else {
            (p.x, p.y)
        };
        let w1 = (wx * 3.8 + wz * 2.7).sin() + (wx * 2.1 - wz * 4.3).cos();
        let w2 = (wx * 6.2 - wz * 5.1).sin() * 0.6 + (wx * 4.7 + wz * 6.4).cos() * 0.5;
        let val = w1 * 0.55 + w2 * 0.45;
        if val < -0.65 { 0 } else if val < -0.10 { 1 } else if val < 0.42 { 2 } else if val < 0.92 { 3 } else { 4 }
    }

    fn tier_style(&self, tier: usize, base_col: Vec3) -> (f32, Vec3, Vec3) {
        const HEIGHTS: [f32; 5] = [0.002, 0.012, 0.024, 0.036, 0.048];
        const SCALES: [Vec3; 5] = [
            Vec3::new(0.78, 0.82, 0.76),
            Vec3::new(0.90, 0.92, 0.88),
            Vec3::new(1.02, 1.00, 0.98),
            Vec3::new(1.22, 1.12, 1.10),
            Vec3::new(1.44, 1.24, 1.25),
        ];
        let idx = tier.min(4);
        let top_c = (base_col * SCALES[idx]).clamp(Vec3::ZERO, Vec3::ONE);
        (HEIGHTS[idx], top_c, (top_c * 0.88).clamp(Vec3::ZERO, Vec3::ONE))
    }

    /// Emits Tiefen-Variante 1: 5-tier 3D contour terraces that flow seamlessly across 0.5m block borders.
    pub fn emit_contour_terraces(&self, corners: [Vec3; 4], normal: Vec3, top_col: Vec3, full_detail: bool, target: &mut dyn MeshTarget) {
        let [v0, v1, v2, v3] = corners;
        if (v1 - v0).length_squared() < 0.08 || (v3 - v0).length_squared() < 0.08 || (v2 - v3).length_squared() < 1e-4 {
            return;
        }
        let n = if full_detail { 4usize } else { 2usize };
        let inv = 1.0 / (n as f32);
        let (nu, nv) = ((v1 - v0).normalize_or_zero(), (v3 - v0).normalize_or_zero());
        let sample_tier = |ix: i32, iz: i32| {
            let (u, v) = ((ix as f32 + 0.5) * inv, (iz as f32 + 0.5) * inv);
            self.depth_tier_at(v0 + (v1 - v0) * u + (v3 - v0) * v)
        };
        for iz in 0..(n as i32) {
            let (v_a, v_b) = ((iz as f32) * inv, ((iz + 1) as f32) * inv);
            let mut ix = 0i32;
            while ix < (n as i32) {
                let tier = sample_tier(ix, iz);
                let mut ix_end = ix + 1;
                while ix_end < (n as i32) && sample_tier(ix_end, iz) == tier {
                    ix_end += 1;
                }
                let (u_a, u_b) = ((ix as f32) * inv, (ix_end as f32) * inv);
                let (dh, top_c, _) = self.tier_style(tier, top_col);
                let lift = self.up * dh;
                let [p0, p1, p2, p3] = [
                    self.bilerp(corners, u_a, v_a) + lift,
                    self.bilerp(corners, u_b, v_a) + lift,
                    self.bilerp(corners, u_b, v_b) + lift,
                    self.bilerp(corners, u_a, v_b) + lift,
                ];
                target.push_quad(Quad::new(p0, p1, p2, p3, normal, top_c));
                ix = ix_end;
            }
            if !full_detail {
                continue;
            }
            for cell_x in 0..(n as i32) {
                let tier = sample_tier(cell_x, iz);
                let (dh, _, wall_c) = self.tier_style(tier, top_col);
                let (u_a, u_b) = ((cell_x as f32) * inv, ((cell_x + 1) as f32) * inv);
                let [b0, b1, b2, b3] = [
                    self.bilerp(corners, u_a, v_a),
                    self.bilerp(corners, u_b, v_a),
                    self.bilerp(corners, u_b, v_b),
                    self.bilerp(corners, u_a, v_b),
                ];
                for (nb_t, e0, e1, wn) in [
                    (sample_tier(cell_x, iz - 1), b0, b1, -nv),
                    (sample_tier(cell_x, iz + 1), b3, b2, nv),
                    (sample_tier(cell_x - 1, iz), b3, b0, -nu),
                    (sample_tier(cell_x + 1, iz), b1, b2, nu),
                ] {
                    if tier > nb_t {
                        let (nb_h, _, _) = self.tier_style(nb_t, top_col);
                        let (low, high) = (self.up * nb_h, self.up * dh);
                        target.push_quad(Quad::new(e0 + low, e1 + low, e1 + high, e0 + high, wn, wall_c));
                    }
                }
            }
        }
    }

    /// Emits a 3-wing cross-billboard star fan whose wings splay outward at a 30° angle from vertical.
    #[allow(clippy::too_many_arguments)]
    fn emit_splayed_cross_fan(
        &self,
        corners: [Vec3; 4],
        fu: f32,
        fv: f32,
        yaw_idx: usize,
        r_uv: f32,
        height: f32,
        seed_base: f32,
        color: Vec3,
        target: &mut dyn MeshTarget,
    ) {
        const WING_DIRS: [[(f32, f32); 3]; 8] = [
            [(0.0000, 1.0000), (0.8660, -0.5000), (-0.8660, -0.5000)],
            [(0.3827, 0.9239), (0.6088, -0.7934), (-0.9914, -0.1305)],
            [(0.7071, 0.7071), (0.2588, -0.9659), (-0.9659, 0.2588)],
            [(0.9239, 0.3827), (-0.1305, -0.9914), (-0.7934, 0.6088)],
            [(1.0000, 0.0000), (-0.5000, -0.8660), (-0.5000, 0.8660)],
            [(0.9239, -0.3827), (-0.7934, -0.6088), (-0.1305, 0.9914)],
            [(0.7071, -0.7071), (-0.9659, -0.2588), (0.2588, 0.9659)],
            [(0.3827, -0.9239), (-0.9914, 0.1305), (0.6088, 0.7934)],
        ];
        let root_pt = self.bilerp(corners, fu, fv);
        let (terrace_dh, _, _) = self.tier_style(self.depth_tier_at(root_pt), color);
        let base_lift = self.up * terrace_dh;
        let up_h = base_lift + self.up * (height * 0.866_025_4);
        let splay_uv = height * 0.85;
        let r_top = r_uv * 1.18;
        for (wing, &(sy, cy)) in WING_DIRS[yaw_idx & 7].iter().enumerate() {
            let b0 = self.bilerp(corners, (fu - cy * r_uv).clamp(0.02, 0.98), (fv - sy * r_uv).clamp(0.02, 0.98)) + base_lift;
            let b1 = self.bilerp(corners, (fu + cy * r_uv).clamp(0.02, 0.98), (fv + sy * r_uv).clamp(0.02, 0.98)) + base_lift;
            let (tu, tv) = (fu - sy * splay_uv, fv + cy * splay_uv);
            let t0 = self.bilerp(corners, (tu - cy * r_top).clamp(0.02, 0.98), (tv - sy * r_top).clamp(0.02, 0.98)) + up_h;
            let t1 = self.bilerp(corners, (tu + cy * r_top).clamp(0.02, 0.98), (tv + sy * r_top).clamp(0.02, 0.98)) + up_h;
            let glow_seed = 2.0 + seed_base + (wing as f32) * 1.37;
            target.push_quad(Quad::glowing(b0, b1, t1, t0, self.up, color, glow_seed));
        }
    }

    /// Emits a dense meadow of 6 30°-splayed cross-billboard fans (18 quads = 126 cutout grass blades) inside `corners`.
    pub fn emit_hybrid(&self, corners: [Vec3; 4], _normal: Vec3, color: Vec3, target: &mut dyn MeshTarget) {
        self.emit_tiered(corners, 1.0, 6, color, target);
    }

    /// Emits a world-anchored natural meadow patch with organic clearings, varied tuft counts, and tall grass islands.
    pub fn emit_natural_patch(&self, corners: [Vec3; 4], color: Vec3, target: &mut dyn MeshTarget) {
        let (gu, gv) = self.cell.feed_coordinates(|u, v| (u, v));
        let tri = |x: f32| 1.0 - ((x - x.floor() - 0.5).abs() * 4.0);
        let jitter = (((gu.wrapping_mul(374_761_393) ^ gv.wrapping_mul(668_265_263)).abs() % 100) as f32) * 0.006 - 0.30;
        let wave = tri((gu as f32) * 0.037 + (gv as f32) * 0.027) * 0.55
            + tri((gu as f32) * 0.018 - (gv as f32) * 0.046 + 0.25) * 0.45
            + jitter;
        if wave < 0.06 {
            return;
        }
        let (clusters, height_scale) = if wave < 0.36 {
            (1, 0.95 + (wave - 0.06) * 1.5)
        } else if wave < 0.64 {
            (2, 1.45 + (wave - 0.36) * 2.2)
        } else {
            (3, 2.10 + (wave - 0.64).min(0.40) * 2.0)
        };
        self.emit_tiered(corners, height_scale, clusters, color, target);
    }

    /// Emits a 30°-splayed 3-wing star-fan meadow tier (`cluster_count` in `1..=6`) inside `corners`.
    pub fn emit_tiered(&self, corners: [Vec3; 4], height_scale: f32, cluster_count: usize, color: Vec3, target: &mut dyn MeshTarget) {
        let [v0, v1, v2, v3] = corners;
        if (v1 - v0).length_squared() < 0.048 || (v3 - v0).length_squared() < 0.048 || (v2 - v3).length_squared() < 0.040 || cluster_count == 0 {
            return;
        }
        let s = self.seed.abs();
        let anchors: [(f32, f32); 6] = [
            (0.26, 0.27), (0.74, 0.25), (0.50, 0.52),
            (0.25, 0.73), (0.73, 0.72), (0.48, 0.24),
        ];
        let count = cluster_count.min(6);
        let r_boost = if count == 1 { 1.42 } else if count == 2 { 1.24 } else { 1.08 };
        for step in 0..count {
            let idx = ((step * 5) + (s as usize)) % 6;
            let (bu, bv) = anchors[idx];
            let h = s.wrapping_add((step as i32) * 1_664_525);
            let ju = ((h % 29) - 14) as f32 * 0.008;
            let jv = (((h / 29) % 29) - 14) as f32 * 0.008;
            let (fu, fv) = ((bu + ju).clamp(0.16, 0.84), (bv + jv).clamp(0.16, 0.84));
            let yaw_idx = (h.abs() as usize) & 7;
            let size_jitter = 0.82 + ((h.abs() % 37) as f32) * 0.012;
            let seed_base = ((h.abs() % 89) as f32) * 0.11;
            let (r_uv, h_raw, tone) = match (step + (s as usize)) % 3 {
                0 => (0.225 * r_boost, 0.142, Vec3::new(1.06, 1.12, 0.86)),
                1 => (0.195 * r_boost, 0.102, Vec3::new(0.96, 1.03, 0.85)),
                _ => (0.245 * r_boost, 0.072, Vec3::new(0.86, 0.95, 0.80)),
            };
            let col = (color * tone).clamp(Vec3::ZERO, Vec3::ONE);
            self.emit_splayed_cross_fan(corners, fu, fv, yaw_idx, r_uv, h_raw * size_jitter * height_scale, seed_base, col, target);
        }
    }

    /// Emits overhanging 3D green turf rim (`Grasnarben-Rand`) and soil clods along the top edge of a world-cube step wall quad.
    pub fn emit_wall_turf(&self, corners: [Vec3; 4], normal: Vec3, top_col: Vec3, full_boxes: bool, target: &mut dyn MeshTarget) {
        crate::block::EarthClods::new(self.cell, 0.0).emit_wall_turf(corners, self.up, normal, top_col, full_boxes, target);
    }
}
