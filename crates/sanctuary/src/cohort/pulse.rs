use std::collections::HashMap;
use std::net::UdpSocket;
use glam::Vec3;
use world::{
    CompanionPresence, CosmicFrame, CraftIdentity, DrivenCraft, GravityBubble, SolarProportions,
    SounderPresence, VesselBerth, WildlifePulse,
};
use crate::audience::{
    MtuBudgetAudience, PulseAudience, INTIMATE_NEIGHBOR_CAP, NEARBY_SIGHT_METERS,
    PACKET_COMPANION_CAP,
};
use crate::aura::{HostAura, HostPresence};
use crate::bubble_audience::{CosmicBubbleAudience, CosmicCraftSight};
use crate::chat_medium::{ChatLocus, ChatMedium, SuitTransceiverMedium};
use crate::chronicle_log::ChronicleStore;
use crate::envelope::SanctuaryEnvelope;
use crate::seal_batch::{OutboundFrame, ParallelSeals, PulseDelivery, SequentialDelivery};
use crate::spatial_chronicle::HorizonStream;
use crate::ward::GuestWard;

fn sounder_near_anchor(frame: CosmicFrame, sounder: SounderPresence, anchor: Vec3) -> SounderPresence {
    if !frame.earth_centered() {
        return SounderPresence::Unseen;
    }
    match sounder {
        SounderPresence::Sighted(boars)
            if boars.iter().any(|b| (b.position() - anchor).length_squared() <= NEARBY_SIGHT_METERS * NEARBY_SIGHT_METERS) =>
        {
            SounderPresence::Sighted(boars)
        }
        _ => SounderPresence::Unseen,
    }
}

/// Focused tick-pulse assembler delivering Hill-bubble-culled and hull-anchored `HostPulse` frames (`<= 4` fields).
pub(super) struct CohortPulse {
    proportions: SolarProportions,
    epoch_secs: f64,
}

impl CohortPulse {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(proportions: SolarProportions, epoch_secs: f64) -> Self {
        Self { proportions, epoch_secs }
    }

    fn resolve_ship(
        craft_id: CraftIdentity,
        crafts: &[(CosmicFrame, DrivenCraft, f32)],
    ) -> Option<(CosmicFrame, DrivenCraft)> {
        crafts.iter().find(|(_, dc, _)| dc.identity() == craft_id).map(|&(f, dc, _)| (f, dc))
    }

