mod binds;
mod canvas;
mod layouts;
mod pipelines;
mod scratch;
mod storage_arena;
mod targets;
mod uniforms;

use glam::{Mat4, Vec3};
use crate::atmosphere_lut::{SKY_LUT_HEIGHT, SKY_LUT_WIDTH};
use crate::lighting::{PointLightCluster, SunShadowCascade};
use crate::power_profile::GpuPowerProfile;
use crate::quad_batch::QuadBatch;
use crate::upload_ring::BufferSlot;
use crate::vram_ledger::{StrictVramGuard, VramLedger};
use crate::{Mesh, SlotReclamation};
use self::binds::{PolaroidReadback, WgpuBindGroups, WgpuDeviceQueue};
use self::layouts::SceneBindLayouts;
use self::pipelines::StagePipelines;
use self::storage_arena::WgpuBufferArena;
use self::targets::{WgpuTargets, MONITOR_ATLAS_SLOTS};
use self::uniforms::{AlignedMonitorUniform, SceneUniformBlock};

pub use self::canvas::PassDraft;
pub use self::targets::{MONITOR_TILE_H, MONITOR_TILE_W};
pub use self::uniforms::FirmamentUniform;

/// Complete `wgpu` rendering stage with Optimus GPU selection, `< 250 MB` VRAM guard, and `<= 1.5 MB/Frame` PCIe ring (`4` fields).
pub struct WgpuStage {
    core: WgpuDeviceQueue,
    pipelines: StagePipelines,
    targets: WgpuTargets,
    arena_binds: (WgpuBufferArena, WgpuBindGroups),
}

impl WgpuStage {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(
        core: WgpuDeviceQueue,
        pipelines: StagePipelines,
        targets: WgpuTargets,
        arena_binds: (WgpuBufferArena, WgpuBindGroups),
    ) -> Self {
        Self { core, pipelines, targets, arena_binds }
    }

