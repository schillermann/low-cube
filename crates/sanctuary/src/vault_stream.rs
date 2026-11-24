use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Read, Write};
use std::path::Path;
use std::sync::Mutex;
use world::WorldFact;

/// Polymorphic buffered WAL handle avoiding nullable `Option<BufWriter<File>>` state and per-fact `open()` syscalls.
pub enum VaultStream {
    Dormant,
    Open(Mutex<BufWriter<File>>),
}

impl VaultStream {
    pub(crate) fn open_append(path: &Path) -> std::io::Result<Self> {
        let file = OpenOptions::new().create(true).append(true).open(path)?;
        Ok(Self::Open(Mutex::new(BufWriter::with_capacity(16_384, file))))
    }

    pub(crate) fn write_frame(&mut self, path: &Path, payload: &[u8]) {
        if matches!(self, Self::Dormant) {
            if let Ok(opened) = Self::open_append(path) {
                *self = opened;
            }
        }
        if let Self::Open(lock) = self {
            if let Ok(mut out) = lock.lock() {
                let len_bytes = (payload.len() as u32).to_le_bytes();
                let _ = out.write_all(&len_bytes);
                let _ = out.write_all(payload);
            }
        }
    }

    pub(crate) fn flush_pending(&mut self) {
        if let Self::Open(lock) = self {
            if let Ok(mut out) = lock.lock() {
                let _ = out.flush();
            }
        }
    }

    pub(crate) fn read_decoded_frames(path: &Path) -> std::io::Result<Option<Vec<WorldFact>>> {
        let Ok(mut file) = File::open(path) else {
            return Ok(None);
        };
        let mut buf = Vec::new();
        file.read_to_end(&mut buf)?;
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
        Ok(Some(out))
    }

    pub(crate) fn rewrite_atomic(journal_path: &Path, facts: &[WorldFact]) -> std::io::Result<()> {
        let tmp_path = std::path::PathBuf::from(format!("{}.tmp", journal_path.display()));
        {
            let file = OpenOptions::new().create(true).write(true).truncate(true).open(&tmp_path)?;
            let mut writer = BufWriter::with_capacity(16_384, file);
            let mut payload = Vec::with_capacity(128);
            for fact in facts {
                payload.clear();
                fact.inscribe(&mut payload);
                writer.write_all(&(payload.len() as u32).to_le_bytes())?;
                writer.write_all(&payload)?;
            }
            writer.flush()?;
        }
        std::fs::rename(&tmp_path, journal_path)
    }
}
