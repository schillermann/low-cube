use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use world::{CraftIdentity, PlanetaryWorld, WorldFact};
use crate::chronicle_log::ChronicleStore;
pub use crate::vault_stream::VaultStream;

/// `ChronicleStore` decorator deduplicating idempotent stratum disclosures and vehicle blueprint commissions.
pub struct CompactedChronicle<S> {
    origin: S,
    disclosed_strata: HashSet<(u16, u8, i32, i32, i32)>,
    commissioned_crafts: HashSet<CraftIdentity>,
}

impl<S> CompactedChronicle<S> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(
        origin: S,
        disclosed_strata: HashSet<(u16, u8, i32, i32, i32)>,
        commissioned_crafts: HashSet<CraftIdentity>,
    ) -> Self {
        Self { origin, disclosed_strata, commissioned_crafts }
    }

    pub fn wrapping(origin: S) -> Self {
        Self::new(origin, HashSet::new(), HashSet::new())
    }

    fn redundant(&mut self, body_code: u16, fact: &WorldFact) -> bool {
        match fact {
            WorldFact::StratumDisclosed { face, stratum } => {
                let key = stratum.transfer(|u, v, elev, _| (body_code, *face, u, v, elev));
                !self.disclosed_strata.insert(key)
            }
            WorldFact::CraftCommissioned { blueprint } if blueprint.identity().assigned() => {
                !self.commissioned_crafts.insert(blueprint.identity())
            }
            WorldFact::OffWorld { body, fact: inner } => self.redundant(body.raw_code(), inner),
            _ => false,
        }
    }
}

impl<S: ChronicleStore> ChronicleStore for CompactedChronicle<S> {
    fn total(&self) -> u32 {
        self.origin.total()
    }

    fn record(&mut self, fact: WorldFact) {
        if !fact.durable() || self.redundant(0, &fact) {
            return;
        }
        self.origin.record(fact);
    }

    fn batch_from(&self, ack_seq: u32) -> Option<(u32, Vec<WorldFact>)> {
        self.origin.batch_from(ack_seq)
    }

    fn fact_at(&self, index: usize) -> Option<&WorldFact> {
        self.origin.fact_at(index)
    }

    fn supersede(&mut self, index: u32) {
        self.origin.supersede(index);
    }

    fn compact_sectors(&mut self, earth_half: f32, world: &mut dyn PlanetaryWorld) -> Vec<u32> {
        self.origin.compact_sectors(earth_half, world)
    }

    fn flush_tick(&mut self) {
        self.origin.flush_tick();
    }
}

/// Append-only buffered SSD write-ahead journal decorator persisting durable `WorldFact`s across server restarts.
pub struct DiskVaultChronicle<S> {
    origin: S,
    journal_path: PathBuf,
    stream: VaultStream,
    dirty: bool,
}

impl<S> DiskVaultChronicle<S> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn with_stream(origin: S, journal_path: PathBuf, stream: VaultStream, dirty: bool) -> Self {
        Self { origin, journal_path, stream, dirty }
    }

    pub const fn new(origin: S, journal_path: PathBuf) -> Self {
        Self::with_stream(origin, journal_path, VaultStream::Dormant, false)
    }
}

impl<S> Drop for DiskVaultChronicle<S> {
    fn drop(&mut self) {
        self.stream.flush_pending();
    }
}

impl<S: ChronicleStore> DiskVaultChronicle<S> {
    /// Replays existing WAL entries onto `world` + `origin`, compacting historical raycast logs into `SectorSnapshot`s.
    pub fn replay_and_open(
        path: impl AsRef<Path>,
        mut origin: S,
        world: &mut dyn PlanetaryWorld,
    ) -> std::io::Result<Self> {
        let journal_path = path.as_ref().to_path_buf();
        if let Some(decoded) = VaultStream::read_decoded_frames(&journal_path)? {
            let folded = Self::fold_fleet_history(decoded);
            let foldable_count = folded.iter().filter(|f| f.snapshot_foldable()).count();
            for fact in &folded {
                let _ = fact.enact_on(world);
            }
            let mut stale_disc = Vec::new();
            world.drain_disclosures(&mut stale_disc);
            if foldable_count >= 8 {
                let compacted = Self::build_snapshot_log(folded, world);
                VaultStream::rewrite_atomic(&journal_path, &compacted)?;
                for fact in compacted {
                    origin.record(fact);
                }
            } else {
                for fact in folded {
                    origin.record(fact);
                }
            }
        }
        let stream = VaultStream::open_append(&journal_path)?;
        Ok(Self::with_stream(origin, journal_path, stream, false))
    }