    pub fn initialize_for_surface(
        instance: &wgpu::Instance,
        surface: &wgpu::Surface<'static>,
        profile: GpuPowerProfile,
        dimensions: (u32, u32),
    ) -> (Self, wgpu::SurfaceConfiguration) {
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: profile.wgpu_preference(),
            compatible_surface: Some(surface),
            force_fallback_adapter: false,
        }))
        .expect("Failed to acquire wgpu Vulkan/DX12 adapter");

        let (device, queue) = pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: Some("low_cube_wgpu_device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                memory_hints: wgpu::MemoryHints::MemoryUsage,
            },
            None,
        ))
        .expect("Failed to open wgpu device");

        let caps = surface.get_capabilities(&adapter);
        let format = caps.formats.iter().copied().find(|f| !f.is_srgb()).unwrap_or(caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: dimensions.0.max(1),
            height: dimensions.1.max(1),
            present_mode: profile.paced_present_mode(&caps.present_modes),
            alpha_mode: caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        let mut vram = StrictVramGuard::new(VramLedger::thinkpad_250mb());
        let layouts = SceneBindLayouts::assembled(&device);
        let pipelines = StagePipelines::assembled(&device, &layouts, format);
        let targets = WgpuTargets::allocate(&device, (config.width, config.height), format, &mut vram);
        let arena = WgpuBufferArena::allocate(&device, &queue, &layouts, &mut vram);
        let binds = WgpuBindGroups::allocate(&device, layouts, &targets, &mut vram);
        (Self::new(WgpuDeviceQueue::new(device, queue, vram, format), pipelines, targets, (arena, binds)), config)
    }

    pub const fn device(&self) -> &wgpu::Device { self.core.device() }

    pub fn resize_targets(&mut self, width: u32, height: u32) {
        let (dev, _, vram) = self.core.split_mut();
        self.targets.resize(dev, width, height, vram);
    }

    pub fn begin_frame_uploads(&mut self) {
        self.arena_binds.0.begin_frame(self.core.queue());
    }

    pub fn upload_patch(&mut self, batch: &QuadBatch) -> Box<dyn Mesh> {
        let (dev, queue, vram) = self.core.split_mut();
        Box::new(self.arena_binds.0.upload_patch(dev, queue, self.arena_binds.1.layouts(), vram, batch))
    }

    fn upload_lut_if_present(&self, draft: &mut PassDraft) {
        if let Some(bytes) = draft.state_mut().take_lut() {
            self.core.queue().write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: self.targets.shadow_lut().lut_texture(),
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                &bytes,
                wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(SKY_LUT_WIDTH * 4), rows_per_image: Some(SKY_LUT_HEIGHT) },
                wgpu::Extent3d { width: SKY_LUT_WIDTH, height: SKY_LUT_HEIGHT, depth_or_array_layers: 1 },
            );
        }
    }

    /// Submits a recorded main scene `PassDraft` (with `1024²` Sun Shadow Pass) onto `target_view`.
    pub fn submit_main_draft(
        &self,
        target_view: &wgpu::TextureView,
        draft: &mut PassDraft,
        viewer_up_haze: (Vec3, Vec3, f32),
        lights: &PointLightCluster,
    ) {
        self.upload_lut_if_present(draft);
        let (vp, sun_dir) = draft.state_mut().camera();
        let light_vp = SunShadowCascade::near_field().light_view_proj(viewer_up_haze.0, sun_dir);
        let globals = SceneUniformBlock::composed((vp, light_vp), (sun_dir, 1.0), viewer_up_haze, lights);
        let queue = self.core.queue();
        let binds = &self.arena_binds.1;
        binds.write_scene(queue, globals.raw_bytes());
        binds.write_firmament(queue, draft.state_mut().firmament().raw_bytes());
        draft.flush_streams(queue, &self.arena_binds.0);

        let mut enc = self.core.device().create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("main_enc") });
        draft.execute_shadow_pass(&mut enc, self.targets.shadow_lut().shadow_view(), &self.pipelines, binds.scene_bg(), &self.arena_binds.0);
        draft.execute_color_pass(
            &mut enc,
            (target_view, self.targets.main_depth()),
            (false, None),
            &self.pipelines,
            (binds.scene_bg(), binds.firmament_bg()),
            &self.arena_binds.0,
        );
        queue.submit(core::iter::once(enc.finish()));
    }

    /// Renders AT MOST ONE `240 × 160` shadow-free phosphor monitor tile (`tile_idx`) and copies it into the atlas.
    pub fn submit_monitor_tile(&self, tile_idx: u32, draft: &mut PassDraft, viewer_up: (Vec3, Vec3), lights: &PointLightCluster) {
        self.upload_lut_if_present(draft);
        let (vp, sun_dir) = draft.state_mut().camera();
        let globals = SceneUniformBlock::composed((vp, Mat4::IDENTITY), (sun_dir, 0.0), (viewer_up.0, viewer_up.1, 0.0), lights);
        let queue = self.core.queue();
        let binds = &self.arena_binds.1;
        binds.write_scene(queue, globals.raw_bytes());
        binds.write_firmament(queue, draft.state_mut().firmament().raw_bytes());
        draft.flush_streams(queue, &self.arena_binds.0);

        let mon = self.targets.monitors();
        let mut enc = self.core.device().create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("mon_tile_enc") });
        draft.execute_color_pass(
            &mut enc,
            (mon.tile_color_view(), mon.tile_depth_view()),
            (false, None),
            &self.pipelines,
            (binds.scene_bg(), binds.firmament_bg()),
            &self.arena_binds.0,
        );
        enc.copy_texture_to_texture(
            wgpu::TexelCopyTextureInfo { texture: mon.tile_color_texture(), mip_level: 0, origin: wgpu::Origin3d::ZERO, aspect: wgpu::TextureAspect::All },
            wgpu::TexelCopyTextureInfo {
                texture: mon.atlas_texture(),
                mip_level: 0,
                origin: wgpu::Origin3d { x: (tile_idx % MONITOR_ATLAS_SLOTS) * MONITOR_TILE_W, y: 0, z: 0 },
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::Extent3d { width: MONITOR_TILE_W, height: MONITOR_TILE_H, depth_or_array_layers: 1 },
        );
        queue.submit(core::iter::once(enc.finish()));
    }

    /// Blits active `240 × 160` phosphor monitor tiles onto `target_view`.
    pub fn blit_monitor_strip(&self, target_view: &wgpu::TextureView, tiles: &[([f32; 4], [f32; 2], f32)]) {
        let cap = tiles.len().min(MONITOR_ATLAS_SLOTS as usize);
        if cap == 0 { return; }
        let aligned: Vec<AlignedMonitorUniform> = tiles[..cap].iter().map(|&(r, u, p)| AlignedMonitorUniform::tile(r, u, p)).collect();
        let bytes = unsafe {
            core::slice::from_raw_parts(aligned.as_ptr().cast::<u8>(), cap * core::mem::size_of::<AlignedMonitorUniform>())
        };
        let queue = self.core.queue();
        let binds = &self.arena_binds.1;
        binds.write_monitors(queue, bytes);

        let mut enc = self.core.device().create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("mon_blit_enc") });
        {
            let mut rp = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("mon_blit_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: target_view,
                    resolve_target: None,
                    ops: wgpu::Operations { load: wgpu::LoadOp::Load, store: wgpu::StoreOp::Store },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            rp.set_pipeline(self.pipelines.monitor_blit());
            for idx in 0..cap {
                rp.set_bind_group(0, binds.monitor_bg(), &[(idx as u32) * 256]);
                rp.draw(0..6, 0..1);
            }
        }
        queue.submit(core::iter::once(enc.finish()));
    }

    /// Submits an overlay/intercom `PassDraft` onto `target_view` preserving prior color.
    pub fn submit_overlay_draft(&self, target_view: &wgpu::TextureView, scissor: Option<[u32; 4]>, draft: &mut PassDraft, lights: &PointLightCluster) {
        let (vp, sun_dir) = draft.state_mut().camera();
        let globals = SceneUniformBlock::composed((vp, Mat4::IDENTITY), (sun_dir, 0.0), (Vec3::ZERO, Vec3::Y, 0.0), lights);
        let queue = self.core.queue();
        let binds = &self.arena_binds.1;
        binds.write_scene(queue, globals.raw_bytes());
        draft.flush_streams(queue, &self.arena_binds.0);

        let mut enc = self.core.device().create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("overlay_enc") });
        draft.execute_color_pass(
            &mut enc,
            (target_view, self.targets.main_depth()),
            (true, scissor),
            &self.pipelines,
            (binds.scene_bg(), binds.firmament_bg()),
            &self.arena_binds.0,
        );
        queue.submit(core::iter::once(enc.finish()));
    }

    /// Renders `draft` into an offscreen Polaroid plate and reads back bottom-up RGBA8 bytes.
    pub fn capture_polaroid_rgba(&self, (w, h): (u32, u32), draft: &mut PassDraft, viewer_up_haze: (Vec3, Vec3, f32), lights: &PointLightCluster) -> Vec<u8> {
        let (pw, ph) = (w.max(1), h.max(1));
        let fmt = self.core.color_format();
        let tex = self.core.device().create_texture(&wgpu::TextureDescriptor {
            label: Some("polaroid_target"),
            size: wgpu::Extent3d { width: pw, height: ph, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: fmt,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let view = tex.create_view(&wgpu::TextureViewDescriptor::default());
        self.submit_main_draft(&view, draft, viewer_up_haze, lights);
        let bgra = matches!(fmt, wgpu::TextureFormat::Bgra8Unorm | wgpu::TextureFormat::Bgra8UnormSrgb);
        let rgba = PolaroidReadback::new(pw, ph).read_bottom_up_rgba(self.core.device(), self.core.queue(), &tex, bgra);
        tex.destroy();
        rgba
    }
}

impl SlotReclamation for WgpuStage {
    fn reclaim_slot(&mut self, slot: BufferSlot) {
        self.arena_binds.0.reclaim(slot, self.core.vram_mut());
    }
}
