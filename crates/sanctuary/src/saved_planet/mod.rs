mod earthworks;
mod facets;
mod ledger;

use std::path::{Path, PathBuf};
use world::{
    Callsign, CelestialSystem, PilotCraft, PlanetaryWorld, SatchelManifest, SolarProportions,
    WorldEarthworks as _, WorldFact,
};
use crate::chronicle_decorators::{CompactedChronicle, DiskVaultChronicle};
use crate::chronicle_log::{ChronicleLog, ChronicleStore};
use crate::chronicled_planet::ChronicledPlanet;
use crate::fleet_ledger::FleetLedger;
use crate::seal::HandshakeIdentity;
use crate::snapshot_chronicle::SnapshottedChronicle;
use crate::survivor_ledger::{DiskVaultSurvivors, MemorySurvivors, SurvivorStore};
use self::ledger::{CheckpointDraft, LocalRealmLedger, OwnerCheckpoint};

/// Durable solitary `PlanetaryWorld` decorator persisting earthworks, vehicles, and player inventory to `worlds/` (`4` fields).
pub struct SavedPlanet {
    world: Box<dyn PlanetaryWorld>,
    ledger: LocalRealmLedger,
    owner: OwnerCheckpoint,
    piloted: PilotCraft,
}

impl SavedPlanet {
    /// Primary constructor doing nothing except assigning arguments to fields.
    fn from_parts(
        world: Box<dyn PlanetaryWorld>,
        ledger: LocalRealmLedger,
        owner: OwnerCheckpoint,
        piloted: PilotCraft,
    ) -> Self {
        Self { world, ledger, owner, piloted }
    }

    /// Secondary constructor assembling fleet and owner checkpoint from stores.
    pub fn new(
        world: Box<dyn PlanetaryWorld>,
        proportions: SolarProportions,
        chronicle: Box<dyn ChronicleStore>,
        survivors: Box<dyn SurvivorStore>,
        pubkey: [u8; 32],
        callsign: Callsign,
    ) -> Self {
        let mut fleet = FleetLedger::empty();
        for idx in 0..(chronicle.total() as usize) {
            if let Some(WorldFact::CraftCommissioned { blueprint }) = chronicle.fact_at(idx) {
                let _ = fleet.register_blueprint(blueprint.clone());
            }
        }
        let welcome = survivors.admit_or_restore(pubkey);
        Self::from_parts(
            world,
            LocalRealmLedger::new(proportions, chronicle, survivors, fleet),
            OwnerCheckpoint::new(pubkey, callsign, welcome, CheckpointDraft::Settled),
            PilotCraft::Pedestrian,
        )
    }

    /// Replays `<journal_path>` and `<journal_path>.survivors` onto a scaled & seeded `HeliosphereWorld`.
    pub fn open(
        journal_path: impl AsRef<Path>,
        callsign: Callsign,
        proportions: SolarProportions,
        starter_kit: SatchelManifest,
    ) -> std::io::Result<Self> {
        let journal = journal_path.as_ref().to_path_buf();
        if let Some(parent) = journal.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let host_key_path = PathBuf::from(format!("{}.host.key", journal.display()));
        let surv_path = PathBuf::from(format!("{}.survivors", journal.display()));
        let keys = HandshakeIdentity::load_or_create(&host_key_path);
        let bevel = (proportions.earth_half() * 0.125).clamp(0.5, 1.0);
        let mut world = CelestialSystem::heliosphere(proportions, bevel);
        let disk = DiskVaultChronicle::replay_and_open(
            &journal,
            CompactedChronicle::wrapping(ChronicleLog::empty()),
            world.as_mut(),
        )?;
        let chronicle = Box::new(SnapshottedChronicle::wrapping(disk));
        let survivors = Box::new(DiskVaultSurvivors::replay_and_open(
            surv_path,
            MemorySurvivors::with_starter(starter_kit),
        )?);
        let mut built = Self::new(world, proportions, chronicle, survivors, keys.public_bytes(), callsign);
        if let Some(rec) = built.owner.welcome().restored_record() {
            built.world.anchor_to_frame(rec.frame(), rec.position());
        }
        Ok(built)
    }

    pub(super) fn with_chronicled<R>(&mut self, op: impl FnOnce(&mut ChronicledPlanet<'_>) -> R) -> R {
        let out = {
            let mut decorated = ChronicledPlanet::new(self.world.as_mut(), &mut self.ledger);
            op(&mut decorated)
        };
        self.ledger.flush_disclosures(self.world.as_mut());
        self.ledger.flush_tick();
        out
    }
}

impl Drop for SavedPlanet {
    fn drop(&mut self) {
        if let PilotCraft::Driving(driven) = self.piloted {
            let id = driven.identity();
            let pose = driven.pose();
            if self.ledger.park_by_id(id, pose, self.world.as_mut()) {
                let mut decorated = ChronicledPlanet::new(self.world.as_mut(), &mut self.ledger);
                let _ = decorated.park_craft_by_id(id, pose);
            }
        }
        self.owner.flush_final(self.ledger.survivors_mut());
        self.ledger.flush_tick();
    }
}
