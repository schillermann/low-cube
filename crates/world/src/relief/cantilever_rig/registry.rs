use std::collections::HashMap;
use crate::block::{ElevatedShape, GridCell};
use crate::frame::SurfaceFrame;
use crate::MeshTarget;
use super::super::natural_strata::NaturalStrata;
use super::super::shaft_excavation::ShaftExcavation;
use super::{CantileverRig, BLOCK_STEP, MAX_HOIST_STEPS};

/// Collection of active polyhedral cantilever rigs across a planetary face.
pub struct RigRegistry {
    rigs: HashMap<(i32, i32), CantileverRig>,
}

impl RigRegistry {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(rigs: HashMap<(i32, i32), CantileverRig>) -> Self {
        Self { rigs }
    }

    pub fn untouched() -> Self {
        Self::new(HashMap::new())
    }

    pub fn empty(&self) -> bool {
        self.rigs.is_empty()
    }

    pub fn present_at(&self, u: i32, v: i32) -> bool {
        self.rigs.contains_key(&(u, v))
    }

    pub fn occupies_layer(&self, u: i32, v: i32, layer_elev: f32) -> bool {
        self.rigs.get(&(u, v)).is_some_and(|r| r.occupies_mast_layer(layer_elev))
    }

    pub fn constrain_support(&self, u: i32, v: i32, support_h: f32, foot_elev: f32) -> f32 {
        self.rigs.get(&(u, v)).map_or(support_h, |r| r.constrain_mast_support(support_h, foot_elev))
    }

    pub fn hit_top(&self, u: i32, v: i32, ray_elev: f32) -> Option<f32> {
        self.rigs.get(&(u, v)).and_then(|r| r.occupies_mast_layer(ray_elev).then(|| r.mast_ceiling()))
    }

    pub fn emit_at(&self, cell: GridCell, frame: &SurfaceFrame, target: &mut dyn MeshTarget) -> bool {
        let (u, v) = cell.feed_coordinates(|u, v| (u, v));
        self.rigs.get(&(u, v)).map(|rig| rig.geometry().emit(cell, frame, target)).is_some()
    }

    pub fn deploy(&mut self, u: i32, v: i32, ground_top: f32) {
        self.rigs.insert((u, v), CantileverRig::deployed(ground_top, 0));
    }

    pub fn mount_target(
        &self, u: i32, v: i32, blocks: &HashMap<(i32, i32), Vec<ElevatedShape>>,
    ) -> Option<(i32, i32, f32)> {
        self.rigs.iter().find_map(|(&(mu, mv), rig)| rig.mount_target(mu, mv, u, v, blocks))
    }

    pub fn attach_elevation(
        &self, u: i32, v: i32, blocks: &HashMap<(i32, i32), Vec<ElevatedShape>>,
    ) -> Option<f32> {
        self.rigs.iter().find_map(|(&(mu, mv), rig)| {
            (rig.hoisted()
                && (u, v) != (mu, mv)
                && rig.connected_cluster(mu, mv, blocks).iter().any(|&(cu, cv, _)| (cu - u).abs() + (cv - v).abs() <= 1))
            .then(|| rig.hoist_elevation())
        })
    }

    pub fn release_cluster(&mut self, cluster: &[(i32, i32, usize)]) {
        for (&(mu, mv), rig) in &mut self.rigs {
            let (gu, gv) = rig.gripped_cell(mu, mv);
            if cluster.iter().any(|&(cu, cv, _)| (cu, cv) == (gu, gv)) {
                rig.reset_arm();
            }
        }
    }

    pub fn lower_at(
        &mut self, strata: &NaturalStrata, shafts: &ShaftExcavation, u: i32, v: i32,
        blocks: &mut HashMap<(i32, i32), Vec<ElevatedShape>>,
    ) -> bool {
        self.adjust_at(strata, shafts, u, v, -MAX_HOIST_STEPS, false, blocks)
    }

    pub fn lower_mast_at(
        &mut self, strata: &NaturalStrata, shafts: &ShaftExcavation, u: i32, v: i32,
        blocks: &mut HashMap<(i32, i32), Vec<ElevatedShape>>,
    ) -> bool {
        self.rigs.contains_key(&(u, v)) && self.lower_at(strata, shafts, u, v, blocks)
    }

    pub fn retract_for_block(&mut self, placed_u: i32, placed_v: i32, placed_base: f32) {
        let relocated = self.rigs.iter_mut().find_map(|(&(mu, mv), rig)| {
            rig.retract_for_placed_block(mu, mv, placed_u, placed_v, placed_base).map(|new_pos| ((mu, mv), new_pos))
        });
        if let Some((old_pos, new_pos)) = relocated {
            if let Some(rig) = self.rigs.remove(&old_pos) {
                self.rigs.insert(new_pos, rig);
            }
        }
    }

    pub fn pack_up(&mut self, u: i32, v: i32, ground_top: f32, blocks: &mut HashMap<(i32, i32), Vec<ElevatedShape>>) -> bool {
        let Some(mut rig) = self.rigs.remove(&(u, v)) else { return false };
        let cluster = rig.connected_cluster(u, v, blocks);
        let _ = rig.adjust_assembly(u, v, -8, false, blocks, |_, _| ground_top);
        for (cu, cv, _) in cluster {
            if let Some(list) = blocks.get_mut(&(cu, cv)) {
                list.retain(|b| !b.block_medium().vehicle_engine());
                if list.is_empty() {
                    blocks.remove(&(cu, cv));
                }
            }
        }
        true
    }

    pub fn adjust_at(
        &mut self, strata: &NaturalStrata, shafts: &ShaftExcavation, u: i32, v: i32, delta_steps: i32, rotate_quarter: bool,
        blocks: &mut HashMap<(i32, i32), Vec<ElevatedShape>>,
    ) -> bool {
        let target_mast = if self.rigs.contains_key(&(u, v)) {
            Some((u, v))
        } else {
            self.rigs.iter().find_map(|(&(mu, mv), rig)| {
                rig.connected_cluster(mu, mv, blocks).iter().any(|&(cu, cv, _)| (cu, cv) == (u, v)).then_some((mu, mv))
            })
        };
        let Some((mu, mv)) = target_mast else { return false };
        let Some(mut rig) = self.rigs.remove(&(mu, mv)) else { return false };
        let changed = rig.adjust_assembly(mu, mv, delta_steps, rotate_quarter, blocks, |cu, cv| {
            (shafts.elevation_at(strata, cu, cv) as f32) * BLOCK_STEP
        });
        self.rigs.insert((mu, mv), rig);
        changed
    }
}
