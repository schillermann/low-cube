use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use world::{SatchelManifest, SurvivorRecord, SurvivorWelcome};
use crate::chronicle_decorators::VaultStream;

/// Authoritative per-X25519-identity survivor inventory and position store.
pub trait SurvivorStore: Send {
    fn admit_or_restore(&self, pubkey: [u8; 32]) -> SurvivorWelcome;
    fn lookup(&self, pubkey: [u8; 32]) -> Option<SurvivorRecord>;
    fn commit(&mut self, pubkey: [u8; 32], record: SurvivorRecord, final_checkpoint: bool);
    fn flush_tick(&mut self) {}
}

/// In-memory `SurvivorStore` mapping `[u8; 32]` X25519 public keys to `SurvivorRecord`s (`<= 4` fields).
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

    fn commit(&mut self, pubkey: [u8; 32], record: SurvivorRecord, _final_checkpoint: bool) {
        if pubkey != [0u8; 32] {
            self.records.insert(pubkey, record);
        }
    }
}

/// Buffered SSD write-ahead `SurvivorStore` decorator persisting survivor checkpoints across restarts (`<= 4` fields).
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

impl<S> Drop for DiskVaultSurvivors<S> {
    fn drop(&mut self) {
        self.stream.flush_pending();
    }
}

impl<S: SurvivorStore> DiskVaultSurvivors<S> {
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
        Ok(Self::with_stream(origin, vault_path, stream, false))
    }
}

impl<S: SurvivorStore> SurvivorStore for DiskVaultSurvivors<S> {
    fn admit_or_restore(&self, pubkey: [u8; 32]) -> SurvivorWelcome {
        self.origin.admit_or_restore(pubkey)
    }

    fn lookup(&self, pubkey: [u8; 32]) -> Option<SurvivorRecord> {
        self.origin.lookup(pubkey)
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
                    || (prev.position() - record.position()).length_squared() >= 64.0
            }
        };
        self.origin.commit(pubkey, record, final_checkpoint);
        if significant {
            let mut payload = Vec::with_capacity(96);
            payload.extend_from_slice(&pubkey);
            record.inscribe(&mut payload);
            self.stream.write_frame(&self.vault_path, &payload);
            self.dirty = true;
        }
    }

    fn flush_tick(&mut self) {
        if self.dirty {
            self.stream.flush_pending();
            self.dirty = false;
        }
        self.origin.flush_tick();
    }
}
