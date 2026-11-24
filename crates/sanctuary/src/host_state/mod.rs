mod court;
mod journal;

use std::collections::VecDeque;
use std::net::{SocketAddr, UdpSocket};
use glam::Vec3;
use world::{
    CompanionPresence, CosmicFrame, DrivenCraft, ExposedStratum, FrameHandoff, GravityBubble,
    MeshTarget, PilotCraft, PlanetaryWorld, ProximityUtterance, SolarProportions, WorldFact,
};
use fauna::BoarSounder;
use crate::admin_ward::{AdminBlockWard, GuestFactStamp};
use crate::aura::{HostAura, HostIdentity};
use crate::chronicle_decorators::CompactedChronicle;
use crate::chronicle_log::{ChronicleLog, ChronicleStore};
use crate::chronicled_planet::ChronicledPlanet;
use crate::cohort::GuestCohort;
use crate::envelope::{CrackNotice, SanctuaryEnvelope};
use crate::fleet_ledger::FleetLedger;
use crate::host_sounder::HostSounder;
use crate::ticket::PortalTicket;
use crate::warded_planet::WardedPlanet;
use crate::wormhole::{PortalCode, WormholeGate};
use self::court::AbandonedDock;

/// Strictly encapsulated chronicle store, guest cohort, host aura, and SPAKE2 wormhole gate (`<= 4` fields).
pub struct HostState {
    chronicle: Box<dyn ChronicleStore>,
    cohort: GuestCohort,
    aura: HostAura,
    wormhole: WormholeGate,
}

impl HostState {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn with_store(
        chronicle: Box<dyn ChronicleStore>,
        cohort: GuestCohort,
        aura: HostAura,
        wormhole: WormholeGate,
    ) -> Self {
        Self { chronicle, cohort, aura, wormhole }
    }

    /// Secondary constructor decorating `chronicle` with `CompactedChronicle`.
    pub fn new(
        chronicle: ChronicleLog,
        cohort: GuestCohort,
        aura: HostAura,
        wormhole: WormholeGate,
    ) -> Self {
        Self::with_store(Box::new(CompactedChronicle::wrapping(chronicle)), cohort, aura, wormhole)
    }

    pub const fn code(&self) -> PortalCode { self.wormhole.code() }
    pub const fn epoch_secs(&self) -> f64 { self.aura.epoch_secs() }
    pub fn sync_epoch(&mut self, epoch_secs: f64) { self.aura.sync_epoch(epoch_secs); }
    pub fn reanchor_host(&mut self, handoff: FrameHandoff) { self.aura.reanchor_host(handoff); }
    pub fn punch_peer(&mut self, socket: &UdpSocket, peer: SocketAddr) { self.wormhole.punch_peer(socket, peer); }
    pub fn occupy_host(&mut self, presence: CompanionPresence) { self.aura.occupy(presence); }
    pub fn pilot_host(&mut self, craft: PilotCraft) { self.aura.pilot(craft); }
    pub fn active_craft_count(&self) -> usize { self.cohort.active_craft_count() }

    pub fn guest_companions(&self, proportions: SolarProportions, out: &mut Vec<CompanionPresence>) {
        let bubble = GravityBubble::new(proportions, self.aura.epoch_secs());
        self.cohort.guest_companions(self.aura.host_frame(), self.aura.occupied_position(), bubble, out);
    }

    pub fn guest_crafts(&self, proportions: SolarProportions, out: &mut Vec<DrivenCraft>) {
        let bubble = GravityBubble::new(proportions, self.aura.epoch_secs());
        self.cohort.guest_crafts(self.aura.host_frame(), self.aura.occupied_position(), bubble, out);
    }

    pub fn constrain_guest_crafts(&self, position: Vec3, player_height: f32, up: Vec3) -> Option<Vec3> {
        self.cohort.constrain_crafts(self.aura.fleet(), self.aura.host_frame(), position, player_height, up)
    }

    pub fn sample_guest_deck(&self, position: Vec3, player_height: f32) -> Option<world::DeckFrame> {
        self.cohort.sample_deck_at(self.aura.fleet(), self.aura.host_frame(), position, player_height)
    }

    pub const fn fleet(&self) -> &FleetLedger { self.aura.fleet() }
    pub(crate) fn fleet_mut(&mut self) -> &mut FleetLedger { self.aura.fleet_mut() }
    pub const fn wildlife(&self) -> &HostSounder { self.aura.wildlife() }
    pub fn wildlife_mut(&mut self) -> &mut HostSounder { self.aura.wildlife_mut() }
    pub fn blend_subtick(&mut self, subtick_secs: f32) { self.cohort.blend_subtick(subtick_secs); }
    pub fn refresh_beacons(&mut self, world: &dyn PlanetaryWorld) { self.aura.beacons_mut().refresh_from(world); }
    pub fn drain_host_utterances(&mut self, out: &mut Vec<ProximityUtterance>) { self.aura.drain_utterances(out); }

