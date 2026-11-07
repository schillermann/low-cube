use glam::Vec3;
use crate::{MeshTarget, Quad};

/// Decorator counting the number of quads pushed to an underlying target.
pub struct CountedMeshTarget<'a> {
    target: &'a mut dyn MeshTarget,
    count: usize,
}

impl<'a> CountedMeshTarget<'a> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(target: &'a mut dyn MeshTarget) -> Self {
        Self { target, count: 0 }
    }

    /// Number of quads successfully pushed through this decorator.
    pub fn count(&self) -> usize {
        self.count
    }
}

impl<'a> MeshTarget for CountedMeshTarget<'a> {
    fn push_quad(&mut self, quad: Quad) {
        self.count += 1;
        self.target.push_quad(quad);
    }
}

/// Decorator subdividing pushed quads into recessed 3D digging craters for stages 1..=3.
pub struct CrateredMeshTarget<'a> {
    target: &'a mut dyn MeshTarget,
    stage: u8,
}

impl<'a> CrateredMeshTarget<'a> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(target: &'a mut dyn MeshTarget, stage: u8) -> Self {
        Self { target, stage }
    }
}

impl MeshTarget for CrateredMeshTarget<'_> {
    fn push_quad(&mut self, quad: Quad) {
        quad.emit_crater(self.stage, self.target);
    }
}

struct TurfCoat {
    up: Vec3,
    top_color: Vec3,
    wall_color: Vec3,
}

/// Decorator coloring upward canopy faces and vertical walls while sealing unemitted undersides.
pub struct TurfMeshTarget<'a> {
    target: &'a mut dyn MeshTarget,
    coat: TurfCoat,
    base_corners: [Vec3; 4],
    bottom_emitted: bool,
}

impl<'a> TurfMeshTarget<'a> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(
        target: &'a mut dyn MeshTarget,
        up: Vec3,
        base_corners: [Vec3; 4],
        top_color: Vec3,
        wall_color: Vec3,
        bottom_emitted: bool,
    ) -> Self {
        Self {
            target,
            coat: TurfCoat { up, top_color, wall_color },
            base_corners,
            bottom_emitted,
        }
    }

    /// Emits a sealing underside quad if the decorated shape did not emit its own bottom face.
    pub fn seal_underside(&mut self) {
        if !self.bottom_emitted {
            let [b0, b1, b2, b3] = self.base_corners;
            self.target.push_quad(Quad::new(b0, b3, b2, b1, -self.coat.up, self.coat.wall_color));
            self.bottom_emitted = true;
        }
    }
}

impl MeshTarget for TurfMeshTarget<'_> {
    fn push_quad(&mut self, quad: Quad) {
        let mut top_surface = false;
        let recolored = quad.transfer(|v0, v1, v2, v3, normal, _| {
            if normal.dot(self.coat.up) < -0.9 {
                self.bottom_emitted = true;
            }
            top_surface = normal.dot(self.coat.up) > 0.25;
            let col = if top_surface { self.coat.top_color } else { self.coat.wall_color };
            let glow = if top_surface && MeadowCrown::verdant(col) { 0.02 } else { 0.0 };
            Quad::glowing(v0, v1, v2, v3, normal, col, glow)
        });
        self.target.push_quad(recolored);
        if top_surface && MeadowCrown::verdant(self.coat.top_color) {
            recolored.transfer(|v0, v1, v2, v3, normal, col| {
                MeadowCrown::new(self.coat.up, 7).emit_hybrid([v0, v1, v2, v3], normal, col, self.target);
            });
        }
    }
}

/// Sculpted 3D low-poly grass pyramid and turf composer atop verdant quads.
#[derive(Clone, Copy, Debug)]
pub struct MeadowCrown {
    up: Vec3,
    seed: i32,
}

impl MeadowCrown {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(up: Vec3, seed: i32) -> Self {
        Self { up, seed }
    }

    /// Secondary constructor deriving a deterministic meadow seed from a discrete grid cell.
    pub fn for_cell(cell: crate::block::GridCell, up: Vec3) -> Self {
        let (u, v) = cell.feed_coordinates(|u, v| (u, v));
        let hash = (u.wrapping_mul(73_856_093) ^ v.wrapping_mul(19_349_663)).abs();
        Self::new(up, hash)
    }

