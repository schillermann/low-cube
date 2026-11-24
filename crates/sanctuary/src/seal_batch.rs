use std::net::{SocketAddr, UdpSocket};
use crate::ward::GuestWard;

const PARALLEL_SEAL_THRESHOLD: usize = 12;

/// Prepared plaintext pulse and chronicle frames paired with their recipient session ward (`<= 4` fields).
pub struct OutboundFrame<'a> {
    companion_id: u32,
    ward: &'a mut GuestWard,
    payloads: Vec<Vec<u8>>,
    sealed: Vec<(SocketAddr, Vec<u8>)>,
}

impl<'a> OutboundFrame<'a> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(
        companion_id: u32,
        ward: &'a mut GuestWard,
        payloads: Vec<Vec<u8>>,
        sealed: Vec<(SocketAddr, Vec<u8>)>,
    ) -> Self {
        Self { companion_id, ward, payloads, sealed }
    }

    pub fn prepare(companion_id: u32, ward: &'a mut GuestWard, payloads: Vec<Vec<u8>>) -> Self {
        let cap = payloads.len();
        Self::new(companion_id, ward, payloads, Vec::with_capacity(cap))
    }

    fn seal_frames(&mut self) {
        for plain in &self.payloads {
            let mut pkt = Vec::with_capacity(plain.len() + 30);
            let peer = self.ward.seal_outbound(self.companion_id, plain, &mut pkt);
            self.sealed.push((peer, pkt));
        }
    }

    fn transmit(&self, socket: &UdpSocket) {
        for (peer, pkt) in &self.sealed {
            let _ = socket.send_to(pkt, *peer);
        }
    }
}

/// Narrow contract sealing and transmitting prepared outbound companion frames.
pub trait PulseDelivery {
    fn deliver(&self, socket: &UdpSocket, frames: &mut [OutboundFrame<'_>]);
}

/// Sequential low-latency frame delivery used for small co-op cohorts (`< 12` companions).
pub struct SequentialDelivery;

impl SequentialDelivery {
    /// Primary constructor initializing sequential frame delivery.
    pub const fn new() -> Self {
        Self
    }
}

impl PulseDelivery for SequentialDelivery {
    fn deliver(&self, socket: &UdpSocket, frames: &mut [OutboundFrame<'_>]) {
        for frame in frames {
            frame.seal_frames();
            frame.transmit(socket);
        }
    }
}

/// `PulseDelivery` decorator parallelizing ChaCha20-Poly1305 AEAD sealing across CPU cores for large cohorts.
pub struct ParallelSeals<D> {
    origin: D,
    lanes: usize,
}

impl<D> ParallelSeals<D> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(origin: D, lanes: usize) -> Self {
        Self { origin, lanes }
    }

    pub fn hardware(origin: D) -> Self {
        let lanes = std::thread::available_parallelism().map_or(4, usize::from).clamp(2, 16);
        Self::new(origin, lanes)
    }
}

impl<D: PulseDelivery> PulseDelivery for ParallelSeals<D> {
    fn deliver(&self, socket: &UdpSocket, frames: &mut [OutboundFrame<'_>]) {
        if frames.len() < PARALLEL_SEAL_THRESHOLD {
            self.origin.deliver(socket, frames);
            return;
        }
        let chunk_size = frames.len().div_ceil(self.lanes).max(1);
        std::thread::scope(|scope| {
            for chunk in frames.chunks_mut(chunk_size) {
                scope.spawn(move || {
                    for frame in chunk {
                        frame.seal_frames();
                    }
                });
            }
        });
        for frame in frames {
            frame.transmit(socket);
        }
    }
}
