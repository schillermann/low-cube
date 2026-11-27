mod stream;

use std::collections::HashMap;
use glam::Vec3;
use world::{BodyId, ClimateZone, CraftIdentity, WorldFact};
use crate::chronicle_log::ChronicleStore;
pub use self::stream::HorizonStream;

pub(super) const EARTH_SECTOR_METERS: f32 = 128.0;
pub(super) const HORIZON_FACT_METERS: f32 = 256.0;

fn earth_face_and_sector(pos: Vec3, earth_half: f32) -> (u8, i32, i32) {
    let zone = ClimateZone::dominant(pos);
    let idx = zone.index() as u8;
    let frame = zone.surface_frame(earth_half);
    let (u, v) = frame.decompose(pos);
    (
        idx,
        (u / EARTH_SECTOR_METERS).floor() as i32,
        (v / EARTH_SECTOR_METERS).floor() as i32,
    )
}

/// Spatial index partitioning `ChronicleStore` fact IDs by `(ClimateZone, Sector)`, `BodyId`, and `CraftIdentity` (`<= 4` fields).
pub struct SpatialFactIndex {
    synced_len: u32,
    pub(super) earth_sectors: HashMap<(u8, i32, i32), Vec<u32>>,
    pub(super) offworld: HashMap<BodyId, Vec<u32>>,
    pub(super) fleet: (HashMap<CraftIdentity, u32>, HashMap<CraftIdentity, (u32, Vec3)>),
}

impl SpatialFactIndex {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(
        synced_len: u32,
        earth_sectors: HashMap<(u8, i32, i32), Vec<u32>>,
        offworld: HashMap<BodyId, Vec<u32>>,
        fleet: (HashMap<CraftIdentity, u32>, HashMap<CraftIdentity, (u32, Vec3)>),
    ) -> Self {
        Self { synced_len, earth_sectors, offworld, fleet }
    }

    pub fn empty() -> Self {
        Self::new(0, HashMap::new(), HashMap::new(), (HashMap::new(), HashMap::new()))
    }

    pub fn sync_with(&mut self, store: &dyn ChronicleStore, earth_half: f32) {
        let total = store.total();
        while self.synced_len < total {
            let idx = self.synced_len;
            self.synced_len += 1;
            let Some(fact) = store.fact_at(idx as usize) else { continue };
            self.index_fact(idx, fact, earth_half);
        }
    }

    pub fn prune_superseded(&mut self, dead_sorted: &[u32]) {
        if dead_sorted.is_empty() {
            return;
        }
        for ids in self.earth_sectors.values_mut() {
            ids.retain(|id| dead_sorted.binary_search(id).is_err());
        }
        for ids in self.offworld.values_mut() {
            ids.retain(|id| dead_sorted.binary_search(id).is_err());
        }
    }

    fn index_fact(&mut self, idx: u32, fact: &WorldFact, earth_half: f32) {
        match fact {
            WorldFact::OffWorld { body, fact: inner } => {
                if let WorldFact::CraftCommissioned { blueprint } = inner.as_ref() {
                    if blueprint.identity().assigned() {
                        self.fleet.0.insert(blueprint.identity(), idx);
                    }
                }
                self.offworld.entry(*body).or_default().push(idx);
            }
            WorldFact::SectorSnapshot { snapshot } => {
                let (gsu, gsv) = snapshot.sector();
                let key = (snapshot.face(), gsu.div_euclid(32), gsv.div_euclid(32));
                self.earth_sectors.entry(key).or_default().push(idx);
            }
            WorldFact::CraftCommissioned { blueprint } => {
                if blueprint.identity().assigned() {
                    self.fleet.0.insert(blueprint.identity(), idx);
                }
            }
            WorldFact::HelmClaimed { craft, eye, .. } => {
                if craft.assigned() {
                    self.fleet.1.insert(*craft, (idx, *eye));
                } else {
                    let key = earth_face_and_sector(*eye, earth_half);
                    self.earth_sectors.entry(key).or_default().push(idx);
                }
            }
            WorldFact::HelmReleased { craft, pose } => {
                if craft.assigned() {
                    self.fleet.1.insert(*craft, (idx, pose.origin()));
                } else {
                    let key = earth_face_and_sector(pose.origin(), earth_half);
                    self.earth_sectors.entry(key).or_default().push(idx);
                }
            }
            WorldFact::StratumDisclosed { face, stratum } => {
                let key = stratum.transfer(|u, v, _, _| (*face, u.div_euclid(256), v.div_euclid(256)));
                self.earth_sectors.entry(key).or_default().push(idx);
            }
            WorldFact::Harvested { eye, .. }
            | WorldFact::MoldPlaced { eye, .. }
            | WorldFact::TimberStruck { eye, .. }
            | WorldFact::Foraged { eye, .. }
            | WorldFact::RigAdjusted { eye, .. }
            | WorldFact::RigLowered { eye, .. }
            | WorldFact::LitterCast { eye, .. }
            | WorldFact::RigBlueprintBuilt { eye, .. } => {
                if let WorldFact::RigBlueprintBuilt { blueprint, .. } = fact {
                    if blueprint.identity().assigned() {
                        self.fleet.0.insert(blueprint.identity(), idx);
                    }
                }
                let key = earth_face_and_sector(*eye, earth_half);
                self.earth_sectors.entry(key).or_default().push(idx);
            }
            WorldFact::Loosened { .. } => {}
        }
    }
}
