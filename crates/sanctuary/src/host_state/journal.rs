use world::{CosmicFrame, CraftBlueprint, CraftIdentity, ExposedStratum, PlanetaryWorld, WorldFact};
use crate::admin_ward::{AdminBlockWard, GuestFactStamp};
use crate::chronicled_planet::FactJournal;
use crate::envelope::CrackNotice;
use crate::warded_planet::WardedPlanet;
use super::HostState;

impl FactJournal for HostState {
    fn inscribe_fact(&mut self, fact: WorldFact) {
        self.record_authored_fact(1, fact);
    }

    fn local_slot(&self) -> u32 {
        1
    }

    fn register_local_blueprint(&mut self, blueprint: CraftBlueprint) -> bool {
        self.fleet_mut().register_blueprint(blueprint)
    }

    fn claim_local_helm(&mut self, craft: CraftIdentity, pilot_slot: u32) -> bool {
        self.fleet_mut().claim_helm(craft, pilot_slot)
    }

    fn release_local_helm(&mut self, craft: CraftIdentity) {
        let _ = self.fleet_mut().release_helm(craft, 1);
    }
}

/// Focused chronicle mutation and guest petition enactment contract for `HostState`.
pub trait AuthoredChronicle {
    fn record_authored_fact(&mut self, author_slot: u32, fact: WorldFact);
    fn record_disclosure(&mut self, face: u8, stratum: ExposedStratum);
    fn enact_guest_petition(&mut self, cid: u32, fact: WorldFact, world: &mut dyn PlanetaryWorld) -> bool;
}

impl AuthoredChronicle for HostState {
    fn record_authored_fact(&mut self, author_slot: u32, fact: WorldFact) {
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

    fn record_disclosure(&mut self, face: u8, stratum: ExposedStratum) {
        self.chronicle.record(WorldFact::StratumDisclosed { face, stratum });
    }

    fn enact_guest_petition(&mut self, cid: u32, fact: WorldFact, world: &mut dyn PlanetaryWorld) -> bool {
        match fact {
            WorldFact::StratumDisclosed { .. } | WorldFact::SectorSnapshot { .. } => false,
            WorldFact::OffWorld { ref fact, .. }
                if matches!(**fact, WorldFact::StratumDisclosed { .. } | WorldFact::SectorSnapshot { .. }) =>
            {
                false
            }
            other => {
                let stamped = GuestFactStamp::new(cid, other).stamped();
                let fleet = self.aura.fleet_mut();
                let enacted = self.cohort.ward_mut(cid).is_some_and(|ward| {
                    let mut guarded = AdminBlockWard::new(WardedPlanet::new(world, ward, fleet, cid));
                    stamped.enact_on(&mut guarded)
                });
                if enacted {
                    self.cohort.commit_survivor(cid, &self.aura, true);
                    self.record_authored_fact(cid, stamped);
                }
                enacted
            }
        }
    }
}
