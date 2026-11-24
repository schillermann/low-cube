mod clock;
mod curve;
mod roster;

use glam::Vec3;
use world::{
    CompanionPresence, CraftPose, DeckFrame, DrivenCraft, PilotCraft, VesselBerth,
};
pub use self::clock::{GlideClock, TimelineClock};
pub use self::curve::{CircularStrideCurve, PresenceCurve, VesselDeckCurve, WaypointCurve};
pub use self::roster::SmoothRoster;

/// Polymorphic visibility role distinguishing `<= 160 m` companions from `160..2400 m` distant spacecraft.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SightRole {
    FullActor(VesselBerth),
    DistantVessel,
}

/// Immutable companion trajectory pairing posture, driven vehicle, `GlideClock`, and `SightRole` (`<= 4` fields).
#[derive(Clone, Copy, Debug)]
pub struct CompanionGlide {
    from: (CompanionPresence, PilotCraft),
    to: (CompanionPresence, PilotCraft),
    clock: GlideClock,
    role: SightRole,
}

impl CompanionGlide {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn with_role(
        from: (CompanionPresence, PilotCraft),
        to: (CompanionPresence, PilotCraft),
        clock: GlideClock,
        role: SightRole,
    ) -> Self {
        Self { from, to, clock, role }
    }

    pub const fn new(
        from: (CompanionPresence, PilotCraft),
        to: (CompanionPresence, PilotCraft),
        clock: GlideClock,
    ) -> Self {
        Self::with_role(from, to, clock, SightRole::FullActor(VesselBerth::planetary()))
    }

    pub const fn anchored_with_berth(
        now: f64,
        presence: CompanionPresence,
        craft: PilotCraft,
        berth: VesselBerth,
    ) -> Self {
        Self::with_role((presence, craft), (presence, craft), GlideClock::settled(now), SightRole::FullActor(berth))
    }

    pub const fn anchored(now: f64, presence: CompanionPresence, craft: PilotCraft) -> Self {
        Self::anchored_with_berth(now, presence, craft, VesselBerth::planetary())
    }

    pub const fn awaiting_first(presence: CompanionPresence) -> Self {
        Self::new(
            (presence, PilotCraft::Pedestrian),
            (presence, PilotCraft::Pedestrian),
            GlideClock::awaiting_first(),
        )
    }

    pub const fn live(self) -> bool { self.clock.live() }
    pub const fn full_actor(self) -> bool { matches!(self.role, SightRole::FullActor(_)) }
    pub const fn berth(self) -> VesselBerth {
        match self.role {
            SightRole::FullActor(b) => b,
            SightRole::DistantVessel => VesselBerth::planetary(),
        }
    }
    pub fn lease_expired_at(self, now: f64) -> bool { self.clock.lease_expired_at(now) }
    pub const fn authoritative_presence(self) -> CompanionPresence { self.to.0 }
    pub const fn authoritative_craft(self) -> PilotCraft { self.to.1 }
    pub const fn into_authoritative_craft(self) -> PilotCraft { self.to.1 }

    pub fn push_craft_with_speed(self, out: &mut Vec<(DrivenCraft, f32)>) {
        let PilotCraft::Driving(cur) = self.to.1 else { return };
        let step_speed = match self.from.1 {
            PilotCraft::Driving(prev) if prev.identity() == cur.identity() => {
                (cur.pose().origin() - prev.pose().origin()).length() / self.clock.cadence_secs().max(0.01)
            }
            _ => 0.0,
        };
        let thrust_speed = DeckFrame::thrust_from_pose(cur.pose()) * 180.0;
        out.push((cur, step_speed.max(thrust_speed)));
    }

    pub fn presence_at(self, render_secs: f64) -> CompanionPresence {
        self.presence_at_with_hull(render_secs, None)
    }

    pub fn presence_at_with_hull(self, render_secs: f64, hull_pose: Option<CraftPose>) -> CompanionPresence {
        let local_curve = CircularStrideCurve::new(
            WaypointCurve::new(self.from.0, self.to.0, self.clock),
            (self.from.0, self.to.0),
            self.clock,
        );
        match (self.berth(), hull_pose) {
            (VesselBerth::Aboard(_), Some(pose)) => VesselDeckCurve::new(local_curve, pose).sample_at(render_secs),
            _ => local_curve.sample_at(render_secs),
        }
    }

    pub fn craft_at(self, render_secs: f64) -> PilotCraft {
        self.from.1.interpolated(self.to.1, self.clock.alpha_at(render_secs))
    }

    pub fn shifted(self, delta: Vec3) -> Self {
        let aboard = self.berth().aboard_any();
        let next_from_p = if aboard { self.from.0 } else { self.from.0.shifted(delta) };
        let next_to_p = if aboard { self.to.0 } else { self.to.0.shifted(delta) };
        Self::with_role(
            (next_from_p, self.from.1.shifted(delta)),
            (next_to_p, self.to.1.shifted(delta)),
            self.clock,
            self.role,
        )
    }

    pub fn absorbed(self, now: f64, next_presence: CompanionPresence, next_craft: PilotCraft) -> Self {
        self.absorbed_with_role(now, next_presence, next_craft, self.role)
    }

    pub fn absorbed_with_berth(
        self,
        now: f64,
        next_presence: CompanionPresence,
        next_craft: PilotCraft,
        next_berth: VesselBerth,
    ) -> Self {
        self.absorbed_with_role(now, next_presence, next_craft, SightRole::FullActor(next_berth))
    }

    pub fn absorbed_with_role(
        self,
        now: f64,
        next_presence: CompanionPresence,
        next_craft: PilotCraft,
        next_role: SightRole,
    ) -> Self {
        if self.role != next_role {
            return Self::with_role((next_presence, next_craft), (next_presence, next_craft), GlideClock::settled(now), next_role);
        }
        if self.clock.live() && self.to.0 == next_presence && self.to.1 == next_craft {
            return self;
        }
        let cur_pres = self.presence_at(now);
        let teleport = matches!(next_role, SightRole::FullActor(_))
            && !next_presence.within_reach(
                cur_pres.transfer(|_, kin, _, _| kin.transfer(|p, _, _, _, _| p)),
                8.0,
            );
        if !self.clock.live() || teleport || self.clock.stale_at(now) {
            return Self::with_role((next_presence, next_craft), (next_presence, next_craft), GlideClock::settled(now), next_role);
        }
        let cur_craft = self.craft_at(now);
        Self::with_role((cur_pres, cur_craft), (next_presence, next_craft), self.clock.absorbed(now), next_role)
    }
}
