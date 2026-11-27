use glam::{Mat4, Vec3};
use world::{MeshTarget, Quad};

/// Compact `1024 × 1024` near-field directional Sun shadow map resolution (`4 MB` `Depth32Float`).
pub const SUN_SHADOW_MAP_DIM: u32 = 1024;

/// Strict forward point-light ceiling (`<= 16` active lights: `HelmetLamp`, `ArcFurnace`, beacons, thrusters).
pub const MAX_POINT_LIGHTS: usize = 16;

/// GPU-aligned point light (`32 Bytes`, `2` private fields).
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PointLightEntry {
    pos_radius: [f32; 4],
    color_intensity: [f32; 4],
}

impl PointLightEntry {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(pos_radius: [f32; 4], color_intensity: [f32; 4]) -> Self {
        Self { pos_radius, color_intensity }
    }

    pub const fn unlit() -> Self {
        Self::new([0.0; 4], [0.0; 4])
    }

    pub const fn emitter(position: Vec3, radius: f32, color: Vec3, intensity: f32) -> Self {
        Self::new(
            [position.x, position.y, position.z, radius],
            [color.x, color.y, color.z, intensity],
        )
    }

    pub fn position(self) -> Vec3 {
        Vec3::new(self.pos_radius[0], self.pos_radius[1], self.pos_radius[2])
    }

    pub const fn radius(self) -> f32 {
        self.pos_radius[3]
    }
}

/// Bounded `<= 16` dynamic point-light cluster ranked by viewer proximity (`3` private fields).
#[derive(Clone, Debug)]
pub struct PointLightCluster {
    entries: [PointLightEntry; MAX_POINT_LIGHTS],
    active_count: usize,
    viewer: Vec3,
}

impl PointLightCluster {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(
        entries: [PointLightEntry; MAX_POINT_LIGHTS],
        active_count: usize,
        viewer: Vec3,
    ) -> Self {
        Self { entries, active_count, viewer }
    }

    pub const fn centered_on(viewer: Vec3) -> Self {
        Self::new([PointLightEntry::unlit(); MAX_POINT_LIGHTS], 0, viewer)
    }

    pub fn reset(&mut self, viewer: Vec3) {
        self.active_count = 0;
        self.viewer = viewer;
    }

    pub const fn active_count(&self) -> usize {
        self.active_count
    }

    pub const fn entries(&self) -> &[PointLightEntry; MAX_POINT_LIGHTS] {
        &self.entries
    }

    /// Registers a point light if within `96 m`, evicting the farthest light when all `16` slots are occupied.
    pub fn offer(&mut self, candidate: PointLightEntry) {
        let cand_dist_sq = candidate.position().distance_squared(self.viewer);
        if cand_dist_sq > 96.0 * 96.0 || candidate.radius() <= 0.05 {
            return;
        }
        for existing in &self.entries[..self.active_count] {
            if existing.position().distance_squared(candidate.position()) < 0.85 {
                return;
            }
        }
        if self.active_count < MAX_POINT_LIGHTS {
            self.entries[self.active_count] = candidate;
            self.active_count += 1;
            return;
        }
        let mut farthest_idx = 0usize;
        let mut farthest_d2 = -1.0_f32;
        for (idx, light) in self.entries.iter().enumerate() {
            let d2 = light.position().distance_squared(self.viewer);
            if d2 > farthest_d2 {
                farthest_d2 = d2;
                farthest_idx = idx;
            }
        }
        if cand_dist_sq < farthest_d2 {
            self.entries[farthest_idx] = candidate;
        }
    }
}

/// Yegor `MeshTarget` Decorator harvesting `<= 16` dynamic point lights from radiant quads (`2` private fields).
pub struct LuminanceHarvestTarget<'a, T> {
    origin: &'a mut T,
    cluster: &'a mut PointLightCluster,
}

impl<'a, T> LuminanceHarvestTarget<'a, T> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(origin: &'a mut T, cluster: &'a mut PointLightCluster) -> Self {
        Self { origin, cluster }
    }
}

impl<T: MeshTarget> MeshTarget for LuminanceHarvestTarget<'_, T> {
    fn push_quad(&mut self, quad: Quad) {
        let glow = quad.radiate(|g| g);
        if (0.78..1.98).contains(&glow) || (-3.0..-1.5).contains(&glow) {
            quad.transfer(|v0, v1, v2, v3, _, color| {
                let center = (v0 + v1 + v2 + v3) * 0.25;
                let (radius, intensity) = if glow < -1.5 {
                    (14.0, (-glow - 1.5).clamp(0.4, 1.8))
                } else if glow > 1.5 {
                    (11.0, 1.35)
                } else {
                    (7.5, glow)
                };
                self.cluster.offer(PointLightEntry::emitter(center, radius, color, intensity));
            });
        }
        self.origin.push_quad(quad);
    }
}

/// Texel-snapped `1024²` orthographic directional Sun shadow cascade (`2` private fields).
#[derive(Clone, Copy, Debug)]
pub struct SunShadowCascade {
    half_extent: f32,
    depth_span: f32,
}

impl SunShadowCascade {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(half_extent: f32, depth_span: f32) -> Self {
        Self { half_extent, depth_span }
    }

    pub const fn near_field() -> Self {
        Self::new(48.0, 160.0)
    }

    /// Builds a `[0, 1]` WebGPU depth orthographic light-space matrix snapped to `1024²` shadow texels.
    pub fn light_view_proj(self, viewer: Vec3, sun_dir: Vec3) -> Mat4 {
        let dir = sun_dir.try_normalize().unwrap_or(Vec3::Y);
        let up_ref = if dir.y.abs() > 0.94 { Vec3::Z } else { Vec3::Y };
        let light_pos = viewer + dir * (self.depth_span * 0.5);
        let raw_view = Mat4::look_at_rh(light_pos, viewer, up_ref);
        let texel_world = (self.half_extent * 2.0) / (SUN_SHADOW_MAP_DIM as f32);
        let mut proj = Mat4::orthographic_rh(
            -self.half_extent,
            self.half_extent,
            -self.half_extent,
            self.half_extent,
            0.5,
            self.depth_span,
        );
        let origin_ls = raw_view.transform_point3(Vec3::ZERO);
        let snapped_x = (origin_ls.x / texel_world).round() * texel_world;
        let snapped_y = (origin_ls.y / texel_world).round() * texel_world;
        proj.w_axis.x += (snapped_x - origin_ls.x) / self.half_extent;
        proj.w_axis.y += (snapped_y - origin_ls.y) / self.half_extent;
        proj * raw_view
    }
}
