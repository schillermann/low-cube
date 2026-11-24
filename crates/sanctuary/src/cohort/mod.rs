mod pulse;

use std::collections::HashMap;
use std::net::{SocketAddr, UdpSocket};
use glam::Vec3;
use world::{
    CompanionPresence, CosmicFrame, DeckFrame, DrivenCraft, GravityBubble, PilotCraft,
    PlanetaryWorld, PresenceAttire, ProximityUtterance, SolarProportions,
};
use crate::admin_ward::SpawnBeacons;
use crate::audience::{CHAT_EARSHOT_METERS, NEARBY_SIGHT_METERS};
use crate::aura::{HostAura, HostIdentity};
use crate::chronicle_log::ChronicleStore;
use crate::envelope::SanctuaryEnvelope;
use crate::fleet_ledger::FleetLedger;
use crate::glide::TimelineClock;
use crate::spatial_chronicle::HorizonStream;
use crate::ward::{GuestWard, WardVitals};
use self::pulse::CohortPulse;

const PULSE_INTERVAL_SECS: f32 = 1.0 / 60.0;

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

    pub const fn tick_secs(&self) -> f64 { self.timeline.0.tick_secs() }

    pub fn active_craft_count(&self) -> usize {
        let at = self.timeline.0.render_secs();
        self.wards.values().filter(|(w, _)| matches!(w.craft_at(at), PilotCraft::Driving(_))).count()
    }

    pub fn ward_mut(&mut self, cid: u32) -> Option<&mut GuestWard> {
        self.wards.get_mut(&cid).map(|(w, _)| w)
    }

    pub fn ward_locus(&self, cid: u32) -> Option<(CosmicFrame, Vec3)> {
        self.wards.get(&cid).map(|(w, _)| (w.cosmic_frame(), w.anchor_position()))
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

    pub fn guest_companions(
        &self,
        host_frame: CosmicFrame,
        host_anchor: Option<Vec3>,
        bubble: GravityBubble,
        out: &mut Vec<CompanionPresence>,
    ) {
        let at = self.timeline.0.render_secs();
        let reach_sq = f64::from(NEARBY_SIGHT_METERS * NEARBY_SIGHT_METERS);
        out.clear();
        for (ward, _) in self.wards.values() {
            let wf = ward.cosmic_frame();
            let p = ward.presence_at(at);
            let pos = p.transfer(|_, kin, _, _| kin.transfer(|pt, _, _, _, _| pt));
            if wf == host_frame {
                if host_anchor.is_none_or(|anchor| p.within_reach(anchor, NEARBY_SIGHT_METERS)) {
                    out.push(p);
                }
            } else if let Some(anchor) = host_anchor {
                let mapped = bubble.map_position(wf, pos, host_frame);
                if (mapped - anchor.as_dvec3()).length_squared() <= reach_sq {
                    out.push(p.with_position(mapped.as_vec3()));
                }
            }
        }
    }

    pub fn guest_crafts(
        &self,
        host_frame: CosmicFrame,
        host_anchor: Option<Vec3>,
        bubble: GravityBubble,
        out: &mut Vec<DrivenCraft>,
    ) {
        let at = self.timeline.0.render_secs();
        let reach_sq = 2_400.0_f64 * 2_400.0_f64;
        out.clear();
        for (ward, _) in self.wards.values() {
            let wf = ward.cosmic_frame();
            let PilotCraft::Driving(dc) = ward.craft_at(at) else { continue };
            if wf == host_frame {
                if host_anchor.is_none_or(|anchor| dc.within_reach(anchor, 2_400.0)) {
                    out.push(dc);
                }
            } else if let Some(anchor) = host_anchor {
                let mapped = bubble.map_position(wf, dc.pose().origin(), host_frame);
                if (mapped - anchor.as_dvec3()).length_squared() <= reach_sq {
                    out.push(dc.with_origin(mapped.as_vec3()));
                }
            }
        }
    }

    pub fn constrain_crafts(
        &self,
        fleet: &FleetLedger,
        host_frame: CosmicFrame,
        position: Vec3,
        player_height: f32,
        up: Vec3,
    ) -> Option<Vec3> {
        let at = self.timeline.0.render_secs();
        let mut best: Option<Vec3> = None;
        for (ward, _) in self.wards.values().filter(|(w, _)| w.cosmic_frame() == host_frame) {
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

    pub fn sample_deck_at(
        &self,
        fleet: &FleetLedger,
        host_frame: CosmicFrame,
        position: Vec3,
        player_height: f32,
    ) -> Option<DeckFrame> {
        let at = self.timeline.0.render_secs();
        for (ward, _) in self.wards.values().filter(|(w, _)| w.cosmic_frame() == host_frame) {
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

    pub fn advance_wards(&mut self, dt: f32, abandoned_out: &mut Vec<(u32, CosmicFrame, PilotCraft)>) -> bool {
        self.timeline.0 = self.timeline.0.advanced(dt);
        for (ward, _) in self.wards.values_mut() {
            ward.step(dt);
        }
        let expired: Vec<u32> = self.wards.iter().filter_map(|(&cid, (w, _))| (!w.alive()).then_some(cid)).collect();
        for cid in expired {
            if let Some((dead, _)) = self.wards.remove(&cid) {
                let frame = dead.cosmic_frame();
                abandoned_out.push((cid, frame, dead.into_pilot_craft()));
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

    pub fn relay_utterance(
        &mut self,
        socket: &UdpSocket,
        speaker_frame: CosmicFrame,
        utterance: ProximityUtterance,
        aura: &mut HostAura,
    ) {
        let speaker = utterance.transfer(|slot, _, _| slot);
        if speaker != 1 {
            aura.receive_utterance(speaker_frame, utterance.clone(), CHAT_EARSHOT_METERS);
        }
        let mut payload = Vec::with_capacity(64);
        SanctuaryEnvelope::HostUtterance { utterance: utterance.clone() }.inscribe(&mut payload);
        let mut scratch = Vec::with_capacity(96);
        for (&cid, (ward, _)) in &mut self.wards {
            if cid != speaker
                && ward.cosmic_frame() == speaker_frame
                && utterance.audible_at(ward.anchor_position(), CHAT_EARSHOT_METERS)
            {
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
        CohortPulse::new(proportions, epoch_secs).deliver(socket, &mut self.wards, chronicle, aura);
    }
}
