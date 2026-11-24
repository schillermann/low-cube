use std::collections::HashMap;
use glam::Vec3;
use world::{ClimateZone, CompanionPresence, CosmicFrame, DrivenCraft, GravityBubble};
use crate::audience::{PulseAudience, NEARBY_SIGHT_METERS};
use crate::sector_audience::{grid_cell, same_horizon_hemisphere};

const MAX_VELOCITY_SIGHT: f32 = 2_400.0;

/// `O(N)` Hill-bubble `PulseAudience` partitioning actors by `(CosmicFrame, Sector)` + `65 km` heliocentric seam cells (`<= 4` fields).
pub struct CosmicBubbleAudience<'a> {
    companions: &'a [(CosmicFrame, CompanionPresence)],
    frame_buckets: HashMap<(CosmicFrame, i32, i32, i32), Vec<usize>>,
    macro_buckets: HashMap<(i64, i64, i64), Vec<usize>>,
    bubble: GravityBubble,
}

impl<'a> CosmicBubbleAudience<'a> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(
        companions: &'a [(CosmicFrame, CompanionPresence)],
        frame_buckets: HashMap<(CosmicFrame, i32, i32, i32), Vec<usize>>,
        macro_buckets: HashMap<(i64, i64, i64), Vec<usize>>,
        bubble: GravityBubble,
    ) -> Self {
        Self { companions, frame_buckets, macro_buckets, bubble }
    }

    /// Indexes all connected companions once per tick across local `160 m` sectors and `65,536 m` heliocentric cells.
    pub fn indexed(companions: &'a [(CosmicFrame, CompanionPresence)], bubble: GravityBubble) -> Self {
        let cap = companions.len().min(512);
        let mut frame_buckets: HashMap<(CosmicFrame, i32, i32, i32), Vec<usize>> = HashMap::with_capacity(cap);
        let mut macro_buckets: HashMap<(i64, i64, i64), Vec<usize>> = HashMap::with_capacity(cap);
        for (idx, &(frame, comp)) in companions.iter().enumerate() {
            let pos = comp.transfer(|_, kin, _, _| kin.transfer(|p, _, _, _, _| p));
            if !pos.is_finite() {
                continue;
            }
            let (cx, cy, cz) = grid_cell(pos);
            frame_buckets.entry((frame, cx, cy, cz)).or_default().push(idx);
            macro_buckets.entry(bubble.macro_cell(frame, pos)).or_default().push(idx);
        }
        Self::new(companions, frame_buckets, macro_buckets, bubble)
    }
}

impl PulseAudience for CosmicBubbleAudience<'_> {
    fn select_in_frame(
        &self,
        recipient_slot: u32,
        recipient_frame: CosmicFrame,
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
        let body_half = self.bubble.frame_body_half(recipient_frame);
        let (cx, cy, cz) = grid_cell(anchor_pos);
        for dz in -1..=1 {
            for dy in -1..=1 {
                for dx in -1..=1 {
                    let Some(indices) = self.frame_buckets.get(&(recipient_frame, cx + dx, cy + dy, cz + dz)) else {
                        continue;
                    };
                    for &idx in indices {
                        let (_, comp) = self.companions[idx];
                        if comp.same_slot(recipient_slot) || !comp.within_reach(anchor_pos, NEARBY_SIGHT_METERS) {
                            continue;
                        }
                        let peer_pos = comp.transfer(|_, kin, _, _| kin.transfer(|p, _, _, _, _| p));
                        if same_horizon_hemisphere(anchor_pos, peer_pos, body_half) {
                            out_companions.push(comp);
                        }
                    }
                }
            }
        }
        let reach_sq = f64::from(NEARBY_SIGHT_METERS) * f64::from(NEARBY_SIGHT_METERS);
        let anchor_d = anchor_pos.as_dvec3();
        let (mx, my, mz) = self.bubble.macro_cell(recipient_frame, anchor_pos);
        for dz in -1..=1 {
            for dy in -1..=1 {
                for dx in -1..=1 {
                    let Some(indices) = self.macro_buckets.get(&(mx + dx, my + dy, mz + dz)) else {
                        continue;
                    };
                    for &idx in indices {
                        let (peer_frame, comp) = self.companions[idx];
                        if peer_frame == recipient_frame || comp.same_slot(recipient_slot) {
                            continue;
                        }
                        let peer_pos = comp.transfer(|_, kin, _, _| kin.transfer(|p, _, _, _, _| p));
                        let mapped = self.bubble.map_position(peer_frame, peer_pos, recipient_frame);
                        if (mapped - anchor_d).length_squared() <= reach_sq {
                            out_companions.push(comp.with_position(mapped.as_vec3()));
                        }
                    }
                }
            }
        }
    }
}

