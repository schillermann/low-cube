mod clock;

use glam::Vec3;
use world::{CompanionPresence, DeckFrame, DrivenCraft, PilotCraft, WildlifePulse};
use crate::fleet_ledger::FleetLedger;
use crate::remote_wildlife::RemoteWildlife;
pub use self::clock::GlideClock;

/// Per-companion smooth interpolation buffer locking driver posture and driven vehicle in phase.
#[derive(Clone, Debug)]
pub struct CompanionGlide {
    from: (CompanionPresence, PilotCraft),
    to: (CompanionPresence, PilotCraft),
    clock: GlideClock,
    subtick_secs: f32,
}

impl CompanionGlide {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(
        from: (CompanionPresence, PilotCraft),
        to: (CompanionPresence, PilotCraft),
        clock: GlideClock,
        subtick_secs: f32,
    ) -> Self {
        Self { from, to, clock, subtick_secs }
    }

    pub fn anchored(presence: CompanionPresence, craft: PilotCraft) -> Self {
        Self::new((presence, craft), (presence, craft), GlideClock::settled(), 0.0)
    }

    pub fn awaiting_first(presence: CompanionPresence) -> Self {
        Self::new(
            (presence, PilotCraft::Pedestrian),
            (presence, PilotCraft::Pedestrian),
            GlideClock::awaiting_first(),
            0.0,
        )
    }

    pub const fn live(&self) -> bool {
        self.clock.live()
    }

    pub const fn authoritative_presence(&self) -> CompanionPresence {
        self.to.0
    }

    pub const fn authoritative_craft(&self) -> PilotCraft {
        self.to.1
    }

    pub const fn into_authoritative_craft(self) -> PilotCraft {
        self.to.1
    }

    pub fn smooth_presence(&self) -> CompanionPresence {
        self.from.0.interpolated(self.to.0, self.clock.alpha(self.subtick_secs))
    }

    pub fn smooth_craft(&self) -> PilotCraft {
        self.from.1.interpolated(self.to.1, self.clock.alpha(self.subtick_secs))
    }

    pub fn absorb(&mut self, next_presence: CompanionPresence, next_craft: PilotCraft) {
        if self.clock.live() && self.to.0 == next_presence && self.to.1 == next_craft {
            return;
        }
        let cur_pres = self.smooth_presence();
        let teleport = !next_presence.within_reach(
            cur_pres.transfer(|_, kin, _, _| kin.transfer(|p, _, _, _, _| p)),
            8.0,
        );
        if !self.clock.live() || teleport || self.clock.stale() {
            *self = Self::anchored(next_presence, next_craft);
            return;
        }
        let cur_craft = self.smooth_craft();
        self.from = (cur_pres, cur_craft);
        self.to = (next_presence, next_craft);
        self.clock = self.clock.absorbed();
        self.subtick_secs = 0.0;
    }

    pub fn advance_tick(&mut self, dt: f32) {
        self.clock = self.clock.advance_tick(dt);
        self.subtick_secs = 0.0;
    }

    pub fn blend_subtick(&mut self, subtick_secs: f32) {
        self.subtick_secs = subtick_secs.max(0.0);
    }
}

/// Smoothly interpolated remote companion, vehicle, and wild-boar roster on a P2P guest client.
pub struct SmoothRoster {
    entries: Vec<(u32, CompanionGlide)>,
    wildlife: RemoteWildlife,
}

impl SmoothRoster {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(entries: Vec<(u32, CompanionGlide)>, wildlife: RemoteWildlife) -> Self {
        Self { entries, wildlife }
    }

    pub const fn empty() -> Self {
        Self::new(Vec::new(), RemoteWildlife::calm())
    }

    pub const fn wildlife(&self) -> &RemoteWildlife {
        &self.wildlife
    }

    pub fn wildlife_mut(&mut self) -> &mut RemoteWildlife {
        &mut self.wildlife
    }

    pub fn reconcile_wildlife(&mut self, pulse: WildlifePulse) {
        self.wildlife.absorb(pulse);
    }

    pub fn reconcile_snapshot(&mut self, companions: &[CompanionPresence], crafts: &[DrivenCraft]) {
        self.entries.retain(|(slot, _)| companions.iter().any(|c| c.same_slot(*slot)));
        for &comp in companions {
            let slot = comp.slot();
            let pilot_craft = crafts
                .iter()
                .find(|c| c.same_slot(slot))
                .copied()
                .map_or(PilotCraft::Pedestrian, PilotCraft::Driving);
            if let Some((_, glide)) = self.entries.iter_mut().find(|(s, _)| *s == slot) {
                glide.absorb(comp, pilot_craft);
            } else {
                self.entries.push((slot, CompanionGlide::anchored(comp, pilot_craft)));
            }
        }
    }

    pub fn advance_tick(&mut self, dt: f32) {
        for (_, glide) in &mut self.entries {
            glide.advance_tick(dt);
        }
        self.wildlife.advance_tick(dt);
    }

    pub fn blend_subtick(&mut self, subtick_secs: f32) {
        for (_, glide) in &mut self.entries {
            glide.blend_subtick(subtick_secs);
        }
        self.wildlife.blend_subtick(subtick_secs);
    }

    pub fn copy_companions(&self, out: &mut Vec<CompanionPresence>) {
        out.clear();
        for (_, glide) in &self.entries {
            out.push(glide.smooth_presence());
        }
    }

    pub fn copy_crafts(&self, out: &mut Vec<DrivenCraft>) {
        out.clear();
        for (_, glide) in &self.entries {
            glide.smooth_craft().push_active(out);
        }
    }

    pub fn constrain_crafts(
        &self,
        fleet: &FleetLedger,
        position: Vec3,
        player_height: f32,
        up: Vec3,
    ) -> Option<Vec3> {
        let mut best: Option<Vec3> = None;
        for (_, glide) in &self.entries {
            if let PilotCraft::Driving(craft) = glide.smooth_craft() {
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
        for (_, glide) in &self.entries {
            if let PilotCraft::Driving(craft) = glide.smooth_craft() {
                if let Some(deck) = fleet.sample_deck_at(std::slice::from_ref(&craft), position, player_height) {
                    return Some(deck);
                }
            }
        }
        None
    }
}
