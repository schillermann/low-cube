use std::collections::HashMap;
use crate::{MeshTarget, Quad, QuadSphere, TerrainMesh, WorldFact};
use super::super::alien_worlds::AlienWorld;
use super::super::catalog::BodyId;
use super::super::proportions::SolarProportions;

const MAX_WARM_BODIES: usize = 32;

/// Idle duration in seconds after which an uninhabited off-world `QuadSphere` folds into cold `SectorSnapshot`s and frees RAM.
pub const OFFWORLD_IDLE_TTL_SECS: f32 = 180.0;

struct DrainSink;

impl MeshTarget for DrainSink {
    fn push_quad(&mut self, _q: Quad) {}
}

/// Persistent off-world chronicle and warm authoritative `QuadSphere` pool with Idle-TTL cold eviction (`<= 4` fields).
pub struct OffworldVault {
    authoritative: bool,
    chronicles: HashMap<BodyId, Vec<WorldFact>>,
    warm: HashMap<BodyId, (QuadSphere, f32, f32)>,
}

impl OffworldVault {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(
        authoritative: bool,
        chronicles: HashMap<BodyId, Vec<WorldFact>>,
        warm: HashMap<BodyId, (QuadSphere, f32, f32)>,
    ) -> Self {
        Self { authoritative, chronicles, warm }
    }

    pub fn empty(authoritative: bool) -> Self {
        Self::new(authoritative, HashMap::new(), HashMap::new())
    }

    pub const fn authoritative(&self) -> bool {
        self.authoritative
    }

    pub fn warm_body_count(&self) -> usize {
        self.warm.len()
    }

    pub fn cold_chronicle_count(&self, id: BodyId) -> usize {
        self.chronicles.get(&id).map_or(0, Vec::len)
    }

    pub fn record_fact(&mut self, id: BodyId, fact: WorldFact) {
        self.chronicles.entry(id).or_default().push(fact);
    }

    /// Folds `id`'s modified sectors into compact raycast-free `SectorSnapshot`s and drops `QuadSphere` from RAM.
    pub fn evict_to_cold(&mut self, id: BodyId) -> bool {
        use crate::WorldEarthworks as _;
        let Some((cube, _, _)) = self.warm.remove(&id) else {
            return false;
        };
        let mut snaps = Vec::new();
        cube.collect_sector_snapshots(&mut snaps);
        if !snaps.is_empty() || self.chronicles.contains_key(&id) {
            let list = self.chronicles.entry(id).or_default();
            list.retain(|f| !f.snapshot_foldable());
            list.extend(snaps.into_iter().map(|snapshot| WorldFact::SectorSnapshot { snapshot }));
            if list.is_empty() {
                self.chronicles.remove(&id);
            }
        }
        true
    }

    pub fn shelve(&mut self, id: BodyId, cube: QuadSphere, half: f32) {
        if !self.authoritative {
            return;
        }
        if self.warm.len() >= MAX_WARM_BODIES && !self.warm.contains_key(&id) {
            if let Some(victim) = self
                .warm
                .iter()
                .max_by(|a, b| (a.1).2.total_cmp(&(b.1).2))
                .map(|(&k, _)| k)
            {
                let _ = self.evict_to_cold(victim);
            }
        }
        self.warm.insert(id, (cube, half, 0.0));
    }

    pub fn checkout(&mut self, id: BodyId, proportions: SolarProportions) -> (QuadSphere, f32) {
        if let Some((cube, half, _)) = self.warm.remove(&id) {
            return (cube, half);
        }
        let (mut cube, half) = AlienWorld::build_body(id, proportions, self.authoritative);
        if let Some(facts) = self.chronicles.get(&id) {
            for fact in facts {
                let _ = fact.clone().enact_on(&mut cube);
            }
            let mut sink = DrainSink;
            while cube.drain_modified_patch(&mut sink).is_some() {}
            while cube.drain_modified_slice(&mut sink).is_some() {}
        }
        (cube, half)
    }

    pub fn harvest_inactive(
        &mut self,
        id: BodyId,
        eye: glam::Vec3,
        direction: glam::Vec3,
        proportions: SolarProportions,
    ) -> Option<crate::BlockMedium> {
        use crate::WorldEarthworks as _;
        let (mut cube, half) = self.checkout(id, proportions);
        let medium = cube.harvest_surface(eye, direction);
        if medium.is_some() {
            self.record_fact(id, WorldFact::Harvested { eye, direction });
        }
        self.shelve(id, cube, half);
        medium
    }

