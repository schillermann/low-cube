use glam::Vec3;
use world::{MeshTarget, PlanetaryWorld, SightCone, SounderPresence, WildlifePulse};
use fauna::RemoteSounder;
use crate::glide::GlideClock;
use crate::host_sounder::PendingShove;

/// Smoothly interpolated client mirror of the host's authoritative `SounderPresence` and `GoreStamp`.
#[derive(Clone, Copy, Debug)]
pub struct RemoteWildlife {
    span: (SounderPresence, SounderPresence),
    clock: GlideClock,
    gore_state: (u16, PendingShove),
    observer: RemoteSounder,
}

impl RemoteWildlife {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(
        span: (SounderPresence, SounderPresence),
        clock: GlideClock,
        gore_state: (u16, PendingShove),
        observer: RemoteSounder,
    ) -> Self {
        Self { span, clock, gore_state, observer }
    }

    /// Secondary constructor starting in a calm, unseen state before the first `HostPulse`.
    pub const fn calm() -> Self {
        Self::new(
            (SounderPresence::Unseen, SounderPresence::Unseen),
            GlideClock::awaiting_first(),
            (0, PendingShove::Calm),
            RemoteSounder::calm(),
        )
    }

    pub fn sounder_at(&self, render_secs: f64) -> SounderPresence {
        self.span.0.interpolated(self.span.1, self.clock.alpha_at(render_secs))
    }

    pub fn absorb(&mut self, now: f64, pulse: WildlifePulse) {
        if let Some(shove) = pulse.gore().absorb_into(&mut self.gore_state.0) {
            self.gore_state.1 = PendingShove::Gored(shove);
        }
        let next = pulse.sounder();
        if self.span.1 == next {
            return;
        }
        if matches!(self.span.1, SounderPresence::Unseen) || self.clock.stale_at(now) {
            self.span = (next, next);
            self.clock = GlideClock::settled(now);
            return;
        }
        let cur = self.sounder_at(now);
        self.span = (cur, next);
        self.clock = self.clock.absorbed(now);
    }

    pub fn step_listener(
        &mut self,
        render_secs: f64,
        dt: f32,
        listener: Vec3,
        view_dir: Vec3,
        world: &dyn PlanetaryWorld,
    ) -> Option<Vec3> {
        let cur = self.sounder_at(render_secs);
        self.observer.observe(dt, cur, listener, view_dir, world);
        self.gore_state.1.take()
    }

    pub fn intercept_strike(
        &mut self,
        render_secs: f64,
        eye: Vec3,
        view_dir: Vec3,
        max_reach: f32,
        world: &dyn PlanetaryWorld,
    ) -> bool {
        let cur = self.sounder_at(render_secs);
        self.observer.intercept_strike(cur, eye, view_dir, max_reach, world)
    }

    pub fn emit_visible(
        &self,
        render_secs: f64,
        cone: SightCone,
        world: &dyn PlanetaryWorld,
        target: &mut dyn MeshTarget,
    ) {
        self.observer.emit_visible(self.sounder_at(render_secs), cone, world, target);
    }
}
