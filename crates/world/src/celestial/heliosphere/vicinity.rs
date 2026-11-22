use std::collections::HashMap;
use glam::Vec3;
use crate::{TerrainMesh, TranslatedWorld, WorldFact};
use super::super::alien_worlds::AlienWorld;
use super::super::belt::AsteroidBelt;
use super::super::catalog::BodyId;
use super::super::ephemeris::SolarEphemeris;
use super::super::proportions::SolarProportions;
use super::awakened::{ActiveVicinity, AwakenedBody};

/// Dual-slot LRU off-world `CubePlanet` pool (`slices 6..11` & `18..23`) with `WorldFact` persistence (`<= 4` fields).
pub struct VicinityCache {
    slots: [ActiveVicinity; 2],
    stamps: ([u64; 2], u64),
    authoritative: bool,
    chronicles: HashMap<BodyId, Vec<WorldFact>>,
}

impl VicinityCache {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(
        slots: [ActiveVicinity; 2],
        stamps: ([u64; 2], u64),
        authoritative: bool,
        chronicles: HashMap<BodyId, Vec<WorldFact>>,
    ) -> Self {
        Self { slots, stamps, authoritative, chronicles }
    }

    pub fn empty(authoritative: bool) -> Self {
        Self::new([ActiveVicinity::Dormant, ActiveVicinity::Dormant], ([0, 0], 0), authoritative, HashMap::new())
    }

    pub const fn active_ids(&self) -> [Option<BodyId>; 2] {
        [self.slots[0].active_id(), self.slots[1].active_id()]
    }

    pub const fn uploaded_ids(&self) -> [Option<BodyId>; 2] {
        [self.slots[0].uploaded_id(), self.slots[1].uploaded_id()]
    }

    pub const fn any_active(&self) -> bool {
        self.slots[0].body().is_some() || self.slots[1].body().is_some()
    }

    pub const fn slot_body(&self, idx: usize) -> Option<&AwakenedBody> {
        if idx < 2 { self.slots[idx].body() } else { None }
    }

    pub fn slot_body_mut(&mut self, idx: usize) -> Option<&mut AwakenedBody> {
        if idx < 2 { self.slots[idx].body_mut() } else { None }
    }

    pub fn body_near(&self, pos: Vec3, pad: f32) -> Option<&AwakenedBody> {
        [0usize, 1]
            .into_iter()
            .filter_map(|i| self.slots[i].body())
            .filter(|b| b.near(pos, pad))
            .min_by(|a, b| a.surface_gap(pos).total_cmp(&b.surface_gap(pos)))
    }

    pub fn find_body(&self, id: BodyId) -> Option<&AwakenedBody> {
        [0usize, 1].into_iter().filter_map(|i| self.slots[i].body()).find(|b| b.body_id() == id)
    }

    pub fn record_local_fact(&mut self, id: BodyId, fact: WorldFact) {
        self.chronicles.entry(id).or_default().push(fact);
    }

    fn mount_in_slot(&mut self, id: BodyId, center: Vec3, proportions: SolarProportions) -> usize {
        for idx in 0..2 {
            if self.slots[idx].body().is_some_and(|b| b.body_id() == id) {
                self.stamps.1 += 1;
                self.stamps.0[idx] = self.stamps.1;
                return idx;
            }
        }
        let target_idx = if self.slots[0].body().is_none() {
            0
        } else if self.slots[1].body().is_none() {
            1
        } else if self.stamps.0[0] <= self.stamps.0[1] {
            0
        } else {
            1
        };
        let (mut cube, half) = AlienWorld::build_body(id, proportions, self.authoritative);
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
        self.stamps.1 += 1;
        self.stamps.0[target_idx] = self.stamps.1;
        self.slots[target_idx] = ActiveVicinity::Awakened(AwakenedBody::new(
            (id, half),
            center,
            TranslatedWorld::new(cube, center),
            0b0011_1111,
        ));
        target_idx
    }

    fn prune_distant(&mut self, probe: Vec3) {
        for slot in &mut self.slots {
            let keep = slot.body().is_some_and(|b| {
                let pad = if b.body_id().belt_slot().is_some() {
                    96.0
                } else {
                    (b.half_size() * 26.0).clamp(144.0, 1_050.0)
                };
                b.near(probe, pad)
            });
            if !keep {
                *slot = ActiveVicinity::Dormant;
            }
        }
    }

