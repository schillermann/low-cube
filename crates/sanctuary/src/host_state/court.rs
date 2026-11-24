use std::collections::VecDeque;
use std::net::{SocketAddr, UdpSocket};
use glam::Vec3;
use world::{
    CosmicFrame, DrivenCraft, GravityBubble, PilotCraft, PlanetaryWorld, ProximityUtterance,
    SolarProportions, WorldFact,
};
use crate::aura::HostAura;
use crate::chronicle_log::ChronicleStore;
use crate::cohort::GuestCohort;
use crate::envelope::SanctuaryEnvelope;

/// Focused custodian docking vehicles abandoned by disconnected companions on Earth or off-world bodies (`<= 4` fields).
pub(super) struct AbandonedDock<'a> {
    chronicle: &'a mut dyn ChronicleStore,
    aura: &'a mut HostAura,
}

impl<'a> AbandonedDock<'a> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(chronicle: &'a mut dyn ChronicleStore, aura: &'a mut HostAura) -> Self {
        Self { chronicle, aura }
    }

    pub fn dock_expired(
        &mut self,
        abandoned: Vec<(u32, CosmicFrame, PilotCraft)>,
        world: &mut dyn PlanetaryWorld,
    ) -> bool {
        let mut parked_any = false;
        for (cid, frame, pilot_craft) in abandoned {
            if let PilotCraft::Driving(dc) = pilot_craft {
                parked_any |= self.park_craft(frame, dc, world);
            }
            self.aura.fleet_mut().release_slot(cid);
        }
        parked_any
    }

    fn park_craft(&mut self, frame: CosmicFrame, dc: DrivenCraft, world: &mut dyn PlanetaryWorld) -> bool {
        match frame.anchored_body() {
            Some(b) if !frame.earth_centered() => {
                if let Some(assembly) = self.aura.fleet().reconstruct_assembly(dc.identity(), dc.pose()) {
                    let _ = world.enact_offworld(
                        b,
                        &WorldFact::CraftCommissioned { blueprint: assembly.blueprint().clone() },
                    );
                }
                self.aura.fleet_mut().force_release(dc.identity());
                let rel = WorldFact::HelmReleased { craft: dc.identity(), pose: dc.pose() };
                let ok = world.enact_offworld(b, &rel);
                if ok {
                    self.chronicle.record(WorldFact::OffWorld { body: b, fact: Box::new(rel) });
                }
                ok
            }
            _ => {
                let ok = self.aura.fleet_mut().park_by_id(dc.identity(), dc.pose(), world);
                if ok {
                    self.chronicle.record(WorldFact::HelmReleased { craft: dc.identity(), pose: dc.pose() });
                }
                ok
            }
        }
    }

    pub fn scope_standby_release(frame: CosmicFrame, dc: DrivenCraft) -> WorldFact {
        let release = WorldFact::HelmReleased { craft: dc.identity(), pose: dc.pose() };
        match frame.anchored_body() {
            Some(b) if !frame.earth_centered() => WorldFact::OffWorld { body: b, fact: Box::new(release) },
            _ => release,
        }
    }

    pub fn valid_boar_reach(frame: CosmicFrame, eye: Vec3, direction: Vec3, ward: &crate::ward::GuestWard) -> bool {
        frame.earth_centered() && ward.eye_within_reach(eye, direction)
    }
}

/// Focused standby UDP envelope dispatcher updating cohort pulses, inventory notices, and deferred petitions (`<= 4` fields).
pub(super) struct StandbyCourt<'a> {
    cohort: &'a mut GuestCohort,
    aura: &'a mut HostAura,
}

impl<'a> StandbyCourt<'a> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(cohort: &'a mut GuestCohort, aura: &'a mut HostAura) -> Self {
        Self { cohort, aura }
    }

    pub fn absorb(
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
            Some(SanctuaryEnvelope::GuestPulse { ack_seq, presence, craft, frame, berth }) => {
                let vetted = match craft {
                    PilotCraft::Driving(dc) if !self.aura.fleet().helmed_by(dc.identity(), cid) => PilotCraft::Pedestrian,
                    other => other,
                };
                if let Some(ward) = self.cohort.ward_mut(cid) {
                    ward.absorb_standby_pulse(cid, ack_seq, now, frame, berth, presence, vetted, bubble);
                }
                self.cohort.commit_survivor(cid, self.aura, false);
            }
            Some(SanctuaryEnvelope::GuestPetition { fact }) => {
                if deferred.len() < 64 {
                    deferred.push_back((cid, fact));
                }
            }
            Some(SanctuaryEnvelope::GuestUtterance { text }) => {
                if let Some((wf, pos)) = self.cohort.ward_world_locus(cid, self.aura) {
                    let speaker_name = self.cohort.ward_callsign(cid);
                    if let Some(utterance) = ProximityUtterance::named(cid, speaker_name, pos, &text) {
                        self.cohort.relay_utterance(socket, wf, utterance, self.aura);
                    }
                }
            }
            Some(SanctuaryEnvelope::GuestInventory { notice }) => {
                let _ = self.cohort.apply_inventory_notice(cid, notice, self.aura);
            }
            _ => {}
        }
    }
}
