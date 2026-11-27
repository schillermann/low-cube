use crate::atmosphere_lut::{SKY_LUT_HEIGHT, SKY_LUT_WIDTH};
use crate::lighting::SUN_SHADOW_MAP_DIM;
use crate::vram_ledger::VramAccount;

/// Resolution (`240 × 160`) of each Round-Robin diegetic phosphor flight monitor tile.
pub const MONITOR_TILE_W: u32 = 240;
pub const MONITOR_TILE_H: u32 = 160;
pub const MONITOR_ATLAS_SLOTS: u32 = 8;

/// Resizable `Depth32Float` attachment tracked in `VramAccount` (`4` private fields).
pub struct DepthAttachment {
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    dimensions: (u32, u32),
    byte_footprint: u64,
}

impl DepthAttachment {
    pub const fn new(
        texture: wgpu::Texture,
        view: wgpu::TextureView,
        dimensions: (u32, u32),
        byte_footprint: u64,
    ) -> Self {
        Self { texture, view, dimensions, byte_footprint }
    }

    pub fn allocate(device: &wgpu::Device, width: u32, height: u32, vram: &mut dyn VramAccount) -> Self {
        let (w, h) = (width.max(1), height.max(1));
        let bytes = (w as u64) * (h as u64) * 4;
        vram.credit_allocation(bytes);
        let tex = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("main_depth32"),
            size: wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        let view = tex.create_view(&wgpu::TextureViewDescriptor::default());
        Self::new(tex, view, (w, h), bytes)
    }

    pub fn resize_if_needed(&mut self, device: &wgpu::Device, width: u32, height: u32, vram: &mut dyn VramAccount) {
        let (w, h) = (width.max(1), height.max(1));
        if self.dimensions == (w, h) {
            return;
        }
        vram.debit_release(self.byte_footprint);
        self.texture.destroy();
        *self = Self::allocate(device, w, h, vram);
    }

    pub const fn view(&self) -> &wgpu::TextureView { &self.view }
}

/// `1024²` Sun shadow map (`4 MB`) + `128 × 64` Atmosphere Sky-LUT (`32 KB`, `4` private fields).
pub struct ShadowLutPair {
    shadow_tex: wgpu::Texture,
    shadow_view: wgpu::TextureView,
    lut_tex: wgpu::Texture,
    lut_view: wgpu::TextureView,
}

impl ShadowLutPair {
    pub const fn new(
        shadow_tex: wgpu::Texture,
        shadow_view: wgpu::TextureView,
        lut_tex: wgpu::Texture,
        lut_view: wgpu::TextureView,
    ) -> Self {
        Self { shadow_tex, shadow_view, lut_tex, lut_view }
    }