    /// Zero-allocation stack scan awakening up to 2 simultaneously nearby bodies (e.g. Mars + Phobos).
    pub fn awaken_visible(&mut self, probe: Vec3, ephemeris: SolarEphemeris) {
        let earth_h = ephemeris.proportions().earth_half();
        if probe.abs().max_element() <= earth_h + 64.0 {
            return;
        }
        self.prune_distant(probe);
        let probe_d = probe.as_dvec3();
        let snaps = ephemeris.snapshot_all(0.0);
        let mut first: Option<(f64, BodyId, Vec3)> = None;
        let mut second: Option<(f64, BodyId, Vec3)> = None;
        for snap in snaps {
            if snap.id() == BodyId::EARTH || !snap.nature().landable_crust() {
                continue;
            }
            let awaken_d = (snap.half_size() * 22.0).clamp(96.0, 900.0);
            let surf_d = (probe_d - snap.center()).abs().max_element() - snap.half_size();
            if surf_d <= awaken_d {
                let entry = (surf_d, snap.id(), snap.center().as_vec3());
                if first.map_or(true, |f| surf_d < f.0) {
                    second = first;
                    first = Some(entry);
                } else if second.map_or(true, |s| surf_d < s.0) {
                    second = Some(entry);
                }
            }
        }
        if first.is_none() {
            let sun = snaps[0];
            let belt = AsteroidBelt::new(ephemeris.proportions(), ephemeris.epoch_secs());
            if let Some(rock) = belt.nearest_rock(probe_d, sun.center(), sun.gm(), 64.0) {
                let _ = self.mount_in_slot(rock.id(), rock.center().as_vec3(), ephemeris.proportions());
            }
            return;
        }
        for hit in [first, second].into_iter().flatten() {
            let _ = self.mount_in_slot(hit.1, hit.2, ephemeris.proportions());
        }
    }

    /// Ensures the landable off-world body or belt asteroid nearest `probe` is awakened in an LRU slot.
    pub fn ensure_near(&mut self, probe: Vec3, ephemeris: SolarEphemeris) -> Option<&AwakenedBody> {
        let earth_h = ephemeris.proportions().earth_half();
        if probe.abs().max_element() <= earth_h + 64.0 {
            return None;
        }
        let mut best_slot: Option<(usize, f32)> = None;
        for idx in 0..2 {
            if let Some(b) = self.slots[idx].body() {
                let pad = if b.body_id().belt_slot().is_some() {
                    96.0
                } else {
                    (b.half_size() * 26.0).clamp(144.0, 1_050.0)
                };
                if b.near(probe, pad) {
                    let gap = b.surface_gap(probe);
                    if best_slot.map_or(true, |(_, g)| gap < g) {
                        best_slot = Some((idx, gap));
                    }
                }
            }
        }
        if let Some((idx, _)) = best_slot {
            self.stamps.1 += 1;
            self.stamps.0[idx] = self.stamps.1;
            return self.slots[idx].body();
        }
        let probe_d = probe.as_dvec3();
        let snaps = ephemeris.snapshot_all(0.0);
        let mut best: Option<(f64, BodyId, Vec3)> = None;
        for snap in snaps {
            if snap.id() == BodyId::EARTH || !snap.nature().landable_crust() {
                continue;
            }
            let awaken_d = (snap.half_size() * 22.0).clamp(96.0, 900.0);
            let surf_d = (probe_d - snap.center()).abs().max_element() - snap.half_size();
            if surf_d <= awaken_d && best.map_or(true, |(b, _, _)| surf_d < b) {
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
        let (id, center) = candidate?;
        let slot_idx = self.mount_in_slot(id, center, ephemeris.proportions());
        self.slots[slot_idx].body()
    }

    pub fn ensure_near_mut(&mut self, probe: Vec3, ephemeris: SolarEphemeris) -> Option<&mut AwakenedBody> {
        let _ = self.ensure_near(probe, ephemeris)?;
        let idx = [0usize, 1]
            .into_iter()
            .filter(|&i| self.slots[i].body().is_some())
            .min_by(|&a, &b| {
                self.slots[a].body().unwrap().surface_gap(probe).total_cmp(&self.slots[b].body().unwrap().surface_gap(probe))
            })?;
        self.slots[idx].body_mut()
    }

    pub fn enact_on_body(&mut self, id: BodyId, local_fact: &WorldFact, proportions: SolarProportions) -> bool {
        for slot in &mut self.slots {
            if let Some(b) = slot.body_mut().filter(|b| b.body_id() == id) {
                let ok = local_fact.enact_on(b.planet_mut().inner_mut());
                if ok {
                    b.mark_all_dirty();
                    if local_fact.durable() {
                        self.chronicles.entry(id).or_default().push(local_fact.clone());
                    }
                }
                return ok;
            }
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