    /// Indicates whether a surface pigment represents living green turf rather than rock, sand, snow, or soil.
    pub fn verdant(color: Vec3) -> bool {
        color.y > color.x * 1.15 && color.y > color.z * 1.15
    }

    fn bilerp(corners: [Vec3; 4], u: f32, v: f32) -> Vec3 {
        let [v0, v1, v2, v3] = corners;
        let (cu, cv) = (u.clamp(0.02, 0.98), v.clamp(0.02, 0.98));
        v0 * ((1.0 - cu) * (1.0 - cv))
            + v1 * (cu * (1.0 - cv))
            + v2 * (cu * cv)
            + v3 * ((1.0 - cu) * cv)
    }

    /// Variant 1 (Tall): 2-segment curving grass blade with a steeper lower stem and an overarching tip segment.
    #[allow(clippy::too_many_arguments)]
    fn emit_arched_blade(
        &self, corners: [Vec3; 4], fu: f32, fv: f32, yaw: f32, height: f32, color: Vec3, target: &mut dyn MeshTarget,
    ) {
        let (sy, cy) = yaw.sin_cos();
        let (wx, wy, lx, ly) = (cy * 0.046, sy * 0.046, -sy, cy);
        let b0 = Self::bilerp(corners, fu - wx, fv - wy);
        let b1 = Self::bilerp(corners, fu + wx, fv + wy);
        let m_center = Self::bilerp(corners, fu + lx * 0.065, fv + ly * 0.065) + self.up * (height * 0.56);
        let m_off = (b1 - b0) * 0.42;
        let (m0, m1) = (m_center - m_off, m_center + m_off);
        let t_center = Self::bilerp(corners, fu + lx * 0.195, fv + ly * 0.195) + self.up * height;
        let t_off = (b1 - b0) * 0.18;
        let (t0, t1) = (t_center - t_off, t_center + t_off);

        let n_low = ((b1 - b0).cross(m0 - b0).normalize_or_zero() + self.up * 0.65).normalize_or_zero();
        let n_high = ((m1 - m0).cross(t0 - m0).normalize_or_zero() + self.up * 0.85).normalize_or_zero();
        let c_low = (color * Vec3::new(0.86, 0.95, 0.82)).clamp(Vec3::ZERO, Vec3::ONE);
        let c_high = (color * Vec3::new(1.16, 1.18, 0.84)).clamp(Vec3::ZERO, Vec3::ONE);
        target.push_quad(Quad::glowing(b0, b1, m1, m0, n_low, c_low, 2.0));
        target.push_quad(Quad::glowing(m0, m1, t1, t0, n_high, c_high, 2.0));
    }

    /// Variant 2 (Medium): 3-blade fanned grass tuft radiating outward from a shared root cluster.
    #[allow(clippy::too_many_arguments)]
    fn emit_fan_tuft(
        &self, corners: [Vec3; 4], fu: f32, fv: f32, yaw: f32, height: f32, color: Vec3, target: &mut dyn MeshTarget,
    ) {
        for (idx, (da, h_mul, lean, tone)) in [
            (0.0f32, 1.00f32, 0.165f32, Vec3::new(1.08, 1.12, 0.88)),
            (2.05, 0.84, 0.150, Vec3::new(0.94, 1.02, 0.86)),
            (4.15, 0.74, 0.135, Vec3::new(1.02, 1.06, 0.90)),
        ]
        .into_iter()
        .enumerate()
        {
            let (sy, cy) = (yaw + da).sin_cos();
            let root_u = fu + cy * 0.022 * (idx as f32 - 1.0);
            let root_v = fv + sy * 0.022 * (idx as f32 - 1.0);
            let b0 = Self::bilerp(corners, root_u - cy * 0.044, root_v - sy * 0.044);
            let b1 = Self::bilerp(corners, root_u + cy * 0.044, root_v + sy * 0.044);
            let tip_c = Self::bilerp(corners, root_u - sy * lean, root_v + cy * lean) + self.up * (height * h_mul);
            let t_off = (b1 - b0) * 0.24;
            let norm = ((b1 - b0).cross(tip_c - b0).normalize_or_zero() + self.up * 0.75).normalize_or_zero();
            let col = (color * tone).clamp(Vec3::ZERO, Vec3::ONE);
            target.push_quad(Quad::glowing(b0, b1, tip_c + t_off, tip_c - t_off, norm, col, 2.0));
        }
    }