    pub fn allocate(device: &wgpu::Device, vram: &mut dyn VramAccount) -> Self {
        let shadow_bytes = (SUN_SHADOW_MAP_DIM as u64) * (SUN_SHADOW_MAP_DIM as u64) * 4;
        let lut_bytes = (SKY_LUT_WIDTH as u64) * (SKY_LUT_HEIGHT as u64) * 4;
        vram.credit_allocation(shadow_bytes + lut_bytes);

        let shadow_tex = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("sun_shadow_1024"),
            size: wgpu::Extent3d { width: SUN_SHADOW_MAP_DIM, height: SUN_SHADOW_MAP_DIM, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let shadow_view = shadow_tex.create_view(&wgpu::TextureViewDescriptor::default());

        let lut_tex = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("atmo_sky_lut_128x64"),
            size: wgpu::Extent3d { width: SKY_LUT_WIDTH, height: SKY_LUT_HEIGHT, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let lut_view = lut_tex.create_view(&wgpu::TextureViewDescriptor::default());
        Self::new(shadow_tex, shadow_view, lut_tex, lut_view)
    }

    pub const fn shadow_view(&self) -> &wgpu::TextureView { &self.shadow_view }
    pub const fn lut_texture(&self) -> &wgpu::Texture { &self.lut_tex }
    pub const fn lut_view(&self) -> &wgpu::TextureView { &self.lut_view }
}

/// Offscreen `240 × 160` Round-Robin monitor tile + `(240 * 8) × 160` phosphor atlas (`3` private fields).
pub struct MonitorTarget {
    atlas: (wgpu::Texture, wgpu::TextureView),
    tile_color: (wgpu::Texture, wgpu::TextureView),
    tile_depth: (wgpu::Texture, wgpu::TextureView),
}

impl MonitorTarget {
    pub const fn new(
        atlas: (wgpu::Texture, wgpu::TextureView),
        tile_color: (wgpu::Texture, wgpu::TextureView),
        tile_depth: (wgpu::Texture, wgpu::TextureView),
    ) -> Self {
        Self { atlas, tile_color, tile_depth }
    }

    pub fn allocate(device: &wgpu::Device, color_fmt: wgpu::TextureFormat, vram: &mut dyn VramAccount) -> Self {
        let aw = MONITOR_TILE_W * MONITOR_ATLAS_SLOTS;
        let ah = MONITOR_TILE_H;
        let bytes = ((aw * ah + MONITOR_TILE_W * MONITOR_TILE_H * 2) as u64) * 4;
        vram.credit_allocation(bytes);

        let atlas_tex = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("phosphor_monitor_atlas"),
            size: wgpu::Extent3d { width: aw, height: ah, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: color_fmt,
            usage: wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let tile_c = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("phosphor_tile_color"),
            size: wgpu::Extent3d { width: MONITOR_TILE_W, height: MONITOR_TILE_H, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: color_fmt,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let tile_d = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("phosphor_tile_depth"),
            size: wgpu::Extent3d { width: MONITOR_TILE_W, height: MONITOR_TILE_H, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        let av = atlas_tex.create_view(&wgpu::TextureViewDescriptor::default());
        let cv = tile_c.create_view(&wgpu::TextureViewDescriptor::default());
        let dv = tile_d.create_view(&wgpu::TextureViewDescriptor::default());
        Self::new((atlas_tex, av), (tile_c, cv), (tile_d, dv))
    }

    pub const fn atlas_texture(&self) -> &wgpu::Texture { &self.atlas.0 }
    pub const fn atlas_view(&self) -> &wgpu::TextureView { &self.atlas.1 }
    pub const fn tile_color_texture(&self) -> &wgpu::Texture { &self.tile_color.0 }
    pub const fn tile_color_view(&self) -> &wgpu::TextureView { &self.tile_color.1 }
    pub const fn tile_depth_view(&self) -> &wgpu::TextureView { &self.tile_depth.1 }
}

/// Aggregated GPU render targets and samplers (`4` private fields).
pub struct WgpuTargets {
    main_depth: DepthAttachment,
    shadow_lut: ShadowLutPair,
    monitors: MonitorTarget,
    samplers: (wgpu::Sampler, wgpu::Sampler),
}

impl WgpuTargets {
    pub const fn new(
        main_depth: DepthAttachment,
        shadow_lut: ShadowLutPair,
        monitors: MonitorTarget,
        samplers: (wgpu::Sampler, wgpu::Sampler),
    ) -> Self {
        Self { main_depth, shadow_lut, monitors, samplers }
    }

    pub fn allocate(
        device: &wgpu::Device,
        (width, height): (u32, u32),
        color_fmt: wgpu::TextureFormat,
        vram: &mut dyn VramAccount,
    ) -> Self {
        let shadow_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("shadow_cmp_sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            compare: Some(wgpu::CompareFunction::LessEqual),
            ..Default::default()
        });
        let linear_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("lut_linear_sampler"),
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });
        let shadow_lut = ShadowLutPair::allocate(device, vram);
        let _ = &shadow_lut.shadow_tex;
        Self::new(
            DepthAttachment::allocate(device, width, height, vram),
            shadow_lut,
            MonitorTarget::allocate(device, color_fmt, vram),
            (shadow_sampler, linear_sampler),
        )
    }

    pub fn resize(&mut self, device: &wgpu::Device, width: u32, height: u32, vram: &mut dyn VramAccount) {
        self.main_depth.resize_if_needed(device, width, height, vram);
        let _ = &self.monitors.tile_depth.0;
    }

    pub const fn main_depth(&self) -> &wgpu::TextureView { self.main_depth.view() }
    pub const fn shadow_lut(&self) -> &ShadowLutPair { &self.shadow_lut }
    pub const fn monitors(&self) -> &MonitorTarget { &self.monitors }
    pub const fn shadow_sampler(&self) -> &wgpu::Sampler { &self.samplers.0 }
    pub const fn linear_sampler(&self) -> &wgpu::Sampler { &self.samplers.1 }
}
