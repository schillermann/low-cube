mod cosmos;

use std::env;
use std::fs;
use std::path::PathBuf;
use std::thread;
use std::time::{Duration, Instant};
use world::{CelestialSystem, PlanetaryWorld, SatchelManifest, SolarProportions};
use crate::chronicle_decorators::{CompactedChronicle, DiskVaultChronicle};
use crate::chronicle_log::{ChronicleLog, ChronicleStore};
use crate::survivor_ledger::{DiskVaultSurvivors, MemorySurvivors, SurvivorStore};
use crate::SanctuaryHost;
use self::cosmos::ServerCosmosScale;

/// Encapsulated `server.toml` text resolving dedicated sanctuary properties without external crates.
#[derive(Clone, Debug)]
pub struct SanctuaryToml {
    content: String,
}

impl SanctuaryToml {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(content: String) -> Self {
        Self { content }
    }

    /// Secondary constructor loading configuration text from disk when present.
    pub fn from_file(path: &str) -> Self {
        Self::new(fs::read_to_string(path).unwrap_or_default())
    }

    pub fn section_property(&self, section: &str, key: &str, top_level_allowed: bool) -> Option<String> {
        let mut active = top_level_allowed;
        for line in self.content.lines().map(str::trim) {
            if line.starts_with('[') && line.ends_with(']') {
                let header = line[1..line.len() - 1].trim();
                active = header.eq_ignore_ascii_case(section);
                continue;
            }
            if active && !line.is_empty() && !line.starts_with('#') {
                if let Some((k, raw_val)) = line.split_once('=') {
                    if k.trim().eq_ignore_ascii_case(key) {
                        let uncommented = raw_val.split('#').next().unwrap_or("").trim();
                        let cleaned = uncommented.trim_matches('"').trim_matches('\'').trim();
                        return Some(cleaned.to_string());
                    }
                }
            }
        }
        None
    }
}

/// Encapsulated UDP binding port for a dedicated P2P `SanctuaryHost`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SanctuaryEndpoint {
    port: u16,
}

/// Polymorphic dedicated-server persistence mode avoiding nullable `Option<PathBuf>` fields.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum VaultPersistence {
    Ephemeral,
    Journaled(PathBuf),
}

impl SanctuaryEndpoint {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(port: u16) -> Self {
        Self { port }
    }

    pub const fn port(&self) -> u16 {
        self.port
    }

    fn serve(
        self,
        proportions: SolarProportions,
        persistence: VaultPersistence,
        starter_kit: SatchelManifest,
    ) -> std::io::Result<()> {
        let blocks_per_edge = proportions.earth_blocks();
        let bind_addr = format!("0.0.0.0:{}", self.port);
        let bevel = ((blocks_per_edge as f32) * 0.5 * 0.125).clamp(0.5, 1.0);
        let mut world: Box<dyn PlanetaryWorld> = CelestialSystem::heliosphere(proportions, bevel);
        let mem_survivors = MemorySurvivors::with_starter(starter_kit);
        let (store, survivors): (Box<dyn ChronicleStore>, Box<dyn SurvivorStore>) = match persistence {
            VaultPersistence::Journaled(path) => {
                let surv_path = PathBuf::from(format!("{}.survivors", path.display()));
                let base = CompactedChronicle::wrapping(ChronicleLog::empty());
                let chronicle = Box::new(DiskVaultChronicle::replay_and_open(path, base, world.as_mut())?);
                let ledger = Box::new(DiskVaultSurvivors::replay_and_open(surv_path, mem_survivors)?);
                (chronicle, ledger)
            }
            VaultPersistence::Ephemeral => (
                Box::new(CompactedChronicle::wrapping(ChronicleLog::empty())),
                Box::new(mem_survivors),
            ),
        };
        let mut host = SanctuaryHost::bind_cosmos_with_stores(&bind_addr, world, proportions, store, survivors)?;
        let bound = host.local_addr()?;
        let ticket = host.ticket().encode();
        println!("=== LOW-CUBE DEDICATED SERVER ===");
        println!("Listening UDP:  {bound} ({blocks_per_edge} blocks/edge)");
        println!(
            "Solar Scale:    dist={:.4} body={:.2} sun={:.2} belt={:.2}",
            proportions.distance_scale(),
            proportions.body_scale(),
            proportions.sun_scale(),
            proportions.belt_density(),
        );
        println!("Portal Code:    {}", host.code().encode());
        println!("Direct Address: {}", host.ticket().direct());
        println!("Invite Ticket:  {ticket}");
        println!("Players join in-game via Esc -> MULTIPLAYER -> JOIN");
        let mut last = Instant::now();
        loop {
            let now = Instant::now();
            let dt = (now - last).as_secs_f32().min(0.1);
            last = now;
            let _ = host.tick(dt);
            thread::sleep(Duration::from_millis(15));
        }
    }
}

/// Assembled dedicated server configuration combining `server.toml`, environment variables, and CLI flags (`<= 4` fields).
#[derive(Clone, Debug, PartialEq)]
pub struct SanctuaryPlan {
    endpoint: SanctuaryEndpoint,
    proportions: SolarProportions,
    persistence: VaultPersistence,
    starter_kit: SatchelManifest,
}

impl SanctuaryPlan {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn with_starter(
        endpoint: SanctuaryEndpoint,
        proportions: SolarProportions,
        persistence: VaultPersistence,
        starter_kit: SatchelManifest,
    ) -> Self {
        Self { endpoint, proportions, persistence, starter_kit }
    }

    pub fn with_persistence(
        endpoint: SanctuaryEndpoint,
        proportions: SolarProportions,
        persistence: VaultPersistence,
    ) -> Self {
        Self::with_starter(endpoint, proportions, persistence, SatchelManifest::starter_kit())
    }