    /// Variant 3 (Short & Wide): Broad V-folded ground sprout with two wide, low-leaning leaves.
    #[allow(clippy::too_many_arguments)]
    fn emit_broad_sprout(
        &self, corners: [Vec3; 4], fu: f32, fv: f32, yaw: f32, height: f32, color: Vec3, target: &mut dyn MeshTarget,
    ) {
        for (da, h_mul, tone) in [
            (0.0f32, 1.0f32, Vec3::new(0.90, 0.98, 0.82)),
            (2.85, 0.82, Vec3::new(0.82, 0.92, 0.78)),
        ] {
            let (sy, cy) = (yaw + da).sin_cos();
            let b0 = Self::bilerp(corners, fu - cy * 0.060, fv - sy * 0.060);
            let b1 = Self::bilerp(corners, fu + cy * 0.060, fv + sy * 0.060);
            let tip_c = Self::bilerp(corners, fu - sy * 0.185, fv + cy * 0.185) + self.up * (height * h_mul);
            let t_off = (b1 - b0) * 0.34;
            let norm = ((b1 - b0).cross(tip_c - b0).normalize_or_zero() + self.up * 0.80).normalize_or_zero();
            let col = (color * tone).clamp(Vec3::ZERO, Vec3::ONE);
            target.push_quad(Quad::glowing(b0, b1, tip_c + t_off, tip_c - t_off, norm, col, 2.0));
        }
    }

    /// Emits a dense meadow of 27 clusters (54 individual grass blades in 3 distinct shapes and sizes) inside `corners`.
    pub fn emit_hybrid(&self, corners: [Vec3; 4], _normal: Vec3, color: Vec3, target: &mut dyn MeshTarget) {
        self.emit_tiered(corners, 1.0, 27, color, target);
    }

    /// Emits a distance-scaled meadow tier (`cluster_count` in `3`, `9`, or `27`) inside `corners` in O(1) local camera space.
    pub fn emit_tiered(&self, corners: [Vec3; 4], height_scale: f32, cluster_count: usize, color: Vec3, target: &mut dyn MeshTarget) {
        let [v0, v1, v2, v3] = corners;
        let (span_u, span_v) = ((v1 - v0).length(), (v3 - v0).length());
        if span_u < 0.22 || span_v < 0.22 || (v2 - v3).length() < 0.20 {
            return;
        }
        let scale = (span_u.min(span_v) / 0.5).clamp(0.15, 1.0) * height_scale;
        let s = self.seed.abs();
        let count = cluster_count.min(27);
        let stride = (27 / count.max(1)).max(1);
        for step in 0..count {
            let idx = (step * stride) % 27;
            let (row, col_idx) = (idx / 5, idx % 5);
            let (bu, bv) = if idx < 25 {
                (0.12 + (col_idx as f32) * 0.19, 0.12 + (row as f32) * 0.19)
            } else {
                (0.31 + ((idx - 25) as f32) * 0.38, 0.50)
            };
            let h = s.wrapping_add((idx as i32) * 1_664_525);
            let ju = ((h % 19) - 9) as f32 * 0.0048;
            let jv = (((h / 19) % 19) - 9) as f32 * 0.0048;
            let (fu, fv) = ((bu + ju).clamp(0.08, 0.92), (bv + jv).clamp(0.08, 0.92));
            let yaw = ((h % 360) as f32).to_radians();
            let size_jitter = 0.86 + ((h.abs() % 29) as f32) * 0.01;
            match (step + (s as usize)) % 3 {
                0 => self.emit_arched_blade(corners, fu, fv, yaw, 0.112 * size_jitter * scale, color, target),
                1 => self.emit_fan_tuft(corners, fu, fv, yaw, 0.078 * size_jitter * scale, color, target),
                _ => self.emit_broad_sprout(corners, fu, fv, yaw, 0.048 * size_jitter * scale, color, target),
            }
        }
    }
}
