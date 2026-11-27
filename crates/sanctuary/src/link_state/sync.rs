use std::collections::VecDeque;
use std::net::{SocketAddr, UdpSocket};
use glam::{Quat, Vec3};
use world::{
    CompanionPresence, CosmicFrame, DrivenCraft, FrameHandoff, PilotCraft, PlanetaryWorld,
    PresenceKinematics, ProximityUtterance, SurvivorWelcome, VesselBerth, WorldFact,
};
use crate::envelope::SanctuaryEnvelope;
use crate::fleet_ledger::FleetLedger;
use crate::glide::SmoothRoster;
use crate::seal::SessionSeal;
use super::arrival::WelcomeArrival;

const PULSE_INTERVAL_SECS: f32 = 1.0 / 60.0;

/// Synchronized chronicle cursor, smoothly interpolated companion roster, avatar + berth + frame, and arrival (`<= 4` fields).
pub struct LinkSync {
    cursors: (u32, f32, bool),
    roster: SmoothRoster,
    local_avatar: (CompanionPresence, PilotCraft, CosmicFrame, VesselBerth),
    arrival: WelcomeArrival,
}

impl LinkSync {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn with_arrival(
        cursors: (u32, f32, bool),
        roster: SmoothRoster,
        local_avatar: (CompanionPresence, PilotCraft, CosmicFrame, VesselBerth),
        arrival: WelcomeArrival,
    ) -> Self {
        Self { cursors, roster, local_avatar, arrival }
    }

    pub const fn new(
        ack_seq: u32,
        pulse_clock: f32,
        roster: SmoothRoster,
        local_avatar: (CompanionPresence, PilotCraft),
    ) -> Self {
        Self::with_arrival(
            (ack_seq, pulse_clock, false),
            roster,
            (local_avatar.0, local_avatar.1, CosmicFrame::earth(), VesselBerth::planetary()),
            WelcomeArrival::natural(),
        )
    }

    pub const fn local_slot(&self) -> u32 { self.local_avatar.0.slot() }
    pub const fn urgent_pulse(&self) -> bool { self.cursors.1 >= PULSE_INTERVAL_SECS }
    pub const fn assigned_spawn(&self) -> Option<Vec3> { self.arrival.assigned_spawn() }
    pub const fn assigned_survivor(&self) -> Option<SurvivorWelcome> { self.arrival.assigned_survivor() }
    pub fn constrain_arrival(&self, pos: Vec3, height: f32, ground: Vec3, on_earth: bool) -> Vec3 {
        if on_earth && self.cursors.0 == 0 { self.arrival.constrain_unstreamed(pos, height, ground) } else { ground }
    }
    pub fn assign_welcome_epoch(&mut self, epoch_secs: f64) { self.arrival.assign_epoch(epoch_secs); }
    pub fn take_pending_epoch(&mut self) -> Option<f64> { self.arrival.take_pending_epoch() }
    pub fn take_pending_anchor(&mut self) -> Option<(CosmicFrame, Vec3)> { self.arrival.take_pending_anchor() }

    pub fn assign_survivor_welcome(&mut self, welcome: SurvivorWelcome, cid: u32) {
        self.arrival.assign_survivor(welcome);
        if let Some(rec) = welcome.restored_record() {
            let attire = self.local_avatar.0.transfer(|_, _, _, a| a);
            let kin = PresenceKinematics::new(rec.position(), Quat::IDENTITY, Quat::IDENTITY, (0.0, 0.0));
            self.local_avatar.0 = CompanionPresence::new(cid, kin, (0.0, 0.0, 0.0, 0.0), attire);
            self.local_avatar.2 = rec.frame();
            self.local_avatar.3 = VesselBerth::planetary();
            self.cursors.1 = 1.0;
        }
    }

    pub fn apply_frame_handoff(&mut self, handoff: FrameHandoff) {
        if !self.local_avatar.3.aboard_any() {
            self.local_avatar.0 = self.local_avatar.0.shifted(handoff.pos_shift());
        }
        self.local_avatar.1 = self.local_avatar.1.shifted(handoff.pos_shift());
        self.local_avatar.2 = handoff.next_frame();
        self.roster.shift_frame(handoff.pos_shift());
        self.cursors.1 = 1.0;
    }

