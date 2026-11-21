/// Monotonic tick, EMA arrival cadence, and playout horizon for jitter-free snapshot interpolation.
#[derive(Clone, Copy, Debug)]
pub struct GlideClock {
    tick_elapsed: f32,
    span_secs: f32,
    since_arrival: f32,
    cadence_secs: f32,
}

impl GlideClock {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(tick_elapsed: f32, span_secs: f32, since_arrival: f32, cadence_secs: f32) -> Self {
        Self { tick_elapsed, span_secs, since_arrival, cadence_secs }
    }

    pub const fn settled() -> Self {
        Self::new(0.036, 0.036, 0.0, 0.017)
    }

    pub const fn awaiting_first() -> Self {
        Self::new(0.0, 0.0, 1.0, 0.017)
    }

    pub const fn live(self) -> bool {
        self.span_secs > 0.0
    }

    pub const fn stale(self) -> bool {
        self.since_arrival >= 0.50
    }

    pub fn absorbed(self) -> Self {
        let sample = self.since_arrival.clamp(0.010, 0.160);
        let next_cadence = if sample > self.cadence_secs {
            self.cadence_secs * 0.65 + sample * 0.35
        } else {
            self.cadence_secs * 0.88 + sample * 0.12
        };
        let span = (next_cadence * 2.15).clamp(0.036, 0.220);
        Self::new(0.0, span, 0.0, next_cadence)
    }

    pub(crate) fn advance_tick(self, dt: f32) -> Self {
        Self::new(
            (self.tick_elapsed + dt).min(self.span_secs),
            self.span_secs,
            (self.since_arrival + dt).min(1.0),
            self.cadence_secs,
        )
    }

    pub(crate) fn alpha(self, subtick_secs: f32) -> f32 {
        ((self.tick_elapsed + subtick_secs) / self.span_secs.max(1e-4)).clamp(0.0, 1.0)
    }
}
