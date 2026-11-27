use crate::instanced_module::{ModuleInstance, ModuleVertex};
use crate::wgsl::{
    FIRMAMENT_LUT_WGSL, FREE_QUAD_WGSL, MODULE_INSTANCE_WGSL, MONITOR_BLIT_WGSL, PACKED_QUAD_WGSL,
};
use super::layouts::SceneBindLayouts;

/// Color-pass vertex-pulled quad pipelines (`4` private fields).
pub struct QuadPipelines {
    packed_opaque: wgpu::RenderPipeline,
    free_opaque: wgpu::RenderPipeline,
    free_plume: wgpu::RenderPipeline,
    free_overlay: wgpu::RenderPipeline,
}

impl QuadPipelines {
    pub const fn new(
        packed_opaque: wgpu::RenderPipeline,
        free_opaque: wgpu::RenderPipeline,
        free_plume: wgpu::RenderPipeline,
        free_overlay: wgpu::RenderPipeline,
    ) -> Self {
        Self { packed_opaque, free_opaque, free_plume, free_overlay }
    }

    pub const fn packed_opaque(&self) -> &wgpu::RenderPipeline { &self.packed_opaque }
    pub const fn free_opaque(&self) -> &wgpu::RenderPipeline { &self.free_opaque }
    pub const fn free_plume(&self) -> &wgpu::RenderPipeline { &self.free_plume }
    pub const fn free_overlay(&self) -> &wgpu::RenderPipeline { &self.free_overlay }
}

/// `1024²` depth-only Sun shadow map pipelines (`3` private fields).
pub struct ShadowPipelines {
    packed_shadow: wgpu::RenderPipeline,
    free_shadow: wgpu::RenderPipeline,
    module_shadow: wgpu::RenderPipeline,
}

impl ShadowPipelines {
    pub const fn new(
        packed_shadow: wgpu::RenderPipeline,
        free_shadow: wgpu::RenderPipeline,
        module_shadow: wgpu::RenderPipeline,
    ) -> Self {
        Self { packed_shadow, free_shadow, module_shadow }
    }

    pub const fn packed_shadow(&self) -> &wgpu::RenderPipeline { &self.packed_shadow }
    pub const fn free_shadow(&self) -> &wgpu::RenderPipeline { &self.free_shadow }
    pub const fn module_shadow(&self) -> &wgpu::RenderPipeline { &self.module_shadow }
}

/// Complete compiled `wgpu::RenderPipeline` suite (`4` private fields).
pub struct StagePipelines {
    quads: QuadPipelines,
    shadows: ShadowPipelines,
    module_opaque: wgpu::RenderPipeline,
    firmament_monitor: (wgpu::RenderPipeline, wgpu::RenderPipeline),
}

impl StagePipelines {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(
        quads: QuadPipelines,
        shadows: ShadowPipelines,
        module_opaque: wgpu::RenderPipeline,
        firmament_monitor: (wgpu::RenderPipeline, wgpu::RenderPipeline),
    ) -> Self {
        Self { quads, shadows, module_opaque, firmament_monitor }
    }

