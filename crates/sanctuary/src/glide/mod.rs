mod clock;
mod curve;

use glam::Vec3;
use world::{CompanionPresence, DeckFrame, DrivenCraft, PilotCraft, WildlifePulse};
use crate::fleet_ledger::FleetLedger;
use crate::remote_wildlife::RemoteWildlife;
pub use self::clock::{GlideClock, TimelineClock};
pub use self::curve::{CircularStrideCurve, PresenceCurve, WaypointCurve};

/// Immutable companion trajectory pairing driver posture and driven vehicle across a `GlideClock` horizon.
#[derive(Clone, Copy, Debug)]
pub struct CompanionGlide {
    from: (CompanionPresence, PilotCraft),
    to: (CompanionPresence, PilotCraft),
    clock: GlideClock,
}

impl CompanionGlide {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(
        from: (CompanionPresence, PilotCraft),
        to: (CompanionPresence, PilotCraft),
        clock: GlideClock,
    ) -> Self {
        Self { from, to, clock }
    }

    pub const fn anchored(now: f64, presence: CompanionPresence, craft: PilotCraft) -> Self {
        Self::new((presence, craft), (presence, craft), GlideClock::settled(now))
    }

    pub const fn awaiting_first(presence: CompanionPresence) -> Self {
        Self::new(
            (presence, PilotCraft::Pedestrian),
            (presence, PilotCraft::Pedestrian),
            GlideClock::awaiting_first(),
        )
    }

    pub const fn live(self) -> bool {
        self.clock.live()
    }

    pub const fn authoritative_presence(self) -> CompanionPresence {
        self.to.0
    }

    pub const fn authoritative_craft(self) -> PilotCraft {
        self.to.1
    }

    pub const fn into_authoritative_craft(self) -> PilotCraft {
        self.to.1
    }

    pub fn presence_at(self, render_secs: f64) -> CompanionPresence {
        CircularStrideCurve::new(
            WaypointCurve::new(self.from.0, self.to.0, self.clock),
            (self.from.0, self.to.0),
            self.clock,
        )
        .sample_at(render_secs)
    }

    pub fn craft_at(self, render_secs: f64) -> PilotCraft {
        self.from.1.interpolated(self.to.1, self.clock.alpha_at(render_secs))
    }

    pub fn absorbed(self, now: f64, next_presence: CompanionPresence, next_craft: PilotCraft) -> Self {
        if self.clock.live() && self.to.0 == next_presence && self.to.1 == next_craft {
            return self;
        }
        let cur_pres = self.presence_at(now);
        let teleport = !next_presence.within_reach(
            cur_pres.transfer(|_, kin, _, _| kin.transfer(|p, _, _, _, _| p)),
            8.0,
        );
        if !self.clock.live() || teleport || self.clock.stale_at(now) {
            return Self::anchored(now, next_presence, next_craft);
        }
        let cur_craft = self.craft_at(now);
        Self::new((cur_pres, cur_craft), (next_presence, next_craft), self.clock.absorbed(now))
    }
}

/// Smoothly interpolated remote companion, vehicle, and wild-boar roster evaluated over `TimelineClock`.
pub struct SmoothRoster {
    timeline: TimelineClock,
    entries: Vec<(u32, CompanionGlide)>,
    wildlife: RemoteWildlife,
}

impl SmoothRoster {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(
        timeline: TimelineClock,
        entries: Vec<(u32, CompanionGlide)>,
        wildlife: RemoteWildlife,
    ) -> Self {
        Self { timeline, entries, wildlife }
    }

    pub const fn empty() -> Self {
        Self::new(TimelineClock::origin(), Vec::new(), RemoteWildlife::calm())
    }

    pub const fn render_secs(&self) -> f64 {
        self.timeline.render_secs()
    }

    pub const fn wildlife(&self) -> &RemoteWildlife {
        &self.wildlife
    }

    pub fn wildlife_mut(&mut self) -> &mut RemoteWildlife {
        &mut self.wildlife
    }

    pub fn active_craft_count(&self) -> usize {
        let at = self.timeline.render_secs();
        self.entries
            .iter()
            .filter(|(_, g)| matches!(g.craft_at(at), PilotCraft::Driving(_)))
            .count()
    }

    pub fn reconcile_wildlife(&mut self, pulse: WildlifePulse) {
        self.wildlife.absorb(self.timeline.tick_secs(), pulse);
    }

    pub fn reconcile_snapshot(&mut self, companions: &[CompanionPresence], crafts: &[DrivenCraft]) {
        let now = self.timeline.tick_secs();
        self.entries.retain(|(slot, _)| companions.iter().any(|c| c.same_slot(*slot)));
        for &comp in companions {
            let slot = comp.slot();
            let pilot_craft = crafts
                .iter()
                .find(|c| c.same_slot(slot))
                .copied()
                .map_or(PilotCraft::Pedestrian, PilotCraft::Driving);
            if let Some((_, glide)) = self.entries.iter_mut().find(|(s, _)| *s == slot) {
                *glide = glide.absorbed(now, comp, pilot_craft);
            } else {
                self.entries.push((slot, CompanionGlide::anchored(now, comp, pilot_craft)));
            }
        }
    }

    pub fn advance_tick(&mut self, dt: f32) {
        self.timeline = self.timeline.advanced(dt);
    }

    pub fn blend_subtick(&mut self, subtick_secs: f32) {
        self.timeline = self.timeline.blended(subtick_secs);
    }

    pub fn copy_companions(&self, out: &mut Vec<CompanionPresence>) {
        let at = self.timeline.render_secs();
        out.clear();
        for (_, glide) in &self.entries {
            out.push(glide.presence_at(at));
        }
    }

    pub fn copy_crafts(&self, out: &mut Vec<DrivenCraft>) {
        let at = self.timeline.render_secs();
        out.clear();
        for (_, glide) in &self.entries {
            glide.craft_at(at).push_active(out);
        }
    }

    pub fn constrain_crafts(
        &self,
        fleet: &FleetLedger,
        position: Vec3,
        player_height: f32,
        up: Vec3,
    ) -> Option<Vec3> {
        let at = self.timeline.render_secs();
        let mut best: Option<Vec3> = None;
        for (_, glide) in &self.entries {
            if let PilotCraft::Driving(craft) = glide.craft_at(at) {
                if let Some(top) = fleet.constrain_crafts(std::slice::from_ref(&craft), position, player_height) {
                    if best.is_none_or(|prev| top.dot(up) > prev.dot(up)) {
                        best = Some(top);
                    }
                }
            }
        }
        best
    }

    pub fn sample_deck_at(
        &self,
        fleet: &FleetLedger,
        position: Vec3,
        player_height: f32,
    ) -> Option<DeckFrame> {
        let at = self.timeline.render_secs();
        for (_, glide) in &self.entries {
            if let PilotCraft::Driving(craft) = glide.craft_at(at) {
                if let Some(deck) = fleet.sample_deck_at(std::slice::from_ref(&craft), position, player_height) {
                    return Some(deck);
                }
            }
        }
        None
    }
}
