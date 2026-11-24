use std::collections::HashMap;
use std::net::{SocketAddr, UdpSocket};
use glam::Vec3;
use world::{
    CompanionPresence, DeckFrame, DrivenCraft, PilotCraft, PlanetaryWorld, PresenceAttire,
    ProximityUtterance, SolarProportions, SounderPresence, WildlifePulse,
};
use crate::admin_ward::SpawnBeacons;
use crate::audience::{
    MtuBudgetAudience, PulseAudience, CHAT_EARSHOT_METERS, INTIMATE_NEIGHBOR_CAP,
    NEARBY_SIGHT_METERS, PACKET_COMPANION_CAP,
};
use crate::aura::{HostAura, HostIdentity, HostPresence};
use crate::chronicle_log::ChronicleStore;
use crate::envelope::SanctuaryEnvelope;
use crate::fleet_ledger::FleetLedger;
use crate::glide::TimelineClock;
use crate::seal_batch::{OutboundFrame, ParallelSeals, PulseDelivery, SequentialDelivery};
use crate::sector_audience::{SectorGridAudience, VelocityScaledSight};
use crate::spatial_chronicle::HorizonStream;
use crate::ward::{GuestWard, WardVitals};

const PULSE_INTERVAL_SECS: f32 = 1.0 / 60.0;

fn sounder_near_anchor(sounder: SounderPresence, anchor: Vec3) -> SounderPresence {
    match sounder {
        SounderPresence::Sighted(boars)
            if boars.iter().any(|b| (b.position() - anchor).length_squared() <= NEARBY_SIGHT_METERS * NEARBY_SIGHT_METERS) =>
        {
            SounderPresence::Sighted(boars)
        }
        _ => SounderPresence::Unseen,
    }
}

/// Connected companion wards and horizon chronicle streams indexed by `companion_id` and X25519 key (`<= 4` fields).
pub struct GuestCohort {
    wards: HashMap<u32, (GuestWard, HorizonStream)>,
    key_index: HashMap<[u8; 32], u32>,
    next_id: u32,
    timeline: (TimelineClock, f32),
}

impl GuestCohort {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn with_streams(
        wards: HashMap<u32, (GuestWard, HorizonStream)>,
        key_index: HashMap<[u8; 32], u32>,
        next_id: u32,
        timeline: (TimelineClock, f32),
    ) -> Self {
        Self { wards, key_index, next_id, timeline }
    }

    pub fn new(
        wards: HashMap<u32, GuestWard>,
        key_index: HashMap<[u8; 32], u32>,
        next_id: u32,
        timeline: (TimelineClock, f32),
    ) -> Self {
        let paired = wards.into_iter().map(|(k, w)| (k, (w, HorizonStream::empty()))).collect();
        Self::with_streams(paired, key_index, next_id, timeline)
    }

    pub fn empty() -> Self {
        Self::with_streams(HashMap::new(), HashMap::new(), 2, (TimelineClock::origin(), 0.0))
    }

    pub const fn tick_secs(&self) -> f64 {
        self.timeline.0.tick_secs()
    }

    pub fn active_craft_count(&self) -> usize {
        let at = self.timeline.0.render_secs();
        self.wards
            .values()
            .filter(|(w, _)| matches!(w.craft_at(at), PilotCraft::Driving(_)))
            .count()
    }

    pub fn ward_mut(&mut self, cid: u32) -> Option<&mut GuestWard> {
        self.wards.get_mut(&cid).map(|(w, _)| w)
    }

    pub fn ward_anchor(&self, cid: u32) -> Option<Vec3> {
        self.wards.get(&cid).map(|(w, _)| w.anchor_position())
    }

    pub fn ensure_authored_fact(&mut self, cid: u32, fact_id: u32) {
        if let Some((_, stream)) = self.wards.get_mut(&cid) {
            stream.ensure_fact(fact_id);
        }
    }

    pub fn record_guest_gore(&mut self, cid: u32, impulse: Vec3) {
        if let Some((ward, _)) = self.wards.get_mut(&cid) {
            ward.record_gore(impulse);
        }
    }

    pub fn push_pedestrian_observers(&self, world: &dyn PlanetaryWorld, out: &mut Vec<(u32, Vec3, bool)>) {
        for (&cid, (ward, _)) in &self.wards {
            if let Some((pos, crouching)) = ward.pedestrian_observer(world) {
                out.push((cid, pos, crouching));
            }
        }
    }

    pub fn guest_companions(&self, host_anchor: Option<Vec3>, out: &mut Vec<CompanionPresence>) {
        let at = self.timeline.0.render_secs();
        out.clear();
        for (ward, _) in self.wards.values() {
            let p = ward.presence_at(at);
            if host_anchor.is_none_or(|anchor| p.within_reach(anchor, NEARBY_SIGHT_METERS)) {
                out.push(p);
            }
        }
    }

