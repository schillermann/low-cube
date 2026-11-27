use glam::{Mat4, Vec3, Vec4};
use crate::instanced_module::ModuleInstance;
use crate::quad_batch::QuadBatch;
use crate::upload_ring::BufferSlot;
use crate::{BlendPassMode, FirmamentUniform, FramePass};
use super::pipelines::StagePipelines;
use super::scratch::{CapturedPassState, FrameStreamScratch, RecordedOp};
use super::storage_arena::{WgpuBufferArena, STREAM_FREE_CAP, STREAM_MODULE_CAP, STREAM_PACKED_CAP};

/// Decoupled `FramePass` recorder eliminating mid-pass borrow conflicts (`3` private fields).
pub struct PassDraft {
    ops: Vec<RecordedOp>,
    scratch: FrameStreamScratch,
    state: CapturedPassState,
}

impl PassDraft {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(ops: Vec<RecordedOp>, scratch: FrameStreamScratch, state: CapturedPassState) -> Self {
        Self { ops, scratch, state }
    }

    pub fn empty() -> Self {
        Self::new(Vec::with_capacity(512), FrameStreamScratch::new(), CapturedPassState::initial())
    }

    pub fn reset(&mut self) {
        self.ops.clear();
        self.scratch.clear();
        self.state = CapturedPassState::initial();
    }

    pub fn state_mut(&mut self) -> &mut CapturedPassState { &mut self.state }
    pub fn flush_streams(&self, queue: &wgpu::Queue, arena: &WgpuBufferArena) {
        self.scratch.flush_to_belt(queue, arena);
    }

