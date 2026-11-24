use std::collections::HashMap;
use std::net::{SocketAddr, UdpSocket};
use glam::Vec3;
use world::{
    CompanionPresence, DeckFrame, DrivenCraft, PilotCraft, PlanetaryWorld, PresenceAttire,
    ProximityUtterance, WildlifePulse,
};
use crate::admin_ward::SpawnBeacons;
use crate::audience::{
    MtuBudgetAudience, ProximityAudience, PulseAudience, WorldAudience, CHAT_EARSHOT_METERS,
    INTIMATE_NEIGHBOR_CAP, NEARBY_SIGHT_METERS, PACKET_COMPANION_CAP,
};
use crate::aura::{HostAura, HostIdentity, HostPresence, PeerCrack};
use crate::chronicle_log::ChronicleStore;
use crate::envelope::{CrackNotice, SanctuaryEnvelope};
use crate::fleet_ledger::FleetLedger;
use crate::glide::TimelineClock;
use crate::seal_batch::{OutboundFrame, ParallelSeals, PulseDelivery, SequentialDelivery};
use crate::ward::{GuestWard, WardVitals};

const PULSE_INTERVAL_SECS: f32 = 1.0 / 60.0;

/// Connected companion wards indexed by `companion_id` and X25519 public key (`<= 4` fields).
pub struct GuestCohort {
    wards: HashMap<u32, GuestWard>,
    key_index: HashMap<[u8; 32], u32>,
    next_id: u32,
    timeline: (TimelineClock, f32),
}

impl GuestCohort {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(
        wards: HashMap<u32, GuestWard>,
        key_index: HashMap<[u8; 32], u32>,
        next_id: u32,
        timeline: (TimelineClock, f32),
    ) -> Self {
        Self { wards, key_index, next_id, timeline }
    }

    pub fn empty() -> Self {
        Self::new(HashMap::new(), HashMap::new(), 2, (TimelineClock::origin(), 0.0))
    }

    pub const fn tick_secs(&self) -> f64 {
        self.timeline.0.tick_secs()
    }

    pub fn active_craft_count(&self) -> usize {
        let at = self.timeline.0.render_secs();
        self.wards
            .values()
            .filter(|w| matches!(w.craft_at(at), PilotCraft::Driving(_)))
            .count()
    }

    pub fn ward_mut(&mut self, cid: u32) -> Option<&mut GuestWard> {
        self.wards.get_mut(&cid)
    }

    pub fn ward_anchor(&self, cid: u32) -> Option<Vec3> {
        self.wards.get(&cid).map(GuestWard::anchor_position)
    }

    pub fn record_guest_gore(&mut self, cid: u32, impulse: Vec3) {
        if let Some(ward) = self.wards.get_mut(&cid) {
            ward.record_gore(impulse);
        }
    }

    pub fn push_pedestrian_observers(&self, world: &dyn PlanetaryWorld, out: &mut Vec<(u32, Vec3, bool)>) {
        for (&cid, ward) in &self.wards {
            if let Some((pos, crouching)) = ward.pedestrian_observer(world) {
                out.push((cid, pos, crouching));
            }
        }
    }

    pub fn guest_companions(&self, host_anchor: Option<Vec3>, out: &mut Vec<CompanionPresence>) {
        let at = self.timeline.0.render_secs();
        out.clear();
        for ward in self.wards.values() {
            let p = ward.presence_at(at);
            if host_anchor.is_none_or(|anchor| p.within_reach(anchor, NEARBY_SIGHT_METERS)) {
                out.push(p);
            }
        }
    }

    pub fn guest_crafts(&self, host_anchor: Option<Vec3>, out: &mut Vec<DrivenCraft>) {
        let at = self.timeline.0.render_secs();
        out.clear();
        for ward in self.wards.values() {
            if let PilotCraft::Driving(dc) = ward.craft_at(at) {
                if host_anchor.is_none_or(|anchor| dc.within_reach(anchor, NEARBY_SIGHT_METERS)) {
                    out.push(dc);
                }
            }
        }
    }