    pub fn utter_from_host(&mut self, socket: &UdpSocket, text: &str) {
        if let Some(pos) = self.aura.occupied_position() {
            if let Some(utterance) = ProximityUtterance::sanitized(1, pos, text) {
                let hf = self.aura.host_frame();
                self.cohort.relay_utterance(socket, hf, utterance, &mut self.aura);
            }
        }
    }

    pub fn answer_wormhole(&mut self, socket: &UdpSocket, src: SocketAddr, pkt: &[u8], ticket: PortalTicket) {
        self.wormhole.answer_hello(socket, src, pkt, ticket);
    }

    pub fn emit_guest_crafts(&self, proportions: SolarProportions, target: &mut dyn MeshTarget) {
        let mut crafts = Vec::new();
        self.guest_crafts(proportions, &mut crafts);
        self.aura.fleet().emit_crafts(&crafts, target);
    }

    pub fn admit_guest(&mut self, identity: &HostIdentity, peer: SocketAddr, guest_pub: [u8; 32]) -> (u32, Option<Vec3>) {
        self.cohort.admit_guest(identity, self.aura.beacons_mut(), peer, guest_pub)
    }

    pub fn record_authored_fact(&mut self, author_slot: u32, fact: WorldFact) {
        match fact {
            WorldFact::Loosened { eye, direction, stage } => {
                let frame = if author_slot == 1 {
                    self.aura.host_frame()
                } else {
                    self.cohort.ward_locus(author_slot).map_or(CosmicFrame::earth(), |(f, _)| f)
                };
                self.aura.record_crack(author_slot, frame, CrackNotice::new(eye, direction, stage.min(4)));
            }
            WorldFact::OffWorld { body, ref fact } => {
                if let WorldFact::Loosened { eye, direction, stage } = **fact {
                    self.aura.record_crack(author_slot, CosmicFrame::Body(body), CrackNotice::new(eye, direction, stage.min(4)));
                }
            }
            _ => {}
        }
        let before = self.chronicle.total();
        self.chronicle.record(fact);
        if self.chronicle.total() > before && author_slot >= 2 {
            self.cohort.ensure_authored_fact(author_slot, before);
        }
    }

    pub fn record_disclosure(&mut self, face: u8, stratum: ExposedStratum) {
        self.chronicle.record(WorldFact::StratumDisclosed { face, stratum });
    }

    pub fn enact_guest_petition(&mut self, cid: u32, fact: WorldFact, world: &mut dyn PlanetaryWorld) -> bool {
        match fact {
            WorldFact::StratumDisclosed { .. } => false,
            WorldFact::BoarStruck { eye, direction, power, reach_cm } => {
                if !self.cohort.ward_mut(cid).is_some_and(|w| AbandonedDock::valid_boar_reach(w.cosmic_frame(), eye, direction, w)) {
                    return false;
                }
                let (_, drops) = self
                    .aura
                    .wildlife_mut()
                    .evaluate_strike(eye, direction, power.clamp(1, 4), (f32::from(reach_cm) * 0.01).clamp(0.5, 2.75), false, world);
                let Some(meat_rays) = drops else { return false };
                let mut decorated = ChronicledPlanet::new(world, self);
                BoarSounder::cast_meat_drops(meat_rays, &mut decorated)
            }
            other => {
                let stamped = GuestFactStamp::new(cid, other).stamped();
                let fleet = self.aura.fleet_mut();
                let enacted = self.cohort.ward_mut(cid).is_some_and(|ward| {
                    let mut guarded = AdminBlockWard::new(WardedPlanet::new(world, ward, fleet, cid));
                    stamped.enact_on(&mut guarded)
                });
                if enacted {
                    self.record_authored_fact(cid, stamped);
                }
                enacted
            }
        }
    }