    pub fn with_proportions(endpoint: SanctuaryEndpoint, proportions: SolarProportions) -> Self {
        Self::with_persistence(endpoint, proportions, VaultPersistence::Ephemeral)
    }

    /// Secondary constructor initializing true-scale solar proportions for `blocks_per_edge`.
    pub fn new(endpoint: SanctuaryEndpoint, blocks_per_edge: u32) -> Self {
        Self::with_proportions(endpoint, SolarProportions::true_scale(blocks_per_edge))
    }

    pub const fn endpoint(&self) -> SanctuaryEndpoint { self.endpoint }
    pub const fn blocks_per_edge(&self) -> u32 { self.proportions.earth_blocks() }
    pub const fn proportions(&self) -> SolarProportions { self.proportions }
    pub const fn starter_kit(&self) -> SatchelManifest { self.starter_kit }

    fn parse_scale_factor(raw: &str, source: &str) -> u32 {
        let factor = raw.trim().parse::<i64>().unwrap_or_else(|_| {
            panic!("Invalid world-cube size '{raw}' in {source}: expected integer 1..=32")
        });
        let max_f = i64::from(SolarProportions::MAX_WORLD_SCALE_FACTOR);
        assert!(
            (1..=max_f).contains(&factor),
            "Invalid world-cube size '{raw}' in {source}: allowed range is 1..={max_f}"
        );
        factor as u32
    }

    /// Secondary constructor assembling configuration in priority order: CLI > Env > `server.toml` > Baseline.
    pub fn assembled(args: &[String], toml: &SanctuaryToml) -> Self {
        let mut port = 27777u16;
        let mut blocks = 128u32;
        let mut persistence = toml
            .section_property("storage", "journal", false)
            .filter(|s| !s.is_empty())
            .map_or(VaultPersistence::Ephemeral, |s| VaultPersistence::Journaled(PathBuf::from(s)));
        let mut starter_kit = toml
            .section_property("inventory", "items", false)
            .or_else(|| toml.section_property("inventory", "satchel", false))
            .map_or_else(SatchelManifest::starter_kit, |raw| {
                SatchelManifest::parse_server_kit(&raw, "server.toml")
            });

        if let Some(raw_size) = toml
            .section_property("world", "size", true)
            .or_else(|| toml.section_property("world", "scale", true))
        {
            blocks = 128 * Self::parse_scale_factor(&raw_size, "server.toml");
        }
        if let Some(raw_port) = toml.section_property("server", "port", true) {
            port = raw_port.parse::<u16>().unwrap_or_else(|_| {
                panic!("Invalid server port '{raw_port}' in server.toml: expected 1..=65535")
            });
        }

        if let Ok(env_size) = env::var("LOW_CUBE_SIZE").or_else(|_| env::var("LOW_CUBE_SCALE")) {
            blocks = 128 * Self::parse_scale_factor(&env_size, "LOW_CUBE_SIZE");
        }
        if let Ok(env_journal) = env::var("LOW_CUBE_JOURNAL") {
            if !env_journal.trim().is_empty() {
                persistence = VaultPersistence::Journaled(PathBuf::from(env_journal.trim()));
            }
        }
        if let Ok(env_inv) = env::var("LOW_CUBE_INVENTORY").or_else(|_| env::var("LOW_CUBE_ITEMS")) {
            starter_kit = SatchelManifest::parse_server_kit(&env_inv, "LOW_CUBE_INVENTORY");
        }
        if let Ok(env_port) = env::var("LOW_CUBE_PORT") {
            if let Ok(parsed) = env_port.trim().parse::<u16>() {
                port = parsed;
            }
        }

        let mut idx = 1;
        while idx < args.len() {
            let arg = args[idx].as_str();
            if let Some(val) = arg.strip_prefix("--size=").or_else(|| arg.strip_prefix("--scale=")) {
                blocks = 128 * Self::parse_scale_factor(val, "--size");
                idx += 1;
            } else if matches!(arg, "--size" | "--scale") && idx + 1 < args.len() {
                blocks = 128 * Self::parse_scale_factor(&args[idx + 1], "--size");
                idx += 2;
            } else if let Some(val) = arg.strip_prefix("--journal=") {
                if !val.trim().is_empty() {
                    persistence = VaultPersistence::Journaled(PathBuf::from(val.trim()));
                }
                idx += 1;
            } else if arg == "--journal" && idx + 1 < args.len() {
                persistence = VaultPersistence::Journaled(PathBuf::from(args[idx + 1].trim()));
                idx += 2;
            } else if let Some(val) = arg.strip_prefix("--inventory=").or_else(|| arg.strip_prefix("--items=")) {
                starter_kit = SatchelManifest::parse_server_kit(val, "--inventory");
                idx += 1;
            } else if matches!(arg, "--inventory" | "--items") && idx + 1 < args.len() {
                starter_kit = SatchelManifest::parse_server_kit(&args[idx + 1], "--inventory");
                idx += 2;
            } else if arg == "--blocks" && idx + 1 < args.len() {
                if let Ok(b) = args[idx + 1].parse::<u32>() {
                    blocks = b.clamp(16, 4_096);
                }
                idx += 2;
            } else if arg == "--port" && idx + 1 < args.len() {
                if let Ok(p) = args[idx + 1].parse::<u16>() {
                    port = p;
                }
                idx += 2;
            } else {
                idx += 1;
            }
        }

        let proportions = ServerCosmosScale::new(args, toml).resolve(blocks);
        Self::with_starter(SanctuaryEndpoint::new(port), proportions, persistence, starter_kit)
    }

    pub fn execute(self) -> std::io::Result<()> {
        self.endpoint.serve(self.proportions, self.persistence, self.starter_kit)
    }
}
