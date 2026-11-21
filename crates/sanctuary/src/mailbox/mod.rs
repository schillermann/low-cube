mod socket_loop;

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::Instant;
pub use self::socket_loop::SocketLoop;

const STANDBY_THRESHOLD_NANOS: u64 = 20_000_000;

/// Lock-free atomic status and main-loop heartbeat mailbox coordinating `SocketLoop` standby failover.
pub struct PortalMailbox {
    epoch: Instant,
    main_beat_nanos: AtomicU64,
    alive: AtomicBool,
    surface_dirty: AtomicBool,
}

impl PortalMailbox {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(
        epoch: Instant,
        main_beat_nanos: AtomicU64,
        alive: AtomicBool,
        surface_dirty: AtomicBool,
    ) -> Self {
        Self { epoch, main_beat_nanos, alive, surface_dirty }
    }

    pub fn active() -> Self {
        Self::new(
            Instant::now(),
            AtomicU64::new(0),
            AtomicBool::new(true),
            AtomicBool::new(false),
        )
    }

    pub fn running(&self) -> bool {
        self.alive.load(Ordering::Acquire)
    }

    pub fn halt(&self) {
        self.alive.store(false, Ordering::Release);
    }

    pub fn record_main_beat(&self, now: Instant) {
        let nanos = now.saturating_duration_since(self.epoch).as_nanos().min(u64::MAX as u128) as u64;
        self.main_beat_nanos.store(nanos, Ordering::Release);
    }

    pub fn standby_needed(&self, now: Instant) -> bool {
        let last = self.main_beat_nanos.load(Ordering::Acquire);
        let cur = now.saturating_duration_since(self.epoch).as_nanos().min(u64::MAX as u128) as u64;
        cur.saturating_sub(last) >= STANDBY_THRESHOLD_NANOS
    }

    pub fn note_surface_change(&self) {
        self.surface_dirty.store(true, Ordering::Release);
    }

    pub fn take_surface_change(&self) -> bool {
        self.surface_dirty.swap(false, Ordering::AcqRel)
    }
}
