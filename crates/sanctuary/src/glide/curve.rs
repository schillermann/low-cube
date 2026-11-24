use world::{CompanionPresence, CraftPose, PresenceRhythm};
use super::GlideClock;

/// Focused 1-method immutable companion trajectory contract evaluated purely at `render_secs`.
pub trait PresenceCurve {
    fn sample_at(&self, render_secs: f64) -> CompanionPresence;
}

/// Spatial waypoint trajectory interpolating 3D `PresenceKinematics` across a `GlideClock` horizon.
#[derive(Clone, Copy, Debug)]
pub struct WaypointCurve {
    from: CompanionPresence,
    to: CompanionPresence,
    clock: GlideClock,
}

impl WaypointCurve {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(from: CompanionPresence, to: CompanionPresence, clock: GlideClock) -> Self {
        Self { from, to, clock }
    }
}

impl PresenceCurve for WaypointCurve {
    fn sample_at(&self, render_secs: f64) -> CompanionPresence {
        let alpha = self.clock.alpha_at(render_secs);
        self.from.transfer(|_, from_kin, from_rhythm, _| {
            self.to.transfer(|slot, to_kin, _, attire| {
                CompanionPresence::new(slot, from_kin.interpolated(to_kin, alpha), from_rhythm, attire)
            })
        })
    }
}

/// `PresenceCurve` decorator wrapping stride phase modulo `TAU` and completing swing arcs forward.
#[derive(Clone, Copy, Debug)]
pub struct CircularStrideCurve<C> {
    origin: C,
    endpoints: (CompanionPresence, CompanionPresence),
    clock: GlideClock,
}

impl<C> CircularStrideCurve<C> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(
        origin: C,
        endpoints: (CompanionPresence, CompanionPresence),
        clock: GlideClock,
    ) -> Self {
        Self { origin, endpoints, clock }
    }
}

impl<C: PresenceCurve> PresenceCurve for CircularStrideCurve<C> {
    fn sample_at(&self, render_secs: f64) -> CompanionPresence {
        let alpha = self.clock.alpha_at(render_secs);
        let spatial = self.origin.sample_at(render_secs);
        let from_r = self.endpoints.0.transfer(|_, _, r, _| PresenceRhythm::from_wire(r));
        let to_r = self.endpoints.1.transfer(|_, _, r, _| PresenceRhythm::from_wire(r));
        let rhythm = from_r.interpolated(to_r, alpha).wire_tuple();
        spatial.transfer(|slot, kin, _, attire| CompanionPresence::new(slot, kin, rhythm, attire))
    }
}

/// `PresenceCurve` decorator folding hull-local crew trajectories onto a live 6-DOF `CraftPose` (`0.000 m` slip at `240 km/s`).
#[derive(Clone, Copy, Debug)]
pub struct VesselDeckCurve<C> {
    origin: C,
    hull_pose: CraftPose,
}

impl<C> VesselDeckCurve<C> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(origin: C, hull_pose: CraftPose) -> Self {
        Self { origin, hull_pose }
    }
}

impl<C: PresenceCurve> PresenceCurve for VesselDeckCurve<C> {
    fn sample_at(&self, render_secs: f64) -> CompanionPresence {
        self.origin.sample_at(render_secs).folded_onto_hull(self.hull_pose)
    }
}
