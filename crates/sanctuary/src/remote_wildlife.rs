use glam::Vec3;
use world::{MeshTarget, PlanetaryWorld, SightCone, SounderPresence, WildlifePulse};
use fauna::RemoteSounder;
use crate::glide::GlideClock;
use crate::host_sounder::PendingShove;

/// Smoothly interpolated client mirror of the host's authoritative `SounderPresence` and `GoreStamp`.
#[derive(Clone, Copy, Debug)]
pub struct RemoteWildlife {
    span: (SounderPresence, SounderPresence),
    clock: (GlideClock, f32),
    gore_state: (u16, PendingShove),
    observer: RemoteSounder,
}

impl RemoteWildlife {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(
        span: (SounderPresence, SounderPresence),
        clock: (GlideClock, f32),
        gore_state: (u16, PendingShove),
        observer: RemoteSounder,
    ) -> Self {
        Self { span, clock, gore_state, observer }
    }

    /// Secondary constructor starting in a calm, unseen state before the first `HostPulse`.
    pub const fn calm() -> Self {
        Self::new(
            (SounderPresence::Unseen, SounderPresence::Unseen),
            (GlideClock::settled(), 0.0),
            (0, PendingShove::Calm),
            RemoteSounder::calm(),
        )
    }

    pub fn smooth_sounder(&self) -> SounderPresence {
        self.span.0.interpolated(self.span.1, self.clock.0.alpha(self.clock.1))
    }

    pub fn absorb(&mut self, pulse: WildlifePulse) {
        if let Some(shove) = pulse.gore().absorb_into(&mut self.gore_state.0) {
            self.gore_state.1 = PendingShove::Gored(shove);
        }
        let next = pulse.sounder();
        if self.span.1 == next {
            return;
        }
        if matches!(self.span.1, SounderPresence::Unseen) || self.clock.0.stale() {
            self.span = (next, next);
            self.clock = (GlideClock::settled(), 0.0);
            return;
        }
        let cur = self.smooth_sounder();
        self.span = (cur, next);
        self.clock = (self.clock.0.absorbed(), 0.0);
    }

    pub fn advance_tick(&mut self, dt: f32) {
        self.clock = (self.clock.0.advance_tick(dt), 0.0);
    }

    pub fn blend_subtick(&mut self, subtick_secs: f32) {
        self.clock.1 = subtick_secs.max(0.0);
    }

    pub fn step_listener(
        &mut self,
        dt: f32,
        listener: Vec3,
        view_dir: Vec3,
        world: &dyn PlanetaryWorld,
    ) -> Option<Vec3> {
        let cur = self.smooth_sounder();
        self.observer.observe(dt, cur, listener, view_dir, world);
        self.gore_state.1.take()
    }

    pub fn intercept_strike(
        &mut self,
        eye: Vec3,
        view_dir: Vec3,
        max_reach: f32,
        world: &dyn PlanetaryWorld,
    ) -> bool {
        let cur = self.smooth_sounder();
        self.observer.intercept_strike(cur, eye, view_dir, max_reach, world)
    }

    pub fn emit_visible(&self, cone: SightCone, world: &dyn PlanetaryWorld, target: &mut dyn MeshTarget) {
        self.observer.emit_visible(self.smooth_sounder(), cone, world, target);
    }
}