    pub fn assembled(
        device: &wgpu::Device,
        layouts: &SceneBindLayouts,
        color_fmt: wgpu::TextureFormat,
    ) -> Self {
        let sh_packed = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("packed_quad_wgsl"),
            source: wgpu::ShaderSource::Wgsl(PACKED_QUAD_WGSL.into()),
        });
        let sh_free = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("free_quad_wgsl"),
            source: wgpu::ShaderSource::Wgsl(FREE_QUAD_WGSL.into()),
        });
        let sh_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("module_instance_wgsl"),
            source: wgpu::ShaderSource::Wgsl(MODULE_INSTANCE_WGSL.into()),
        });
        let sh_firm = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("firmament_lut_wgsl"),
            source: wgpu::ShaderSource::Wgsl(FIRMAMENT_LUT_WGSL.into()),
        });
        let sh_mon = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("monitor_blit_wgsl"),
            source: wgpu::ShaderSource::Wgsl(MONITOR_BLIT_WGSL.into()),
        });

        let quad_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("quad_pl"),
            bind_group_layouts: &[layouts.scene(), layouts.patch()],
            push_constant_ranges: &[],
        });
        let module_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("module_pl"),
            bind_group_layouts: &[layouts.scene()],
            push_constant_ranges: &[],
        });
        let firm_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("firm_pl"),
            bind_group_layouts: &[layouts.firmament()],
            push_constant_ranges: &[],
        });
        let mon_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("mon_pl"),
            bind_group_layouts: &[layouts.monitor()],
            push_constant_ranges: &[],
        });

        let alpha_blend = wgpu::BlendState {
            color: wgpu::BlendComponent {
                src_factor: wgpu::BlendFactor::SrcAlpha,
                dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                operation: wgpu::BlendOperation::Add,
            },
            alpha: wgpu::BlendComponent {
                src_factor: wgpu::BlendFactor::One,
                dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                operation: wgpu::BlendOperation::Add,
            },
        };
        let depth_opaque = Some(wgpu::DepthStencilState {
            format: wgpu::TextureFormat::Depth32Float,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::LessEqual,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        });
        let depth_plume = Some(wgpu::DepthStencilState {
            format: wgpu::TextureFormat::Depth32Float,
            depth_write_enabled: false,
            depth_compare: wgpu::CompareFunction::LessEqual,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        });
        let depth_shadow = Some(wgpu::DepthStencilState {
            format: wgpu::TextureFormat::Depth32Float,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::LessEqual,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState { constant: 2, slope_scale: 2.0, clamp: 0.0 },
        });
        let depth_sky = Some(wgpu::DepthStencilState {
            format: wgpu::TextureFormat::Depth32Float,
            depth_write_enabled: false,
            depth_compare: wgpu::CompareFunction::LessEqual,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        });

        let mk_quad = |sh: &wgpu::ShaderModule, blend: Option<wgpu::BlendState>, depth: Option<wgpu::DepthStencilState>, cull: Option<wgpu::Face>| {
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("quad_rp"),
                layout: Some(&quad_layout),
                vertex: wgpu::VertexState { module: sh, entry_point: Some("vs_main"), compilation_options: Default::default(), buffers: &[] },
                primitive: wgpu::PrimitiveState { cull_mode: cull, ..Default::default() },
                depth_stencil: depth,
                multisample: wgpu::MultisampleState::default(),
                fragment: Some(wgpu::FragmentState {
                    module: sh,
                    entry_point: Some("fs_main"),
                    compilation_options: Default::default(),
                    targets: &[Some(wgpu::ColorTargetState { format: color_fmt, blend, write_mask: wgpu::ColorWrites::ALL })],
                }),
                multiview: None,
                cache: None,
            })
        };

        let mod_vbuf = [
            wgpu::VertexBufferLayout {
                array_stride: core::mem::size_of::<ModuleVertex>() as u64,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &wgpu::vertex_attr_array![0 => Float32x4, 1 => Float32x4, 2 => Float32x4],
            },
            wgpu::VertexBufferLayout {
                array_stride: core::mem::size_of::<ModuleInstance>() as u64,
                step_mode: wgpu::VertexStepMode::Instance,
                attributes: &wgpu::vertex_attr_array![3 => Float32x4, 4 => Float32x4, 5 => Float32x4, 6 => Float32x4, 7 => Float32x4],
            },
        ];

        let module_opaque = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("module_opaque_rp"),
            layout: Some(&module_layout),
            vertex: wgpu::VertexState { module: &sh_module, entry_point: Some("vs_main"), compilation_options: Default::default(), buffers: &mod_vbuf },
            primitive: wgpu::PrimitiveState { cull_mode: Some(wgpu::Face::Back), ..Default::default() },
            depth_stencil: depth_opaque.clone(),
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: &sh_module,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState { format: color_fmt, blend: None, write_mask: wgpu::ColorWrites::ALL })],
            }),
            multiview: None,
            cache: None,
        });

        let mk_shadow = |pl: &wgpu::PipelineLayout, sh: &wgpu::ShaderModule, bufs: &[wgpu::VertexBufferLayout]| {
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("shadow_rp"),
                layout: Some(pl),
                vertex: wgpu::VertexState { module: sh, entry_point: Some("vs_shadow"), compilation_options: Default::default(), buffers: bufs },
                primitive: wgpu::PrimitiveState { cull_mode: None, ..Default::default() },
                depth_stencil: depth_shadow.clone(),
                multisample: wgpu::MultisampleState::default(),
                fragment: None,
                multiview: None,
                cache: None,
            })
        };

        let firm_rp = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("firmament_rp"),
            layout: Some(&firm_layout),
            vertex: wgpu::VertexState { module: &sh_firm, entry_point: Some("vs_main"), compilation_options: Default::default(), buffers: &[] },
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: depth_sky,
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: &sh_firm,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState { format: color_fmt, blend: None, write_mask: wgpu::ColorWrites::ALL })],
            }),
            multiview: None,
            cache: None,
        });

        let mon_rp = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("monitor_blit_rp"),
            layout: Some(&mon_layout),
            vertex: wgpu::VertexState { module: &sh_mon, entry_point: Some("vs_main"), compilation_options: Default::default(), buffers: &[] },
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: &sh_mon,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState { format: color_fmt, blend: None, write_mask: wgpu::ColorWrites::ALL })],
            }),
            multiview: None,
            cache: None,
        });

        Self::new(
            QuadPipelines::new(
                mk_quad(&sh_packed, None, depth_opaque.clone(), Some(wgpu::Face::Back)),
                mk_quad(&sh_free, None, depth_opaque, None),
                mk_quad(&sh_free, Some(alpha_blend), depth_plume, None),
                mk_quad(&sh_free, Some(alpha_blend), None, None),
            ),
            ShadowPipelines::new(
                mk_shadow(&quad_layout, &sh_packed, &[]),
                mk_shadow(&quad_layout, &sh_free, &[]),
                mk_shadow(&module_layout, &sh_module, &mod_vbuf),
            ),
            module_opaque,
            (firm_rp, mon_rp),
        )
    }

    pub const fn quads(&self) -> &QuadPipelines { &self.quads }
    pub const fn shadows(&self) -> &ShadowPipelines { &self.shadows }
    pub const fn module_opaque(&self) -> &wgpu::RenderPipeline { &self.module_opaque }
    pub const fn firmament(&self) -> &wgpu::RenderPipeline { &self.firmament_monitor.0 }
    pub const fn monitor_blit(&self) -> &wgpu::RenderPipeline { &self.firmament_monitor.1 }
}
