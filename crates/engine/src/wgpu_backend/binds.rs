use std::num::NonZeroU64;
use crate::vram_ledger::{StrictVramGuard, VramAccount, VramLedger};
use super::layouts::SceneBindLayouts;
use super::targets::{WgpuTargets, MONITOR_ATLAS_SLOTS};
use super::uniforms::{AlignedMonitorUniform, FirmamentUniform, SceneUniformBlock};

/// Encapsulated `wgpu` device, queue, surface format, and strict `< 250 MB` `VramLedger` (`4` strictly private fields).
pub struct WgpuDeviceQueue {
    device: wgpu::Device,
    queue: wgpu::Queue,
    vram: StrictVramGuard<VramLedger>,
    color_format: wgpu::TextureFormat,
}

impl WgpuDeviceQueue {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(
        device: wgpu::Device,
        queue: wgpu::Queue,
        vram: StrictVramGuard<VramLedger>,
        color_format: wgpu::TextureFormat,
    ) -> Self {
        Self { device, queue, vram, color_format }
    }

    pub const fn device(&self) -> &wgpu::Device { &self.device }
    pub const fn queue(&self) -> &wgpu::Queue { &self.queue }
    pub const fn color_format(&self) -> wgpu::TextureFormat { self.color_format }
    pub fn vram_mut(&mut self) -> &mut dyn VramAccount { &mut self.vram }
    pub fn split_mut(&mut self) -> (&wgpu::Device, &wgpu::Queue, &mut dyn VramAccount) {
        (&self.device, &self.queue, &mut self.vram)
    }
}

/// Global `@group(0)` uniform buffers and bind groups (`4` strictly private fields).
pub struct WgpuBindGroups {
    scene: (wgpu::Buffer, wgpu::BindGroup),
    firmament: (wgpu::Buffer, wgpu::BindGroup),
    monitor: (wgpu::Buffer, wgpu::BindGroup),
    layouts: SceneBindLayouts,
}

impl WgpuBindGroups {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(
        scene: (wgpu::Buffer, wgpu::BindGroup),
        firmament: (wgpu::Buffer, wgpu::BindGroup),
        monitor: (wgpu::Buffer, wgpu::BindGroup),
        layouts: SceneBindLayouts,
    ) -> Self {
        Self { scene, firmament, monitor, layouts }
    }

    pub const fn layouts(&self) -> &SceneBindLayouts { &self.layouts }
    pub const fn scene_bg(&self) -> &wgpu::BindGroup { &self.scene.1 }
    pub const fn firmament_bg(&self) -> &wgpu::BindGroup { &self.firmament.1 }
    pub const fn monitor_bg(&self) -> &wgpu::BindGroup { &self.monitor.1 }
    pub fn write_scene(&self, queue: &wgpu::Queue, bytes: &[u8]) { queue.write_buffer(&self.scene.0, 0, bytes); }
    pub fn write_firmament(&self, queue: &wgpu::Queue, bytes: &[u8]) { queue.write_buffer(&self.firmament.0, 0, bytes); }
    pub fn write_monitors(&self, queue: &wgpu::Queue, bytes: &[u8]) { queue.write_buffer(&self.monitor.0, 0, bytes); }

    pub fn allocate(
        device: &wgpu::Device,
        layouts: SceneBindLayouts,
        targets: &WgpuTargets,
        vram: &mut dyn VramAccount,
    ) -> Self {
        let sc_sz = core::mem::size_of::<SceneUniformBlock>() as u64;
        let fi_sz = core::mem::size_of::<FirmamentUniform>() as u64;
        let mo_sz = (MONITOR_ATLAS_SLOTS as u64) * (core::mem::size_of::<AlignedMonitorUniform>() as u64);
        vram.credit_allocation(sc_sz + fi_sz + mo_sz);

        let sc_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("scene_globals_ub"),
            size: sc_sz,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let sc_bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("scene_globals_bg"),
            layout: layouts.scene(),
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: sc_buf.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(targets.shadow_lut().shadow_view()) },
                wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::Sampler(targets.shadow_sampler()) },
                wgpu::BindGroupEntry { binding: 3, resource: wgpu::BindingResource::TextureView(targets.shadow_lut().lut_view()) },
                wgpu::BindGroupEntry { binding: 4, resource: wgpu::BindingResource::Sampler(targets.linear_sampler()) },
            ],
        });

        let fi_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("firmament_ub"),
            size: fi_sz,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let fi_bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("firmament_bg"),
            layout: layouts.firmament(),
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: fi_buf.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(targets.shadow_lut().lut_view()) },
                wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::Sampler(targets.linear_sampler()) },
            ],
        });

        let mo_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("monitor_tiles_ub"),
            size: mo_sz,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let mo_bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("monitor_tiles_bg"),
            layout: layouts.monitor(),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &mo_buf,
                        offset: 0,
                        size: NonZeroU64::new(32),
                    }),
                },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(targets.monitors().atlas_view()) },
                wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::Sampler(targets.linear_sampler()) },
            ],
        });
        Self::new((sc_buf, sc_bg), (fi_buf, fi_bg), (mo_buf, mo_bg), layouts)
    }
}

/// Ephemeral Polaroid shutter readback extracting bottom-up RGBA8 pixels from an offscreen `wgpu::Texture` (`2` fields).
pub struct PolaroidReadback {
    width: u32,
    height: u32,
}

impl PolaroidReadback {
    pub const fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }

    pub fn read_bottom_up_rgba(self, device: &wgpu::Device, queue: &wgpu::Queue, tex: &wgpu::Texture, bgra: bool) -> Vec<u8> {
        let row_bytes = self.width * 4;
        let padded_row = (row_bytes + 255) & !255;
        let buf_size = (padded_row as u64) * (self.height as u64);
        let staging = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("polaroid_readback"),
            size: buf_size,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let mut enc = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("polaroid_copy") });
        enc.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo { texture: tex, mip_level: 0, origin: wgpu::Origin3d::ZERO, aspect: wgpu::TextureAspect::All },
            wgpu::TexelCopyBufferInfo {
                buffer: &staging,
                layout: wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(padded_row), rows_per_image: Some(self.height) },
            },
            wgpu::Extent3d { width: self.width, height: self.height, depth_or_array_layers: 1 },
        );
        queue.submit(core::iter::once(enc.finish()));
        let slice = staging.slice(..);
        slice.map_async(wgpu::MapMode::Read, |_| {});
        device.poll(wgpu::Maintain::Wait);
        let mapped = slice.get_mapped_range();
        let mut out = vec![0u8; (row_bytes * self.height) as usize];
        for y in 0..self.height {
            let src_y = self.height - 1 - y;
            let src = &mapped[(src_y * padded_row) as usize..(src_y * padded_row + row_bytes) as usize];
            let dst = &mut out[(y * row_bytes) as usize..((y + 1) * row_bytes) as usize];
            dst.copy_from_slice(src);
            if bgra {
                for px in dst.chunks_exact_mut(4) {
                    px.swap(0, 2);
                }
            }
        }
        drop(mapped);
        staging.unmap();
        staging.destroy();
        out
    }
}
