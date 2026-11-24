use std::collections::HashMap;
use glam::Vec3;
use world::{ClimateZone, CompanionPresence, DrivenCraft};
use crate::audience::{PulseAudience, NEARBY_SIGHT_METERS};

const CELL_SPAN: f32 = NEARBY_SIGHT_METERS;
const MAX_VELOCITY_SIGHT: f32 = 2_400.0;

fn grid_cell(pos: Vec3) -> (i32, i32, i32) {
    (
        (pos.x / CELL_SPAN).floor() as i32,
        (pos.y / CELL_SPAN).floor() as i32,
        (pos.z / CELL_SPAN).floor() as i32,
    )
}

fn same_horizon_hemisphere(anchor: Vec3, peer: Vec3, earth_half: f32) -> bool {
    let crust_bound = earth_half + 48.0;
    if anchor.abs().max_element() > crust_bound || peer.abs().max_element() > crust_bound {
        return true;
    }
    let anchor_n = ClimateZone::dominant(anchor).normal();
    let peer_n = ClimateZone::dominant(peer).normal();
    if anchor_n.dot(peer_n) < -0.15 {
        return false;
    }
    if earth_half > 128.0 && anchor_n.dot(peer_n) < 0.5 {
        return anchor.dot(peer_n) >= earth_half - 180.0;
    }
    true
}

/// `O(N)` spatial-bucketed `PulseAudience` isolating `ClimateZone` continents and distant sectors (`<= 4` fields).
pub struct SectorGridAudience<'a> {
    companions: &'a [CompanionPresence],
    buckets: HashMap<(i32, i32, i32), Vec<usize>>,
    radius_meters: f32,
    earth_half: f32,
}

impl<'a> SectorGridAudience<'a> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn with_buckets(
        companions: &'a [CompanionPresence],
        buckets: HashMap<(i32, i32, i32), Vec<usize>>,
        radius_meters: f32,
        earth_half: f32,
    ) -> Self {
        Self { companions, buckets, radius_meters, earth_half }
    }

    /// Builds a `160 m` 3D sector index once per tick in `O(N)`.
    pub fn indexed(companions: &'a [CompanionPresence], radius_meters: f32, earth_half: f32) -> Self {
        let mut buckets: HashMap<(i32, i32, i32), Vec<usize>> = HashMap::with_capacity(companions.len().min(512));
        for (idx, comp) in companions.iter().enumerate() {
            let pos = comp.transfer(|_, kin, _, _| kin.transfer(|p, _, _, _, _| p));
            if pos.is_finite() {
                buckets.entry(grid_cell(pos)).or_default().push(idx);
            }
        }
        Self::with_buckets(companions, buckets, radius_meters, earth_half)
    }
}

impl PulseAudience for SectorGridAudience<'_> {
    fn select_for(
        &self,
        recipient_slot: u32,
        anchor_pos: Vec3,
        _cursor: &mut usize,
        out_companions: &mut Vec<CompanionPresence>,
        out_crafts: &mut Vec<DrivenCraft>,
    ) {
        out_companions.clear();
        out_crafts.clear();
        if !anchor_pos.is_finite() {
            return;
        }
        let (cx, cy, cz) = grid_cell(anchor_pos);
        for dz in -1..=1 {
            for dy in -1..=1 {
                for dx in -1..=1 {
                    let Some(indices) = self.buckets.get(&(cx + dx, cy + dy, cz + dz)) else {
                        continue;
                    };
                    for &idx in indices {
                        let comp = self.companions[idx];
                        if comp.same_slot(recipient_slot) || !comp.within_reach(anchor_pos, self.radius_meters) {
                            continue;
                        }
                        let peer_pos = comp.transfer(|_, kin, _, _| kin.transfer(|p, _, _, _, _| p));
                        if same_horizon_hemisphere(anchor_pos, peer_pos, self.earth_half) {
                            out_companions.push(comp);
                        }
                    }
                }
            }
        }
    }
}

/// `PulseAudience` decorator scaling driven spacecraft sight distance by approach velocity within the local sky dome.
pub struct VelocityScaledSight<'a, A> {
    origin: A,
    crafts: &'a [(DrivenCraft, f32)],
    earth_half: f32,
}

impl<'a, A> VelocityScaledSight<'a, A> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(origin: A, crafts: &'a [(DrivenCraft, f32)], earth_half: f32) -> Self {
        Self { origin, crafts, earth_half }
    }
}

impl<A: PulseAudience> PulseAudience for VelocityScaledSight<'_, A> {
    fn select_for(
        &self,
        recipient_slot: u32,
        anchor_pos: Vec3,
        cursor: &mut usize,
        out_companions: &mut Vec<CompanionPresence>,
        out_crafts: &mut Vec<DrivenCraft>,
    ) {
        self.origin.select_for(recipient_slot, anchor_pos, cursor, out_companions, out_crafts);
        let on_crust = anchor_pos.abs().max_element() <= self.earth_half + 48.0;
        let local_up = if on_crust {
            ClimateZone::dominant(anchor_pos).normal()
        } else {
            anchor_pos.normalize_or_zero()
        };
        for &(dc, speed_mps) in self.crafts {
            if dc.same_slot(recipient_slot) {
                continue;
            }
            let craft_pos = dc.pose().origin();
            let reach = (NEARBY_SIGHT_METERS + speed_mps * 3.0).clamp(NEARBY_SIGHT_METERS, MAX_VELOCITY_SIGHT);
            if !dc.within_reach(anchor_pos, reach) {
                continue;
            }
            if dc.within_reach(anchor_pos, NEARBY_SIGHT_METERS) {
                if same_horizon_hemisphere(anchor_pos, craft_pos, self.earth_half) {
                    out_crafts.push(dc);
                }
            } else if !on_crust || (craft_pos - anchor_pos).dot(local_up) > -24.0 {
                out_crafts.push(dc);
            }
        }
    }
}
