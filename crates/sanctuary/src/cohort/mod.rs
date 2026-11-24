mod deck_view;
mod directory;
mod pulse;

use std::collections::HashMap;
use std::net::{SocketAddr, UdpSocket};
use glam::{Quat, Vec3};
use world::{
    Callsign, CompanionPresence, CosmicFrame, DeckFrame, DrivenCraft, GravityBubble, PilotCraft,
    PlanetaryWorld, PresenceAttire, PresenceKinematics, ProximityUtterance, SolarProportions,
    SurvivorRecord, SurvivorWelcome,
};
use crate::audience::CHAT_EARSHOT_METERS;
use crate::aura::{HostAura, HostIdentity};
use crate::chronicle_log::ChronicleStore;
use crate::envelope::{InventoryNotice, SanctuaryEnvelope};
use crate::fleet_ledger::FleetLedger;
use crate::glide::{CompanionGlide, TimelineClock};
use crate::link_state::WelcomeSpawn;
#[cfg(not(test))]
use crate::passport::HexKey;
use crate::passport::{KnockFrame, KnockIntent};
use crate::spatial_chronicle::HorizonStream;
use crate::survivor_ledger::{MemorySurvivors, SurvivorStore};
use crate::ward::{GuestWard, WardVitals};
use self::deck_view::HostDeckView;
pub use self::directory::{CohortAdmission, IdentityGrant, SlotLedger};
use self::pulse::CohortPulse;

const PULSE_INTERVAL_SECS: f32 = 1.0 / 60.0;

/// Connected companion wards, server-issued `SlotLedger`, and persistent `SurvivorStore` (`<= 4` fields).
pub struct GuestCohort {
    wards: HashMap<u32, (GuestWard, HorizonStream)>,
    identities: SlotLedger,
    survivors: Box<dyn SurvivorStore>,
    timeline: (TimelineClock, f32),
}

impl GuestCohort {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn with_ledger(
        wards: HashMap<u32, (GuestWard, HorizonStream)>,
        identities: SlotLedger,
        survivors: Box<dyn SurvivorStore>,
        timeline: (TimelineClock, f32),
    ) -> Self {
        Self { wards, identities, survivors, timeline }
    }

    pub fn with_survivors(
        wards: HashMap<u32, (GuestWard, HorizonStream)>,
        identities: (HashMap<[u8; 32], u32>, u32),
        survivors: Box<dyn SurvivorStore>,
        timeline: (TimelineClock, f32),
    ) -> Self {
        Self::with_ledger(wards, SlotLedger::from_index(identities.0, identities.1), survivors, timeline)
    }

