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
        let crown = MeadowCrown::new(self.coat.up, 7);
        let recolored = quad.transfer(|v0, v1, v2, v3, normal, _| {
            if normal.dot(self.coat.up) < -0.9 {
                self.bottom_emitted = true;
            }
            top_surface = normal.dot(self.coat.up) > 0.25;
            let col = if top_surface { self.coat.top_color } else { self.coat.wall_color };
            let glow = if top_surface && crown.verdant(col) { 0.02 } else { 0.0 };
            Quad::glowing(v0, v1, v2, v3, normal, col, glow)
        });
        self.target.push_quad(recolored);
        if top_surface && crown.verdant(self.coat.top_color) {
            recolored.transfer(|v0, v1, v2, v3, normal, col| {
                crown.emit_hybrid([v0, v1, v2, v3], normal, col, self.target);
            });
        }
    }
}

/// Sculpted 3D low-poly grass pyramid and turf composer atop verdant quads.
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

    fn bilerp(&self, corners: [Vec3; 4], u: f32, v: f32) -> Vec3 {
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
        let up_h = self.up * (height * 0.866_025_4);
        let splay_uv = height * 0.85;
        let r_top = r_uv * 1.18;
        for (wing, &(sy, cy)) in WING_DIRS[yaw_idx & 7].iter().enumerate() {
            let b0 = self.bilerp(corners, fu - cy * r_uv, fv - sy * r_uv);
            let b1 = self.bilerp(corners, fu + cy * r_uv, fv + sy * r_uv);
            let (tu, tv) = (fu - sy * splay_uv, fv + cy * splay_uv);
            let t0 = self.bilerp(corners, tu - cy * r_top, tv - sy * r_top) + up_h;
            let t1 = self.bilerp(corners, tu + cy * r_top, tv + sy * r_top) + up_h;
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
}

/// Decorator over MeshTarget that sprouts natural meadow grass patches atop uncratered verdant quads.
pub struct NaturalMeadowTarget<'a> {
    target: &'a mut dyn MeshTarget,
    crown: MeadowCrown,
}

impl<'a> NaturalMeadowTarget<'a> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(target: &'a mut dyn MeshTarget, crown: MeadowCrown) -> Self {
        Self { target, crown }
    }

    /// Secondary constructor decorating a target for a specific grid cell and surface normal.
    pub fn for_cell(target: &'a mut dyn MeshTarget, cell: crate::block::GridCell, up: Vec3) -> Self {
        Self::new(target, MeadowCrown::for_cell(cell, up))
    }
}

impl MeshTarget for NaturalMeadowTarget<'_> {
    fn push_quad(&mut self, quad: Quad) {
        if quad.radiate(|g| g.abs() < 1e-4) {
            let crown = self.crown;
            quad.transfer(|v0, v1, v2, v3, normal, col| {
                if crown.upward_verdant(normal, col) {
                    crown.emit_natural_patch([v0, v1, v2, v3], col, self.target);
                }
            });
        }
    }
}
