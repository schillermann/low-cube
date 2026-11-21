mod cadence;

use std::time::{Duration, Instant};
use winit::event_loop::ActiveEventLoop;
use crate::clock::{MonotonicTime, SystemClock};
use super::display::SessionDisplay;
use self::cadence::PulseCadence;

const TICKS_PER_SEC: u128 = 120;
const NANOS_PER_SEC: u128 = 1_000_000_000;
const TICK_SECS: f32 = 1.0 / 120.0;
const MAX_CATCHUP_TICKS: u64 = 12;

/// Immutable quantized tick count, sub-tick interpolation phase, and elastic sight scale for one frame.
#[derive(Clone, Copy, Debug)]
pub(super) struct PulseBeat {
    due_ticks: u32,
    subtick_blend: f32,
    sight_scale: f32,
}

impl PulseBeat {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub(super) const fn new(due_ticks: u32, subtick_blend: f32, sight_scale: f32) -> Self {
        Self { due_ticks, subtick_blend, sight_scale }
    }

    pub(super) const fn due_ticks(self) -> u32 {
        self.due_ticks
    }

    pub(super) const fn tick_seconds(self) -> f32 {
        TICK_SECS
    }

    pub(super) fn modal_seconds(self) -> f32 {
        (self.due_ticks as f32) * TICK_SECS
    }

    pub(super) const fn subtick_blend(self) -> f32 {
        self.subtick_blend
    }

    pub(super) const fn sight_scale(self) -> f32 {
        self.sight_scale
    }
}

/// Unified monotonic epoch heartbeat quantizing causal world ticks decoupled from GPU frame rate.
pub(super) struct EpochPulse {
    clock: Box<dyn MonotonicTime>,
    epoch: Instant,
    completed_ticks: u64,
    cadence: PulseCadence,
}

impl EpochPulse {
    /// Primary constructor doing nothing except assigning arguments to fields.
    fn new(clock: Box<dyn MonotonicTime>, epoch: Instant, completed_ticks: u64, cadence: PulseCadence) -> Self {
        Self { clock, epoch, completed_ticks, cadence }
    }

    pub(super) fn anchored_to(clock: Box<dyn MonotonicTime>) -> Self {
        let now = clock.current_instant();
        Self::new(clock, now, 0, PulseCadence::initial(now))
    }

    pub(super) fn realtime() -> Self {
        Self::anchored_to(Box::new(SystemClock::new()))
    }

    pub(super) fn mark_urgent(&mut self) {
        self.cadence = self.cadence.marked_urgent();
    }

    pub(super) fn begin_frame(&mut self) -> (Instant, PulseBeat) {
        let now = self.clock.current_instant();
        self.cadence = self.cadence.frame_begun(now);
        let elapsed_nanos = now.duration_since(self.epoch).as_nanos();
        let target_ticks = ((elapsed_nanos * TICKS_PER_SEC) / NANOS_PER_SEC) as u64;
        let raw_due = target_ticks.saturating_sub(self.completed_ticks);
        let due_ticks = raw_due.min(MAX_CATCHUP_TICKS) as u32;
        self.completed_ticks = target_ticks;
        let tick_origin = ((target_ticks as u128) * NANOS_PER_SEC) / TICKS_PER_SEC;
        let tick_span = (NANOS_PER_SEC as f32) / (TICKS_PER_SEC as f32);
        let subtick_blend = ((elapsed_nanos.saturating_sub(tick_origin) as f32) / tick_span).clamp(0.0, 1.0);
        (now, PulseBeat::new(due_ticks, subtick_blend, self.cadence.sight_scale()))
    }

    #[cfg(test)]
    pub(super) fn record_work(&mut self, work_started: Instant) {
        let work_done = self.clock.current_instant();
        let work_elapsed = work_done.duration_since(work_started);
        self.cadence = self.cadence.frame_completed(work_elapsed);
    }

    #[cfg(test)]
    pub(super) fn next_deadline(&self, modal_open: bool) -> Instant {
        self.cadence.deadline_at(self.next_tick_instant(), modal_open)
    }

    pub(super) fn complete_frame(&mut self, work_started: Instant, display: &SessionDisplay) {
        let work_done = self.clock.current_instant();
        let work_elapsed = work_done.duration_since(work_started);
        display.present();
        self.cadence = self.cadence.frame_completed(work_elapsed);
    }

    fn next_tick_instant(&self) -> Instant {
        let next_tick = self.completed_ticks.saturating_add(1) as u128;
        let next_nanos = (next_tick * NANOS_PER_SEC).div_ceil(TICKS_PER_SEC) as u64;
        self.epoch + Duration::from_nanos(next_nanos)
    }

    pub(super) fn pace_event_loop(&mut self, modal_open: bool, display: &SessionDisplay, event_loop: &ActiveEventLoop) {
        let now = self.clock.current_instant();
        self.cadence = self.cadence.pace_loop(now, self.next_tick_instant(), modal_open, display, event_loop);
    }
}