    fn build_snapshot_log(folded: Vec<WorldFact>, world: &mut dyn PlanetaryWorld) -> Vec<WorldFact> {
        let mut out = Vec::new();
        for fact in folded {
            if let WorldFact::RigBlueprintBuilt { ref blueprint, .. } = fact {
                if blueprint.identity().assigned() {
                    out.push(WorldFact::CraftCommissioned { blueprint: blueprint.clone() });
                }
            } else if let WorldFact::OffWorld { ref fact, .. } = fact {
                if let WorldFact::RigBlueprintBuilt { ref blueprint, .. } = **fact {
                    if blueprint.identity().assigned() {
                        out.push(WorldFact::CraftCommissioned { blueprint: blueprint.clone() });
                    }
                }
            }
            if !fact.snapshot_foldable() {
                out.push(fact);
            }
        }
        let mut earth_snaps = Vec::new();
        world.collect_sector_snapshots(&mut earth_snaps);
        for snapshot in earth_snaps {
            out.push(WorldFact::SectorSnapshot { snapshot });
        }
        let mut off_snaps = Vec::new();
        world.collect_offworld_snapshots(&mut off_snaps);
        for (body, snapshot) in off_snaps {
            out.push(WorldFact::OffWorld { body, fact: Box::new(WorldFact::SectorSnapshot { snapshot }) });
        }
        out
    }

    fn fold_fleet_history(facts: Vec<WorldFact>) -> Vec<WorldFact> {
        let mut last_park_idx = HashMap::<CraftIdentity, usize>::new();
        for (idx, fact) in facts.iter().enumerate() {
            match fact {
                WorldFact::HelmReleased { craft, .. } if craft.assigned() => {
                    last_park_idx.insert(*craft, idx);
                }
                WorldFact::OffWorld { fact: inner, .. } => {
                    if let WorldFact::HelmReleased { craft, .. } = **inner {
                        if craft.assigned() {
                            last_park_idx.insert(craft, idx);
                        }
                    }
                }
                _ => {}
            }
        }
        facts
            .into_iter()
            .enumerate()
            .filter(|(idx, fact)| match fact {
                WorldFact::HelmClaimed { craft, .. } if last_park_idx.get(craft).is_some_and(|&p| p > *idx) => false,
                WorldFact::HelmReleased { craft, .. } if last_park_idx.get(craft).is_some_and(|&p| p != *idx) => false,
                WorldFact::OffWorld { fact: inner, .. } => match **inner {
                    WorldFact::HelmClaimed { craft, .. } if last_park_idx.get(&craft).is_some_and(|&p| p > *idx) => false,
                    WorldFact::HelmReleased { craft, .. } if last_park_idx.get(&craft).is_some_and(|&p| p != *idx) => false,
                    _ => true,
                },
                _ => true,
            })
            .map(|(_, f)| f)
            .collect()
    }
}

impl<S: ChronicleStore> ChronicleStore for DiskVaultChronicle<S> {
    fn total(&self) -> u32 {
        self.origin.total()
    }

    fn record(&mut self, fact: WorldFact) {
        let before = self.origin.total();
        self.origin.record(fact.clone());
        if self.origin.total() > before {
            let mut payload = Vec::with_capacity(64);
            fact.inscribe(&mut payload);
            self.stream.write_frame(&self.journal_path, &payload);
            self.dirty = true;
        }
    }

    fn batch_from(&self, ack_seq: u32) -> Option<(u32, Vec<WorldFact>)> {
        self.origin.batch_from(ack_seq)
    }

    fn fact_at(&self, index: usize) -> Option<&WorldFact> {
        self.origin.fact_at(index)
    }

    fn supersede(&mut self, index: u32) {
        self.origin.supersede(index);
    }

    fn compact_sectors(&mut self, earth_half: f32, world: &mut dyn PlanetaryWorld) -> Vec<u32> {
        self.origin.compact_sectors(earth_half, world)
    }

    fn flush_tick(&mut self) {
        if self.dirty {
            self.stream.flush_pending();
            self.dirty = false;
        }
        self.origin.flush_tick();
    }
}
