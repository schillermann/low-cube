use glam::{Mat4, Vec3, Vec4};
use crate::instanced_module::ModuleInstance;
use crate::packed_quad::{FreeQuad, PackedQuad};
use crate::upload_ring::BufferSlot;
use crate::{BlendPassMode, FirmamentUniform};
use super::storage_arena::{WgpuBufferArena, MAX_PATCH_DRAWS};
use super::uniforms::AlignedPatchUniform;

#[derive(Clone, Copy)]
pub enum RecordedOp {
    ClearSky(Vec4),
    ClearDepth,
    DrawSkyLut,
    SelectBlend(BlendPassMode),
    DrawSlotPacked { slot: BufferSlot, count: u32, patch_idx: u32 },
    DrawSlotFree { slot: BufferSlot, count: u32, patch_idx: u32 },
    DrawStreamPacked { start: u32, count: u32, patch_idx: u32 },
    DrawStreamFree { start: u32, count: u32, patch_idx: u32 },
    DrawModules { code_idx: usize, start: u32, count: u32 },
}

/// Streaming payload buffers accumulated during a `FramePass` prior to GPU submission (`4` private fields).
pub struct FrameStreamScratch {
    patches: Vec<AlignedPatchUniform>,
    packed: Vec<PackedQuad>,
    free: Vec<FreeQuad>,
    modules: Vec<ModuleInstance>,
}

impl FrameStreamScratch {
    pub fn new() -> Self {
        Self {
            patches: Vec::with_capacity(256),
            packed: Vec::with_capacity(2048),
            free: Vec::with_capacity(2048),
            modules: Vec::with_capacity(64),
        }
    }

    pub fn clear(&mut self) {
        self.patches.clear();
        self.packed.clear();
        self.free.clear();
        self.modules.clear();
    }

    pub fn push_patch(&mut self, model: Mat4, origin: [f32; 4]) -> Option<u32> {
        if self.patches.len() >= MAX_PATCH_DRAWS {
            return None;
        }
        let idx = self.patches.len() as u32;
        self.patches.push(AlignedPatchUniform::from_pose(model, origin));
        Some(idx)
    }

    pub fn append_packed(&mut self, slice: &[PackedQuad], cap: usize) -> Option<u32> {
        if slice.is_empty() || self.packed.len() + slice.len() > cap {
            return None;
        }
        let start = self.packed.len() as u32;
        self.packed.extend_from_slice(slice);
        Some(start)
    }

    pub fn append_free(&mut self, slice: &[FreeQuad], cap: usize) -> Option<u32> {
        if slice.is_empty() || self.free.len() + slice.len() > cap {
            return None;
        }
        let start = self.free.len() as u32;
        self.free.extend_from_slice(slice);
        Some(start)
    }

    pub fn append_modules_by_code(&mut self, code: u8, instances: &[ModuleInstance], cap: usize) -> (u32, u32) {
        let start = self.modules.len() as u32;
        for inst in instances {
            if inst.module().code() == code && self.modules.len() < cap {
                self.modules.push(*inst);
            }
        }
        (start, (self.modules.len() as u32) - start)
    }

    pub fn flush_to_belt(&self, queue: &wgpu::Queue, arena: &WgpuBufferArena) {
        let belt = arena.belt();
        if !self.patches.is_empty() {
            let bytes = unsafe {
                core::slice::from_raw_parts(
                    self.patches.as_ptr().cast::<u8>(),
                    self.patches.len() * core::mem::size_of::<AlignedPatchUniform>(),
                )
            };
            queue.write_buffer(belt.patch_uniforms(), 0, bytes);
        }
        if !self.packed.is_empty() {
            let bytes = unsafe {
                core::slice::from_raw_parts(
                    self.packed.as_ptr().cast::<u8>(),
                    self.packed.len() * core::mem::size_of::<PackedQuad>(),
                )
            };
            queue.write_buffer(&belt.packed_stream().0, 0, bytes);
        }
        if !self.free.is_empty() {
            let bytes = unsafe {
                core::slice::from_raw_parts(
                    self.free.as_ptr().cast::<u8>(),
                    self.free.len() * core::mem::size_of::<FreeQuad>(),
                )
            };
            queue.write_buffer(&belt.free_stream().0, 0, bytes);
        }
        if !self.modules.is_empty() {
            let bytes = unsafe {
                core::slice::from_raw_parts(
                    self.modules.as_ptr().cast::<u8>(),
                    self.modules.len() * core::mem::size_of::<ModuleInstance>(),
                )
            };
            queue.write_buffer(belt.module_instances(), 0, bytes);
        }
    }
}

enum LutSyncState {
    Unchanged,
    Pending(Vec<u8>),
}

/// Camera & sky state captured during a `FramePass` (`4` strictly private fields, `0` `Option` fields).
pub struct CapturedPassState {
    view_proj_light: (Mat4, Vec3),
    current_model: Mat4,
    firmament: FirmamentUniform,
    lut_sync: LutSyncState,
}

impl CapturedPassState {
    pub const fn initial() -> Self {
        Self {
            view_proj_light: (Mat4::IDENTITY, Vec3::Y),
            current_model: Mat4::IDENTITY,
            firmament: FirmamentUniform::dormant(),
            lut_sync: LutSyncState::Unchanged,
        }
    }

    pub fn record_camera(&mut self, vp: Mat4, light: Vec3) {
        self.view_proj_light = (vp, light);
        self.current_model = Mat4::IDENTITY;
    }

    pub fn record_model(&mut self, model: Mat4) { self.current_model = model; }
    pub const fn current_model(&self) -> Mat4 { self.current_model }
    pub const fn camera(&self) -> (Mat4, Vec3) { self.view_proj_light }
    pub fn record_firmament(&mut self, f: FirmamentUniform) { self.firmament = f; }
    pub const fn firmament(&self) -> &FirmamentUniform { &self.firmament }
    pub fn record_lut(&mut self, bytes: &[u8]) { self.lut_sync = LutSyncState::Pending(bytes.to_vec()); }
    pub fn take_lut(&mut self) -> Option<Vec<u8>> {
        match core::mem::replace(&mut self.lut_sync, LutSyncState::Unchanged) {
            LutSyncState::Unchanged => None,
            LutSyncState::Pending(bytes) => Some(bytes),
        }
    }
}
