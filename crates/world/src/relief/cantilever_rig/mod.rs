mod mesh;
mod registry;

use std::collections::{HashMap, HashSet, VecDeque};
use crate::block::{ElevatedShape, GridCell};
use self::mesh::RigGeometry;
pub use self::registry::RigRegistry;

pub(super) const BLOCK_STEP: f32 = GridCell::SPAN;
const MIN_HOIST_STEPS: i32 = 0;
pub(super) const MAX_HOIST_STEPS: i32 = 8;
const INITIAL_HOIST_STEPS: i32 = 3;
const INITIAL_REACH_STEPS: i32 = 2;
const LAYER_MATCH_TOLERANCE: f32 = 0.30;

/// Non-block polyhedral 3D cantilever crane (`Galgen-Kranarm`) with auto-retracting telescopic gripper.
#[derive(Clone, Copy, Debug)]
pub struct CantileverRig {
    ground_top: f32,
    hoist_steps: i32,
    reach_steps: i32,
    turn: u8,
}

impl CantileverRig {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(ground_top: f32, hoist_steps: i32, reach_steps: i32, turn: u8) -> Self {
        Self { ground_top, hoist_steps, reach_steps, turn }
    }

    pub const fn deployed(ground_top: f32, turn: u8) -> Self {
        Self::new(ground_top, INITIAL_HOIST_STEPS, INITIAL_REACH_STEPS, turn & 3)
    }

    pub const fn hoisted(&self) -> bool {
        self.hoist_steps > 0
    }

    pub fn reset_arm(&mut self) {
        self.hoist_steps = INITIAL_HOIST_STEPS;
        self.reach_steps = INITIAL_REACH_STEPS;
    }

    pub const fn step_vectors(&self) -> (i32, i32) {
        match self.turn & 3 {
            0 => (1, 0),
            1 => (0, 1),
            2 => (-1, 0),
            _ => (0, -1),
        }
    }

    pub const fn hoist_elevation(&self) -> f32 {
        self.ground_top + (self.hoist_steps as f32) * BLOCK_STEP
    }

    pub const fn mast_ceiling(&self) -> f32 {
        self.hoist_elevation() + 0.65
    }

    pub const fn geometry(&self) -> RigGeometry {
        RigGeometry::new(self.ground_top, self.hoist_elevation(), self.reach_steps, self.turn)
    }

    pub const fn gripped_cell(&self, mast_u: i32, mast_v: i32) -> (i32, i32) {
        let (du, dv) = self.step_vectors();
        (mast_u + du * self.reach_steps, mast_v + dv * self.reach_steps)
    }

    pub const fn claw_slot(&self, mast_u: i32, mast_v: i32) -> (i32, i32) {
        let (du, dv) = self.step_vectors();
        let step = self.reach_steps - 1;
        (mast_u + du * step, mast_v + dv * step)
    }

    pub fn occupies_mast_layer(&self, layer_elev: f32) -> bool {
        layer_elev >= self.ground_top - 0.05 && layer_elev <= self.mast_ceiling()
    }

    pub fn constrain_mast_support(&self, support_h: f32, foot_elev: f32) -> f32 {
        if foot_elev + 0.08 <= self.ground_top {
            support_h
        } else {
            support_h.max(self.hoist_elevation() + BLOCK_STEP)
        }
    }

    pub fn mount_target(
        &self, mast_u: i32, mast_v: i32, u: i32, v: i32, blocks: &HashMap<(i32, i32), Vec<ElevatedShape>>,
    ) -> Option<(i32, i32, f32)> {
        let span = self.reach_steps.max(1);
        if !self.hoisted() || (u - mast_u).abs() > span || (v - mast_v).abs() > span {
            return None;
        }
        let ((eu, ev), hoist_z) = (self.gripped_cell(mast_u, mast_v), self.hoist_elevation());
        let occupied = blocks.get(&(eu, ev)).is_some_and(|l| l.iter().any(|b| (b.floor() - hoist_z).abs() <= LAYER_MATCH_TOLERANCE));
        (!occupied).then_some((eu, ev, hoist_z))
    }

    pub fn retract_for_placed_block(
        &mut self, mast_u: i32, mast_v: i32, placed_u: i32, placed_v: i32, placed_base: f32,
    ) -> Option<(i32, i32)> {
        if (placed_base - self.hoist_elevation()).abs() > LAYER_MATCH_TOLERANCE
            || (placed_u, placed_v) != self.claw_slot(mast_u, mast_v)
        {
            return None;
        }
        if self.reach_steps > 1 {
            self.reach_steps -= 1;
            None
        } else {
            let (du, dv) = self.step_vectors();
            Some((mast_u - du, mast_v - dv))
        }
    }