    pub fn anchor_local_frame(&mut self, frame: CosmicFrame) {
        self.local_avatar.2 = frame;
        self.cursors.1 = 1.0;
    }

    pub fn update_presence(&mut self, presence: CompanionPresence, cid: u32) {
        self.update_berth_presence(presence, VesselBerth::planetary(), cid);
    }

    pub fn update_berth_presence(&mut self, presence: CompanionPresence, berth: VesselBerth, cid: u32) {
        let prev_pos = self.local_avatar.0.transfer(|_, kin, _, _| kin.transfer(|p, _, _, _, _| p));
        if self.local_avatar.3 != berth || !presence.within_reach(prev_pos, 4.0) {
            self.cursors.1 = 1.0;
        }
        self.local_avatar.0 = presence.with_slot(cid);
        self.local_avatar.3 = berth;
    }

    pub fn update_craft(&mut self, craft: PilotCraft, cid: u32) {
        self.local_avatar.1 = craft.with_slot(cid);
    }

    pub fn update_sky_watch(&mut self, active: bool) {
        if self.cursors.2 != active {
            self.cursors.2 = active;
            self.cursors.1 = 1.0;
        }
    }

    pub fn assign_slot(&mut self, cid: u32) {
        self.local_avatar.0 = self.local_avatar.0.with_slot(cid);
        self.local_avatar.1 = self.local_avatar.1.with_slot(cid);
    }

    pub fn assign_spawn_point(&mut self, spawn: Vec3, cid: u32) {
        self.arrival.assign_spawn(spawn);
        let attire = self.local_avatar.0.transfer(|_, _, _, a| a);
        let kin = PresenceKinematics::new(spawn, Quat::IDENTITY, Quat::IDENTITY, (0.0, 0.0));
        self.local_avatar.0 = CompanionPresence::new(cid, kin, (0.0, 0.0, 0.0, 0.0), attire);
        self.local_avatar.3 = VesselBerth::planetary();
    }

    pub const fn roster(&self) -> &SmoothRoster { &self.roster }
    pub fn copy_companions(&self, out: &mut Vec<CompanionPresence>) {
        self.roster.copy_companions_aboard(self.local_avatar.1, out);
    }
    pub fn copy_crafts(&self, out: &mut Vec<DrivenCraft>) { self.roster.copy_crafts(out); }
    pub fn drain_utterances(&mut self, out: &mut Vec<ProximityUtterance>) { self.roster.drain_utterances(out); }
    pub fn blend_subtick(&mut self, subtick_secs: f32) { self.roster.blend_subtick(subtick_secs); }

    pub fn flush_pulse(&mut self, cid: u32, seal: &mut SessionSeal, peer: SocketAddr, socket: &UdpSocket) {
        self.cursors.1 = 0.0;
        let (mut payload, mut frame) = (Vec::new(), Vec::new());
        let pulse = SanctuaryEnvelope::GuestPulse {
            ack_seq: self.cursors.0,
            presence: self.local_avatar.0,
            craft: self.local_avatar.1,
            frame: self.local_avatar.2,
            berth: self.local_avatar.3,
            sky_watch: self.cursors.2,
        };
        pulse.inscribe(&mut payload);
        seal.seal(cid, &payload, &mut frame);
        let _ = socket.send_to(&frame, peer);
    }

    pub fn step_pulse(&mut self, dt: f32, cid: u32, seal: &mut SessionSeal, peer: SocketAddr, socket: &UdpSocket) {
        self.roster.advance_tick(dt);
        self.cursors.1 += dt;
        if self.cursors.1 >= PULSE_INTERVAL_SECS {
            let remainder = (self.cursors.1 - PULSE_INTERVAL_SECS).min(PULSE_INTERVAL_SECS);
            self.flush_pulse(cid, seal, peer, socket);
            self.cursors.1 = remainder;
        }
    }

