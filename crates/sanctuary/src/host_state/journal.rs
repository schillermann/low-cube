use world::{CraftBlueprint, CraftIdentity, WorldFact};
use crate::chronicled_planet::FactJournal;
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
