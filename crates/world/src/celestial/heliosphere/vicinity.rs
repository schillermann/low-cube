use std::collections::HashMap;
use glam::Vec3;
use crate::{CubePlanet, TerrainMesh, TranslatedWorld, WorldFact};
use super::super::alien_worlds::AlienWorld;
use super::super::belt::AsteroidBelt;
use super::super::catalog::BodyId;
use super::super::ephemeris::SolarEphemeris;

/// Live off-world rocky planet, moon, or landed belt asteroid occupying slices `6..11`.
pub struct AwakenedBody {
    id_and_half: (BodyId, f32),
    center: Vec3,
    planet: TranslatedWorld<CubePlanet>,
    dirty_slices: u8,
}

impl AwakenedBody {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(
        id_and_half: (BodyId, f32),
        center: Vec3,
        planet: TranslatedWorld<CubePlanet>,
        dirty_slices: u8,
    ) -> Self {
        Self { id_and_half, center, planet, dirty_slices }
    }

    pub const fn body_id(&self) -> BodyId { self.id_and_half.0 }
    pub const fn half_size(&self) -> f32 { self.id_and_half.1 }
    pub const fn center(&self) -> Vec3 { self.center }
    pub const fn planet(&self) -> &TranslatedWorld<CubePlanet> { &self.planet }
    pub fn planet_mut(&mut self) -> &mut TranslatedWorld<CubePlanet> { &mut self.planet }

    pub fn near(&self, pos: Vec3, pad: f32) -> bool {
        (pos - self.center).abs().max_element() <= self.id_and_half.1 + pad
    }

    pub const fn pending_upload(&self) -> bool {
        self.dirty_slices != 0
    }

    pub fn mark_all_dirty(&mut self) {
        self.dirty_slices = 0b0011_1111;
    }

    pub fn take_dirty_slice(&mut self) -> Option<usize> {
        for bit in 0..6u8 {
            if (self.dirty_slices & (1 << bit)) != 0 {
                self.dirty_slices &= !(1 << bit);
                return Some(usize::from(bit));
            }
        }
        None
    }
}

/// Polymorphic off-world terrain slot avoiding `Option<AwakenedBody>`.
pub enum ActiveVicinity {
    Dormant,
    Awakened(AwakenedBody),
}

/// On-demand off-world `CubePlanet` cache with per-`BodyId` `WorldFact` persistence (`<= 4` fields).
pub struct VicinityCache {
    slot: ActiveVicinity,
    authoritative: bool,
    chronicles: HashMap<BodyId, Vec<WorldFact>>,
}

impl VicinityCache {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(
        slot: ActiveVicinity,
        authoritative: bool,
        chronicles: HashMap<BodyId, Vec<WorldFact>>,
    ) -> Self {
        Self { slot, authoritative, chronicles }
    }

    pub fn empty(authoritative: bool) -> Self {
        Self::new(ActiveVicinity::Dormant, authoritative, HashMap::new())
    }

    pub const fn active_id(&self) -> Option<BodyId> {
        match &self.slot {
            ActiveVicinity::Dormant => None,
            ActiveVicinity::Awakened(b) => Some(b.body_id()),
        }
    }

    pub const fn active_body(&self) -> Option<&AwakenedBody> {
        match &self.slot {
            ActiveVicinity::Dormant => None,
            ActiveVicinity::Awakened(b) => Some(b),
        }
    }

    pub fn active_body_mut(&mut self) -> Option<&mut AwakenedBody> {
        match &mut self.slot {
            ActiveVicinity::Dormant => None,
            ActiveVicinity::Awakened(b) => Some(b),
        }
    }

    pub fn record_local_fact(&mut self, id: BodyId, fact: WorldFact) {
        self.chronicles.entry(id).or_default().push(fact);
    }

    /// Ensures the landable off-world body or belt asteroid nearest `probe` (within landing range) is awakened.
    pub fn ensure_near(&mut self, probe: Vec3, ephemeris: SolarEphemeris) -> Option<&AwakenedBody> {
        let earth_h = ephemeris.proportions().earth_half();
        if probe.abs().max_element() <= earth_h + 64.0 {
            return None;
        }
        if let ActiveVicinity::Awakened(ref existing) = self.slot {
            if existing.near(probe, 24.0) {
                return self.active_body();
            }
        }

        let probe_d = probe.as_dvec3();
        let snaps = ephemeris.snapshot_all(0.0);
        let mut best: Option<(f64, BodyId, Vec3)> = None;
        for snap in snaps {
            if snap.id() == BodyId::EARTH || !snap.nature().landable_crust() {
                continue;
            }
            let surf_d = (probe_d - snap.center()).abs().max_element() - snap.half_size();
            if surf_d <= 96.0 && best.map_or(true, |(b, _, _)| surf_d < b) {
                best = Some((surf_d, snap.id(), snap.center().as_vec3()));
            }
        }
        let mut candidate = best.map(|(_, id, c)| (id, c));
        if candidate.is_none() {
            let sun = snaps[0];
            let belt = AsteroidBelt::new(ephemeris.proportions(), ephemeris.epoch_secs());
            if let Some(rock) = belt.nearest_rock(probe_d, sun.center(), sun.gm(), 64.0) {
                candidate = Some((rock.id(), rock.center().as_vec3()));
            }
        }

        let Some((id, center)) = candidate else {
            self.slot = ActiveVicinity::Dormant;
            return None;
        };
        if self.active_id() != Some(id) {
            let (mut cube, half) = AlienWorld::build_body(id, ephemeris.proportions(), self.authoritative);
            if let Some(facts) = self.chronicles.get(&id) {
                for fact in facts {
                    let _ = fact.clone().enact_on(&mut cube);
                }
                struct DrainSink;
                impl crate::MeshTarget for DrainSink {
                    fn push_quad(&mut self, _q: crate::Quad) {}
                }
                let mut sink = DrainSink;
                while cube.drain_modified_patch(&mut sink).is_some() {}
                while cube.drain_modified_slice(&mut sink).is_some() {}
            }
            let translated = TranslatedWorld::new(cube, center);
            self.slot = ActiveVicinity::Awakened(AwakenedBody::new((id, half), center, translated, 0b0011_1111));
        }
        self.active_body()
    }

    pub fn ensure_near_mut(&mut self, probe: Vec3, ephemeris: SolarEphemeris) -> Option<&mut AwakenedBody> {
        let _ = self.ensure_near(probe, ephemeris)?;
        self.active_body_mut()
    }

    /// Enacts a body-local `WorldFact` onto `id` without evicting a distinct locally awakened world.
    pub fn enact_on_body(
        &mut self,
        id: BodyId,
        local_fact: &WorldFact,
        proportions: super::super::proportions::SolarProportions,
    ) -> bool {
        if self.active_id() == Some(id) {
            let ok = self
                .active_body_mut()
                .is_some_and(|b| {
                    let applied = local_fact.enact_on(b.planet_mut().inner_mut());
                    if applied {
                        b.mark_all_dirty();
                    }
                    applied
                });
            if ok && local_fact.durable() {
                self.record_local_fact(id, local_fact.clone());
            }
            return ok;
        }
        let (mut cube, _) = AlienWorld::build_body(id, proportions, self.authoritative);
        if let Some(facts) = self.chronicles.get(&id) {
            for fact in facts {
                let _ = fact.enact_on(&mut cube);
            }
        }
        let ok = local_fact.enact_on(&mut cube);
        if ok && local_fact.durable() {
            self.record_local_fact(id, local_fact.clone());
        }
        ok
    }
}
