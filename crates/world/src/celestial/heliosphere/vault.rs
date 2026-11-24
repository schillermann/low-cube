use std::collections::HashMap;
use crate::{CubePlanet, MeshTarget, Quad, TerrainMesh, WorldFact};
use super::super::alien_worlds::AlienWorld;
use super::super::catalog::BodyId;
use super::super::proportions::SolarProportions;

const MAX_WARM_BODIES: usize = 32;

struct DrainSink;

impl MeshTarget for DrainSink {
    fn push_quad(&mut self, _q: Quad) {}
}

/// Persistent off-world chronicle and warm authoritative `CubePlanet` pool avoiding 2-slot server thrashing (`<= 4` fields).
pub struct OffworldVault {
    authoritative: bool,
    chronicles: HashMap<BodyId, Vec<WorldFact>>,
    warm: HashMap<BodyId, (CubePlanet, f32)>,
}

impl OffworldVault {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(
        authoritative: bool,
        chronicles: HashMap<BodyId, Vec<WorldFact>>,
        warm: HashMap<BodyId, (CubePlanet, f32)>,
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

    pub fn record_fact(&mut self, id: BodyId, fact: WorldFact) {
        self.chronicles.entry(id).or_default().push(fact);
    }

    pub fn shelve(&mut self, id: BodyId, cube: CubePlanet, half: f32) {
        if !self.authoritative {
            return;
        }
        if self.warm.len() >= MAX_WARM_BODIES && !self.warm.contains_key(&id) {
            if let Some(&victim) = self.warm.keys().next() {
                self.warm.remove(&victim);
            }
        }
        self.warm.insert(id, (cube, half));
    }

    pub fn checkout(&mut self, id: BodyId, proportions: SolarProportions) -> (CubePlanet, f32) {
        if let Some(existing) = self.warm.remove(&id) {
            return existing;
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
}
