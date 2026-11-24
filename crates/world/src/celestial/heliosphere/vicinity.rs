use std::collections::HashMap;
use glam::Vec3;
use crate::{TranslatedWorld, WorldFact};
use super::super::catalog::BodyId;
use super::super::ephemeris::SolarEphemeris;
use super::super::proportions::SolarProportions;
use super::awakened::{ActiveVicinity, AwakenedBody};
use super::scanner::BodyProximity;
use super::vault::OffworldVault;

/// Dual-slot client GPU window (`slices 6..11` & `18..23`) backed by warm authoritative `OffworldVault` (`<= 4` fields).
pub struct VicinityCache {
    slots: [ActiveVicinity; 2],
    stamps: ([u64; 2], u64),
    vault: OffworldVault,
}

impl VicinityCache {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn with_vault(
        slots: [ActiveVicinity; 2],
        stamps: ([u64; 2], u64),
        vault: OffworldVault,
    ) -> Self {
        Self { slots, stamps, vault }
    }

    pub fn new(
        slots: [ActiveVicinity; 2],
        stamps: ([u64; 2], u64),
        authoritative: bool,
        chronicles: HashMap<BodyId, Vec<WorldFact>>,
    ) -> Self {
        Self::with_vault(slots, stamps, OffworldVault::new(authoritative, chronicles, HashMap::new()))
    }

    pub fn empty(authoritative: bool) -> Self {
        Self::with_vault([ActiveVicinity::Dormant, ActiveVicinity::Dormant], ([0, 0], 0), OffworldVault::empty(authoritative))
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

    pub fn warm_body_count(&self) -> usize {
        self.vault.warm_body_count()
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
        self.vault.record_fact(id, fact);
    }

    fn evict_slot(&mut self, idx: usize) {
        let prev = std::mem::replace(&mut self.slots[idx], ActiveVicinity::Dormant);
        if let ActiveVicinity::Awakened(body) = prev {
            let (id, half, cube) = body.into_parts();
            self.vault.shelve(id, cube, half);
        }
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
        self.evict_slot(target_idx);
        let (cube, half) = self.vault.checkout(id, proportions);
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
        for idx in 0..2 {
            let keep = self.slots[idx].body().is_some_and(|b| {
                let pad = if b.body_id().belt_slot().is_some() {
                    96.0
                } else {
                    (b.half_size() * 26.0).clamp(144.0, 1_050.0)
                };
                b.near(probe, pad)
            });
            if !keep {
                self.evict_slot(idx);
            }
        }
    }

    /// Zero-allocation stack scan awakening up to 2 simultaneously nearby bodies (e.g. Mars + Phobos).
    pub fn awaken_visible(&mut self, probe: Vec3, ephemeris: SolarEphemeris) {
        let earth_h = ephemeris.proportions().earth_half();
        if ephemeris.frame().earth_centered() && probe.abs().max_element() <= earth_h + 64.0 {
            return;
        }
        self.prune_distant(probe);
        for hit in BodyProximity::new(probe, ephemeris).top_two().into_iter().flatten() {
            let _ = self.mount_in_slot(hit.0, hit.1, ephemeris.proportions());
        }
    }

    /// Ensures the landable off-world body or belt asteroid nearest `probe` is awakened in an LRU slot.
    pub fn ensure_near(&mut self, probe: Vec3, ephemeris: SolarEphemeris) -> Option<&AwakenedBody> {
        let earth_h = ephemeris.proportions().earth_half();
        if ephemeris.frame().earth_centered() && probe.abs().max_element() <= earth_h + 64.0 {
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
        let (id, center) = BodyProximity::new(probe, ephemeris).top_two()[0]?;
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

    pub fn harvest_on_body(
        &mut self,
        id: BodyId,
        eye: Vec3,
        direction: Vec3,
        proportions: SolarProportions,
    ) -> Option<crate::BlockMedium> {
        use crate::WorldEarthworks as _;
        for slot in &mut self.slots {
            if let Some(b) = slot.body_mut().filter(|b| b.body_id() == id) {
                let medium = b.planet_mut().inner_mut().harvest_surface(eye, direction);
                if medium.is_some() {
                    b.mark_all_dirty();
                    self.vault.record_fact(id, WorldFact::Harvested { eye, direction });
                }
                return medium;
            }
        }
        self.vault.harvest_inactive(id, eye, direction, proportions)
    }

    pub fn enact_on_body(&mut self, id: BodyId, local_fact: &WorldFact, proportions: SolarProportions) -> bool {
        for slot in &mut self.slots {
            if let Some(b) = slot.body_mut().filter(|b| b.body_id() == id) {
                let ok = local_fact.enact_on(b.planet_mut().inner_mut());
                if ok {
                    b.mark_all_dirty();
                    if local_fact.durable() {
                        self.vault.record_fact(id, local_fact.clone());
                    }
                }
                return ok;
            }
        }
        self.vault.enact_inactive(id, local_fact, proportions)
    }

    pub fn recenter_active(&mut self, ephemeris: SolarEphemeris) {
        if !self.any_active() {
            return;
        }
        let snaps = ephemeris.snapshot_all(0.0);
        let frame = ephemeris.frame();
        for slot in &mut self.slots {
            let Some(b) = slot.body_mut() else { continue };
            let id = b.body_id();
            if frame.anchored_body() == Some(id) {
                b.relocate(Vec3::ZERO);
            } else if let Some(snap) = snaps.iter().find(|s| s.id() == id) {
                b.relocate(snap.center().as_vec3());
            }
        }
    }

    pub fn capture_body_sector(
        &mut self,
        id: BodyId,
        (face, gsu, gsv): (u8, i32, i32),
        proportions: SolarProportions,
    ) -> crate::SectorSnapshot {
        use crate::WorldEarthworks as _;
        for slot in &self.slots {
            if let Some(b) = slot.body().filter(|b| b.body_id() == id) {
                return b.planet().capture_sector_snapshot(face, gsu, gsv);
            }
        }
        self.vault.capture_inactive_sector(id, face, gsu, gsv, proportions)
    }

    pub fn collect_offworld_snapshots(
        &mut self,
        proportions: SolarProportions,
        out: &mut Vec<(BodyId, crate::SectorSnapshot)>,
    ) {
        use crate::WorldEarthworks as _;
        let mut local = Vec::new();
        for slot in &self.slots {
            if let Some(b) = slot.body() {
                local.clear();
                b.planet().collect_sector_snapshots(&mut local);
                for snap in local.drain(..) {
                    out.push((b.body_id(), snap));
                }
            }
        }
        self.vault.collect_all_snapshots(proportions, self.active_ids(), out);
    }

    pub fn step_active_bodies(&mut self, dt: f32, active: &[BodyId]) -> bool {
        let mut changed = false;
        for slot in &mut self.slots {
            if let Some(b) = slot.body_mut() {
                let mask = if active.contains(&b.body_id()) { 0b0011_1111 } else { 0 };
                changed |= crate::PlanetaryWorld::step_active_zones(b.planet_mut(), dt, mask, active);
            }
        }
        changed | self.vault.step_warm_bodies(dt, active)
    }
}