    pub fn deliver(
        self,
        socket: &UdpSocket,
        wards: &mut HashMap<u32, (GuestWard, HorizonStream)>,
        chronicle: &mut dyn ChronicleStore,
        aura: &mut HostAura,
    ) {
        chronicle.flush_tick();
        aura.fact_index_mut().sync_with(chronicle, self.proportions.earth_half());
        let bubble = GravityBubble::new(self.proportions, self.epoch_secs);

        let mut crafts = Vec::new();
        if matches!(aura.presence(), HostPresence::Occupied(_)) {
            aura.push_host_craft_with_speed(&mut crafts);
        }
        for (ward, _) in wards.values() {
            ward.push_authoritative_craft_with_speed(&mut crafts);
        }

        let mut raw_berths: HashMap<u32, (CraftIdentity, CompanionPresence)> = HashMap::new();
        let mut roster = Vec::with_capacity(wards.len() + 1);
        if let HostPresence::Occupied(p) = aura.presence() {
            match aura.host_berth() {
                VesselBerth::Aboard(id) => {
                    if let Some((sf, dc)) = Self::resolve_ship(id, &crafts) {
                        raw_berths.insert(1, (id, p));
                        roster.push((sf, p.folded_onto_hull(dc.pose())));
                    }
                }
                VesselBerth::Planetary => roster.push((aura.host_frame(), p)),
            }
        }
        for (&cid, (ward, _)) in wards.iter() {
            let p = ward.presence();
            match ward.berth() {
                VesselBerth::Aboard(id) => {
                    if let Some((sf, dc)) = Self::resolve_ship(id, &crafts) {
                        raw_berths.insert(cid, (id, p));
                        roster.push((sf, p.folded_onto_hull(dc.pose())));
                    }
                }
                VesselBerth::Planetary => roster.push((ward.cosmic_frame(), p)),
            }
        }

        let audience = MtuBudgetAudience::new(
            CosmicCraftSight::indexed(
                CosmicBubbleAudience::indexed(&roster, bubble),
                &crafts,
                bubble,
            ),
            INTIMATE_NEIGHBOR_CAP,
            PACKET_COMPANION_CAP,
        );
        let sounder = aura.wildlife().presence();
        let sector_cracks = aura.drain_cracks();
        let mut frames = Vec::with_capacity(wards.len());
        let fact_index = aura.fact_index_mut() as &_;
        for (&cid, (ward, stream)) in wards {
            let (wf, anchor, own_ship) = match ward.berth().craft_id().and_then(|id| Self::resolve_ship(id, &crafts)) {
                Some((sf, dc)) => (sf, dc.pose().world_point_at(ward.anchor_position()), Some(dc.identity())),
                None => (ward.cosmic_frame(), ward.anchor_position(), None),
            };
            let (mut others, mut other_crafts) = (Vec::new(), Vec::new());
            audience.select_in_frame(cid, wf, anchor, ward.sight_cursor_mut(), &mut others, &mut other_crafts);
            if let Some(ship_id) = own_ship {
                if !other_crafts.iter().any(|dc| dc.identity() == ship_id) {
                    if let Some((_, dc)) = Self::resolve_ship(ship_id, &crafts) {
                        if !dc.same_slot(cid) {
                            other_crafts.push(dc);
                        }
                    }
                }
            }
            let mut berths = Vec::new();
            for comp in &mut others {
                if let Some(&(ship_id, local_pres)) = raw_berths.get(&comp.slot()) {
                    let has_hull = own_ship == Some(ship_id)
                        || other_crafts.iter().any(|dc| dc.identity() == ship_id);
                    if has_hull {
                        *comp = local_pres;
                        berths.push((comp.slot(), ship_id));
                    }
                }
            }
            if ward.live_presence() {
                stream.gather_in_frame(wf, anchor, &other_crafts, fact_index, self.proportions, self.epoch_secs);
            }
            let mut payloads = Vec::with_capacity(2);
            if let Some((base_seq, facts)) = stream.batch_from(ward.ack_seq(), chronicle) {
                let mut out = Vec::new();
                SanctuaryEnvelope::HostChronicle { base_seq, facts }.inscribe(&mut out);
                payloads.push(out);
            }
            let cracks = sector_cracks.collect_in_frame(wf, anchor, cid, NEARBY_SIGHT_METERS);
            let wildlife = WildlifePulse::new(sounder_near_anchor(wf, sounder, anchor), ward.gore_stamp());
            let mut pulse_bytes = Vec::new();
            SanctuaryEnvelope::HostPulse {
                total_facts: stream.total(),
                cracks,
                companions: others,
                crafts: other_crafts,
                berths,
                wildlife,
            }
            .inscribe(&mut pulse_bytes);
            payloads.push(pulse_bytes);
            frames.push(OutboundFrame::prepare(cid, ward, payloads));
        }
        let delivery = ParallelSeals::hardware(SequentialDelivery::new());
        delivery.deliver(socket, &mut frames);
    }

    pub fn relay_utterance(
        socket: &UdpSocket,
        wards: &mut HashMap<u32, (GuestWard, HorizonStream)>,
        speaker_frame: CosmicFrame,
        utterance: world::ProximityUtterance,
        aura: &mut HostAura,
        render_secs: f64,
    ) {
        let (speaker, origin) = utterance.transfer(|slot, pos, _| (slot, pos));
        let speaker_berth = if speaker == 1 {
            aura.effective_berth()
        } else {
            wards.get(&speaker).map_or(VesselBerth::planetary(), |(w, _)| w.effective_berth())
        };
        let speaker_locus = ChatLocus::new(speaker_frame, speaker_berth, origin, !utterance.radio_silent());
        let medium = SuitTransceiverMedium::cosmos();
        if speaker != 1 {
            aura.receive_utterance(speaker_locus, utterance.clone(), &medium);
        }
        let mut payload = Vec::with_capacity(80);
        SanctuaryEnvelope::HostUtterance { utterance: utterance.clone() }.inscribe(&mut payload);
        let mut scratch = Vec::with_capacity(112);
        let view = super::deck_view::HostDeckView::new(wards, render_secs);
        let recipient_ids: Vec<u32> = wards
            .iter()
            .filter_map(|(&cid, (ward, _))| {
                if cid == speaker {
                    return None;
                }
                let raw = ward.anchor_position();
                let (wf, pos) = match ward.berth().craft_id().and_then(|id| view.active_ship(id, aura)) {
                    Some((sf, pose)) => (sf, pose.world_point_at(raw)),
                    None => (ward.cosmic_frame(), raw),
                };
                let listener = ChatLocus::new(wf, ward.effective_berth(), pos, true);
                medium.carries(speaker_locus, listener).then_some(cid)
            })
            .collect();
        for cid in recipient_ids {
            if let Some((ward, _)) = wards.get_mut(&cid) {
                ward.send_sealed(socket, cid, &payload, &mut scratch);
            }
        }
    }
}
