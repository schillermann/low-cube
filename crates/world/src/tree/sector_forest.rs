use std::collections::HashMap;
use glam::Vec3;
use crate::block::GridCell;
use crate::frame::SurfaceFrame;
use crate::topology::ClimateZone;
use crate::{Face, MeshTarget, WorldSeed};
use super::arboreal::Arboreal;
use super::depository::WoodlandDepository;
use super::floor_litter::FloorLitter;
use super::sector_grove::SectorGrove;
use super::timber_yield::{CleaveOutcome, SurfaceFind};
use super::zoned_forest::ZonedForest;

const MAX_GROVES: usize = 1_024;
const RING_KEEP_SECTORS: i32 = 32;

/// Polymorphic canopy seed mode avoiding nullable `Option<ClimateZone>` state.
#[derive(Clone, Copy)]
pub enum CanopySeed {
    Procedural(ClimateZone, f32, WorldSeed),
    Custom(f32),
}

impl CanopySeed {
    pub const fn continental(self) -> bool {
        self.half_size() > 128.0
    }

    pub const fn half_size(self) -> f32 {
        match self {
            Self::Procedural(_, h, _) | Self::Custom(h) => h,
        }
    }
}

/// Lazy sector-partitioned woodland canopy materializing trees only inside queried `4m × 4m` sectors (`<= 4` fields).
pub struct SectorForest {
    seed: CanopySeed,
    groves: HashMap<(i32, i32), SectorGrove>,
    active: Vec<(i32, i32)>,
    settled: bool,
}

impl SectorForest {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(
        seed: CanopySeed,
        groves: HashMap<(i32, i32), SectorGrove>,
        active: Vec<(i32, i32)>,
        settled: bool,
    ) -> Self {
        Self { seed, groves, active, settled }
    }

    pub fn lazy_zone(zone: ClimateZone, half_size: f32) -> Self {
        Self::seeded_zone(zone, half_size, WorldSeed::origin())
    }

    pub fn seeded_zone(zone: ClimateZone, half_size: f32, seed: WorldSeed) -> Self {
        Self::new(CanopySeed::Procedural(zone, half_size, seed), HashMap::new(), Vec::new(), false)
    }

    pub fn with_specimens(half_size: f32, trees: Vec<Box<dyn Arboreal>>) -> Self {
        let mut groves = HashMap::new();
        groves.insert((0, 0), SectorGrove::populated(trees));
        Self::new(CanopySeed::Custom(half_size), groves, Vec::new(), false)
    }

    pub const fn continental(&self) -> bool { self.seed.continental() }
    pub fn materialized_grove_count(&self) -> usize { self.groves.len() }
    pub fn active_simulation(&self) -> bool { self.settled || !self.active.is_empty() }

    pub fn sector_uv(u: f32, v: f32) -> (i32, i32) {
        let (cu, cv) = GridCell::at_planar(u, v).feed_coordinates(|gu, gv| (gu, gv));
        (cu.div_euclid(8), cv.div_euclid(8))
    }

    pub fn sector_at(frame: &SurfaceFrame, point: Vec3) -> (i32, i32) {
        let (u, v) = frame.decompose(point);
        Self::sector_uv(u, v)
    }

    pub fn sector_revision(&self, gsu: i32, gsv: i32) -> u32 {
        self.groves.get(&(gsu, gsv)).map_or(0, SectorGrove::revision)
    }

    fn ensure_grove(&mut self, key: (i32, i32), origin: &dyn Face, frame: SurfaceFrame) {
        if self.groves.contains_key(&key) {
            return;
        }
        let CanopySeed::Procedural(zone, half, seed) = self.seed else {
            return;
        };
        if self.groves.len() >= MAX_GROVES {
            self.groves.retain(|&(su, sv), g| {
                !g.pristine() || (su - key.0).abs().max((sv - key.1).abs()) <= RING_KEEP_SECTORS
            });
        }
        let planted = ZonedForest::seeded(zone, seed).populate_sector(origin, frame, half, key);
        self.groves.insert(key, SectorGrove::populated(planted));
    }

    pub fn ensure_neighborhood(&mut self, center: (i32, i32), radius: i32, origin: &dyn Face, frame: SurfaceFrame) {
        for dv in -radius..=radius {
            for du in -radius..=radius {
                self.ensure_grove((center.0 + du, center.1 + dv), origin, frame);
            }
        }
    }

    pub fn emit_sector(&mut self, key: (i32, i32), origin: &dyn Face, frame: SurfaceFrame, target: &mut dyn MeshTarget) {
        self.ensure_grove(key, origin, frame);
        if let Some(grove) = self.groves.get(&key) {
            grove.emit(target);
        }
    }

