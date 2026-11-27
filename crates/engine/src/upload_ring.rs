use std::collections::VecDeque;

/// Strict `1.5 MB / Frame` (`1,572,864` bytes) PCIe staging upload budget preventing ThinkPad T480 PCIe 3.0 x2 stalls.
pub const PCIE_FRAME_UPLOAD_CAP_BYTES: usize = 1_572_864;

/// Opaque GPU storage-buffer slot token (`1` private field, `Copy`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct BufferSlot {
    index: u32,
}

impl BufferSlot {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(index: u32) -> Self {
        Self { index }
    }

    pub const fn index(self) -> usize {
        self.index as usize
    }
}

/// Focused contract receiving bounded GPU buffer slices without exposing driver queues.
pub trait StagingTarget {
    fn write_chunk(&mut self, slot: BufferSlot, byte_offset: u64, payload: &[u8]);
}

/// Deferred chunk inside `UploadBudgetRing` (`3` private fields).
#[derive(Clone, Debug)]
struct QueuedChunk {
    slot: BufferSlot,
    byte_offset: u64,
    payload: Vec<u8>,
}

impl QueuedChunk {
    const fn new(slot: BufferSlot, byte_offset: u64, payload: Vec<u8>) -> Self {
        Self { slot, byte_offset, payload }
    }
}

/// Throttled PCIe upload ringbuffer capping GPU transfers to `<= 1.5 MB / Frame` (`3` private fields).
pub struct UploadBudgetRing {
    cap_per_frame: usize,
    remaining_bytes: usize,
    backlog: VecDeque<QueuedChunk>,
}

impl UploadBudgetRing {
    /// Primary constructor doing nothing except assigning arguments to fields.
    fn new(cap_per_frame: usize, remaining_bytes: usize, backlog: VecDeque<QueuedChunk>) -> Self {
        Self { cap_per_frame, remaining_bytes, backlog }
    }

    pub fn thinkpad_budget() -> Self {
        Self::with_cap(PCIE_FRAME_UPLOAD_CAP_BYTES)
    }

    pub fn with_cap(cap_per_frame: usize) -> Self {
        let clamped = cap_per_frame.max(16);
        Self::new(clamped, clamped, VecDeque::new())
    }

    pub const fn remaining_frame_bytes(&self) -> usize {
        self.remaining_bytes
    }

    pub fn queued_backlog_bytes(&self) -> usize {
        self.backlog.iter().map(|c| c.payload.len()).sum()
    }

    /// Resets the per-frame PCIe byte allowance (`<= 1.5 MB`) and flushes queued backlog chunks in FIFO order.
    pub fn begin_frame(&mut self, target: &mut dyn StagingTarget) -> usize {
        self.remaining_bytes = self.cap_per_frame;
        let mut flushed = 0usize;
        while self.remaining_bytes > 0 {
            let Some(mut front) = self.backlog.pop_front() else {
                break;
            };
            let align_cap = self.remaining_bytes & !3usize;
            if align_cap == 0 {
                self.backlog.push_front(front);
                break;
            }
            if front.payload.len() <= align_cap {
                let len = front.payload.len();
                target.write_chunk(front.slot, front.byte_offset, &front.payload);
                self.remaining_bytes -= len;
                flushed += len;
            } else {
                let head = &front.payload[..align_cap];
                target.write_chunk(front.slot, front.byte_offset, head);
                let tail = front.payload.split_off(align_cap);
                let next_offset = front.byte_offset + (align_cap as u64);
                self.backlog.push_front(QueuedChunk::new(front.slot, next_offset, tail));
                self.remaining_bytes -= align_cap;
                flushed += align_cap;
                break;
            }
        }
        flushed
    }

    /// Submits `payload` up to the remaining `<= 1.5 MB` frame allowance, deferring spillover to subsequent frames.
    pub fn submit(&mut self, slot: BufferSlot, byte_offset: u64, payload: &[u8], target: &mut dyn StagingTarget) -> usize {
        if payload.is_empty() {
            return 0;
        }
        if !self.backlog.is_empty() {
            self.backlog.push_back(QueuedChunk::new(slot, byte_offset, payload.to_vec()));
            return 0;
        }
        let align_cap = self.remaining_bytes & !3usize;
        if payload.len() <= align_cap {
            target.write_chunk(slot, byte_offset, payload);
            self.remaining_bytes -= payload.len();
            payload.len()
        } else if align_cap > 0 {
            target.write_chunk(slot, byte_offset, &payload[..align_cap]);
            self.remaining_bytes -= align_cap;
            self.backlog.push_back(QueuedChunk::new(
                slot,
                byte_offset + (align_cap as u64),
                payload[align_cap..].to_vec(),
            ));
            align_cap
        } else {
            self.backlog.push_back(QueuedChunk::new(slot, byte_offset, payload.to_vec()));
            0
        }
    }

    /// Purges pending deferred chunks targeting `slot` when a sector/brick buffer is evicted or replaced.
    pub fn cancel_slot(&mut self, slot: BufferSlot) {
        self.backlog.retain(|c| c.slot != slot);
    }
}