/// `PulseAudience` decorator projecting approaching spacecraft across `CosmicFrame` boundaries in `f64` (`<= 4` fields).
pub struct CosmicCraftSight<'a, A> {
    origin: A,
    crafts: &'a [(CosmicFrame, DrivenCraft, f32)],
    macro_buckets: HashMap<(i64, i64, i64), Vec<usize>>,
    bubble: GravityBubble,
}

impl<'a, A> CosmicCraftSight<'a, A> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(
        origin: A,
        crafts: &'a [(CosmicFrame, DrivenCraft, f32)],
        macro_buckets: HashMap<(i64, i64, i64), Vec<usize>>,
        bubble: GravityBubble,
    ) -> Self {
        Self { origin, crafts, macro_buckets, bubble }
    }

    pub fn indexed(origin: A, crafts: &'a [(CosmicFrame, DrivenCraft, f32)], bubble: GravityBubble) -> Self {
        let mut macro_buckets: HashMap<(i64, i64, i64), Vec<usize>> = HashMap::with_capacity(crafts.len().min(128));
        for (idx, &(frame, dc, _)) in crafts.iter().enumerate() {
            let pos = dc.pose().origin();
            if pos.is_finite() {
                macro_buckets.entry(bubble.macro_cell(frame, pos)).or_default().push(idx);
            }
        }
        Self::new(origin, crafts, macro_buckets, bubble)
    }
}

impl<A: PulseAudience> PulseAudience for CosmicCraftSight<'_, A> {
    fn select_in_frame(
        &self,
        recipient_slot: u32,
        recipient_frame: CosmicFrame,
        anchor_pos: Vec3,
        cursor: &mut usize,
        out_companions: &mut Vec<CompanionPresence>,
        out_crafts: &mut Vec<DrivenCraft>,
    ) {
        self.origin.select_in_frame(recipient_slot, recipient_frame, anchor_pos, cursor, out_companions, out_crafts);
        if !anchor_pos.is_finite() || self.crafts.is_empty() {
            return;
        }
        let body_half = self.bubble.frame_body_half(recipient_frame);
        let on_crust = body_half > 0.0 && anchor_pos.abs().max_element() <= body_half + 48.0;
        let local_up = if on_crust {
            ClimateZone::dominant(anchor_pos).normal()
        } else {
            anchor_pos.normalize_or_zero()
        };
        let anchor_d = anchor_pos.as_dvec3();
        let (mx, my, mz) = self.bubble.macro_cell(recipient_frame, anchor_pos);
        for dz in -1..=1 {
            for dy in -1..=1 {
                for dx in -1..=1 {
                    let Some(indices) = self.macro_buckets.get(&(mx + dx, my + dy, mz + dz)) else {
                        continue;
                    };
                    for &idx in indices {
                        let (craft_frame, dc, speed_mps) = self.crafts[idx];
                        if dc.same_slot(recipient_slot) {
                            continue;
                        }
                        let mapped = self.bubble.map_position(craft_frame, dc.pose().origin(), recipient_frame);
                        let reach = f64::from((NEARBY_SIGHT_METERS + speed_mps * 3.0).clamp(NEARBY_SIGHT_METERS, MAX_VELOCITY_SIGHT));
                        let dist_sq = (mapped - anchor_d).length_squared();
                        if dist_sq > reach * reach {
                            continue;
                        }
                        let local_origin = mapped.as_vec3();
                        let local_dc = dc.with_origin(local_origin);
                        if dist_sq <= f64::from(NEARBY_SIGHT_METERS * NEARBY_SIGHT_METERS) {
                            if same_horizon_hemisphere(anchor_pos, local_origin, body_half) {
                                out_crafts.push(local_dc);
                            }
                        } else if !on_crust || (local_origin - anchor_pos).dot(local_up) > -24.0 {
                            out_crafts.push(local_dc);
                        }
                    }
                }
            }
        }
    }
}
