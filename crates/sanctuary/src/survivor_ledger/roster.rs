use std::fs;
use std::path::{Path, PathBuf};
use world::SurvivorRecord;
use crate::passport::HexKey;

/// Human-readable `<journal>.survivors.toml` inventory ledger mapping callsigns to server-issued identities (`<= 4` fields).
pub struct RosterSheet {
    path: PathBuf,
}

impl RosterSheet {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn for_vault(vault_path: &Path) -> Self {
        Self::new(PathBuf::from(format!("{}.toml", vault_path.display())))
    }

    pub fn write(&self, entries: &[([u8; 32], SurvivorRecord)]) {
        if let Some(parent) = self.path.parent().filter(|p| !p.as_os_str().is_empty()) {
            let _ = fs::create_dir_all(parent);
        }
        let mut text = String::from("# Authoritative Server Survivor Inventory Roster\n");
        for &(pubkey, record) in entries {
            let hex = HexKey::new(pubkey);
            record.inscribe_roster_entry(&hex.short_tag(), &hex.encoded(), &mut text);
        }
        let _ = fs::write(&self.path, text);
    }
}
