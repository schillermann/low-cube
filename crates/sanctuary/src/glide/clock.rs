/// Monotonic simulation tick and render-playhead clock without mutable per-entity subtick fields.
#[derive(Clone, Copy, Debug)]
pub struct TimelineClock {
    tick_secs: f64,
    render_secs: f64,
}

impl TimelineClock {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(tick_secs: f64, render_secs: f64) -> Self {
        Self { tick_secs, render_secs }
    }

    pub const fn origin() -> Self {
        Self::new(0.0, 0.0)
    }

    pub const fn tick_secs(self) -> f64 {
        self.tick_secs
    }

    pub const fn render_secs(self) -> f64 {
        self.render_secs
    }

    pub fn advanced(self, dt: f32) -> Self {
        let next = self.tick_secs + f64::from(dt.max(0.0));
        Self::new(next, next)
    }

    pub fn blended(self, subtick_secs: f32) -> Self {
        Self::new(self.tick_secs, self.tick_secs + f64::from(subtick_secs.max(0.0)))
    }
}

/// Immutable arrival window and asymmetric EMA cadence for jitter-free trajectory evaluation.
#[derive(Clone, Copy, Debug)]
pub struct GlideClock {
    window: (f64, f64),
    last_arrival: f64,
    cadence_secs: f32,
}

impl GlideClock {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(window: (f64, f64), last_arrival: f64, cadence_secs: f32) -> Self {
        Self { window, last_arrival, cadence_secs }
    }

    pub const fn settled(now: f64) -> Self {
        Self::new((now, now + 0.036), now, 0.017)
    }

    pub const fn awaiting_first() -> Self {
        Self::new((0.0, 0.0), -1.0, 0.017)
    }

    pub const fn live(self) -> bool {
        self.last_arrival >= 0.0
    }

    pub const fn cadence_secs(self) -> f32 {
        self.cadence_secs
    }

    pub fn stale_at(self, now: f64) -> bool {
        !self.live() || (now - self.last_arrival) >= 0.50
    }

    pub fn lease_expired_at(self, now: f64) -> bool {
        !self.live() || (now - self.last_arrival) >= 0.85
    }

    pub fn absorbed(self, now: f64) -> Self {
        let sample = ((now - self.last_arrival) as f32).clamp(0.010, 0.160);
        let next_cadence = if sample > self.cadence_secs {
            self.cadence_secs * 0.65 + sample * 0.35
        } else {
            self.cadence_secs * 0.88 + sample * 0.12
        };
        let span = f64::from((next_cadence * 2.15).clamp(0.036, 0.220));
        Self::new((now, now + span), now, next_cadence)
    }

    pub fn alpha_at(self, render_secs: f64) -> f32 {
        if !self.live() {
            return 1.0;
        }
        let span = (self.window.1 - self.window.0).max(1e-4);
        (((render_secs - self.window.0) / span) as f32).clamp(0.0, 1.0)
    }
}
