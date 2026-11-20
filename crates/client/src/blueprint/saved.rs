use world::CraftBlueprint;
use crate::attire::{BeltSatchel, HeldRelic, YouthAttire};

/// Named, serializable vehicle blueprint pairing player-assigned title and canonical `CraftBlueprint`.
#[derive(Clone, Debug, PartialEq)]
pub struct SavedBlueprint {
    title: String,
    blueprint: CraftBlueprint,
}

impl SavedBlueprint {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(title: String, blueprint: CraftBlueprint) -> Self {
        Self { title, blueprint }
    }

    pub fn title_text(&self) -> &str {
        &self.title
    }

    pub const fn blueprint(&self) -> &CraftBlueprint {
        &self.blueprint
    }

    pub fn manifest(&self) -> Vec<(HeldRelic, u32)> {
        let mut counts = [0u32; HeldRelic::COUNT];
        self.blueprint.tally_media(|med| {
            let relic = HeldRelic::from_medium(med);
            counts[relic.code() as usize] = counts[relic.code() as usize].saturating_add(1);
        });
        let mut list = Vec::new();
        for relic in HeldRelic::TRAY_ORDER {
            let qty = counts[relic.code() as usize];
            if qty > 0 {
                list.push((relic, qty));
            }
        }
        list
    }

    pub fn buildable(&self, satchel: &BeltSatchel, attire: YouthAttire) -> bool {
        self.blueprint.drivable()
            && self
                .manifest()
                .into_iter()
                .all(|(relic, needed)| satchel.resource_tally(relic, attire) >= needed)
    }

    pub fn manifest_caption(&self) -> String {
        let manifest = self.manifest();
        let mut items: Vec<String> = manifest
            .iter()
            .take(3)
            .map(|&(r, n)| format!("{} X{}", r.title(), n))
            .collect();
        if manifest.len() > 3 {
            items.push(format!("+{} MORE", manifest.len() - 3));
        }
        items.join("   ")
    }

    pub fn requirement_summary(&self, satchel: &BeltSatchel, attire: YouthAttire) -> String {
        let mut items = self.manifest();
        items.sort_by_key(|&(relic, needed)| u8::from(satchel.resource_tally(relic, attire) >= needed));
        let mut labels: Vec<String> = items
            .iter()
            .take(3)
            .map(|&(relic, needed)| format!("{}: {}/{}", relic.title(), satchel.resource_tally(relic, attire), needed))
            .collect();
        if items.len() > 3 {
            labels.push(format!("+{} MORE", items.len() - 3));
        }
        labels.join("  ")
    }
}
