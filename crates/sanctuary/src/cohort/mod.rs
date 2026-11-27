mod deck_view;
mod directory;
mod pulse;

use std::collections::HashMap;
use std::net::{SocketAddr, UdpSocket};
use glam::Vec3;
use world::{
    Callsign, CompanionPresence, CosmicFrame, DeckFrame, DrivenCraft, GravityBubble, PilotCraft,
    ProximityUtterance, SolarProportions,
};
use crate::aura::HostAura;
use crate::chronicle_log::ChronicleStore;
use crate::envelope::{InventoryNotice, SanctuaryEnvelope};
use crate::fleet_ledger::FleetLedger;
use crate::glide::TimelineClock;
use crate::inbox_batch::{InboundBatch, PulseInbox};
use crate::spatial_chronicle::HorizonStream;
use crate::survivor_ledger::{MemorySurvivors, SurvivorStore};
use crate::ward::GuestWard;
use self::deck_view::HostDeckView;
pub use self::directory::{CohortAdmission, CohortGate, SlotLedger};
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

    pub fn commit_owner_record(&mut self, pubkey: [u8; 32], record: world::SurvivorRecord, final_checkpoint: bool) {
        self.survivors.commit(pubkey, record, final_checkpoint);
    }

    pub fn lookup_owner(&self, pubkey: [u8; 32]) -> Option<world::SurvivorWelcome> {
        self.survivors.lookup(pubkey).map(world::SurvivorWelcome::Restored)
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

    pub fn apply_inventory_notice(
        &mut self,
        cid: u32,
        notice: InventoryNotice,
        proportions: world::SolarProportions,
        aura: &HostAura,
    ) -> bool {
        let in_vacuum = self
            .ward_world_locus(cid, aura)
            .is_some_and(|(wf, wpos)| proportions.vacuum_locus(wf, wpos));
        let applied = self.wards.get_mut(&cid).is_some_and(|(ward, _)| match notice {
            InventoryNotice::Crafted { recipe } => {
                if world::RelicCatalog::vacuum_smelt_recipe(recipe) && !in_vacuum {
                    return false;
                }
                ward.manifest_mut().apply_recipe(recipe)
            }
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

    pub fn relay_utterance(
        &mut self,
        socket: &UdpSocket,
        speaker_frame: CosmicFrame,
        utterance: ProximityUtterance,
        aura: &mut HostAura,
    ) {
        CohortPulse::relay_utterance(socket, &mut self.wards, speaker_frame, utterance, aura, self.timeline.0.render_secs());
    }

    pub fn open_datagram_batches<I: PulseInbox>(
        &mut self,
        inbox: &I,
        mut grouped: HashMap<u32, Vec<(u64, SocketAddr, Vec<u8>)>>,
    ) -> Vec<(u32, SanctuaryEnvelope)> {
        let mut batches = Vec::with_capacity(grouped.len());
        for (&cid, (ward, _)) in &mut self.wards {
            if let Some(raw) = grouped.remove(&cid) {
                batches.push(InboundBatch::prepare(cid, ward, raw));
            }
        }
        inbox.open_batches(&mut batches);
        InboundBatch::collect_ordered(batches)
    }

    pub fn collect_active_zones(&self, aura: &HostAura, zone_mask: &mut u8, bodies: &mut Vec<world::BodyId>) {
        for &cid in self.wards.keys() {
            let Some((wf, pos)) = self.ward_world_locus(cid, aura) else { continue };
            if wf.earth_centered() {
                *zone_mask |= 1u8 << (world::ClimateZone::dominant(pos).index() as u8);
            } else if let Some(b) = wf.anchored_body() {
                if !bodies.contains(&b) {
                    bodies.push(b);
                }
            }
        }
    }

    pub fn broadcast(&mut self, socket: &UdpSocket, chronicle: &mut dyn ChronicleStore, aura: &mut HostAura, proportions: SolarProportions, epoch_secs: f64) {
        self.survivors.flush_tick();
        CohortPulse::new(proportions, epoch_secs).deliver(socket, &mut self.wards, chronicle, aura);
    }
}
