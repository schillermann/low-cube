use glam::{Mat4, Vec3, Vec4};
use crate::instanced_module::ModuleInstance;
use crate::lighting::{PointLightCluster, PointLightEntry, MAX_POINT_LIGHTS};
use crate::quad_batch::QuadBatch;
use crate::upload_ring::BufferSlot;
use crate::{BlendPassMode, FirmamentUniform, FramePass};

const TICKS_PER_METER: f64 = 10_000.0;
const METERS_PER_TICK: f64 = 1.0e-4;

/// Sub-millimeter camera anchor (`[i64; 3]` in `0.1 mm` steps, `1` private field) subtracting world offsets before `f32` GPU handoff.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CameraAnchor {
    eye_ticks: [i64; 3],
}

impl CameraAnchor {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(eye_ticks: [i64; 3]) -> Self {
        Self { eye_ticks }
    }

    pub fn from_eye_f32(eye: Vec3) -> Self {
        Self::new([
            Self::quantize_axis(f64::from(eye.x)),
            Self::quantize_axis(f64::from(eye.y)),
            Self::quantize_axis(f64::from(eye.z)),
        ])
    }

    fn quantize_axis(meters: f64) -> i64 {
        if !meters.is_finite() {
            return 0;
        }
        (meters * TICKS_PER_METER).clamp(-9.0e18, 9.0e18).round() as i64
    }

    pub const fn ticks(self) -> [i64; 3] {
        self.eye_ticks
    }

    pub fn eye_meters_f32(self) -> Vec3 {
        Vec3::new(
            ((self.eye_ticks[0] as f64) * METERS_PER_TICK) as f32,
            ((self.eye_ticks[1] as f64) * METERS_PER_TICK) as f32,
            ((self.eye_ticks[2] as f64) * METERS_PER_TICK) as f32,
        )
    }

    /// Exact `i64` (`0.1 mm`) subtraction `(target_ticks - self.eye_ticks) * 1e-4 -> f32`.
    pub fn relative_from_ticks(self, target_ticks: [i64; 3]) -> Vec3 {
        let dx = target_ticks[0].saturating_sub(self.eye_ticks[0]);
        let dy = target_ticks[1].saturating_sub(self.eye_ticks[1]);
        let dz = target_ticks[2].saturating_sub(self.eye_ticks[2]);
        Vec3::new(
            ((dx as f64) * METERS_PER_TICK) as f32,
            ((dy as f64) * METERS_PER_TICK) as f32,
            ((dz as f64) * METERS_PER_TICK) as f32,
        )
    }

    pub fn relative_from_f32(self, world_pos: Vec3) -> Vec3 {
        let target_ticks = [
            Self::quantize_axis(f64::from(world_pos.x)),
            Self::quantize_axis(f64::from(world_pos.y)),
            Self::quantize_axis(f64::from(world_pos.z)),
        ];
        self.relative_from_ticks(target_ticks)
    }

    /// Translates `view_proj` to a camera-relative `(0, 0, 0)` origin so `rel_vp * (p - anchor) == vp * p`.
    pub fn recenter_view_proj(self, view_proj: Mat4) -> Mat4 {
        view_proj * Mat4::from_translation(self.eye_meters_f32())
    }

    pub fn recenter_model(self, model: Mat4) -> Mat4 {
        let world_t = model.w_axis.truncate();
        let rel_t = self.relative_from_f32(world_t);
        Mat4::from_cols(
            model.x_axis,
            model.y_axis,
            model.z_axis,
            Vec4::new(rel_t.x, rel_t.y, rel_t.z, model.w_axis.w),
        )
    }

    pub fn recenter_lights(self, cluster: &PointLightCluster) -> PointLightCluster {
        let count = cluster.active_count();
        let src = cluster.entries();
        let mut entries = [PointLightEntry::unlit(); MAX_POINT_LIGHTS];
        for i in 0..count {
            entries[i] = src[i].recentered(|pos| self.relative_from_f32(pos));
        }
        PointLightCluster::new(entries, count, Vec3::ZERO)
    }
}

