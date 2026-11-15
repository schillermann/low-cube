use std::time::{Duration, Instant};
use winit::event_loop::{ActiveEventLoop, ControlFlow};
use super::super::display::SessionDisplay;

/// Homeostatic perception scale and rest cadence adapting to measured pre-swap hardware work.
#[derive(Clone, Copy, Debug)]
pub(super) struct ElasticHorizon {
    smoothed_work_secs: f32,
}

impl ElasticHorizon {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub(super) const fn new(smoothed_work_secs: f32) -> Self {
        Self { smoothed_work_secs }
    }

    pub(super) const fn balanced() -> Self {
        Self::new(0.004)
    }

    pub(super) fn absorb_work(self, work: Duration) -> Self {
        let sample = work.as_secs_f32().clamp(0.0001, 0.050);
        Self::new(self.smoothed_work_secs * 0.85 + sample * 0.15)
    }

    pub(super) fn sight_scale(self) -> f32 {
        if self.smoothed_work_secs <= 0.004 {
            let headroom = ((0.004 - self.smoothed_work_secs) / 0.003).clamp(0.0, 1.0);
            1.0 + 0.45 * headroom
        } else {
            let strain = ((self.smoothed_work_secs - 0.004) / 0.010).clamp(0.0, 1.0);
            1.0 - 0.35 * strain
        }
    }

    pub(super) fn urgent_interval(self) -> Duration {
        if self.smoothed_work_secs <= 0.003 {
            Duration::from_nanos(4_166_667)
        } else if self.smoothed_work_secs <= 0.007 {
            Duration::from_nanos(8_333_334)
        } else {
            Duration::from_nanos(16_666_667)
        }
    }

    pub(super) fn strained(self) -> bool {
        self.smoothed_work_secs > 0.006
    }
}

/// Encapsulated frame presentation timestamp and input-urgency latches for kernel sleep pacing.
#[derive(Clone, Copy, Debug)]
pub(super) struct WakeGate {
    last_present: Instant,
    urgent_input: bool,
    redraw_queued: bool,
}

impl WakeGate {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub(super) const fn new(last_present: Instant, urgent_input: bool, redraw_queued: bool) -> Self {
        Self { last_present, urgent_input, redraw_queued }
    }

    pub(super) const fn initial(now: Instant) -> Self {
        Self::new(now, true, false)
    }

    pub(super) const fn queued(self) -> bool {
        self.redraw_queued
    }

    pub(super) const fn marked_urgent(self) -> Self {
        Self::new(self.last_present, true, self.redraw_queued)
    }

    pub(super) const fn queued_now(self) -> Self {
        Self::new(self.last_present, self.urgent_input, true)
    }

    pub(super) const fn frame_begun(self) -> Self {
        Self::new(self.last_present, false, false)
    }

    pub(super) const fn frame_presented(self, presented_at: Instant) -> Self {
        Self::new(presented_at, self.urgent_input, self.redraw_queued)
    }

    pub(super) fn deadline(self, horizon: ElasticHorizon, next_tick_at: Instant, modal_open: bool) -> Instant {
        if self.urgent_input {
            return self.last_present + horizon.urgent_interval();
        }
        if modal_open || horizon.strained() {
            return self.last_present + Duration::from_nanos(16_666_667);
        }
        next_tick_at.max(self.last_present + horizon.urgent_interval())
    }
}

/// Composite adaptive frame cadence uniting `ElasticHorizon` and `WakeGate`.
#[derive(Clone, Copy, Debug)]
pub(super) struct PulseCadence {
    horizon: ElasticHorizon,
    gate: WakeGate,
}

impl PulseCadence {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub(super) const fn new(horizon: ElasticHorizon, gate: WakeGate) -> Self {
        Self { horizon, gate }
    }

    pub(super) const fn initial(now: Instant) -> Self {
        Self::new(ElasticHorizon::balanced(), WakeGate::initial(now))
    }

    pub(super) fn sight_scale(self) -> f32 {
        self.horizon.sight_scale()
    }

    pub(super) const fn marked_urgent(self) -> Self {
        Self::new(self.horizon, self.gate.marked_urgent())
    }

    pub(super) const fn frame_begun(self) -> Self {
        Self::new(self.horizon, self.gate.frame_begun())
    }

    pub(super) fn frame_completed(self, work: Duration, presented_at: Instant) -> Self {
        Self::new(self.horizon.absorb_work(work), self.gate.frame_presented(presented_at))
    }

    pub(super) fn pace_loop(
        self,
        now: Instant,
        next_tick_at: Instant,
        modal_open: bool,
        display: &SessionDisplay,
        event_loop: &ActiveEventLoop,
    ) -> Self {
        if self.gate.queued() {
            event_loop.set_control_flow(ControlFlow::Wait);
            return self;
        }
        let deadline = self.gate.deadline(self.horizon, next_tick_at, modal_open);
        if now >= deadline {
            display.request_redraw();
            event_loop.set_control_flow(ControlFlow::Wait);
            Self::new(self.horizon, self.gate.queued_now())
        } else {
            event_loop.set_control_flow(ControlFlow::WaitUntil(deadline));
            self
        }
    }
}
