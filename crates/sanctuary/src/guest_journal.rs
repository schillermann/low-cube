use std::collections::VecDeque;
use world::{
    CompanionPresence, CraftBlueprint, CraftIdentity, DrivenCraft, PilotCraft, PlanetaryWorld,
    WorldFact,
};
use crate::chronicled_planet::FactJournal;
use crate::fleet_ledger::FleetLedger;
use crate::remote_link::RemoteLink;

/// Optimistic guest petition journal pairing `RemoteLink`, bounded prediction ring, and `FleetLedger`.
pub struct GuestJournal {
    link: RemoteLink,
    predicted: VecDeque<WorldFact>,
    fleet: FleetLedger,
}

impl GuestJournal {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn with_fleet(link: RemoteLink, predicted: VecDeque<WorldFact>, fleet: FleetLedger) -> Self {
        Self { link, predicted, fleet }
    }

    pub fn new(link: RemoteLink, predicted: VecDeque<WorldFact>) -> Self {
        Self::with_fleet(link, predicted, FleetLedger::empty())
    }

    pub fn connected(&self) -> bool { self.link.connected() }
    pub const fn fleet(&self) -> &FleetLedger { &self.fleet }
    pub fn fleet_mut(&mut self) -> &mut FleetLedger { &mut self.fleet }
    pub fn sync_local_presence(&mut self, presence: CompanionPresence) { self.link.sync_local_presence(presence); }
    pub fn sync_local_craft(&mut self, craft: PilotCraft) { self.link.sync_local_craft(craft); }
    pub fn companions(&self, out: &mut Vec<CompanionPresence>) { self.link.companions(out); }
    pub fn crafts(&self, out: &mut Vec<DrivenCraft>) { self.link.crafts(out); }
    pub fn blend_subtick(&mut self, subtick_secs: f32) { self.link.blend_subtick(subtick_secs); }
    pub fn ticket_string(&self) -> Option<String> { self.link.ticket_string() }
    pub fn phrase_string(&self) -> Option<String> { self.link.phrase_string() }

    pub fn pump(&mut self, dt: f32, world: &mut dyn PlanetaryWorld) -> bool {
        self.link.pump(dt, world, &mut self.fleet, &mut self.predicted)
    }
}

impl FactJournal for GuestJournal {
    fn inscribe_fact(&mut self, fact: WorldFact) {
        if fact.durable() {
            if self.predicted.len() >= 64 {
                let _ = self.predicted.pop_front();
            }
            self.predicted.push_back(fact.clone());
        }
        self.link.send_petition(fact);
    }

    fn local_slot(&self) -> u32 {
        self.link.companion_slot()
    }

    fn register_local_blueprint(&mut self, blueprint: CraftBlueprint) -> bool {
        self.fleet.register_blueprint(blueprint)
    }

    fn claim_local_helm(&mut self, craft: CraftIdentity, pilot_slot: u32) -> bool {
        self.fleet.claim_helm(craft, pilot_slot)
    }

    fn release_local_helm(&mut self, craft: CraftIdentity) {
        self.fleet.force_release(craft);
    }
}