#[derive(Clone, Copy, Debug)]
enum PassSpace {
    Perspective3d(Mat4),
    Screen2d,
}

/// Yegor `FramePass` Decorator anchoring `i64` (`0.1 mm`) camera-relative subtraction before `f32` `wgpu` submission (`3` fields).
pub struct CameraRelativePass<'a> {
    origin: &'a mut dyn FramePass,
    anchor: CameraAnchor,
    space: PassSpace,
}

impl<'a> CameraRelativePass<'a> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    fn with_space(origin: &'a mut dyn FramePass, anchor: CameraAnchor, space: PassSpace) -> Self {
        Self { origin, anchor, space }
    }

    pub fn new(origin: &'a mut dyn FramePass, anchor: CameraAnchor) -> Self {
        Self::with_space(origin, anchor, PassSpace::Perspective3d(Mat4::IDENTITY))
    }
}

impl FramePass for CameraRelativePass<'_> {
    fn clear_sky(&mut self, sky_rgba: Vec4) {
        self.origin.clear_sky(sky_rgba);
    }

    fn clear_depth_layer(&mut self) {
        self.origin.clear_depth_layer();
    }

    fn bind_camera(&mut self, view_proj: Mat4, light_dir: Vec3) {
        if view_proj.z_axis.w < -0.5 {
            self.space = PassSpace::Perspective3d(Mat4::IDENTITY);
            self.origin.bind_camera(self.anchor.recenter_view_proj(view_proj), light_dir);
            self.origin.apply_model(self.anchor.recenter_model(Mat4::IDENTITY));
        } else {
            self.space = PassSpace::Screen2d;
            self.origin.bind_camera(view_proj, light_dir);
        }
    }

    fn apply_model(&mut self, model: Mat4) {
        match self.space {
            PassSpace::Perspective3d(_) => {
                self.space = PassSpace::Perspective3d(model);
                self.origin.apply_model(self.anchor.recenter_model(model));
            }
            PassSpace::Screen2d => self.origin.apply_model(model),
        }
    }

    fn select_blend(&mut self, mode: BlendPassMode) {
        self.origin.select_blend(mode);
    }

    fn upload_sky_lut(&mut self, rgba8_texels: &[u8]) {
        self.origin.upload_sky_lut(rgba8_texels);
    }

    fn draw_sky_lut(&mut self, uniform: FirmamentUniform) {
        self.origin.draw_sky_lut(uniform);
    }

    fn draw_packed_quads(&mut self, slot: BufferSlot, quad_count: u32, patch_origin: [f32; 4]) {
        match self.space {
            PassSpace::Perspective3d(raw_model) if raw_model == Mat4::IDENTITY => {
                let rel = self.anchor.relative_from_f32(Vec3::new(patch_origin[0], patch_origin[1], patch_origin[2]));
                self.origin.apply_model(Mat4::IDENTITY);
                self.origin.draw_packed_quads(slot, quad_count, [rel.x, rel.y, rel.z, patch_origin[3]]);
                self.origin.apply_model(self.anchor.recenter_model(Mat4::IDENTITY));
            }
            _ => self.origin.draw_packed_quads(slot, quad_count, patch_origin),
        }
    }

    fn draw_free_quads(&mut self, slot: BufferSlot, quad_count: u32) {
        self.origin.draw_free_quads(slot, quad_count);
    }

    fn stream_quads(&mut self, batch: &QuadBatch) {
        self.origin.stream_quads(batch);
    }

    fn draw_module_instances(&mut self, instances: &[ModuleInstance]) {
        if instances.is_empty() {
            return;
        }
        match self.space {
            PassSpace::Perspective3d(_) => {
                let shifted: Vec<ModuleInstance> = instances
                    .iter()
                    .map(|inst| inst.recentered(|pos| self.anchor.relative_from_f32(pos)))
                    .collect();
                self.origin.draw_module_instances(&shifted);
            }
            PassSpace::Screen2d => self.origin.draw_module_instances(instances),
        }
    }
}
