use std::num::NonZeroU64;
use crate::instanced_module::{ArchetypeSlice, ModuleArchetypeBank, ModuleInstance};
use crate::packed_quad::{FreeQuad, PackedQuad};
use crate::patch_mesh::{SlotPair, UploadedPatchMesh};
use crate::quad_batch::QuadBatch;
use crate::upload_ring::{BufferSlot, StagingTarget, UploadBudgetRing};
use crate::vram_ledger::VramAccount;
use super::layouts::SceneBindLayouts;
use super::uniforms::AlignedPatchUniform;

pub(super) const MAX_PATCH_DRAWS: usize = 2048;
pub(super) const STREAM_PACKED_CAP: usize = 32_768;
pub(super) const STREAM_FREE_CAP: usize = 16_384;
pub(super) const STREAM_MODULE_CAP: usize = 2_048;

enum StorageSlotEntry {
    Vacant,
    Occupied {
        buffer: wgpu::Buffer,
        bind_group: wgpu::BindGroup,
        byte_len: u64,
    },
}

/// Queue-backed `StagingTarget` executing throttled `<= 1.5 MB/Frame` writes (`2` private fields).
struct QueueStaging<'a> {
    queue: &'a wgpu::Queue,
    slots: &'a [StorageSlotEntry],
}

impl StagingTarget for QueueStaging<'_> {
    fn write_chunk(&mut self, slot: BufferSlot, byte_offset: u64, payload: &[u8]) {
        if let Some(StorageSlotEntry::Occupied { buffer, .. }) = self.slots.get(slot.index()) {
            self.queue.write_buffer(buffer, byte_offset, payload);
        }
    }
}

/// Persistent per-frame dynamic uniform & streaming buffers (`4` private fields).
pub struct DynamicFrameBelt {
    patch_uniforms: wgpu::Buffer,
    packed_stream: (wgpu::Buffer, wgpu::BindGroup),
    free_stream: (wgpu::Buffer, wgpu::BindGroup),
    module_instances: wgpu::Buffer,
}

impl DynamicFrameBelt {
    pub const fn new(
        patch_uniforms: wgpu::Buffer,
        packed_stream: (wgpu::Buffer, wgpu::BindGroup),
        free_stream: (wgpu::Buffer, wgpu::BindGroup),
        module_instances: wgpu::Buffer,
    ) -> Self {
        Self { patch_uniforms, packed_stream, free_stream, module_instances }
    }

    pub fn allocate(device: &wgpu::Device, layouts: &SceneBindLayouts, vram: &mut dyn VramAccount) -> Self {
        let pu_bytes = (MAX_PATCH_DRAWS * core::mem::size_of::<AlignedPatchUniform>()) as u64;
        let pk_bytes = (STREAM_PACKED_CAP * core::mem::size_of::<PackedQuad>()) as u64;
        let fr_bytes = (STREAM_FREE_CAP * core::mem::size_of::<FreeQuad>()) as u64;
        let mi_bytes = (STREAM_MODULE_CAP * core::mem::size_of::<ModuleInstance>()) as u64;
        vram.credit_allocation(pu_bytes + pk_bytes + fr_bytes + mi_bytes);

        let patch_uniforms = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("patch_uniform_belt"),
            size: pu_bytes,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let mk_stream = |label: &str, size: u64| {
            let buf = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(label),
                size,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            let bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some(label),
                layout: layouts.patch(),
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                            buffer: &patch_uniforms,
                            offset: 0,
                            size: NonZeroU64::new(80),
                        }),
                    },
                    wgpu::BindGroupEntry { binding: 1, resource: buf.as_entire_binding() },
                ],
            });
            (buf, bg)
        };
        let module_instances = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("module_instance_belt"),
            size: mi_bytes,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let pk_stream = mk_stream("packed_stream", pk_bytes);
        let fr_stream = mk_stream("free_stream", fr_bytes);
        Self::new(patch_uniforms, pk_stream, fr_stream, module_instances)
    }

    pub const fn patch_uniforms(&self) -> &wgpu::Buffer { &self.patch_uniforms }
    pub const fn packed_stream(&self) -> &(wgpu::Buffer, wgpu::BindGroup) { &self.packed_stream }
    pub const fn free_stream(&self) -> &(wgpu::Buffer, wgpu::BindGroup) { &self.free_stream }
    pub const fn module_instances(&self) -> &wgpu::Buffer { &self.module_instances }
}

/// Shared GPU vertex/index bank for all `11` `< 150`-triangle `SpaceModule` archetypes (`3` private fields).
pub struct ModuleGpuBank {
    vertex_buf: wgpu::Buffer,
    index_buf: wgpu::Buffer,
    slices: [ArchetypeSlice; 11],
}

impl ModuleGpuBank {
    pub const fn new(vertex_buf: wgpu::Buffer, index_buf: wgpu::Buffer, slices: [ArchetypeSlice; 11]) -> Self {
        Self { vertex_buf, index_buf, slices }
    }

    pub fn upload(device: &wgpu::Device, queue: &wgpu::Queue, vram: &mut dyn VramAccount) -> Self {
        let cpu = ModuleArchetypeBank::assembled();
        let vb_bytes = cpu.vertex_bytes();
        let ib_bytes = cpu.index_bytes();
        let ib_padded = (ib_bytes.len() + 3) & !3;
        vram.credit_allocation((vb_bytes.len() + ib_padded) as u64);

        let vertex_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("module_archetype_vb"),
            size: vb_bytes.len() as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        queue.write_buffer(&vertex_buf, 0, vb_bytes);

        let mut ib_vec = ib_bytes.to_vec();
        ib_vec.resize(ib_padded, 0);
        let index_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("module_archetype_ib"),
            size: ib_padded as u64,
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        queue.write_buffer(&index_buf, 0, &ib_vec);

        let mut slices = [ArchetypeSlice::new(0, 0, 0); 11];
        for (i, slot) in slices.iter_mut().enumerate() {
            let m = world::SpaceModule::from_code((i as u8) + 1).expect("module 1..=11");
            *slot = cpu.slice_for(m);
        }
        Self::new(vertex_buf, index_buf, slices)
    }

