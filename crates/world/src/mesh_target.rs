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

    /// Emits a 3-wing cross-billboard star fan whose wings splay outward at a 30° angle from vertical.
    #[allow(clippy::too_many_arguments)]
    fn emit_splayed_cross_fan(
        &self,
        corners: [Vec3; 4],
        fu: f32,
        fv: f32,
        yaw: f32,
        r_uv: f32,
        height: f32,
        seed_base: f32,
        color: Vec3,
        target: &mut dyn MeshTarget,
    ) {
        let up_h = self.up * (height * 0.866_025_4);
        let splay_uv = height;
        let r_top = r_uv * 1.18;
        for (wing, da) in [0.0f32, 2.094_395, 4.188_79].into_iter().enumerate() {
            let (sy, cy) = (yaw + da).sin_cos();
            let b0 = Self::bilerp(corners, fu - cy * r_uv, fv - sy * r_uv);
            let b1 = Self::bilerp(corners, fu + cy * r_uv, fv + sy * r_uv);
            let (tu, tv) = (fu - sy * splay_uv, fv + cy * splay_uv);
            let t0 = Self::bilerp(corners, tu - cy * r_top, tv - sy * r_top) + up_h;
            let t1 = Self::bilerp(corners, tu + cy * r_top, tv + sy * r_top) + up_h;
            let norm = ((b1 - b0).cross(t0 - b0).normalize_or_zero() + self.up * 0.65).normalize_or_zero();
            let glow_seed = 2.0 + seed_base + (wing as f32) * 1.37;
            target.push_quad(Quad::glowing(b0, b1, t1, t0, norm, color, glow_seed));
        }
    }

    /// Emits a dense meadow of 6 30°-splayed cross-billboard fans (18 quads = 126 cutout grass blades) inside `corners`.
    pub fn emit_hybrid(&self, corners: [Vec3; 4], _normal: Vec3, color: Vec3, target: &mut dyn MeshTarget) {
        self.emit_tiered(corners, 1.0, 6, color, target);
    }

    /// Emits a distance-scaled 30°-splayed cross-billboard meadow tier (`cluster_count` in `1`, `3`, or `6`) inside `corners`.
    pub fn emit_tiered(&self, corners: [Vec3; 4], height_scale: f32, cluster_count: usize, color: Vec3, target: &mut dyn MeshTarget) {
        let [v0, v1, v2, v3] = corners;
        let (span_u, span_v) = ((v1 - v0).length(), (v3 - v0).length());
        if span_u < 0.22 || span_v < 0.22 || (v2 - v3).length() < 0.20 {
            return;
        }
        let scale = (span_u.min(span_v) / 0.5).clamp(0.15, 1.0) * height_scale;
        let s = self.seed.abs();
        let anchors: [(f32, f32); 6] = [
            (0.25, 0.25), (0.75, 0.26), (0.50, 0.50),
            (0.26, 0.75), (0.74, 0.74), (0.50, 0.22),
        ];
        let count = cluster_count.min(6);
        let stride = (6 / count.max(1)).max(1);
        for step in 0..count {
            let idx = (step * stride) % 6;
            let (bu, bv) = anchors[idx];
            let h = s.wrapping_add((idx as i32) * 1_664_525);
            let ju = ((h % 19) - 9) as f32 * 0.005;
            let jv = (((h / 19) % 19) - 9) as f32 * 0.005;
            let (fu, fv) = ((bu + ju).clamp(0.16, 0.84), (bv + jv).clamp(0.16, 0.84));
            let yaw = ((h % 360) as f32).to_radians();
            let size_jitter = 0.88 + ((h.abs() % 25) as f32) * 0.01;
            let seed_base = ((h.abs() % 89) as f32) * 0.11;
            let (r_uv, h_raw, tone) = match (step + (s as usize)) % 3 {
                0 => (0.215, 0.132, Vec3::new(1.06, 1.10, 0.88)),
                1 => (0.190, 0.096, Vec3::new(0.96, 1.02, 0.86)),
                _ => (0.240, 0.064, Vec3::new(0.86, 0.94, 0.80)),
            };
            let col = (color * tone).clamp(Vec3::ZERO, Vec3::ONE);
            self.emit_splayed_cross_fan(corners, fu, fv, yaw, r_uv, h_raw * size_jitter * scale, seed_base, col, target);
        }
    }
}