    /// Replays opaque shadow casters into the `1024²` directional Sun shadow pass.
    pub fn execute_shadow_pass(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        shadow_view: &wgpu::TextureView,
        pipelines: &StagePipelines,
        scene_bg: &wgpu::BindGroup,
        arena: &WgpuBufferArena,
    ) {
        let mut rp = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("sun_shadow_1024_pass"),
            color_attachments: &[],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: shadow_view,
                depth_ops: Some(wgpu::Operations { load: wgpu::LoadOp::Clear(1.0), store: wgpu::StoreOp::Store }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
        });
        rp.set_bind_group(0, scene_bg, &[]);
        let mut blend = BlendPassMode::OpaqueDepth;
        for &op in &self.ops {
            match op {
                RecordedOp::SelectBlend(m) => blend = m,
                _ if !matches!(blend, BlendPassMode::OpaqueDepth) => {}
                RecordedOp::DrawSlotPacked { slot, count, patch_idx } => {
                    if let Some(bg) = arena.bind_group_at(slot) {
                        rp.set_pipeline(pipelines.shadows().packed_shadow());
                        rp.set_bind_group(1, bg, &[patch_idx * 256]);
                        rp.draw(0..(count * 6), 0..1);
                    }
                }
                RecordedOp::DrawStreamPacked { start, count, patch_idx } => {
                    rp.set_pipeline(pipelines.shadows().packed_shadow());
                    rp.set_bind_group(1, &arena.belt().packed_stream().1, &[patch_idx * 256]);
                    rp.draw((start * 6)..((start + count) * 6), 0..1);
                }
                RecordedOp::DrawStreamFree { start, count, patch_idx } => {
                    rp.set_pipeline(pipelines.shadows().free_shadow());
                    rp.set_bind_group(1, &arena.belt().free_stream().1, &[patch_idx * 256]);
                    rp.draw((start * 6)..((start + count) * 6), 0..1);
                }
                RecordedOp::DrawModules { code_idx, start, count } => {
                    let sl = arena.modules().slice_at(code_idx);
                    rp.set_pipeline(pipelines.shadows().module_shadow());
                    rp.set_vertex_buffer(0, arena.modules().vertex_buf().slice(..));
                    rp.set_vertex_buffer(1, arena.belt().module_instances().slice(..));
                    rp.set_index_buffer(arena.modules().index_buf().slice(..), wgpu::IndexFormat::Uint16);
                    rp.draw_indexed(sl.first_index()..(sl.first_index() + sl.index_count()), sl.vertex_offset(), start..(start + count));
                }
                _ => {}
            }
        }
    }

    /// Replays recorded sky, `8-Byte` `PackedQuad`, `ModuleInstance`, actor, and UI draw ops into `targets`.
    pub fn execute_color_pass(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        targets: (&wgpu::TextureView, &wgpu::TextureView),
        (preserve_color, viewport_scissor): (bool, Option<[u32; 4]>),
        pipelines: &StagePipelines,
        binds: (&wgpu::BindGroup, &wgpu::BindGroup),
        arena: &WgpuBufferArena,
    ) {
        let clear = self.ops.iter().find_map(|op| match *op {
            RecordedOp::ClearSky(c) => Some(c),
            _ => None,
        }).unwrap_or(Vec4::new(0.085, 0.125, 0.175, 1.0));

        let init_color_load = if preserve_color {
            wgpu::LoadOp::Load
        } else {
            wgpu::LoadOp::Clear(wgpu::Color { r: clear.x as f64, g: clear.y as f64, b: clear.z as f64, a: 1.0 })
        };
        let init_depth_load = if preserve_color { wgpu::LoadOp::Load } else { wgpu::LoadOp::Clear(1.0) };

        let mut segments: Vec<&[RecordedOp]> = Vec::new();
        let mut seg_start = 0usize;
        for (i, op) in self.ops.iter().enumerate() {
            if matches!(op, RecordedOp::ClearDepth) {
                if i > seg_start || !preserve_color {
                    segments.push(&self.ops[seg_start..i]);
                }
                seg_start = i + 1;
            }
        }
        segments.push(&self.ops[seg_start..]);

        for (seg_idx, seg_ops) in segments.iter().enumerate() {
            let c_load = if seg_idx == 0 { init_color_load } else { wgpu::LoadOp::Load };
            let d_load = if seg_idx == 0 && seg_start == 0 { init_depth_load } else { wgpu::LoadOp::Clear(1.0) };
            let mut rp = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("scene_color_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: targets.0,
                    resolve_target: None,
                    ops: wgpu::Operations { load: c_load, store: wgpu::StoreOp::Store },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: targets.1,
                    depth_ops: Some(wgpu::Operations { load: d_load, store: wgpu::StoreOp::Store }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            if let Some([vx, vy, vw, vh]) = viewport_scissor {
                rp.set_viewport(vx as f32, vy as f32, vw as f32, vh as f32, 0.0, 1.0);
                rp.set_scissor_rect(vx, vy, vw, vh);
            }
            let mut blend = BlendPassMode::OpaqueDepth;
            for &op in *seg_ops {
                match op {
                    RecordedOp::DrawSkyLut => {
                        rp.set_pipeline(pipelines.firmament());
                        rp.set_bind_group(0, binds.1, &[]);
                        rp.draw(0..3, 0..1);
                    }
                    RecordedOp::SelectBlend(m) => blend = m,
                    RecordedOp::DrawSlotPacked { slot, count, patch_idx } => {
                        if let Some(bg) = arena.bind_group_at(slot) {
                            rp.set_pipeline(pipelines.quads().packed_opaque());
                            rp.set_bind_group(0, binds.0, &[]);
                            rp.set_bind_group(1, bg, &[patch_idx * 256]);
                            rp.draw(0..(count * 6), 0..1);
                        }
                    }
                    RecordedOp::DrawSlotFree { slot, count, patch_idx } => {
                        if let Some(bg) = arena.bind_group_at(slot) {
                            let pl = match blend {
                                BlendPassMode::OpaqueDepth => pipelines.quads().free_opaque(),
                                BlendPassMode::TranslucentPlume => pipelines.quads().free_plume(),
                                BlendPassMode::OverlayHud => pipelines.quads().free_overlay(),
                            };
                            rp.set_pipeline(pl);
                            rp.set_bind_group(0, binds.0, &[]);
                            rp.set_bind_group(1, bg, &[patch_idx * 256]);
                            rp.draw(0..(count * 6), 0..1);
                        }
                    }
                    RecordedOp::DrawStreamPacked { start, count, patch_idx } => {
                        rp.set_pipeline(pipelines.quads().packed_opaque());
                        rp.set_bind_group(0, binds.0, &[]);
                        rp.set_bind_group(1, &arena.belt().packed_stream().1, &[patch_idx * 256]);
                        rp.draw((start * 6)..((start + count) * 6), 0..1);
                    }
                    RecordedOp::DrawStreamFree { start, count, patch_idx } => {
                        let pl = match blend {
                            BlendPassMode::OpaqueDepth => pipelines.quads().free_opaque(),
                            BlendPassMode::TranslucentPlume => pipelines.quads().free_plume(),
                            BlendPassMode::OverlayHud => pipelines.quads().free_overlay(),
                        };
                        rp.set_pipeline(pl);
                        rp.set_bind_group(0, binds.0, &[]);
                        rp.set_bind_group(1, &arena.belt().free_stream().1, &[patch_idx * 256]);
                        rp.draw((start * 6)..((start + count) * 6), 0..1);
                    }
                    RecordedOp::DrawModules { code_idx, start, count } => {
                        let sl = arena.modules().slice_at(code_idx);
                        rp.set_pipeline(pipelines.module_opaque());
                        rp.set_bind_group(0, binds.0, &[]);
                        rp.set_vertex_buffer(0, arena.modules().vertex_buf().slice(..));
                        rp.set_vertex_buffer(1, arena.belt().module_instances().slice(..));
                        rp.set_index_buffer(arena.modules().index_buf().slice(..), wgpu::IndexFormat::Uint16);
                        rp.draw_indexed(sl.first_index()..(sl.first_index() + sl.index_count()), sl.vertex_offset(), start..(start + count));
                    }
                    _ => {}
                }
            }
        }
    }
}

impl FramePass for PassDraft {
    fn clear_sky(&mut self, sky_rgba: Vec4) { self.ops.push(RecordedOp::ClearSky(sky_rgba)); }
    fn clear_depth_layer(&mut self) { self.ops.push(RecordedOp::ClearDepth); }
    fn bind_camera(&mut self, view_proj: Mat4, light_dir: Vec3) { self.state.record_camera(view_proj, light_dir); }
    fn apply_model(&mut self, model: Mat4) { self.state.record_model(model); }
    fn select_blend(&mut self, mode: BlendPassMode) { self.ops.push(RecordedOp::SelectBlend(mode)); }
    fn upload_sky_lut(&mut self, texels: &[u8]) { self.state.record_lut(texels); }
    fn draw_sky_lut(&mut self, uniform: FirmamentUniform) {
        self.state.record_firmament(uniform);
        self.ops.push(RecordedOp::DrawSkyLut);
    }
    fn draw_packed_quads(&mut self, slot: BufferSlot, count: u32, patch_origin: [f32; 4]) {
        if let Some(patch_idx) = self.scratch.push_patch(self.state.current_model(), patch_origin) {
            self.ops.push(RecordedOp::DrawSlotPacked { slot, count, patch_idx });
        }
    }
    fn draw_free_quads(&mut self, slot: BufferSlot, count: u32) {
        if let Some(patch_idx) = self.scratch.push_patch(self.state.current_model(), [0.0, 0.0, 0.0, 0.25]) {
            self.ops.push(RecordedOp::DrawSlotFree { slot, count, patch_idx });
        }
    }
    fn stream_quads(&mut self, batch: &QuadBatch) {
        let model = self.state.current_model();
        if let Some(start) = self.scratch.append_packed(batch.packed_slice(), STREAM_PACKED_CAP) {
            if let Some(patch_idx) = self.scratch.push_patch(model, batch.patch_origin()) {
                self.ops.push(RecordedOp::DrawStreamPacked { start, count: batch.packed_slice().len() as u32, patch_idx });
            }
        }
        if let Some(start) = self.scratch.append_free(batch.free_slice(), STREAM_FREE_CAP) {
            if let Some(patch_idx) = self.scratch.push_patch(model, [0.0, 0.0, 0.0, 0.25]) {
                self.ops.push(RecordedOp::DrawStreamFree { start, count: batch.free_slice().len() as u32, patch_idx });
            }
        }
    }
    fn draw_module_instances(&mut self, instances: &[ModuleInstance]) {
        for code in 1..=11u8 {
            let (start, count) = self.scratch.append_modules_by_code(code, instances, STREAM_MODULE_CAP);
            if count > 0 {
                self.ops.push(RecordedOp::DrawModules { code_idx: (code - 1) as usize, start, count });
            }
        }
    }
}