    pub fn guest_crafts(&self, host_anchor: Option<Vec3>, out: &mut Vec<DrivenCraft>) {
        let at = self.timeline.0.render_secs();
        out.clear();
        for (ward, _) in self.wards.values() {
            if let PilotCraft::Driving(dc) = ward.craft_at(at) {
                if host_anchor.is_none_or(|anchor| dc.within_reach(anchor, 2_400.0)) {
                    out.push(dc);
                }
            }
        }
    }

    pub fn constrain_crafts(&self, fleet: &FleetLedger, position: Vec3, player_height: f32, up: Vec3) -> Option<Vec3> {
        let at = self.timeline.0.render_secs();
        let mut best: Option<Vec3> = None;
        for (ward, _) in self.wards.values() {
            if let PilotCraft::Driving(craft) = ward.craft_at(at) {
                if craft.within_reach(position, 32.0) {
                    if let Some(top) = fleet.constrain_crafts(std::slice::from_ref(&craft), position, player_height) {
                        if best.is_none_or(|prev| top.dot(up) > prev.dot(up)) {
                            best = Some(top);
                        }
                    }
                }
            }
        }
        best
    }

    pub fn sample_deck_at(&self, fleet: &FleetLedger, position: Vec3, player_height: f32) -> Option<DeckFrame> {
        let at = self.timeline.0.render_secs();
        for (ward, _) in self.wards.values() {
            if let PilotCraft::Driving(craft) = ward.craft_at(at) {
                if craft.within_reach(position, 32.0) {
                    if let Some(deck) = fleet.sample_deck_at(std::slice::from_ref(&craft), position, player_height) {
                        return Some(deck);
                    }
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
        self.wards
            .entry(cid)
            .or_insert_with(|| (GuestWard::new(seal, peer, pres, WardVitals::fresh()), HorizonStream::empty()));
        (cid, beacon_stand)
    }

    pub fn advance_wards(&mut self, dt: f32, abandoned_out: &mut Vec<(u32, PilotCraft)>) -> bool {
        self.timeline.0 = self.timeline.0.advanced(dt);
        for (ward, _) in self.wards.values_mut() {
            ward.step(dt);
        }
        let expired: Vec<u32> = self
            .wards
            .iter()
            .filter_map(|(&cid, (w, _))| (!w.alive()).then_some(cid))
            .collect();
        for cid in expired {
            if let Some((dead, _)) = self.wards.remove(&cid) {
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
        for (&cid, (ward, _)) in &mut self.wards {
            if cid != speaker && utterance.audible_at(ward.anchor_position(), CHAT_EARSHOT_METERS) {
                ward.send_sealed(socket, cid, &payload, &mut scratch);
            }
        }
    }

    pub fn broadcast(
        &mut self,
        socket: &UdpSocket,
        chronicle: &mut dyn ChronicleStore,
        aura: &mut HostAura,
        proportions: SolarProportions,
        epoch_secs: f64,
    ) {
        chronicle.flush_tick();
        let earth_half = proportions.earth_half();
        aura.fact_index_mut().sync_with(chronicle, earth_half);
        let mut roster = Vec::with_capacity(self.wards.len() + 1);
        let mut crafts = Vec::new();
        if let HostPresence::Occupied(p) = aura.presence() {
            roster.push(p);
            aura.push_host_craft_with_speed(&mut crafts);
        }
        for (ward, _) in self.wards.values() {
            roster.push(ward.presence());
            ward.push_authoritative_craft_with_speed(&mut crafts);
        }
        let audience = MtuBudgetAudience::new(
            VelocityScaledSight::new(
                SectorGridAudience::indexed(&roster, NEARBY_SIGHT_METERS, earth_half),
                &crafts,
                earth_half,
            ),
            INTIMATE_NEIGHBOR_CAP,
            PACKET_COMPANION_CAP,
        );
        let sounder = aura.wildlife().presence();
        let sector_cracks = aura.drain_cracks();
        let mut frames = Vec::with_capacity(self.wards.len());
        let fact_index = aura.fact_index_mut() as &_;
        for (&cid, (ward, stream)) in &mut self.wards {
            let anchor = ward.anchor_position();
            let (mut others, mut other_crafts) = (Vec::new(), Vec::new());
            audience.select_for(cid, anchor, ward.sight_cursor_mut(), &mut others, &mut other_crafts);
            stream.gather_horizon(anchor, &other_crafts, fact_index, proportions, epoch_secs);
            let mut payloads = Vec::with_capacity(2);
            if let Some((base_seq, facts)) = stream.batch_from(ward.ack_seq(), chronicle) {
                let mut out = Vec::new();
                SanctuaryEnvelope::HostChronicle { base_seq, facts }.inscribe(&mut out);
                payloads.push(out);
            }
            let cracks = sector_cracks.collect_near(anchor, cid, NEARBY_SIGHT_METERS);
            let wildlife = WildlifePulse::new(sounder_near_anchor(sounder, anchor), ward.gore_stamp());
            let mut pulse_bytes = Vec::new();
            SanctuaryEnvelope::HostPulse {
                total_facts: stream.total(),
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
