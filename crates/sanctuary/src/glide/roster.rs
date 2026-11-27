use glam::{Quat, Vec3};
use world::{
    CompanionPresence, CraftIdentity, CraftPose, DeckFrame, DrivenCraft, MeshTarget, PilotCraft,
    PresenceAttire, PresenceKinematics, ProximityUtterance, SightCone, SkySignal, VesselBerth,
};
use crate::audience::PACKET_COMPANION_CAP;
use crate::fleet_ledger::FleetLedger;
use super::{CompanionGlide, GlideClock, SightRole, TimelineClock};

/// Smoothly interpolated remote companion, vehicle, sky-signal, and proximity utterance roster (`<= 4` fields).
pub struct SmoothRoster {
    timeline: TimelineClock,
    entries: Vec<(u32, CompanionGlide)>,
    inbox: (Vec<ProximityUtterance>, Vec<SkySignal>),
}

impl SmoothRoster {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn with_signals(
        timeline: TimelineClock,
        entries: Vec<(u32, CompanionGlide)>,
        inbox: (Vec<ProximityUtterance>, Vec<SkySignal>),
    ) -> Self {
        Self { timeline, entries, inbox }
    }

    pub const fn new(
        timeline: TimelineClock,
        entries: Vec<(u32, CompanionGlide)>,
        utterances: Vec<ProximityUtterance>,
    ) -> Self {
        Self::with_signals(timeline, entries, (utterances, Vec::new()))
    }

    pub const fn empty() -> Self {
        Self::new(TimelineClock::origin(), Vec::new(), Vec::new())
    }

    pub fn record_utterance(&mut self, utterance: ProximityUtterance) {
        if self.inbox.0.len() >= 32 {
            self.inbox.0.remove(0);
        }
        self.inbox.0.push(utterance);
    }

    pub fn drain_utterances(&mut self, out: &mut Vec<ProximityUtterance>) { out.append(&mut self.inbox.0); }
    pub fn reconcile_signals(&mut self, signals: &[SkySignal]) {
        self.inbox.1.clear();
        self.inbox.1.extend_from_slice(signals);
    }
    pub fn emit_sky_signals(&self, cone: SightCone, target: &mut dyn MeshTarget) {
        for &sig in &self.inbox.1 {
            sig.emit(cone, target);
        }
    }
    pub fn active_craft_count(&self) -> usize {
        let at = self.timeline.render_secs();
        self.entries.iter().filter(|(_, g)| matches!(g.craft_at(at), PilotCraft::Driving(_))).count()
    }

    pub fn reconcile_snapshot(&mut self, companions: &[CompanionPresence], crafts: &[DrivenCraft]) {
        self.reconcile_snapshot_with_berths(companions, crafts, &[]);
    }

    pub fn reconcile_snapshot_with_berths(
        &mut self,
        companions: &[CompanionPresence],
        crafts: &[DrivenCraft],
        berths: &[(u32, CraftIdentity)],
    ) {
        let now = self.timeline.tick_secs();
        let saturated = companions.len() >= PACKET_COMPANION_CAP;
        self.entries.retain(|(slot, glide)| {
            companions.iter().any(|c| c.same_slot(*slot))
                || crafts.iter().any(|dc| dc.same_slot(*slot))
                || (saturated && !glide.lease_expired_at(now))
        });
        for &comp in companions {
            let slot = comp.slot();
            let pilot_craft = crafts
                .iter()
                .find(|c| c.same_slot(slot))
                .copied()
                .map_or(PilotCraft::Pedestrian, PilotCraft::Driving);
            let berth = berths
                .iter()
                .find(|(s, _)| *s == slot)
                .map_or(VesselBerth::planetary(), |&(_, id)| VesselBerth::aboard(id));
            if let Some((_, glide)) = self.entries.iter_mut().find(|(s, _)| *s == slot) {
                *glide = glide.absorbed_with_berth(now, comp, pilot_craft, berth);
            } else {
                self.entries.push((slot, CompanionGlide::anchored_with_berth(now, comp, pilot_craft, berth)));
            }
        }
        for &dc in crafts {
            let slot = dc.pilot_slot();
            if companions.iter().any(|c| c.same_slot(slot)) {
                continue;
            }
            let kin = PresenceKinematics::new(dc.pose().origin(), Quat::IDENTITY, Quat::IDENTITY, (0.0, 0.0));
            let stand_in = CompanionPresence::new(slot, kin, (0.0, 0.0, 0.0, 0.0), PresenceAttire::astronaut());
            let pc = PilotCraft::Driving(dc);
            if let Some((_, glide)) = self.entries.iter_mut().find(|(s, _)| *s == slot) {
                *glide = glide.absorbed_with_role(now, stand_in, pc, SightRole::DistantVessel);
            } else {
                self.entries.push((slot, CompanionGlide::with_role((stand_in, pc), (stand_in, pc), GlideClock::settled(now), SightRole::DistantVessel)));
            }
        }
    }

    pub fn advance_tick(&mut self, dt: f32) { self.timeline = self.timeline.advanced(dt); }
    pub fn blend_subtick(&mut self, subtick_secs: f32) { self.timeline = self.timeline.blended(subtick_secs); }

    pub fn shift_frame(&mut self, delta: Vec3) {
        for (_, glide) in &mut self.entries {
            *glide = glide.shifted(delta);
        }
    }

    fn resolve_hull_at(&self, craft_id: CraftIdentity, local_craft: PilotCraft, at: f64) -> Option<CraftPose> {
        if let PilotCraft::Driving(dc) = local_craft {
            if dc.identity() == craft_id {
                return Some(dc.pose());
            }
        }
        self.entries.iter().find_map(|(_, g)| match g.craft_at(at) {
            PilotCraft::Driving(dc) if dc.identity() == craft_id => Some(dc.pose()),
            _ => None,
        })
    }

    pub fn copy_companions(&self, out: &mut Vec<CompanionPresence>) {
        self.copy_companions_aboard(PilotCraft::Pedestrian, out);
    }

    pub fn copy_companions_aboard(&self, local_craft: PilotCraft, out: &mut Vec<CompanionPresence>) {
        let at = self.timeline.render_secs();
        out.clear();
        for (_, glide) in &self.entries {
            if !glide.full_actor() {
                continue;
            }
            let hull = glide.berth().craft_id().and_then(|id| self.resolve_hull_at(id, local_craft, at));
            out.push(glide.presence_at_with_hull(at, hull));
        }
    }

    pub fn copy_crafts(&self, out: &mut Vec<DrivenCraft>) {
        let at = self.timeline.render_secs();
        out.clear();
        for (_, glide) in &self.entries {
            glide.craft_at(at).push_active(out);
        }
    }

    pub fn constrain_crafts(&self, fleet: &FleetLedger, position: Vec3, player_height: f32, up: Vec3) -> Option<Vec3> {
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

    pub fn sample_deck_at(&self, fleet: &FleetLedger, position: Vec3, player_height: f32) -> Option<DeckFrame> {
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
