mod roster;

use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use world::{Callsign, SatchelManifest, SurvivorRecord, SurvivorWelcome};
use crate::chronicle_decorators::VaultStream;
pub use self::roster::RosterSheet;

/// Authoritative per-identity survivor inventory, position, and disambiguated callsign store.
pub trait SurvivorStore: Send {
    fn admit_or_restore(&self, pubkey: [u8; 32]) -> SurvivorWelcome;
    fn lookup(&self, pubkey: [u8; 32]) -> Option<SurvivorRecord>;
    fn resolve_callsign(
        &self,
        pubkey: [u8; 32],
        desired: Callsign,
        active: &[([u8; 32], Callsign)],
    ) -> Callsign;
    fn commit(&mut self, pubkey: [u8; 32], record: SurvivorRecord, final_checkpoint: bool);
    fn export_roster(&self, _out: &mut Vec<([u8; 32], SurvivorRecord)>) {}
    fn flush_tick(&mut self) {}
}

/// In-memory `SurvivorStore` mapping `[u8; 32]` public keys to `SurvivorRecord`s (`<= 4` fields).
pub struct MemorySurvivors {
    records: HashMap<[u8; 32], SurvivorRecord>,
    starter_kit: SatchelManifest,
}

impl MemorySurvivors {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(records: HashMap<[u8; 32], SurvivorRecord>, starter_kit: SatchelManifest) -> Self {
        Self { records, starter_kit }
    }

    pub fn with_starter(starter_kit: SatchelManifest) -> Self {
        Self::new(HashMap::new(), starter_kit.sanitized())
    }

    pub fn empty() -> Self {
        Self::with_starter(SatchelManifest::starter_kit())
    }

    fn occupied_by_other(
        &self,
        pubkey: [u8; 32],
        candidate: Callsign,
        active: &[([u8; 32], Callsign)],
    ) -> bool {
        let in_store = self
            .records
            .iter()
            .any(|(&pk, rec)| pk != pubkey && rec.callsign().matches_case_insensitive(candidate));
        let in_active = active
            .iter()
            .any(|&(pk, cs)| pk != pubkey && cs.matches_case_insensitive(candidate));
        in_store || in_active
    }
}

impl SurvivorStore for MemorySurvivors {
    fn admit_or_restore(&self, pubkey: [u8; 32]) -> SurvivorWelcome {
        if let Some(&existing) = self.records.get(&pubkey) {
            SurvivorWelcome::Restored(existing)
        } else {
            SurvivorWelcome::FreshKit(self.starter_kit)
        }
    }

    fn lookup(&self, pubkey: [u8; 32]) -> Option<SurvivorRecord> {
        self.records.get(&pubkey).copied()
    }

    fn resolve_callsign(
        &self,
        pubkey: [u8; 32],
        desired: Callsign,
        active: &[([u8; 32], Callsign)],
    ) -> Callsign {
        let existing = self.records.get(&pubkey).map_or(Callsign::unnamed(), SurvivorRecord::callsign);
        if !desired.present() && existing.present() {
            return existing;
        }
        if !desired.present() {
            for ordinal in 1..100_000u32 {
                let candidate = Callsign::numbered_survivor(ordinal);
                if !self.occupied_by_other(pubkey, candidate, active) {
                    return candidate;
                }
            }
            return Callsign::numbered_survivor(1);
        }
        for ordinal in 1..100_000u32 {
            let candidate = desired.disambiguated(ordinal);
            if !self.occupied_by_other(pubkey, candidate, active) {
                return candidate;
            }
        }
        desired
    }

    fn commit(&mut self, pubkey: [u8; 32], record: SurvivorRecord, _final_checkpoint: bool) {
        if pubkey != [0u8; 32] {
            self.records.insert(pubkey, record);
        }
    }

    fn export_roster(&self, out: &mut Vec<([u8; 32], SurvivorRecord)>) {
        out.clear();
        out.extend(self.records.iter().map(|(&k, &v)| (k, v)));
        out.sort_by(|a, b| a.1.callsign().as_str().cmp(b.1.callsign().as_str()).then_with(|| a.0.cmp(&b.0)));
    }
}

