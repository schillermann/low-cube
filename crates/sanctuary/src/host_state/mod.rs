mod court;
mod journal;

use std::collections::VecDeque;
use std::net::{SocketAddr, UdpSocket};
use glam::Vec3;
use world::{
    Callsign, CompanionPresence, DrivenCraft, FrameHandoff, GravityBubble, MeshTarget, PilotCraft,
    PlanetaryWorld, ProximityUtterance, SolarProportions, SurvivorWelcome, WorldFact,
};
use crate::aura::{HostAura, HostIdentity};
use crate::chronicle_decorators::CompactedChronicle;
use crate::chronicle_log::{ChronicleLog, ChronicleStore};
use crate::cohort::{CohortAdmission, CohortGate, GuestCohort};
use crate::envelope::SanctuaryEnvelope;
use crate::fleet_ledger::FleetLedger;
use crate::host_sounder::HostSounder;
use crate::inbox_batch::{ParallelInbox, SequentialInbox};
use crate::passport::KnockFrame;
use crate::snapshot_chronicle::SnapshottedChronicle;
use crate::ticket::PortalTicket;
use crate::wormhole::{PortalCode, WormholeGate};
use self::court::{AbandonedDock, StandbyCourt};
pub use self::journal::AuthoredChronicle;

/// Strictly encapsulated chronicle store, guest cohort, host aura, and SPAKE2 wormhole gate (`<= 4` fields).
pub struct HostState {
    chronicle: Box<dyn ChronicleStore>,
    pub(super) cohort: GuestCohort,
    pub(super) aura: HostAura,
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

    /// Secondary constructor decorating `chronicle` with `SnapshottedChronicle(CompactedChronicle)`.
    pub fn new(
        chronicle: ChronicleLog,
        cohort: GuestCohort,
        aura: HostAura,
        wormhole: WormholeGate,
    ) -> Self {
        let store = SnapshottedChronicle::wrapping(CompactedChronicle::wrapping(chronicle));
        Self::with_store(Box::new(store), cohort, aura, wormhole)
    }

    pub fn occupied_zones_and_bodies(&self) -> (u8, Vec<world::BodyId>) {
        AbandonedDock::occupied_zones_and_bodies(&self.cohort, &self.aura)
    }

    pub fn compact_sectors(&mut self, earth_half: f32, world: &mut dyn PlanetaryWorld) {
        let dead = self.chronicle.compact_sectors(earth_half, world);
        self.aura.fact_index_mut().prune_superseded(&dead);
    }

    pub const fn code(&self) -> PortalCode { self.wormhole.code() }
    pub const fn epoch_secs(&self) -> f64 { self.aura.epoch_secs() }
    pub fn sync_epoch(&mut self, epoch_secs: f64) { self.aura.sync_epoch(epoch_secs); }
    pub fn reanchor_host(&mut self, handoff: FrameHandoff) { self.aura.reanchor_host(handoff); }
    pub fn punch_peer(&mut self, socket: &UdpSocket, peer: SocketAddr) { self.wormhole.punch_peer(socket, peer); }
    pub fn occupy_host(&mut self, presence: CompanionPresence) { self.aura.occupy(presence); }
    pub fn occupy_host_berth(&mut self, presence: CompanionPresence, berth: world::VesselBerth) {
        self.aura.occupy_berth(presence, berth);
    }
    pub fn pilot_host(&mut self, craft: PilotCraft) { self.aura.pilot(craft); }
    pub fn commit_host_owner(&mut self, pubkey: [u8; 32], record: world::SurvivorRecord, final_checkpoint: bool) {
        self.cohort.commit_owner_record(pubkey, record, final_checkpoint);
    }
    pub fn lookup_host_owner(&self, pubkey: [u8; 32]) -> Option<SurvivorWelcome> {
        self.cohort.lookup_owner(pubkey)
    }
    pub fn active_craft_count(&self) -> usize { self.cohort.active_craft_count() }

    pub fn guest_companions(&self, proportions: SolarProportions, out: &mut Vec<CompanionPresence>) {
        let bubble = GravityBubble::new(proportions, self.aura.epoch_secs());
        self.cohort.guest_companions(&self.aura, bubble, out);
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

    pub fn utter_from_host(&mut self, socket: &UdpSocket, host_callsign: Callsign, text: &str) {
        if let Some(pos) = self.aura.occupied_position() {
            let speaker_name = if host_callsign.present() { host_callsign } else { Callsign::host() };
            if let Some(utterance) = ProximityUtterance::named(1, speaker_name, pos, text) {
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

    pub fn admit_guest(
        &mut self,
        identity: &HostIdentity,
        peer: SocketAddr,
        guest_pub: [u8; 32],
    ) -> (u32, Option<Vec3>, SurvivorWelcome) {
        self.cohort.admit_guest(identity, &mut self.aura, peer, guest_pub)
    }

    pub fn admit_knock(
        &mut self,
        identity: &HostIdentity,
        peer: SocketAddr,
        knock: KnockFrame,
    ) -> CohortAdmission {
        self.cohort.admit_knock(identity, &mut self.aura, peer, knock)
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
        let Some(body) = self.cohort.ward_mut(cid).and_then(|w| w.open_datagram(src, pkt)) else { return false };
        let Some(env) = SanctuaryEnvelope::decode(&body) else { return false };
        StandbyCourt::apply_authoritative(self, socket, cid, env, proportions, world)
    }

    pub fn process_datagram_batch(
        &mut self,
        socket: &UdpSocket,
        grouped: std::collections::HashMap<u32, Vec<(u64, SocketAddr, Vec<u8>)>>,
        proportions: SolarProportions,
        world: &mut dyn PlanetaryWorld,
    ) -> bool {
        let inbox = ParallelInbox::hardware(SequentialInbox::new());
        let envelopes = self.cohort.open_datagram_batches(&inbox, grouped);
        let mut mutated = false;
        for (cid, env) in envelopes {
            if StandbyCourt::apply_authoritative(self, socket, cid, env, proportions, world) {
                mutated = true;
            }
        }
        mutated
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
        StandbyCourt::new(&mut self.cohort, &mut self.aura).absorb(socket, cid, src, pkt, proportions, deferred);
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
        let pulse_ready = self.cohort.advance_wards(dt, &self.aura, &mut abandoned);
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
        let pulse_ready = self.cohort.advance_wards(dt, &self.aura, &mut abandoned);
        let parked_any = AbandonedDock::new(self.chronicle.as_mut(), &mut self.aura).dock_expired(abandoned, world);
        (pulse_ready, parked_any)
    }

    pub fn broadcast(&mut self, socket: &UdpSocket, proportions: SolarProportions, epoch_secs: f64) {
        self.cohort.broadcast(socket, self.chronicle.as_mut(), &mut self.aura, proportions, epoch_secs);
    }
}