    pub fn constrain_crafts(
        &self,
        fleet: &FleetLedger,
        position: Vec3,
        player_height: f32,
        up: Vec3,
    ) -> Option<Vec3> {
        let at = self.timeline.0.render_secs();
        let mut best: Option<Vec3> = None;
        for ward in self.wards.values() {
            if let PilotCraft::Driving(craft) = ward.craft_at(at) {
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
        let at = self.timeline.0.render_secs();
        for ward in self.wards.values() {
            if let PilotCraft::Driving(craft) = ward.craft_at(at) {
                if let Some(deck) = fleet.sample_deck_at(std::slice::from_ref(&craft), position, player_height) {
                    return Some(deck);
                }
            }
        }
        None
    }

    pub fn blend_subtick(&mut self, subtick_secs: f32) {
        self.timeline.0 = self.timeline.0.blended(subtick_secs);
    }

    pub fn admit_guest(
        &mut self,
        identity: &HostIdentity,
        beacons: &mut SpawnBeacons,
        peer: SocketAddr,
        guest_pub: [u8; 32],
    ) -> (u32, Option<Vec3>) {
        let cid = if let Some(&existing) = self.key_index.get(&guest_pub) {
            existing
        } else {
            let id = self.next_id;
            self.next_id = self.next_id.wrapping_add(1).max(2);
            self.key_index.insert(guest_pub, id);
            id
        };
        let seal = identity.keys().establish(guest_pub, false);
        let half = (identity.blocks_per_edge() as f32) * 0.5;
        let (kin, beacon_stand) = beacons.allocate_kinematics(half);
        let pres = CompanionPresence::new(cid, kin, (0.0, 0.0, 0.0, 0.0), PresenceAttire::squire());
        self.wards.entry(cid).or_insert_with(|| GuestWard::new(seal, peer, pres, WardVitals::fresh()));
        (cid, beacon_stand)
    }

    pub fn advance_wards(&mut self, dt: f32, abandoned_out: &mut Vec<(u32, PilotCraft)>) -> bool {
        self.timeline.0 = self.timeline.0.advanced(dt);
        for ward in self.wards.values_mut() {
            ward.step(dt);
        }
        let expired: Vec<u32> = self
            .wards
            .iter()
            .filter_map(|(&cid, w)| (!w.alive()).then_some(cid))
            .collect();
        for cid in expired {
            if let Some(dead) = self.wards.remove(&cid) {
                abandoned_out.push((cid, dead.into_pilot_craft()));
            }
        }
        self.timeline.1 += dt;
        if self.timeline.1 >= PULSE_INTERVAL_SECS {
            self.timeline.1 = (self.timeline.1 - PULSE_INTERVAL_SECS).min(PULSE_INTERVAL_SECS);
            true
        } else {
            false
        }
    }

    pub fn relay_utterance(&mut self, socket: &UdpSocket, utterance: ProximityUtterance, aura: &mut HostAura) {
        let speaker = utterance.transfer(|slot, _, _| slot);
        if speaker != 1 {
            aura.receive_utterance(utterance.clone(), CHAT_EARSHOT_METERS);
        }
        let mut payload = Vec::with_capacity(64);
        SanctuaryEnvelope::HostUtterance { utterance: utterance.clone() }.inscribe(&mut payload);
        let mut scratch = Vec::with_capacity(96);
        for (&cid, ward) in &mut self.wards {
            if cid != speaker && utterance.audible_at(ward.anchor_position(), CHAT_EARSHOT_METERS) {
                ward.send_sealed(socket, cid, &payload, &mut scratch);
            }
        }
    }

    pub fn broadcast(&mut self, socket: &UdpSocket, chronicle: &dyn ChronicleStore, aura: &mut HostAura) {
        let mut roster = Vec::with_capacity(self.wards.len() + 1);
        let mut crafts = Vec::new();
        if let HostPresence::Occupied(p) = aura.presence() {
            roster.push(p);
            aura.push_host_craft(&mut crafts);
        }
        for ward in self.wards.values() {
            roster.push(ward.presence());
            ward.push_authoritative_craft(&mut crafts);
        }
        let audience = MtuBudgetAudience::new(
            ProximityAudience::new(WorldAudience::new(&roster, &crafts), NEARBY_SIGHT_METERS),
            INTIMATE_NEIGHBOR_CAP,
            PACKET_COMPANION_CAP,
        );
        let sounder = aura.wildlife().presence();
        let peer_cracks = aura.drain_cracks();
        let total_facts = chronicle.total();
        let mut frames = Vec::with_capacity(self.wards.len());
        for (&cid, ward) in &mut self.wards {
            let mut payloads = Vec::with_capacity(2);
            if let Some((base_seq, facts)) = chronicle.batch_from(ward.ack_seq()) {
                let mut out = Vec::new();
                SanctuaryEnvelope::HostChronicle { base_seq, facts }.inscribe(&mut out);
                payloads.push(out);
            }
            let anchor = ward.anchor_position();
            let (mut others, mut other_crafts) = (Vec::new(), Vec::new());
            audience.select_for(cid, anchor, ward.sight_cursor_mut(), &mut others, &mut other_crafts);
            let cracks: Vec<CrackNotice> = peer_cracks
                .iter()
                .copied()
                .filter(|c| !c.authored_by(cid) && c.notice().near(anchor, NEARBY_SIGHT_METERS))
                .map(PeerCrack::notice)
                .collect();
            let wildlife = WildlifePulse::new(sounder, ward.gore_stamp());
            let mut pulse_bytes = Vec::new();
            SanctuaryEnvelope::HostPulse {
                total_facts,
                cracks,
                companions: others,
                crafts: other_crafts,
                wildlife,
            }
            .inscribe(&mut pulse_bytes);
            payloads.push(pulse_bytes);
            frames.push(OutboundFrame::prepare(cid, ward, payloads));
        }
        let delivery = ParallelSeals::hardware(SequentialDelivery::new());
        delivery.deliver(socket, &mut frames);
    }
}
