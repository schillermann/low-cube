use std::fs;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use world::Callsign;
use crate::ticket::JoinTarget;
use super::{HexKey, MemoryPassports, PassportClaim, PassportStamp, PassportVault};

/// Encapsulated `passports.toml` text parsing `[profile]` and `[passports]` entries (`<= 4` fields).
pub struct PassportDocument {
    content: String,
}

impl PassportDocument {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(content: String) -> Self {
        Self { content }
    }

    pub fn parse(&self) -> (Callsign, Vec<PassportStamp>) {
        let mut section = "";
        let mut callsign = Callsign::unnamed();
        let mut stamps = Vec::new();
        for raw_line in self.content.lines() {
            let line = raw_line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if line.starts_with('[') && line.ends_with(']') {
                section = line[1..line.len() - 1].trim();
                continue;
            }
            let Some((k, v)) = line.split_once('=') else { continue };
            let key = k.trim();
            let val = v
                .split('#')
                .next()
                .unwrap_or("")
                .trim()
                .trim_matches('"')
                .trim_matches('\'')
                .trim();
            if section.eq_ignore_ascii_case("profile") && key.eq_ignore_ascii_case("name") {
                callsign = Callsign::sanitized(val);
            } else if section.eq_ignore_ascii_case("passports") {
                let Some(host_key) = HexKey::parsed(key) else { continue };
                let mut parts = val.split('|');
                let Some(secret_hex) = parts.next() else { continue };
                let Some(endpoint_raw) = parts.next() else { continue };
                let assigned_raw = parts.next().unwrap_or("").trim();
                let Some(secret) = HexKey::parsed(secret_hex) else { continue };
                let Ok(endpoint) = endpoint_raw.trim().parse::<SocketAddr>() else { continue };
                let assigned = if assigned_raw.is_empty() {
                    Callsign::unnamed()
                } else {
                    let b = assigned_raw.as_bytes();
                    let count = b.len().min(24);
                    let mut wire = Vec::with_capacity(1 + count);
                    wire.push(count as u8);
                    wire.extend_from_slice(&b[..count]);
                    Callsign::decode(&wire).map_or(Callsign::unnamed(), |(c, _)| c)
                };
                stamps.push(PassportStamp::new(host_key.raw(), endpoint, secret.raw(), assigned));
            }
        }
        (callsign, stamps)
    }
}

/// Persistent TOML decorator over `MemoryPassports` saving server-issued identities on disk (`<= 4` fields).
pub struct DiskPassports {
    origin: MemoryPassports,
    path: PathBuf,
}

impl DiskPassports {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(origin: MemoryPassports, path: PathBuf) -> Self {
        Self { origin, path }
    }

    /// Loads `passports.toml` when present, seeding `fallback_callsign` when no profile name exists yet.
    pub fn open(path: impl AsRef<Path>, fallback_callsign: Callsign) -> Self {
        let vault_path = path.as_ref().to_path_buf();
        let Ok(raw) = fs::read_to_string(&vault_path) else {
            return Self::new(MemoryPassports::with_callsign(fallback_callsign), vault_path);
        };
        let (parsed_name, stamps) = PassportDocument::new(raw).parse();
        let chosen = if parsed_name.present() { parsed_name } else { fallback_callsign };
        Self::new(MemoryPassports::new(stamps, chosen), vault_path)
    }

    pub fn stamp_count(&self) -> usize {
        self.origin.stamp_count()
    }

    fn persist(&self) {
        if let Some(parent) = self.path.parent().filter(|p| !p.as_os_str().is_empty()) {
            let _ = fs::create_dir_all(parent);
        }
        let mut out = String::with_capacity(256);
        self.origin.inscribe_toml(&mut out);
        fs::write(&self.path, out).expect("Failed to persist client server passports to disk");
    }
}

impl PassportVault for DiskPassports {
    fn preferred_callsign(&self) -> Callsign {
        self.origin.preferred_callsign()
    }

    fn claim_for(&self, target: JoinTarget) -> PassportClaim {
        self.origin.claim_for(target)
    }

    fn store_issued(&mut self, stamp: PassportStamp) {
        self.origin.store_issued(stamp);
        self.persist();
    }
}