    pub fn emit_compact_face(&mut self, origin: &dyn Face, frame: SurfaceFrame, target: &mut dyn MeshTarget) {
        if self.seed.continental() {
            return;
        }
        let half_sec = ((self.seed.half_size() * 2.0).ceil() as i32).div_euclid(8) + 1;
        if let CanopySeed::Procedural(_, _, _) = self.seed {
            for gsu in -half_sec..=half_sec {
                for gsv in -half_sec..=half_sec {
                    self.ensure_grove((gsu, gsv), origin, frame);
                }
            }
        }
        let mut keys: Vec<(i32, i32)> = self.groves.keys().copied().collect();
        keys.sort_unstable();
        for key in keys {
            if let Some(grove) = self.groves.get(&key) {
                grove.emit(target);
            }
        }
    }

    pub fn constrain_near(&self, pos: Vec3, u: f32, v: f32, dist: f32, origin: &dyn Face, frame: &SurfaceFrame) -> Vec3 {
        let c = Self::sector_uv(u, v);
        let mut cur = pos;
        for dv in -1..=1 {
            for du in -1..=1 {
                let key = (c.0 + du, c.1 + dv);
                if let Some(g) = self.groves.get(&key) {
                    cur = g.constrain(cur, u, v, frame, dist);
                } else if let CanopySeed::Procedural(zone, half, seed) = self.seed {
                    cur = ZonedForest::seeded(zone, seed)
                        .constrain_untouched_sector(cur, (u, v, dist), origin, frame, half, key);
                }
            }
        }
        cur
    }

    pub fn canopy_drag_near(&self, u: f32, v: f32, elev: f32, base: f32, origin: &dyn Face, frame: &SurfaceFrame) -> f32 {
        let c = Self::sector_uv(u, v);
        let mut peak = base;
        for dv in -1..=1 {
            for du in -1..=1 {
                let key = (c.0 + du, c.1 + dv);
                if let Some(g) = self.groves.get(&key) {
                    peak = g.max_drag(u, v, elev, peak);
                } else if let CanopySeed::Procedural(zone, half, seed) = self.seed {
                    peak = ZonedForest::seeded(zone, seed)
                        .canopy_drag_untouched_sector((u, v, elev), peak, origin, frame, half, key);
                }
            }
        }
        peak
    }

    pub fn strike_near(&mut self, eye: Vec3, direction: Vec3, origin: &dyn Face, frame: &SurfaceFrame) -> CleaveOutcome {
        let c = Self::sector_at(frame, eye);
        self.ensure_neighborhood(c, 1, origin, *frame);
        let mut dep = WoodlandDepository::new();
        for dv in -1..=1 {
            for du in -1..=1 {
                let key = (c.0 + du, c.1 + dv);
                if let Some(g) = self.groves.get_mut(&key) {
                    let out = g.strike(eye, direction, &mut dep);
                    if out.struck() {
                        if !self.active.contains(&key) {
                            self.active.push(key);
                        }
                        return out;
                    }
                }
            }
        }
        CleaveOutcome::Missed
    }

    pub fn forage_near(&mut self, eye: Vec3, direction: Vec3, origin: &dyn Face, frame: &SurfaceFrame) -> SurfaceFind {
        let c = Self::sector_at(frame, eye);
        self.ensure_neighborhood(c, 1, origin, *frame);
        for dv in -1..=1 {
            for du in -1..=1 {
                if let Some(g) = self.groves.get_mut(&(c.0 + du, c.1 + dv)) {
                    let find = g.forage(eye, direction);
                    if find.gathered() {
                        return find;
                    }
                }
            }
        }
        SurfaceFind::EmptyHanded
    }

    pub fn append_litter(&mut self, (u, v, elev, yaw): (f32, f32, f32, f32), find: SurfaceFind, origin: &dyn Face, frame: SurfaceFrame) {
        let key = Self::sector_uv(u, v);
        self.ensure_grove(key, origin, frame);
        let specimen = FloorLitter::new(frame, u, v, elev, find, yaw);
        self.groves.entry(key).or_insert_with(|| SectorGrove::populated(Vec::new())).append_litter(specimen);
    }

    pub fn settle_near(&mut self, center_pt: Vec3, origin: &dyn Face, frame: &SurfaceFrame) {
        let c = Self::sector_at(frame, center_pt);
        self.ensure_neighborhood(c, 1, origin, *frame);
        for dv in -1..=1 {
            for du in -1..=1 {
                if let Some(g) = self.groves.get_mut(&(c.0 + du, c.1 + dv)) {
                    if g.settle_onto(origin, frame) {
                        self.settled = true;
                    }
                }
            }
        }
    }

    pub fn step_active(&mut self, dt: f32) -> bool {
        let mut moved = self.settled;
        self.settled = false;
        let mut remaining = Vec::with_capacity(self.active.len());
        for &key in &self.active {
            if let Some(g) = self.groves.get_mut(&key) {
                if g.step_timber(dt) {
                    moved = true;
                }
                if g.animating() {
                    remaining.push(key);
                }
            }
        }
        self.active = remaining;
        moved
    }
}
