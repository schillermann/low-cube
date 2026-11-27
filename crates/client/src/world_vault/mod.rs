mod domain;
mod entry;
mod seed_scale;

#[cfg(test)]
use std::path::Path;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use sanctuary::{HostedPlanet, SanctuaryHost, SavedPlanet};
use world::{PlanetaryWorld, SatchelManifest, SolarProportions};
use crate::attire::{BeltSatchel, EvaAttire};
use crate::command_flags::CommandFlags;
use crate::menu::PortalInvite;
use crate::portal_config::ClientPassportBook;
use crate::server_satchel::{ServerSatchel, StartingSatchel as _};
use crate::toml_document::TomlDocument;

pub use self::domain::{WorldSlug, WorldTitle};
pub use self::entry::WorldSpec;
pub use self::seed_scale::{ScaleTier, SeedPhrase};

/// Monotonic wall-clock probe object eliminating static timestamp functions (`0` fields).
#[derive(Clone, Copy, Debug)]
pub struct WallEpoch;

impl WallEpoch {
    /// Primary constructor initializing the wall-clock probe.
    pub const fn new() -> Self {
        Self
    }

    pub fn seconds(self) -> u64 {
        SystemTime::now().duration_since(UNIX_EPOCH).map_or(1, |d| d.as_secs())
    }

    pub fn nanos(self) -> u64 {
        SystemTime::now().duration_since(UNIX_EPOCH).map_or(7, |d| d.as_nanos() as u64)
    }
}

/// Focused manifest persistence contract consumed by the `SavedWorldStudio` UI decorator.
pub trait WorldArchive {
    fn load_roster(&self) -> Vec<WorldSpec>;
    fn persist_world(&mut self, spec: &WorldSpec) -> WorldSpec;
    fn erase_world(&mut self, slug: &WorldSlug) -> bool;
}

/// Focused runtime world-mounting contract owned exclusively by `SessionStage` (zero `Rc<RefCell>`).
pub trait RealmPort {
    fn mount_solitary(&self, spec: &WorldSpec) -> Box<dyn PlanetaryWorld>;
    fn mount_hosted(&self, spec: &WorldSpec) -> (Box<dyn PlanetaryWorld>, PortalInvite);
    fn purge_evicted(&mut self, slug: &WorldSlug);
}

/// Filesystem world vault storing player worlds under `worlds/<slug>/` (`3` fields).
#[derive(Clone)]
pub struct DiskWorldVault {
    root: PathBuf,
    belt_density: f32,
    starter_kit: SatchelManifest,
}

impl DiskWorldVault {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(root: PathBuf, belt_density: f32, starter_kit: SatchelManifest) -> Self {
        Self { root, belt_density, starter_kit }
    }

    pub fn live(base_proportions: SolarProportions) -> Self {
        let flags = CommandFlags::process();
        let dir = flags
            .value_of("--worlds-dir")
            .or_else(|| std::env::var("LOW_CUBE_WORLDS_DIR").ok())
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "worlds".to_string());
        let attire = EvaAttire::eva_equipped();
        let starter_kit = ServerSatchel::assembled()
            .provision(BeltSatchel::empty())
            .survivor_manifest(attire);
        Self::new(PathBuf::from(dir), base_proportions.belt_density(), starter_kit)
    }

    #[cfg(test)]
    pub fn at_path(root: impl AsRef<Path>, starter_kit: SatchelManifest) -> Self {
        Self::new(root.as_ref().to_path_buf(), 1.0, starter_kit)
    }

    fn unique_slug(&self, base: WorldSlug) -> WorldSlug {
        if !base.directory_in(&self.root).exists() {
            return base;
        }
        for ordinal in 2..10_000u32 {
            let candidate = base.disambiguated(ordinal);
            if !candidate.directory_in(&self.root).exists() {
                return candidate;
            }
        }
        base.disambiguated(WallEpoch::new().seconds() as u32)
    }
}