/// Buffered SSD write-ahead `SurvivorStore` decorator persisting survivor checkpoints and TOML roster (`<= 4` fields).
pub struct DiskVaultSurvivors<S> {
    origin: S,
    vault_path: PathBuf,
    stream: VaultStream,
    dirty: bool,
}

impl<S> DiskVaultSurvivors<S> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn with_stream(origin: S, vault_path: PathBuf, stream: VaultStream, dirty: bool) -> Self {
        Self { origin, vault_path, stream, dirty }
    }

    pub const fn new(origin: S, vault_path: PathBuf) -> Self {
        Self::with_stream(origin, vault_path, VaultStream::Dormant, false)
    }
}

impl<S: SurvivorStore> DiskVaultSurvivors<S> {
    fn sync_roster_sheet(&self) {
        let mut entries = Vec::new();
        self.origin.export_roster(&mut entries);
        RosterSheet::for_vault(&self.vault_path).write(&entries);
    }

    /// Replays existing `[len (4B), pubkey (32B), SurvivorRecord]` frames into `origin` and opens `path` for append.
    pub fn replay_and_open(path: impl AsRef<Path>, mut origin: S) -> std::io::Result<Self> {
        let vault_path = path.as_ref().to_path_buf();
        if let Ok(mut file) = File::open(&vault_path) {
            let mut buf = Vec::new();
            file.read_to_end(&mut buf)?;
            let mut cursor = 0usize;
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
                if frame_len > 32 {
                    let slice = &buf[cursor..cursor + frame_len];
                    let mut pubkey = [0u8; 32];
                    pubkey.copy_from_slice(&slice[..32]);
                    if let Some((record, _)) = SurvivorRecord::decode(&slice[32..]) {
                        origin.commit(pubkey, record, false);
                    }
                }
                cursor += frame_len;
            }
        }
        let stream = VaultStream::open_append(&vault_path)?;
        let built = Self::with_stream(origin, vault_path, stream, false);
        built.sync_roster_sheet();
        Ok(built)
    }
}

impl<S> Drop for DiskVaultSurvivors<S> {
    fn drop(&mut self) {
        self.stream.flush_pending();
    }
}

impl<S: SurvivorStore> SurvivorStore for DiskVaultSurvivors<S> {
    fn admit_or_restore(&self, pubkey: [u8; 32]) -> SurvivorWelcome {
        self.origin.admit_or_restore(pubkey)
    }

    fn lookup(&self, pubkey: [u8; 32]) -> Option<SurvivorRecord> {
        self.origin.lookup(pubkey)
    }

    fn resolve_callsign(
        &self,
        pubkey: [u8; 32],
        desired: Callsign,
        active: &[([u8; 32], Callsign)],
    ) -> Callsign {
        self.origin.resolve_callsign(pubkey, desired, active)
    }

    fn commit(&mut self, pubkey: [u8; 32], record: SurvivorRecord, final_checkpoint: bool) {
        if pubkey == [0u8; 32] {
            return;
        }
        let significant = match self.origin.lookup(pubkey) {
            None => true,
            Some(prev) => {
                final_checkpoint
                    || prev.manifest() != record.manifest()
                    || prev.frame() != record.frame()
                    || prev.callsign() != record.callsign()
                    || (prev.position() - record.position()).length_squared() >= 64.0
            }
        };
        self.origin.commit(pubkey, record, final_checkpoint);
        if significant {
            let mut payload = Vec::with_capacity(112);
            payload.extend_from_slice(&pubkey);
            record.inscribe(&mut payload);
            self.stream.write_frame(&self.vault_path, &payload);
            self.dirty = true;
            if final_checkpoint {
                self.stream.flush_pending();
                self.sync_roster_sheet();
                self.dirty = false;
            }
        }
    }

    fn export_roster(&self, out: &mut Vec<([u8; 32], SurvivorRecord)>) {
        self.origin.export_roster(out);
    }

    fn flush_tick(&mut self) {
        if self.dirty {
            self.stream.flush_pending();
            self.sync_roster_sheet();
            self.dirty = false;
        }
        self.origin.flush_tick();
    }
}
