mod socket_loop;

use std::sync::atomic::{AtomicBool, Ordering};
pub use self::socket_loop::SocketLoop;

/// Lock-free atomic status mailbox coordinating an autonomous `SocketLoop` with `ConduitPlanet`.
pub struct PortalMailbox {
    alive: AtomicBool,
    surface_dirty: AtomicBool,
}

impl PortalMailbox {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(alive: AtomicBool, surface_dirty: AtomicBool) -> Self {
        Self { alive, surface_dirty }
    }

    pub const fn active() -> Self {
        Self::new(AtomicBool::new(true), AtomicBool::new(false))
    }

    pub fn running(&self) -> bool {
        self.alive.load(Ordering::Acquire)
    }

    pub fn halt(&self) {
        self.alive.store(false, Ordering::Release);
    }

    pub fn note_surface_change(&self) {
        self.surface_dirty.store(true, Ordering::Release);
    }

    pub fn take_surface_change(&self) -> bool {
        self.surface_dirty.swap(false, Ordering::AcqRel)
    }
}