impl WorldArchive for DiskWorldVault {
    fn load_roster(&self) -> Vec<WorldSpec> {
        let Ok(read_dir) = std::fs::read_dir(&self.root) else {
            return Vec::new();
        };
        let mut dated: Vec<(WorldSpec, u64)> = Vec::new();
        for entry in read_dir.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let Some(raw_slug) = path.file_name().and_then(|n| n.to_str()).map(str::to_string) else {
                continue;
            };
            let Some(slug) = WorldSlug::validated(raw_slug) else {
                continue;
            };
            let manifest = path.join("world.toml");
            let Some(doc) = manifest.to_str().and_then(TomlDocument::from_file) else {
                continue;
            };
            if let Some(parsed) = WorldSpec::from_toml(slug, &doc) {
                dated.push(parsed);
            }
        }
        dated.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.slug().compare_key(b.0.slug())));
        dated.into_iter().map(|(spec, _)| spec).collect()
    }

    fn persist_world(&mut self, spec: &WorldSpec) -> WorldSpec {
        let unique = self.unique_slug(spec.slug().clone());
        let resolved = if spec.same_slug(&unique) { spec.clone() } else { spec.with_slug(unique) };
        resolved.write_manifest(&self.root, WallEpoch::new().seconds());
        resolved
    }

    fn erase_world(&mut self, slug: &WorldSlug) -> bool {
        let target = slug.directory_in(&self.root);
        target.is_dir() && std::fs::remove_dir_all(target).is_ok()
    }
}

impl RealmPort for DiskWorldVault {
    fn mount_solitary(&self, spec: &WorldSpec) -> Box<dyn PlanetaryWorld> {
        let callsign = ClientPassportBook::from_process().host_callsign();
        let proportions = spec.proportions(self.belt_density);
        let saved = SavedPlanet::open(
            spec.slug().journal_in(&self.root),
            callsign,
            proportions,
            self.starter_kit,
        )
        .expect("Failed to open saved player world");
        Box::new(saved)
    }

    fn mount_hosted(&self, spec: &WorldSpec) -> (Box<dyn PlanetaryWorld>, PortalInvite) {
        let callsign = ClientPassportBook::from_process().host_callsign();
        let proportions = spec.proportions(self.belt_density);
        let host = SanctuaryHost::bind_journaled_cosmos(
            "0.0.0.0:0",
            spec.slug().journal_in(&self.root),
            callsign,
            proportions,
            self.starter_kit,
        )
        .expect("Failed to bind live co-op portal socket for saved world");
        let code = host.code().encode();
        let ticket = host.ticket().encode();
        spec.announce_portal_banner();
        println!("Portal Code:    {code}");
        println!("Direct Address: {}", host.ticket().direct());
        println!("Invite Ticket:  {ticket}");
        (Box::new(HostedPlanet::spawning(host)), PortalInvite::hosted(code, ticket))
    }

    fn purge_evicted(&mut self, slug: &WorldSlug) {
        let _ = self.erase_world(slug);
    }
}

/// Fast in-memory `RealmPort` for windowless unit tests (`0` fields, zero `Rc<RefCell>`).
#[cfg(test)]
pub struct MemoryRealmPort;

#[cfg(test)]
impl MemoryRealmPort {
    /// Primary constructor initializing the in-memory realm port.
    pub const fn new() -> Self {
        Self
    }
}

#[cfg(test)]
impl RealmPort for MemoryRealmPort {
    fn mount_solitary(&self, spec: &WorldSpec) -> Box<dyn PlanetaryWorld> {
        let prop = spec.proportions(1.0);
        Box::new(world::BiomedQuadSphere::barren_seeded(prop.earth_blocks().min(64), prop.seed()))
    }

    fn mount_hosted(&self, spec: &WorldSpec) -> (Box<dyn PlanetaryWorld>, PortalInvite) {
        (self.mount_solitary(spec), PortalInvite::empty())
    }

    fn purge_evicted(&mut self, _slug: &WorldSlug) {}
}
