use std::collections::VecDeque;
use std::net::{SocketAddr, UdpSocket};
use world::{CompanionPresence, DrivenCraft, PilotCraft, PlanetaryWorld, WorldFact};
use crate::envelope::SanctuaryEnvelope;
use crate::fleet_ledger::FleetLedger;
use crate::glide::SmoothRoster;
use crate::seal::SessionSeal;

const PULSE_INTERVAL_SECS: f32 = 1.0 / 60.0;

/// Synchronized chronicle cursor, smoothly interpolated companion roster, and outbound avatar/craft state.
pub struct LinkSync {
    ack_seq: u32,
    pulse_clock: f32,
    roster: SmoothRoster,
    local_avatar: (CompanionPresence, PilotCraft),
}

impl LinkSync {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(
        ack_seq: u32,
        pulse_clock: f32,
        roster: SmoothRoster,
        local_avatar: (CompanionPresence, PilotCraft),
    ) -> Self {
        Self { ack_seq, pulse_clock, roster, local_avatar }
    }

    pub const fn local_slot(&self) -> u32 {
        self.local_avatar.0.slot()
    }

    pub const fn urgent_pulse(&self) -> bool {
        self.pulse_clock >= PULSE_INTERVAL_SECS
    }

    pub fn update_presence(&mut self, presence: CompanionPresence, cid: u32) {
        let prev_pos = self.local_avatar.0.transfer(|_, kin, _, _| kin.transfer(|p, _, _, _, _| p));
        if !presence.within_reach(prev_pos, 4.0) {
            self.pulse_clock = 1.0;
        }
        self.local_avatar.0 = presence.with_slot(cid);
    }

    pub fn update_craft(&mut self, craft: PilotCraft, cid: u32) {
        self.local_avatar.1 = craft.with_slot(cid);
    }

    pub fn assign_slot(&mut self, cid: u32) {
        self.local_avatar.0 = self.local_avatar.0.with_slot(cid);
        self.local_avatar.1 = self.local_avatar.1.with_slot(cid);
    }

    pub const fn roster(&self) -> &SmoothRoster {
        &self.roster
    }

    pub fn copy_companions(&self, out: &mut Vec<CompanionPresence>) {
        self.roster.copy_companions(out);
    }

    pub fn copy_crafts(&self, out: &mut Vec<DrivenCraft>) {
        self.roster.copy_crafts(out);
    }

    pub fn blend_subtick(&mut self, subtick_secs: f32) {
        self.roster.blend_subtick(subtick_secs);
    }

    pub const fn wildlife(&self) -> &crate::remote_wildlife::RemoteWildlife {
        self.roster.wildlife()
    }

    pub fn wildlife_mut(&mut self) -> &mut crate::remote_wildlife::RemoteWildlife {
        self.roster.wildlife_mut()
    }

    pub fn flush_pulse(&mut self, cid: u32, seal: &mut SessionSeal, peer: SocketAddr, socket: &UdpSocket) {
        self.pulse_clock = 0.0;
        let (mut payload, mut frame) = (Vec::new(), Vec::new());
        let pulse = SanctuaryEnvelope::GuestPulse {
            ack_seq: self.ack_seq,
            presence: self.local_avatar.0,
            craft: self.local_avatar.1,
        };
        pulse.inscribe(&mut payload);
        seal.seal(cid, &payload, &mut frame);
        let _ = socket.send_to(&frame, peer);
    }

    pub fn step_pulse(&mut self, dt: f32, cid: u32, seal: &mut SessionSeal, peer: SocketAddr, socket: &UdpSocket) {
        self.roster.advance_tick(dt);
        self.pulse_clock += dt;
        if self.pulse_clock >= PULSE_INTERVAL_SECS {
            let remainder = (self.pulse_clock - PULSE_INTERVAL_SECS).min(PULSE_INTERVAL_SECS);
            self.flush_pulse(cid, seal, peer, socket);
            self.pulse_clock = remainder;
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
            SanctuaryEnvelope::HostPulse { total_facts, cracks, companions, crafts, wildlife } => {
                self.roster.reconcile_snapshot(&companions, &crafts);
                self.roster.reconcile_wildlife(wildlife);
                if !cracks.is_empty() && deferred.len() < 64 {
                    deferred.push_back(SanctuaryEnvelope::HostPulse {
                        total_facts,
                        cracks,
                        companions: Vec::new(),
                        crafts: Vec::new(),
                        wildlife: world::WildlifePulse::calm(),
                    });
                }
            }
            SanctuaryEnvelope::HostChronicle { .. } => {
                if deferred.len() < 64 {
                    deferred.push_back(env);
                }
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
                if base_seq > self.ack_seq {
                    return false;
                }
                let skip = (self.ack_seq - base_seq) as usize;
                let mut changed = false;
                for fact in facts.into_iter().skip(skip) {
                    self.ack_seq += 1;
                    if self.apply_chronicle_fact(fact, world, fleet, predicted) {
                        changed = true;
                    }
                }
                changed
            }
            SanctuaryEnvelope::HostPulse { cracks, companions, crafts, wildlife, .. } => {
                if !companions.is_empty() || !crafts.is_empty() || !matches!(wildlife.sounder(), world::SounderPresence::Unseen) || wildlife.gore() != world::GoreStamp::calm() {
                    self.roster.reconcile_snapshot(&companions, &crafts);
                    self.roster.reconcile_wildlife(wildlife);
                }
                let mut changed = false;
                for crack in cracks {
                    if crack.enact_on(world) {
                        changed = true;
                    }
                }
                changed
            }
            _ => false,
        }
    }
}
