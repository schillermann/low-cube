use std::collections::HashMap;
use std::net::UdpSocket;
use glam::Vec3;
use world::{
    CosmicFrame, GravityBubble, SolarProportions, SounderPresence, WildlifePulse,
};
use crate::audience::{
    MtuBudgetAudience, PulseAudience, INTIMATE_NEIGHBOR_CAP, NEARBY_SIGHT_METERS,
    PACKET_COMPANION_CAP,
};
use crate::aura::{HostAura, HostPresence};
use crate::bubble_audience::{CosmicBubbleAudience, CosmicCraftSight};
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

/// Focused tick-pulse assembler delivering Hill-bubble-culled `HostPulse` and `HostChronicle` frames (`<= 4` fields).
pub(super) struct CohortPulse {
    proportions: SolarProportions,
    epoch_secs: f64,
}

impl CohortPulse {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(proportions: SolarProportions, epoch_secs: f64) -> Self {
        Self { proportions, epoch_secs }
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
        let mut roster = Vec::with_capacity(wards.len() + 1);
        let mut crafts = Vec::new();
        if let HostPresence::Occupied(p) = aura.presence() {
            roster.push((aura.host_frame(), p));
            aura.push_host_craft_with_speed(&mut crafts);
        }
        for (ward, _) in wards.values() {
            roster.push((ward.cosmic_frame(), ward.presence()));
            ward.push_authoritative_craft_with_speed(&mut crafts);
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
            let wf = ward.cosmic_frame();
            let anchor = ward.anchor_position();
            let (mut others, mut other_crafts) = (Vec::new(), Vec::new());
            audience.select_in_frame(cid, wf, anchor, ward.sight_cursor_mut(), &mut others, &mut other_crafts);
            stream.gather_in_frame(wf, anchor, &other_crafts, fact_index, self.proportions, self.epoch_secs);
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
