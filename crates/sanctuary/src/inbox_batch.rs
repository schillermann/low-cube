use std::net::SocketAddr;
use crate::envelope::SanctuaryEnvelope;
use crate::ward::GuestWard;

const PARALLEL_INBOX_THRESHOLD: usize = 12;

/// Encrypted UDP datagrams grouped for a single `GuestWard` session (`<= 4` fields).
pub struct InboundBatch<'a> {
    companion_id: u32,
    ward: &'a mut GuestWard,
    raw: Vec<(u64, SocketAddr, Vec<u8>)>,
    opened: Vec<(u64, u32, SanctuaryEnvelope)>,
}

impl<'a> InboundBatch<'a> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(
        companion_id: u32,
        ward: &'a mut GuestWard,
        raw: Vec<(u64, SocketAddr, Vec<u8>)>,
        opened: Vec<(u64, u32, SanctuaryEnvelope)>,
    ) -> Self {
        Self { companion_id, ward, raw, opened }
    }

    pub fn prepare(
        companion_id: u32,
        ward: &'a mut GuestWard,
        raw: Vec<(u64, SocketAddr, Vec<u8>)>,
    ) -> Self {
        let cap = raw.len();
        Self::new(companion_id, ward, raw, Vec::with_capacity(cap))
    }

    fn decrypt_and_decode(&mut self) {
        let cid = self.companion_id;
        for (seq, src, pkt) in &self.raw {
            if let Some(body) = self.ward.open_datagram(*src, pkt) {
                if let Some(env) = SanctuaryEnvelope::decode(&body) {
                    self.opened.push((*seq, cid, env));
                }
            }
        }
    }

    pub fn collect_ordered(batches: Vec<Self>) -> Vec<(u32, SanctuaryEnvelope)> {
        let total: usize = batches.iter().map(|b| b.opened.len()).sum();
        let mut merged = Vec::with_capacity(total);
        for batch in batches {
            merged.extend(batch.opened);
        }
        merged.sort_unstable_by_key(|&(seq, cid, _)| (seq, cid));
        merged.into_iter().map(|(_, cid, env)| (cid, env)).collect()
    }
}

/// Narrow contract authenticating and decoding inbound ChaCha20-Poly1305 companion packets.
pub trait PulseInbox {
    fn open_batches(&self, batches: &mut [InboundBatch<'_>]);
}

/// Sequential low-latency packet decryption used for small co-op cohorts (`< 12` active senders).
pub struct SequentialInbox;

impl SequentialInbox {
    /// Primary constructor initializing sequential packet authentication.
    pub const fn new() -> Self {
        Self
    }
}

impl PulseInbox for SequentialInbox {
    fn open_batches(&self, batches: &mut [InboundBatch<'_>]) {
        for batch in batches {
            batch.decrypt_and_decode();
        }
    }
}

/// `PulseInbox` decorator parallelizing ChaCha20-Poly1305 AEAD verification across CPU cores.
pub struct ParallelInbox<D> {
    origin: D,
    lanes: usize,
}

impl<D> ParallelInbox<D> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(origin: D, lanes: usize) -> Self {
        Self { origin, lanes }
    }

    pub fn hardware(origin: D) -> Self {
        let lanes = std::thread::available_parallelism().map_or(4, usize::from).clamp(2, 16);
        Self::new(origin, lanes)
    }
}

impl<D: PulseInbox> PulseInbox for ParallelInbox<D> {
    fn open_batches(&self, batches: &mut [InboundBatch<'_>]) {
        if batches.len() < PARALLEL_INBOX_THRESHOLD {
            self.origin.open_batches(batches);
            return;
        }
        let chunk_size = batches.len().div_ceil(self.lanes).max(1);
        std::thread::scope(|scope| {
            for chunk in batches.chunks_mut(chunk_size) {
                scope.spawn(move || {
                    for batch in chunk {
                        batch.decrypt_and_decode();
                    }
                });
            }
        });
    }
}
