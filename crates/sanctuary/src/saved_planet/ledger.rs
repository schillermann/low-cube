use glam::Vec3;
use world::{
    Callsign, CosmicFrame, CraftBlueprint, CraftIdentity, CraftPose, ExposedStratum, PlanetaryWorld,
    SatchelManifest, SolarProportions, SurvivorRecord, SurvivorWelcome, WorldEarthworks, WorldFact,
};
use crate::chronicle_log::ChronicleStore;
use crate::chronicled_planet::FactJournal;
use crate::fleet_ledger::FleetLedger;
use crate::survivor_ledger::SurvivorStore;

/// Polymorphic local checkpoint buffer avoiding nullable `Option<SurvivorRecord>` fields.
#[derive(Clone, Copy, Debug)]
pub(super) enum CheckpointDraft {
    Settled,
    Pending(SurvivorRecord),
}

/// Strictly encapsulated local world-owner identity and survivor checkpoint state (`4` fields).
pub(super) struct OwnerCheckpoint {
    pubkey: [u8; 32],
    callsign: Callsign,
    welcome: SurvivorWelcome,
    draft: CheckpointDraft,
}

impl OwnerCheckpoint {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub(super) const fn new(
        pubkey: [u8; 32],
        callsign: Callsign,
        welcome: SurvivorWelcome,
        draft: CheckpointDraft,
    ) -> Self {
        Self { pubkey, callsign, welcome, draft }
    }

    pub(super) const fn welcome(&self) -> SurvivorWelcome {
        self.welcome
    }

    pub(super) fn note_state(
        &mut self,
        frame: CosmicFrame,
        position: Vec3,
        manifest: SatchelManifest,
        survivors: &mut dyn SurvivorStore,
    ) {
        let record = SurvivorRecord::with_callsign(frame, position, manifest, self.callsign);
        self.welcome = SurvivorWelcome::Restored(record);
        self.draft = CheckpointDraft::Pending(record);
        survivors.commit(self.pubkey, record, false);
    }

    pub(super) fn flush_final(&mut self, survivors: &mut dyn SurvivorStore) {
        if let CheckpointDraft::Pending(record) = self.draft {
            survivors.commit(self.pubkey, record, true);
            self.draft = CheckpointDraft::Settled;
        }
        survivors.flush_tick();
    }
}

/// Durable local chronicle, survivor store, and vehicle fleet ledger for a player-owned world (`4` fields).
pub(super) struct LocalRealmLedger {
    proportions: SolarProportions,
    chronicle: Box<dyn ChronicleStore>,
    survivors: Box<dyn SurvivorStore>,
    fleet: FleetLedger,
}

impl LocalRealmLedger {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub(super) fn new(
        proportions: SolarProportions,
        chronicle: Box<dyn ChronicleStore>,
        survivors: Box<dyn SurvivorStore>,
        fleet: FleetLedger,
    ) -> Self {
        Self { proportions, chronicle, survivors, fleet }
    }

    pub(super) fn survivors_mut(&mut self) -> &mut dyn SurvivorStore {
        self.survivors.as_mut()
    }

    pub(super) fn park_by_id(
        &mut self,
        craft: CraftIdentity,
        pose: CraftPose,
        world: &mut dyn WorldEarthworks,
    ) -> bool {
        self.fleet.park_by_id(craft, pose, world)
    }

    pub(super) fn flush_disclosures(&mut self, world: &mut dyn PlanetaryWorld) {
        let mut revealed: Vec<(u8, ExposedStratum)> = Vec::new();
        world.drain_disclosures(&mut revealed);
        for (face, stratum) in revealed {
            self.chronicle.record(WorldFact::StratumDisclosed { face, stratum });
        }
        let _ = self.chronicle.compact_sectors(self.proportions.earth_half(), world);
    }

    pub(super) fn flush_tick(&mut self) {
        self.chronicle.flush_tick();
        self.survivors.flush_tick();
    }
}

impl FactJournal for LocalRealmLedger {
    fn inscribe_fact(&mut self, fact: WorldFact) {
        self.chronicle.record(fact);
    }

    fn local_slot(&self) -> u32 {
        1
    }

    fn register_local_blueprint(&mut self, blueprint: CraftBlueprint) -> bool {
        self.fleet.register_blueprint(blueprint)
    }

    fn claim_local_helm(&mut self, craft: CraftIdentity, pilot_slot: u32) -> bool {
        self.fleet.claim_helm(craft, pilot_slot)
    }

    fn release_local_helm(&mut self, craft: CraftIdentity) {
        let _ = self.fleet.release_helm(craft, 1);
    }
}