    fn apply_chronicle_fact(
        &self,
        fact: WorldFact,
        world: &mut dyn PlanetaryWorld,
        fleet: &mut FleetLedger,
        predicted: &mut VecDeque<WorldFact>,
    ) -> bool {
        let my_slot = self.local_slot();
        match fact {
            WorldFact::CraftCommissioned { blueprint } => {
                if let Some(pos) = predicted.iter().position(|p| matches!(p, WorldFact::CraftCommissioned { blueprint: b } if b.identity() == blueprint.identity())) {
                    let _ = predicted.remove(pos);
                }
                let _ = fleet.register_blueprint(blueprint);
                false
            }
            WorldFact::HelmClaimed { craft, pilot_slot, eye, direction } => {
                if pilot_slot == my_slot {
                    if let Some(pos) = predicted.iter().position(|p| matches!(p, WorldFact::HelmClaimed { craft: c, .. } if *c == craft)) {
                        let _ = predicted.remove(pos);
                        return false;
                    }
                }
                if craft.assigned() && fleet.helmed_by(craft, my_slot) {
                    if let Some(pos) = predicted.iter().position(|p| matches!(p, WorldFact::HelmClaimed { craft: c, .. } if *c == craft)) {
                        let _ = predicted.remove(pos);
                    }
                    fleet.revoke_local_helm(craft, pilot_slot, my_slot);
                    return false;
                }
                if craft.assigned() {
                    let _ = fleet.claim_helm(craft, pilot_slot);
                }
                if let Some(assembly) = world.board_vehicle(eye, direction, true) {
                    if assembly.identity().assigned() {
                        let _ = fleet.register_blueprint(assembly.blueprint().clone());
                    }
                    true
                } else {
                    false
                }
            }
            WorldFact::HelmReleased { craft, pose } => {
                if let Some(pos) = predicted.iter().position(|p| matches!(p, WorldFact::HelmReleased { craft: c, .. } if *c == craft)) {
                    let _ = predicted.remove(pos);
                    return false;
                }
                fleet.park_by_id(craft, pose, world)
            }
            other => {
                if let Some(pos) = predicted.iter().position(|p| *p == other) {
                    let _ = predicted.remove(pos);
                    false
                } else {
                    other.enact_on(world)
                }
            }
        }
    }

    pub fn absorb_standby_envelope(&mut self, env: SanctuaryEnvelope, deferred: &mut VecDeque<SanctuaryEnvelope>) {
        match env {
            SanctuaryEnvelope::HostPulse { total_facts, cracks, companions, crafts, berths, signals } => {
                self.roster.reconcile_snapshot_with_berths(&companions, &crafts, &berths);
                self.roster.reconcile_signals(&signals);
                if !cracks.is_empty() && deferred.len() < 64 {
                    deferred.push_back(SanctuaryEnvelope::HostPulse {
                        total_facts,
                        cracks,
                        companions: Vec::new(),
                        crafts: Vec::new(),
                        berths: Vec::new(),
                        signals: Vec::new(),
                    });
                }
            }
            SanctuaryEnvelope::HostChronicle { .. } => {
                if deferred.len() < 64 {
                    deferred.push_back(env);
                }
            }
            SanctuaryEnvelope::HostUtterance { utterance } => {
                self.roster.record_utterance(utterance);
            }
            _ => {}
        }
    }

    pub fn absorb_envelope(
        &mut self,
        env: SanctuaryEnvelope,
        world: &mut dyn PlanetaryWorld,
        fleet: &mut FleetLedger,
        predicted: &mut VecDeque<WorldFact>,
    ) -> bool {
        match env {
            SanctuaryEnvelope::HostChronicle { base_seq, facts } => {
                if base_seq > self.cursors.0 {
                    return false;
                }
                let skip = (self.cursors.0 - base_seq) as usize;
                let mut changed = false;
                for fact in facts.into_iter().skip(skip) {
                    self.cursors.0 += 1;
                    if self.apply_chronicle_fact(fact, world, fleet, predicted) {
                        changed = true;
                    }
                }
                changed
            }
            SanctuaryEnvelope::HostPulse { cracks, companions, crafts, berths, signals, .. } => {
                self.roster.reconcile_snapshot_with_berths(&companions, &crafts, &berths);
                self.roster.reconcile_signals(&signals);
                let mut changed = false;
                for crack in cracks {
                    if crack.enact_on(world) {
                        changed = true;
                    }
                }
                changed
            }
            SanctuaryEnvelope::HostUtterance { utterance } => {
                self.roster.record_utterance(utterance);
                false
            }
            _ => false,
        }
    }
}
