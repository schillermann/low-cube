use std::collections::HashMap;
use glam::Vec3;
use world::{BodyId, ClimateZone, PlanetaryWorld, WorldFact};
use crate::chronicle_log::ChronicleStore;

const LIVE_SECTOR_COMPACT_THRESHOLD: usize = 32;

fn sector_for_eye(body_code: u16, eye: Vec3) -> (u16, u8, i32, i32) {
    let zone = ClimateZone::dominant(eye);
    let face = zone.index() as u8;
    let (u, v) = zone.surface_frame(1.0).cell_at(eye).feed_coordinates(|cu, cv| (cu, cv));
    (body_code, face, u.div_euclid(8), v.div_euclid(8))
}

fn foldable_sector_key(body_code: u16, fact: &WorldFact) -> Option<(u16, u8, i32, i32)> {
    match fact {
        WorldFact::SectorSnapshot { snapshot } => {
            let (gsu, gsv) = snapshot.sector();
            Some((body_code, snapshot.face(), gsu, gsv))
        }
        WorldFact::StratumDisclosed { face, stratum } => {
            Some(stratum.transfer(|u, v, _, _| (body_code, *face, u.div_euclid(8), v.div_euclid(8))))
        }
        WorldFact::Harvested { eye, .. }
        | WorldFact::MoldPlaced { eye, .. }
        | WorldFact::RigAdjusted { eye, .. }
        | WorldFact::RigLowered { eye, .. }
        | WorldFact::RigBlueprintBuilt { eye, .. } => Some(sector_for_eye(body_code, *eye)),
        WorldFact::OffWorld { body, fact: inner } => foldable_sector_key(body.raw_code(), inner),
        _ => None,
    }
}

/// `ChronicleStore` decorator folding aged `4m × 4m` sector raycast logs into raycast-free `SectorSnapshot`s (`<= 4` fields).
pub struct SnapshottedChronicle<S> {
    origin: S,
    sector_facts: HashMap<(u16, u8, i32, i32), Vec<u32>>,
    hot_sectors: Vec<(u16, u8, i32, i32)>,
    threshold: usize,
}

impl<S> SnapshottedChronicle<S> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(
        origin: S,
        sector_facts: HashMap<(u16, u8, i32, i32), Vec<u32>>,
        hot_sectors: Vec<(u16, u8, i32, i32)>,
        threshold: usize,
    ) -> Self {
        Self { origin, sector_facts, hot_sectors, threshold }
    }

    pub fn with_threshold(origin: S, threshold: usize) -> Self {
        Self::new(origin, HashMap::new(), Vec::new(), threshold.max(4))
    }

    pub fn wrapping(origin: S) -> Self {
        Self::with_threshold(origin, LIVE_SECTOR_COMPACT_THRESHOLD)
    }
}

impl<S: ChronicleStore> SnapshottedChronicle<S> {
    fn preserve_rig_commission(&mut self, fact: &WorldFact) {
        match fact {
            WorldFact::RigBlueprintBuilt { blueprint, .. } if blueprint.identity().assigned() => {
                self.origin.record(WorldFact::CraftCommissioned { blueprint: blueprint.clone() });
            }
            WorldFact::OffWorld { fact: inner, .. } => self.preserve_rig_commission(inner),
            _ => {}
        }
    }
}

impl<S: ChronicleStore> ChronicleStore for SnapshottedChronicle<S> {
    fn total(&self) -> u32 {
        self.origin.total()
    }

    fn record(&mut self, fact: WorldFact) {
        let before = self.origin.total();
        self.origin.record(fact.clone());
        if self.origin.total() > before {
            if let Some(key) = foldable_sector_key(0, &fact) {
                let list = self.sector_facts.entry(key).or_default();
                list.push(before);
                if list.len() >= self.threshold && !self.hot_sectors.contains(&key) {
                    self.hot_sectors.push(key);
                }
            }
        }
    }

    fn batch_from(&self, ack_seq: u32) -> Option<(u32, Vec<WorldFact>)> {
        self.origin.batch_from(ack_seq)
    }

    fn fact_at(&self, index: usize) -> Option<&WorldFact> {
        self.origin.fact_at(index)
    }

    fn supersede(&mut self, index: u32) {
        self.origin.supersede(index);
    }

    fn compact_sectors(&mut self, _earth_half: f32, world: &mut dyn PlanetaryWorld) -> Vec<u32> {
        if self.hot_sectors.is_empty() {
            return Vec::new();
        }
        let hot = std::mem::take(&mut self.hot_sectors);
        let mut superseded = Vec::new();
        for (body_code, face, gsu, gsv) in hot {
            let count = self.sector_facts.get(&(body_code, face, gsu, gsv)).map_or(0, Vec::len);
            if count < self.threshold {
                continue;
            }
            for dv in -1..=1 {
                for du in -1..=1 {
                    let nbr = (body_code, face, gsu + du, gsv + dv);
                    let prior = self.sector_facts.remove(&nbr).unwrap_or_default();
                    let had_prior = !prior.is_empty();
                    for old_id in prior {
                        if let Some(f) = self.origin.fact_at(old_id as usize).cloned() {
                            self.preserve_rig_commission(&f);
                        }
                        self.origin.supersede(old_id);
                        superseded.push(old_id);
                    }
                    let snap = if body_code == 0 {
                        world.capture_sector_snapshot(face, gsu + du, gsv + dv)
                    } else {
                        world.capture_offworld_snapshot(BodyId::new(body_code), face, gsu + du, gsv + dv)
                    };
                    if !snap.pristine() || had_prior {
                        let snap_fact = if body_code == 0 {
                            WorldFact::SectorSnapshot { snapshot: snap }
                        } else {
                            WorldFact::OffWorld {
                                body: BodyId::new(body_code),
                                fact: Box::new(WorldFact::SectorSnapshot { snapshot: snap }),
                            }
                        };
                        let idx = self.origin.total();
                        self.origin.record(snap_fact);
                        if self.origin.total() > idx {
                            self.sector_facts.insert(nbr, vec![idx]);
                        }
                    }
                }
            }
        }
        superseded.sort_unstable();
        superseded.dedup();
        superseded
    }

    fn flush_tick(&mut self) {
        self.origin.flush_tick();
    }
}