    pub fn process_companion_datagram(
        &mut self,
        socket: &UdpSocket,
        cid: u32,
        src: SocketAddr,
        pkt: &[u8],
        proportions: SolarProportions,
        world: &mut dyn PlanetaryWorld,
    ) -> bool {
        let Some(body) = self.cohort.ward_mut(cid).and_then(|w| w.open_datagram(src, pkt)) else {
            return false;
        };
        let now = self.cohort.tick_secs();
        let bubble = GravityBubble::new(proportions, self.aura.epoch_secs());
        match SanctuaryEnvelope::decode(&body) {
            Some(SanctuaryEnvelope::GuestPulse { ack_seq, presence, craft, frame }) => {
                let vetted = match craft {
                    PilotCraft::Driving(dc) if !self.aura.fleet().helmed_by(dc.identity(), cid) => PilotCraft::Pedestrian,
                    other => other,
                };
                if let Some(ward) = self.cohort.ward_mut(cid) {
                    ward.absorb_pulse(cid, ack_seq, now, frame, presence, vetted, bubble, world);
                }
                false
            }
            Some(SanctuaryEnvelope::GuestPetition { fact }) => self.enact_guest_petition(cid, fact, world),
            Some(SanctuaryEnvelope::GuestUtterance { text }) => {
                if let Some((wf, pos)) = self.cohort.ward_locus(cid) {
                    if let Some(utterance) = ProximityUtterance::sanitized(cid, pos, &text) {
                        self.cohort.relay_utterance(socket, wf, utterance, &mut self.aura);
                    }
                }
                false
            }
            _ => false,
        }
    }

    pub fn process_standby_datagram(
        &mut self,
        socket: &UdpSocket,
        cid: u32,
        src: SocketAddr,
        pkt: &[u8],
        proportions: SolarProportions,
        deferred: &mut VecDeque<(u32, WorldFact)>,
    ) {
        let Some(body) = self.cohort.ward_mut(cid).and_then(|w| w.open_datagram(src, pkt)) else { return };
        let now = self.cohort.tick_secs();
        let bubble = GravityBubble::new(proportions, self.aura.epoch_secs());
        match SanctuaryEnvelope::decode(&body) {
            Some(SanctuaryEnvelope::GuestPulse { ack_seq, presence, craft, frame }) => {
                let vetted = match craft {
                    PilotCraft::Driving(dc) if !self.aura.fleet().helmed_by(dc.identity(), cid) => PilotCraft::Pedestrian,
                    other => other,
                };
                if let Some(ward) = self.cohort.ward_mut(cid) {
                    ward.absorb_standby_pulse(cid, ack_seq, now, frame, presence, vetted, bubble);
                }
            }
            Some(SanctuaryEnvelope::GuestPetition { fact }) => {
                if deferred.len() < 64 {
                    deferred.push_back((cid, fact));
                }
            }
            Some(SanctuaryEnvelope::GuestUtterance { text }) => {
                if let Some((wf, pos)) = self.cohort.ward_locus(cid) {
                    if let Some(utterance) = ProximityUtterance::sanitized(cid, pos, &text) {
                        self.cohort.relay_utterance(socket, wf, utterance, &mut self.aura);
                    }
                }
            }
            _ => {}
        }
    }

    pub fn step_standby_clocks(
        &mut self,
        socket: &UdpSocket,
        dt: f32,
        proportions: SolarProportions,
        deferred: &mut VecDeque<(u32, WorldFact)>,
    ) {
        self.aura.advance_epoch(dt);
        self.wormhole.step(socket, dt);
        let mut abandoned = Vec::new();
        let pulse_ready = self.cohort.advance_wards(dt, &mut abandoned);
        for (cid, frame, pilot_craft) in abandoned {
            if let PilotCraft::Driving(dc) = pilot_craft {
                if deferred.len() < 64 {
                    deferred.push_back((cid, AbandonedDock::scope_standby_release(frame, dc)));
                }
            }
            self.aura.fleet_mut().release_slot(cid);
        }
        if pulse_ready {
            let epoch = self.aura.epoch_secs();
            self.broadcast(socket, proportions, epoch);
        }
    }

    pub fn step_clocks(&mut self, socket: &UdpSocket, dt: f32, world: &mut dyn PlanetaryWorld) -> (bool, bool) {
        self.wormhole.step(socket, dt);
        let mut observers = Vec::with_capacity(8);
        if let Some((pos, crouching)) = self.aura.pedestrian_observer(world) {
            observers.push((1, pos, crouching));
        }
        self.cohort.push_pedestrian_observers(world, &mut observers);
        if !observers.is_empty() {
            if let Some((cid, shove)) = self.aura.wildlife_mut().step_authoritative(dt, &observers, world) {
                self.cohort.record_guest_gore(cid, shove);
            }
        }
        let mut abandoned = Vec::new();
        let pulse_ready = self.cohort.advance_wards(dt, &mut abandoned);
        let parked_any = AbandonedDock::new(self.chronicle.as_mut(), &mut self.aura).dock_expired(abandoned, world);
        (pulse_ready, parked_any)
    }

    pub fn broadcast(&mut self, socket: &UdpSocket, proportions: SolarProportions, epoch_secs: f64) {
        self.cohort.broadcast(socket, self.chronicle.as_mut(), &mut self.aura, proportions, epoch_secs);
    }
}