    pub const fn vertex_buf(&self) -> &wgpu::Buffer { &self.vertex_buf }
    pub const fn index_buf(&self) -> &wgpu::Buffer { &self.index_buf }
    pub const fn slice_at(&self, code_minus_1: usize) -> ArchetypeSlice { self.slices[code_minus_1] }
}

/// GPU storage-buffer arena enforcing `UploadBudgetRing` (`<= 1.5 MB/Frame`) and `VramAccount` (`4` fields).
pub struct WgpuBufferArena {
    slots: Vec<StorageSlotEntry>,
    free_indices: Vec<u32>,
    upload_ring: UploadBudgetRing,
    belt_and_modules: (DynamicFrameBelt, ModuleGpuBank),
}

impl WgpuBufferArena {
    /// Primary constructor doing nothing except assigning arguments to fields.
    fn new(
        slots: Vec<StorageSlotEntry>,
        free_indices: Vec<u32>,
        upload_ring: UploadBudgetRing,
        belt_and_modules: (DynamicFrameBelt, ModuleGpuBank),
    ) -> Self {
        Self { slots, free_indices, upload_ring, belt_and_modules }
    }

    pub fn allocate(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        layouts: &SceneBindLayouts,
        vram: &mut dyn VramAccount,
    ) -> Self {
        let mut slots = Vec::with_capacity(256);
        slots.push(StorageSlotEntry::Vacant);
        let belt = DynamicFrameBelt::allocate(device, layouts, vram);
        let bank = ModuleGpuBank::upload(device, queue, vram);
        Self::new(slots, Vec::new(), UploadBudgetRing::thinkpad_budget(), (belt, bank))
    }

    pub fn begin_frame(&mut self, queue: &wgpu::Queue) {
        let mut staging = QueueStaging { queue, slots: &self.slots };
        let _ = self.upload_ring.begin_frame(&mut staging);
    }

    pub fn upload_patch(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        layouts: &SceneBindLayouts,
        vram: &mut dyn VramAccount,
        batch: &QuadBatch,
    ) -> UploadedPatchMesh {
        let pk_count = batch.packed_slice().len() as u32;
        let fr_count = batch.free_slice().len() as u32;
        let pk_slot = if pk_count > 0 {
            self.allocate_slot(device, queue, layouts, vram, batch.packed_bytes())
        } else {
            BufferSlot::new(0)
        };
        let fr_slot = if fr_count > 0 {
            self.allocate_slot(device, queue, layouts, vram, batch.free_bytes())
        } else {
            BufferSlot::new(0)
        };
        UploadedPatchMesh::new(SlotPair::new((pk_slot, pk_count), (fr_slot, fr_count)), batch.patch_origin())
    }

    fn allocate_slot(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        layouts: &SceneBindLayouts,
        vram: &mut dyn VramAccount,
        bytes: &[u8],
    ) -> BufferSlot {
        let size = (bytes.len().max(16) as u64 + 15) & !15;
        vram.credit_allocation(size);
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("quad_patch_ssbo"),
            size,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("quad_patch_bg"),
            layout: layouts.patch(),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: self.belt_and_modules.0.patch_uniforms(),
                        offset: 0,
                        size: NonZeroU64::new(80),
                    }),
                },
                wgpu::BindGroupEntry { binding: 1, resource: buffer.as_entire_binding() },
            ],
        });
        let slot = if let Some(idx) = self.free_indices.pop() {
            self.slots[idx as usize] = StorageSlotEntry::Occupied { buffer, bind_group, byte_len: size };
            BufferSlot::new(idx)
        } else {
            let idx = self.slots.len() as u32;
            self.slots.push(StorageSlotEntry::Occupied { buffer, bind_group, byte_len: size });
            BufferSlot::new(idx)
        };
        let mut staging = QueueStaging { queue, slots: &self.slots };
        let _ = self.upload_ring.submit(slot, 0, bytes, &mut staging);
        slot
    }

    pub fn reclaim(&mut self, slot: BufferSlot, vram: &mut dyn VramAccount) {
        let idx = slot.index();
        if idx == 0 || idx >= self.slots.len() {
            return;
        }
        self.upload_ring.cancel_slot(slot);
        let prior = core::mem::replace(&mut self.slots[idx], StorageSlotEntry::Vacant);
        if let StorageSlotEntry::Occupied { buffer, byte_len, .. } = prior {
            vram.debit_release(byte_len);
            buffer.destroy();
            self.free_indices.push(idx as u32);
        }
    }

    pub fn bind_group_at(&self, slot: BufferSlot) -> Option<&wgpu::BindGroup> {
        match self.slots.get(slot.index()) {
            Some(StorageSlotEntry::Occupied { bind_group, .. }) => Some(bind_group),
            _ => None,
        }
    }

    pub const fn belt(&self) -> &DynamicFrameBelt { &self.belt_and_modules.0 }
    pub const fn modules(&self) -> &ModuleGpuBank { &self.belt_and_modules.1 }
}