    pub fn with_streams(
        wards: HashMap<u32, (GuestWard, HorizonStream)>,
        key_index: HashMap<[u8; 32], u32>,
        next_id: u32,
        timeline: (TimelineClock, f32),
    ) -> Self {
        Self::with_survivors(wards, (key_index, next_id), Box::new(MemorySurvivors::empty()), timeline)
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

    pub fn with_store(survivors: Box<dyn SurvivorStore>) -> Self {
        Self::with_ledger(HashMap::new(), SlotLedger::empty(), survivors, (TimelineClock::origin(), 0.0))
    }

    pub fn empty() -> Self {
        Self::with_store(Box::new(MemorySurvivors::empty()))
    }

    pub const fn tick_secs(&self) -> f64 { self.timeline.0.tick_secs() }
    fn view(&self) -> HostDeckView<'_> { HostDeckView::new(&self.wards, self.timeline.0.render_secs()) }

    pub fn active_craft_count(&self) -> usize {
        let at = self.timeline.0.render_secs();
        self.wards.values().filter(|(w, _)| matches!(w.craft_at(at), PilotCraft::Driving(_))).count()
    }

    pub fn ward_mut(&mut self, cid: u32) -> Option<&mut GuestWard> { self.wards.get_mut(&cid).map(|(w, _)| w) }
    pub fn ward_callsign(&self, cid: u32) -> Callsign {
        self.wards.get(&cid).map_or(Callsign::unnamed(), |(w, _)| w.callsign())
    }
    pub fn ward_locus(&self, cid: u32) -> Option<(CosmicFrame, Vec3)> {
        self.wards.get(&cid).map(|(w, _)| (w.cosmic_frame(), w.anchor_position()))
    }

    pub fn ward_world_locus(&self, cid: u32, aura: &HostAura) -> Option<(CosmicFrame, Vec3)> {
        let (ward, _) = self.wards.get(&cid)?;
        let raw = ward.anchor_position();
        if let Some(id) = ward.berth().craft_id() {
            if let Some((sf, pose)) = self.view().active_ship(id, aura) {
                return Some((sf, pose.world_point_at(raw)));
            }
        }
        Some((ward.cosmic_frame(), raw))
    }

    pub fn commit_survivor(&mut self, cid: u32, aura: &HostAura, final_checkpoint: bool) {
        let Some((wf, wpos)) = self.ward_world_locus(cid, aura) else { return };
        if let Some((ward, _)) = self.wards.get(&cid) {
            ward.commit_checkpoint(wf, wpos, final_checkpoint, self.survivors.as_mut());
        }
    }

    pub fn apply_inventory_notice(&mut self, cid: u32, notice: InventoryNotice, aura: &HostAura) -> bool {
        let applied = self.wards.get_mut(&cid).is_some_and(|(ward, _)| match notice {
            InventoryNotice::Crafted { recipe } => ward.manifest_mut().apply_recipe(recipe),
            InventoryNotice::Consumed { relic_code } => ward.manifest_mut().consume_edible(relic_code),
        });
        if applied {
            self.commit_survivor(cid, aura, true);
        }
        applied
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

    pub fn guest_companions(&self, aura: &HostAura, bubble: GravityBubble, out: &mut Vec<CompanionPresence>) {
        self.view().companions(aura, bubble, out);
    }

    pub fn guest_crafts(&self, hf: CosmicFrame, anchor: Option<Vec3>, bubble: GravityBubble, out: &mut Vec<DrivenCraft>) {
        self.view().crafts(hf, anchor, bubble, out);
    }

    pub fn constrain_crafts(&self, fleet: &FleetLedger, hf: CosmicFrame, pos: Vec3, h: f32, up: Vec3) -> Option<Vec3> {
        self.view().constrain_crafts(fleet, hf, pos, h, up)
    }

    pub fn sample_deck_at(&self, fleet: &FleetLedger, hf: CosmicFrame, pos: Vec3, h: f32) -> Option<DeckFrame> {
        self.view().sample_deck_at(fleet, hf, pos, h)
    }

    pub fn blend_subtick(&mut self, subtick_secs: f32) { self.timeline.0 = self.timeline.0.blended(subtick_secs); }

    pub fn admit_guest(
        &mut self,
        identity: &HostIdentity,
        aura: &mut HostAura,
        peer: SocketAddr,
        guest_pub: [u8; 32],
    ) -> (u32, Option<Vec3>, SurvivorWelcome) {
        let knock = KnockFrame::new(guest_pub, KnockIntent::PinnedKey, Callsign::unnamed());
        self.admit_knock(identity, aura, peer, knock).into_legacy_tuple()
    }

    pub fn admit_knock(
        &mut self,
        identity: &HostIdentity,
        aura: &mut HostAura,
        peer: SocketAddr,
        knock: KnockFrame,
    ) -> CohortAdmission {
        let (_, desired_callsign) = knock.transfer(|pk, cs| (pk, cs));
        let (cid, survivor_pub, minted_secret, knock_retry) =
            self.identities.resolve_knock(knock, self.survivors.as_ref());
        if self.wards.get(&cid).is_some_and(|(w, _)| w.established_traffic()) {
            self.commit_survivor(cid, aura, true);
            let _ = self.wards.remove(&cid);
        }
        let active: Vec<([u8; 32], Callsign)> = self
            .wards
            .values()
            .map(|(w, _)| (w.guest_pubkey(), w.callsign()))
            .collect();
        let welcome = match self.wards.get(&cid) {
            Some((w, _)) if knock_retry && !w.established_traffic() => SurvivorWelcome::FreshKit(w.manifest()),
            _ => self.survivors.admit_or_restore(survivor_pub),
        };
        let callsign = self.survivors.resolve_callsign(survivor_pub, desired_callsign, &active);
        let half = (identity.blocks_per_edge() as f32) * 0.5;
        let (kin, beacon_stand) = match welcome.restored_record() {
            Some(rec) => (PresenceKinematics::new(rec.position(), Quat::IDENTITY, Quat::IDENTITY, (0.0, 0.0)), None),
            None => aura.beacons_mut().allocate_kinematics(half),
        };
        if minted_secret.is_some() && !knock_retry {
            let spawn_pos = kin.transfer(|p, _, _, _, _| p);
            let initial = SurvivorRecord::with_callsign(CosmicFrame::earth(), spawn_pos, welcome.manifest(), callsign);
            self.survivors.commit(survivor_pub, initial, true);
            #[cfg(not(test))]
            println!("[SANCTUARY] Issued identity #{} to '{}' (slot P{cid})", HexKey::new(survivor_pub).short_tag(), callsign.as_str());
        } else if let Some(prev) = self.survivors.lookup(survivor_pub) {
            if prev.callsign() != callsign {
                self.survivors.commit(survivor_pub, prev.renamed(callsign), true);
            }
            #[cfg(not(test))]
            println!("[SANCTUARY] Restored survivor '{}' (#{}, slot P{cid})", callsign.as_str(), HexKey::new(survivor_pub).short_tag());
        }
        if !self.wards.contains_key(&cid) {
            let seal = identity.keys().establish(survivor_pub, false);
            let pres = CompanionPresence::new(cid, kin, (0.0, 0.0, 0.0, 0.0), PresenceAttire::squire());
            let frame = welcome.restored_record().map_or(CosmicFrame::earth(), |r| r.frame());
            let vitals = WardVitals::named_survivor(survivor_pub, callsign, welcome.manifest());
            let ward = GuestWard::with_locus(seal, (peer, frame), CompanionGlide::awaiting_first(pres), vitals);
            self.wards.insert(cid, (ward, HorizonStream::empty()));
        }
        let grant = match minted_secret {
            Some(secret) => IdentityGrant::Issued { secret, callsign },
            None => IdentityGrant::Confirmed(callsign),
        };
        CohortAdmission::new(cid, WelcomeSpawn::from_optional(beacon_stand), welcome, grant)
    }

    pub fn advance_wards(&mut self, dt: f32, aura: &HostAura, abandoned_out: &mut Vec<(u32, CosmicFrame, PilotCraft)>) -> bool {
        self.timeline.0 = self.timeline.0.advanced(dt);
        for (ward, _) in self.wards.values_mut() {
            ward.step(dt);
        }
        let expired: Vec<u32> = self.wards.iter().filter_map(|(&cid, (w, _))| (!w.alive()).then_some(cid)).collect();
        for cid in expired {
            self.commit_survivor(cid, aura, true);
            if let Some((dead, _)) = self.wards.remove(&cid) {
                abandoned_out.push((cid, dead.cosmic_frame(), dead.into_pilot_craft()));
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

    pub fn relay_utterance(&mut self, socket: &UdpSocket, speaker_frame: CosmicFrame, utterance: ProximityUtterance, aura: &mut HostAura) {
        let speaker = utterance.transfer(|slot, _, _| slot);
        if speaker != 1 {
            aura.receive_utterance(speaker_frame, utterance.clone(), CHAT_EARSHOT_METERS);
        }
        let mut payload = Vec::with_capacity(80);
        SanctuaryEnvelope::HostUtterance { utterance: utterance.clone() }.inscribe(&mut payload);
        let mut scratch = Vec::with_capacity(112);
        let recipient_ids: Vec<u32> = self
            .wards
            .keys()
            .copied()
            .filter(|&cid| {
                cid != speaker
                    && self
                        .ward_world_locus(cid, aura)
                        .is_some_and(|(wf, pos)| wf == speaker_frame && utterance.audible_at(pos, CHAT_EARSHOT_METERS))
            })
            .collect();
        for cid in recipient_ids {
            if let Some((ward, _)) = self.wards.get_mut(&cid) {
                ward.send_sealed(socket, cid, &payload, &mut scratch);
            }
        }
    }

    pub fn broadcast(&mut self, socket: &UdpSocket, chronicle: &mut dyn ChronicleStore, aura: &mut HostAura, proportions: SolarProportions, epoch_secs: f64) {
        self.survivors.flush_tick();
        CohortPulse::new(proportions, epoch_secs).deliver(socket, &mut self.wards, chronicle, aura);
    }
}
