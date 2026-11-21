use std::time::{Duration, Instant};
use winit::event_loop::{ActiveEventLoop, ControlFlow};
use super::super::display::SessionDisplay;

const MIN_FRAME_PERIOD: Duration = Duration::from_nanos(4_166_667);
const RELAXED_FRAME_PERIOD: Duration = Duration::from_nanos(15_500_000);

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

    pub(super) fn strained(self) -> bool {
        self.smoothed_work_secs > 0.008
    }
}

/// Encapsulated frame-origin timestamp and input-urgency latches for VSync-harmonious pacing.
#[derive(Clone, Copy, Debug)]
pub(super) struct WakeGate {
    frame_origin: Instant,
    urgent_input: bool,
    redraw_queued: bool,
}

impl WakeGate {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub(super) const fn new(frame_origin: Instant, urgent_input: bool, redraw_queued: bool) -> Self {
        Self { frame_origin, urgent_input, redraw_queued }
    }

    pub(super) const fn initial(now: Instant) -> Self {
        Self::new(now, true, false)
    }

    pub(super) const fn queued(self) -> bool {
        self.redraw_queued
    }

    pub(super) const fn marked_urgent(self) -> Self {
        Self::new(self.frame_origin, true, self.redraw_queued)
    }

    pub(super) const fn queued_now(self) -> Self {
        Self::new(self.frame_origin, self.urgent_input, true)
    }

    pub(super) const fn frame_begun(self, started_at: Instant) -> Self {
        Self::new(started_at, false, false)
    }

    pub(super) fn deadline(self, horizon: ElasticHorizon, next_tick_at: Instant, modal_open: bool) -> Instant {
        if modal_open || horizon.strained() {
            return self.frame_origin + RELAXED_FRAME_PERIOD;
        }
        if self.urgent_input {
            return self.frame_origin + MIN_FRAME_PERIOD;
        }
        next_tick_at.max(self.frame_origin + MIN_FRAME_PERIOD)
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

    pub(super) const fn frame_begun(self, started_at: Instant) -> Self {
        Self::new(self.horizon, self.gate.frame_begun(started_at))
    }

    pub(super) fn frame_completed(self, work: Duration) -> Self {
        Self::new(self.horizon.absorb_work(work), self.gate)
    }

    #[cfg(test)]
    pub(super) fn deadline_at(self, next_tick_at: Instant, modal_open: bool) -> Instant {
        self.gate.deadline(self.horizon, next_tick_at, modal_open)
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
