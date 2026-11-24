use std::collections::HashMap;
use glam::Vec3;
use world::{
    BodyId, ClimateZone, CosmicFrame, CraftIdentity, DrivenCraft, SolarEphemeris, SolarProportions,
    WorldFact,
};
use crate::chronicle_log::ChronicleStore;

const EARTH_SECTOR_METERS: f32 = 128.0;
const HORIZON_FACT_METERS: f32 = 256.0;

fn earth_face_and_sector(pos: Vec3, earth_half: f32) -> (u8, i32, i32) {
    let zone = ClimateZone::dominant(pos);
    let idx = ClimateZone::all().iter().position(|&z| z == zone).unwrap_or(0) as u8;
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
    earth_sectors: HashMap<(u8, i32, i32), Vec<u32>>,
    offworld: HashMap<BodyId, Vec<u32>>,
    fleet: (HashMap<CraftIdentity, u32>, HashMap<CraftIdentity, (u32, Vec3)>),
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
                let zones = ClimateZone::all();
                let zone = zones.get(*face as usize).copied().unwrap_or(ClimateZone::Temperate);
                let frame = zone.surface_frame(earth_half);
                let pt = stratum.transfer(|u, v, elev, _| {
                    frame.point_at((u as f32) * 0.5 + 0.25, (v as f32) * 0.5 + 0.25, (elev as f32) * 0.5)
                });
                let key = earth_face_and_sector(pt, earth_half);
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
            WorldFact::Loosened { .. } | WorldFact::BoarStruck { .. } => {}
        }
    }
}

/// Per-guest monotonic `HostChronicle` stream delivering exclusively facts inside the guest's horizon (`<= 4` fields).
pub struct HorizonStream {
    delivered_words: Vec<u64>,
    stream_ids: Vec<u32>,
}

impl HorizonStream {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(delivered_words: Vec<u64>, stream_ids: Vec<u32>) -> Self {
        Self { delivered_words, stream_ids }
    }

    pub const fn empty() -> Self {
        Self::new(Vec::new(), Vec::new())
    }

    pub fn total(&self) -> u32 {
        self.stream_ids.len() as u32
    }

    fn delivered(&self, id: u32) -> bool {
        let word = (id as usize) / 64;
        let bit = (id as usize) % 64;
        self.delivered_words.get(word).is_some_and(|w| (w >> bit) & 1 != 0)
    }

    fn mark_delivered(&mut self, id: u32) {
        let word = (id as usize) / 64;
        let bit = (id as usize) % 64;
        if self.delivered_words.len() <= word {
            self.delivered_words.resize(word + 1, 0);
        }
        self.delivered_words[word] |= 1u64 << bit;
    }

    pub fn ensure_fact(&mut self, id: u32) {
        if !self.delivered(id) {
            self.mark_delivered(id);
            self.stream_ids.push(id);
        }
    }

    pub fn gather_horizon(
        &mut self,
        anchor: Vec3,
        visible_crafts: &[DrivenCraft],
        index: &SpatialFactIndex,
        proportions: SolarProportions,
        epoch_secs: f64,
    ) {
        self.gather_in_frame(CosmicFrame::earth(), anchor, visible_crafts, index, proportions, epoch_secs);
    }

    pub fn gather_in_frame(
        &mut self,
        recipient_frame: CosmicFrame,
        anchor: Vec3,
        visible_crafts: &[DrivenCraft],
        index: &SpatialFactIndex,
        proportions: SolarProportions,
        epoch_secs: f64,
    ) {
        if !anchor.is_finite() {
            return;
        }
        let earth_half = proportions.earth_half();
        let mut discovered = Vec::new();

        for dc in visible_crafts {
            if let Some(&bp_id) = index.fleet.0.get(&dc.identity()) {
                if !self.delivered(bp_id) {
                    discovered.push(bp_id);
                }
            }
        }
        if recipient_frame.earth_centered() {
            for (&craft_id, &(pose_idx, pos)) in &index.fleet.1 {
                if (anchor - pos).length_squared() <= HORIZON_FACT_METERS * HORIZON_FACT_METERS {
                    if let Some(&bp_id) = index.fleet.0.get(&craft_id) {
                        if !self.delivered(bp_id) {
                            discovered.push(bp_id);
                        }
                    }
                    if !self.delivered(pose_idx) {
                        discovered.push(pose_idx);
                    }
                }
            }
        }

        if recipient_frame.earth_centered() && anchor.abs().max_element() <= earth_half + HORIZON_FACT_METERS {
            let dom_n = ClimateZone::dominant(anchor).normal();
            for (face_idx, zone) in ClimateZone::all().into_iter().enumerate() {
                let n = zone.normal();
                if n.dot(dom_n) < -0.15 || (earth_half > 128.0 && anchor.dot(n) < earth_half - 180.0) {
                    continue;
                }
                let frame = zone.surface_frame(earth_half);
                let (u, v) = frame.decompose(anchor);
                let su = (u / EARTH_SECTOR_METERS).floor() as i32;
                let sv = (v / EARTH_SECTOR_METERS).floor() as i32;
                for dv in -2..=2 {
                    for du in -2..=2 {
                        if let Some(ids) = index.earth_sectors.get(&(face_idx as u8, su + du, sv + dv)) {
                            for &id in ids {
                                if !self.delivered(id) {
                                    discovered.push(id);
                                }
                            }
                        }
                    }
                }
            }
        }

        if !index.offworld.is_empty() {
            let eph = SolarEphemeris::with_frame(proportions, epoch_secs, recipient_frame);
            let snaps = eph.snapshot_all(0.0);
            let probe_d = anchor.as_dvec3();
            for (&body_id, ids) in &index.offworld {
                let near = if recipient_frame.anchored_body() == Some(body_id) {
                    true
                } else if let Some(snap) = snaps.iter().find(|s| s.id() == body_id) {
                    let pad = (snap.half_size() * 26.0).max(640.0);
                    (probe_d - snap.center()).abs().max_element() <= snap.half_size() + pad
                } else {
                    let belt = world::AsteroidBelt::new(proportions, epoch_secs);
                    let helio_probe = world::GravityBubble::new(proportions, epoch_secs)
                        .map_position(recipient_frame, anchor, CosmicFrame::Body(BodyId::SOL));
                    belt.nearest_rock(helio_probe, glam::DVec3::ZERO, snaps[0].gm(), 640.0)
                        .is_some_and(|r| r.id() == body_id)
                };
                if near {
                    for &id in ids {
                        if !self.delivered(id) {
                            discovered.push(id);
                        }
                    }
                }
            }
        }

        if !discovered.is_empty() {
            discovered.sort_unstable();
            discovered.dedup();
            for id in discovered {
                self.ensure_fact(id);
            }
        }
    }

    pub fn batch_from(&self, ack_seq: u32, store: &dyn ChronicleStore) -> Option<(u32, Vec<WorldFact>)> {
        let start = ack_seq as usize;
        if start >= self.stream_ids.len() {
            return None;
        }
        let limit = (start + 24).min(self.stream_ids.len());
        let mut scratch = Vec::with_capacity(128);
        let mut used_bytes = 0usize;
        let mut batch = Vec::with_capacity(limit - start);
        for &fact_idx in &self.stream_ids[start..limit] {
            let Some(fact) = store.fact_at(fact_idx as usize) else { continue };
            scratch.clear();
            fact.inscribe(&mut scratch);
            if !batch.is_empty() && used_bytes + scratch.len() > 1_100 {
                break;
            }
            used_bytes += scratch.len();
            batch.push(fact.clone());
        }
        (!batch.is_empty()).then_some((ack_seq, batch))
    }
}