    pub fn enact_inactive(&mut self, id: BodyId, local_fact: &WorldFact, proportions: SolarProportions) -> bool {
        let (mut cube, half) = self.checkout(id, proportions);
        let ok = local_fact.enact_on(&mut cube);
        if ok && local_fact.durable() {
            self.record_fact(id, local_fact.clone());
        }
        self.shelve(id, cube, half);
        ok
    }

    pub fn capture_inactive_sector(
        &mut self,
        id: BodyId,
        face: u8,
        gsu: i32,
        gsv: i32,
        proportions: SolarProportions,
    ) -> crate::SectorSnapshot {
        use crate::WorldEarthworks as _;
        let (cube, half) = self.checkout(id, proportions);
        let snap = cube.capture_sector_snapshot(face, gsu, gsv);
        self.shelve(id, cube, half);
        snap
    }

    pub fn collect_all_snapshots(
        &mut self,
        proportions: SolarProportions,
        skip: [Option<BodyId>; 2],
        out: &mut Vec<(BodyId, crate::SectorSnapshot)>,
    ) {
        use crate::WorldEarthworks as _;
        let mut ids: Vec<BodyId> = self.chronicles.keys().copied().chain(self.warm.keys().copied()).collect();
        ids.sort_unstable_by_key(|b| b.raw_code());
        ids.dedup();
        let mut local = Vec::new();
        for id in ids {
            if skip.contains(&Some(id)) {
                continue;
            }
            if !self.warm.contains_key(&id) {
                if let Some(facts) = self.chronicles.get(&id) {
                    if !facts.is_empty() && facts.iter().all(|f| matches!(f, WorldFact::SectorSnapshot { .. })) {
                        for fact in facts {
                            if let WorldFact::SectorSnapshot { snapshot } = fact {
                                out.push((id, snapshot.clone()));
                            }
                        }
                        continue;
                    }
                }
            }
            let (cube, half) = self.checkout(id, proportions);
            local.clear();
            cube.collect_sector_snapshots(&mut local);
            for snap in local.drain(..) {
                out.push((id, snap));
            }
            self.shelve(id, cube, half);
        }
    }

    pub fn collect_medium_sites(
        &self,
        medium: crate::BlockMedium,
        proportions: SolarProportions,
        skip: [Option<BodyId>; 2],
        out: &mut Vec<(BodyId, glam::Vec3, glam::Vec3)>,
    ) {
        use crate::PlanetaryWorld as _;
        let mut local = Vec::new();
        for (&id, (cube, _, _)) in &self.warm {
            if !skip.contains(&Some(id)) {
                local.clear();
                cube.collect_medium_sites(medium, &mut local);
                out.extend(local.drain(..).map(|(p, n)| (id, p, n)));
            }
        }
        for (&id, facts) in &self.chronicles {
            if !skip.contains(&Some(id)) && !self.warm.contains_key(&id) {
                let half = (AlienWorld::body_blocks(id, proportions) as f32) * 0.5;
                local.clear();
                for fact in facts {
                    if let WorldFact::SectorSnapshot { snapshot } = fact {
                        snapshot.collect_medium_sites(medium, half, &mut local);
                    }
                }
                out.extend(local.drain(..).map(|(p, n)| (id, p, n)));
            }
        }
    }

    pub fn step_warm_bodies(&mut self, dt: f32, active_bodies: &[BodyId]) -> bool {
        let mut stale = Vec::new();
        for (&id, (_, _, idle_secs)) in &mut self.warm {
            if active_bodies.contains(&id) {
                *idle_secs = 0.0;
            } else {
                *idle_secs += dt.max(0.0);
                if *idle_secs >= OFFWORLD_IDLE_TTL_SECS {
                    stale.push(id);
                }
            }
        }
        for id in stale {
            let _ = self.evict_to_cold(id);
        }
        let mut targets: Vec<&mut QuadSphere> = self
            .warm
            .iter_mut()
            .filter_map(|(&id, (cube, _, _))| active_bodies.contains(&id).then_some(cube))
            .collect();
        if targets.is_empty() {
            return false;
        }
        if targets.len() == 1 {
            return crate::PlanetaryWorld::step_surface(targets[0], dt);
        }
        std::thread::scope(|scope| {
            let handles: Vec<_> = targets
                .into_iter()
                .map(|cube| scope.spawn(move || crate::PlanetaryWorld::step_surface(cube, dt)))
                .collect();
            handles.into_iter().fold(false, |acc, h| acc | h.join().expect("OffworldVault shard panicked"))
        })
    }
}
