mod socket_loop;

use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::time::Instant;
pub use self::socket_loop::SocketLoop;

const STANDBY_THRESHOLD_NANOS: u64 = 20_000_000;

/// Focused 1-method background P2P socket heartbeat decoupled from `PlanetaryWorld` terrain and meshes.
pub trait PortalBeat: Send {
    fn step_standby(&mut self, dt: f32) -> usize;
}

/// Lock-free atomic heartbeat and active-fleet mailbox coordinating `SocketLoop` standby failover.
pub struct PortalMailbox {
    epoch: Instant,
    main_beat_nanos: AtomicU64,
    alive: AtomicBool,
    active_crafts: AtomicUsize,
}

impl PortalMailbox {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(
        epoch: Instant,
        main_beat_nanos: AtomicU64,
        alive: AtomicBool,
        active_crafts: AtomicUsize,
    ) -> Self {
        Self { epoch, main_beat_nanos, alive, active_crafts }
    }

    pub fn active() -> Self {
        Self::new(
            Instant::now(),
            AtomicU64::new(0),
            AtomicBool::new(true),
            AtomicUsize::new(0),
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

    pub fn note_active_crafts(&self, count: usize) {
        self.active_crafts.store(count, Ordering::Release);
    }

    pub fn active_crafts(&self) -> usize {
        self.active_crafts.load(Ordering::Acquire)
    }
}
