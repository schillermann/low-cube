use std::collections::HashSet;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use world::{CraftIdentity, PlanetaryWorld, WorldFact};
use crate::chronicle_log::ChronicleStore;

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
}

/// Append-only SSD write-ahead journal decorator persisting durable `WorldFact`s across server restarts.
pub struct DiskVaultChronicle<S> {
    origin: S,
    journal_path: PathBuf,
}

impl<S> DiskVaultChronicle<S> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(origin: S, journal_path: PathBuf) -> Self {
        Self { origin, journal_path }
    }
}

impl<S: ChronicleStore> DiskVaultChronicle<S> {
    /// Replays existing WAL entries onto `world` + `origin` (folding superseded vehicle claims/parks) and opens `path`.
    pub fn replay_and_open(
        path: impl AsRef<Path>,
        mut origin: S,
        world: &mut dyn PlanetaryWorld,
    ) -> std::io::Result<Self> {
        let journal_path = path.as_ref().to_path_buf();
        if let Ok(mut file) = File::open(&journal_path) {
            let mut buf = Vec::new();
            file.read_to_end(&mut buf)?;
            let decoded = Self::decode_frames(&buf);
            let folded = Self::fold_fleet_history(decoded);
            for fact in folded {
                let _ = fact.enact_on(world);
                origin.record(fact);
            }
        }
        Ok(Self::new(origin, journal_path))
    }

    fn decode_frames(buf: &[u8]) -> Vec<WorldFact> {
        let mut cursor = 0usize;
        let mut out = Vec::new();
        while cursor + 4 <= buf.len() {
            let frame_len = u32::from_le_bytes([
                buf[cursor],
                buf[cursor + 1],
                buf[cursor + 2],
                buf[cursor + 3],
            ]) as usize;
            cursor += 4;
            if cursor + frame_len > buf.len() {
                break;
            }
            if let Some((fact, _)) = WorldFact::decode(&buf[cursor..cursor + frame_len]) {
                out.push(fact);
            }
            cursor += frame_len;
        }
        out
    }

    fn fold_fleet_history(facts: Vec<WorldFact>) -> Vec<WorldFact> {
        let mut last_park_idx = std::collections::HashMap::<CraftIdentity, usize>::new();
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

    fn append_frame(&self, fact: &WorldFact) {
        let mut payload = Vec::with_capacity(64);
        fact.inscribe(&mut payload);
        if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(&self.journal_path) {
            let len_bytes = (payload.len() as u32).to_le_bytes();
            let _ = file.write_all(&len_bytes);
            let _ = file.write_all(&payload);
        }
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
            self.append_frame(&fact);
        }
    }

    fn batch_from(&self, ack_seq: u32) -> Option<(u32, Vec<WorldFact>)> {
        self.origin.batch_from(ack_seq)
    }
}
