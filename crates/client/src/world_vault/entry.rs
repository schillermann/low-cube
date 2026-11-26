use std::path::Path;
use world::{SolarProportions, WorldSeed};
use crate::toml_document::TomlDocument;
use super::domain::{WorldSlug, WorldTitle};
use super::seed_scale::{ScaleTier, SeedPhrase};

/// Immutable descriptor of a player-created world composed exclusively of domain objects (`4` fields).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorldSpec {
    slug: WorldSlug,
    title: WorldTitle,
    tier: ScaleTier,
    phrase: SeedPhrase,
}

impl WorldSpec {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(
        slug: WorldSlug,
        title: WorldTitle,
        tier: ScaleTier,
        phrase: SeedPhrase,
    ) -> Self {
        Self { slug, title, tier, phrase }
    }

    pub const fn slug(&self) -> &WorldSlug {
        &self.slug
    }

    pub fn with_slug(&self, slug: WorldSlug) -> Self {
        Self::new(slug, self.title.clone(), self.tier, self.phrase.clone())
    }

    pub fn same_slug(&self, candidate: &WorldSlug) -> bool {
        self.slug.matches(candidate)
    }

    pub fn announce_portal_banner(&self) {
        self.title.announce_portal_banner();
    }

    #[cfg(test)]
    pub fn matches_spec(&self, title: &str, blocks: u32, seed_label: &str, seed: WorldSeed) -> bool {
        self.title.matches_text(title)
            && self.tier.blocks() == blocks
            && self.phrase.matches_label(seed_label)
            && self.phrase.seed() == seed
    }

    pub fn roster_button_label(&self, active: bool) -> String {
        let summary = format!(
            "{}  |  {}  |  SEED {}",
            self.title.short_label(),
            self.tier.compact_label(),
            self.phrase.short_label(),
        );
        if active {
            format!("[ACTIVE] {summary}")
        } else {
            summary
        }
    }

    /// Resolves `SolarProportions` through the `SeededCosmos<EpsteinReachCosmos<...>>` decorator chain.
    pub fn proportions(&self, belt_density: f32) -> SolarProportions {
        self.tier.cosmos_scale(belt_density, self.phrase.seed()).proportions()
    }

    pub fn write_manifest(&self, root: &Path, created_epoch: u64) {
        let dir = self.slug.directory_in(root);
        std::fs::create_dir_all(&dir).expect("Failed to create world save directory");
        let mut content = String::from("[world]\n");
        self.title.inscribe_toml(&mut content);
        self.tier.inscribe_toml(&mut content);
        self.phrase.inscribe_toml(&mut content);
        content.push_str(&format!("created_epoch = {created_epoch}\n"));
        std::fs::write(dir.join("world.toml"), content).expect("Failed to write world.toml");
    }

    pub fn from_toml(slug: WorldSlug, doc: &TomlDocument) -> Option<(Self, u64)> {
        let raw_name = doc.section_property("world", "name", true)?;
        let title = WorldTitle::sanitized(&raw_name);
        if title.blank() {
            return None;
        }
        let tier = ScaleTier::clamped(
            doc.section_property("world", "size", true)
                .and_then(|v| v.parse::<u32>().ok())
                .unwrap_or(1),
        );
        let seed_bits = doc
            .section_property("world", "seed", true)
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(0);
        let raw_label = doc.section_property("world", "seed_label", true).unwrap_or_default();
        let phrase = SeedPhrase::rehydrated(WorldSeed::new(seed_bits), &raw_label);
        let created = doc
            .section_property("world", "created_epoch", true)
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(0);
        Some((Self::new(slug, title, tier, phrase), created))
    }
}