    pub fn connected_cluster(
        &self, mast_u: i32, mast_v: i32, blocks: &HashMap<(i32, i32), Vec<ElevatedShape>>,
    ) -> Vec<(i32, i32, usize)> {
        let (start_u, start_v) = self.gripped_cell(mast_u, mast_v);
        let hoist_z = self.hoist_elevation();
        let Some(start_list) = blocks.get(&(start_u, start_v)) else { return Vec::new() };
        let Some(start_idx) = start_list.iter().position(|b| (b.floor() - hoist_z).abs() <= LAYER_MATCH_TOLERANCE) else {
            return Vec::new();
        };
        let (mut visited, mut queue, mut cluster) = (HashSet::new(), VecDeque::new(), Vec::new());
        visited.insert((start_u, start_v, start_idx));
        queue.push_back((start_u, start_v, start_idx));
        while let Some((cu, cv, idx)) = queue.pop_front() {
            let z = blocks[&(cu, cv)][idx].floor();
            cluster.push((cu, cv, idx));
            for (du, dv) in [(1, 0), (-1, 0), (0, 1), (0, -1), (0, 0)] {
                let (nu, nv) = (cu + du, cv + dv);
                if (nu, nv) != (mast_u, mast_v) {
                    if let Some(nlist) = blocks.get(&(nu, nv)) {
                        for (nidx, nb) in nlist.iter().enumerate() {
                            if (nb.floor() - z).abs() <= BLOCK_STEP + 0.12 && visited.insert((nu, nv, nidx)) {
                                queue.push_back((nu, nv, nidx));
                            }
                        }
                    }
                }
            }
        }
        cluster
    }

    pub fn adjust_assembly(
        &mut self, mast_u: i32, mast_v: i32, delta_steps: i32, rotate_quarter: bool,
        blocks: &mut HashMap<(i32, i32), Vec<ElevatedShape>>, ground_fn: impl Fn(i32, i32) -> f32,
    ) -> bool {
        let cluster = self.connected_cluster(mast_u, mast_v, blocks);
        if cluster.is_empty() {
            return false;
        }
        if delta_steps != 0 {
            let target_steps = (self.hoist_steps + delta_steps).clamp(MIN_HOIST_STEPS, MAX_HOIST_STEPS);
            let mut actual_delta = target_steps - self.hoist_steps;
            for &(cu, cv, idx) in &cluster {
                let min_delta = ((ground_fn(cu, cv) - blocks[&(cu, cv)][idx].floor()) / BLOCK_STEP).ceil() as i32;
                actual_delta = actual_delta.max(min_delta);
            }
            if actual_delta == 0 {
                return false;
            }
            self.hoist_steps += actual_delta;
            let dz = (actual_delta as f32) * BLOCK_STEP;
            for &(cu, cv, idx) in &cluster {
                if let Some(list) = blocks.get_mut(&(cu, cv)) {
                    list[idx].shift_elevation(dz);
                }
            }
            for &(cu, cv, _) in &cluster {
                if let Some(list) = blocks.get_mut(&(cu, cv)) {
                    list.sort_by(|a, b| a.ceiling().total_cmp(&b.ceiling()));
                }
            }
            return true;
        }
        if rotate_quarter {
            let (piv_u, piv_v) = self.gripped_cell(mast_u, mast_v);
            let (mut extracted, mut by_cell): (Vec<_>, HashMap<(i32, i32), Vec<usize>>) = (Vec::with_capacity(cluster.len()), HashMap::new());
            for &(cu, cv, idx) in &cluster {
                by_cell.entry((cu, cv)).or_default().push(idx);
            }
            for ((cu, cv), mut indices) in by_cell {
                indices.sort_unstable_by(|a, b| b.cmp(a));
                if let Some(list) = blocks.get_mut(&(cu, cv)) {
                    for idx in indices {
                        extracted.push((cu, cv, list.remove(idx)));
                    }
                    if list.is_empty() {
                        blocks.remove(&(cu, cv));
                    }
                }
            }
            for (cu, cv, mut shape) in extracted {
                shape.rotate_quarter();
                let (du, dv) = (cu - piv_u, cv - piv_v);
                let list = blocks.entry((piv_u - dv, piv_v + du)).or_default();
                list.push(shape);
                list.sort_by(|a, b| a.ceiling().total_cmp(&b.ceiling()));
            }
            return true;
        }
        false
    }
}
