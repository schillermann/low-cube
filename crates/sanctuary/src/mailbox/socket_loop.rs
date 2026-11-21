use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};
use super::{PortalBeat, PortalMailbox};

const BEAT_NANOS: u64 = 8_333_333;

/// Polymorphic thread handle state avoiding `Option<JoinHandle<()>>`.
enum SocketThread {
    Detached,
    Running(JoinHandle<()>),
}

/// Autonomous 120 Hz P2P standby socket heartbeat taking over only when the main simulation loop stalls.
pub struct SocketLoop {
    mailbox: Arc<PortalMailbox>,
    thread_state: SocketThread,
}

impl SocketLoop {
    /// Primary constructor doing nothing except assigning arguments to fields.
    fn new(mailbox: Arc<PortalMailbox>, thread_state: SocketThread) -> Self {
        Self { mailbox, thread_state }
    }

    pub fn detached(mailbox: Arc<PortalMailbox>) -> Self {
        Self::new(mailbox, SocketThread::Detached)
    }

    pub fn active(&self) -> bool {
        matches!(self.thread_state, SocketThread::Running(_)) && self.mailbox.running()
    }

    pub fn spawn(
        wire: Arc<Mutex<dyn PortalBeat>>,
        mailbox: Arc<PortalMailbox>,
    ) -> Self {
        let loop_box = Arc::clone(&mailbox);
        let handle = std::thread::Builder::new()
            .name("low-cube-sanctuary".to_owned())
            .spawn(move || Self::drive_cadence(wire, loop_box))
            .map_or(SocketThread::Detached, SocketThread::Running);
        Self::new(mailbox, handle)
    }

    fn drive_cadence(wire: Arc<Mutex<dyn PortalBeat>>, mailbox: Arc<PortalMailbox>) {
        let step_dur = Duration::from_nanos(BEAT_NANOS);
        let mut last_beat = Instant::now();
        let mut next_wake = last_beat + step_dur;
        while mailbox.running() {
            let check_at = Instant::now();
            if mailbox.standby_needed(check_at) {
                if let Ok(mut session) = wire.lock() {
                    let beat_at = Instant::now();
                    if mailbox.standby_needed(beat_at) {
                        let dt = beat_at.saturating_duration_since(last_beat).as_secs_f32().clamp(0.0005, 0.020);
                        last_beat = beat_at;
                        let crafts = session.step_standby(dt);
                        mailbox.note_active_crafts(crafts);
                    } else {
                        last_beat = beat_at;
                    }
                }
            } else {
                last_beat = check_at;
            }
            let now = Instant::now();
            if next_wake > now {
                std::thread::park_timeout(next_wake - now);
                next_wake += step_dur;
            } else {
                next_wake = now + step_dur;
            }
        }
    }
}

impl Drop for SocketLoop {
    fn drop(&mut self) {
        if let SocketThread::Running(handle) = std::mem::replace(&mut self.thread_state, SocketThread::Detached) {
            self.mailbox.halt();
            handle.thread().unpark();
            let _ = handle.join();
        }
    }
}
