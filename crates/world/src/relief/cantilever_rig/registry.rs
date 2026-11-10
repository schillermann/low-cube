use std::collections::HashMap;
use glam::Vec3;
use crate::block::{BlockMedium, ElevatedShape, GridCell};
use crate::frame::SurfaceFrame;
use crate::MeshTarget;
use super::super::natural_strata::NaturalStrata;
use super::super::shaft_excavation::ShaftExcavation;
use super::super::sight_ray::SurfaceSightRay;
use super::super::vehicle_assembly::VehicleAssembly;
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

    pub fn hit_top(
        &self, u: i32, v: i32, ray_elev: f32, blocks: &HashMap<(i32, i32), Vec<ElevatedShape>>,
    ) -> Option<f32> {
        if let Some(rig) = self.rigs.get(&(u, v)) {
            if rig.occupies_mast_layer(ray_elev) {
                return Some(rig.mast_ceiling());
            }
        }
        self.rigs.iter().find_map(|(&(mu, mv), rig)| {
            rig.occupies_empty_boom_layer(mu, mv, u, v, ray_elev, blocks).then(|| rig.hoist_elevation() + BLOCK_STEP)
        })
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
        self.rigs.iter().find_map(|(&(mu, mv), rig)| rig.attach_elevation(mu, mv, u, v, blocks))
    }

    pub fn cluster_hoisted(&self, cluster: &[(i32, i32, usize)], blocks: &HashMap<(i32, i32), Vec<ElevatedShape>>) -> bool {
        self.rigs.iter().any(|(&(mu, mv), rig)| {
            if !rig.hoisted() {
                return false;
            }
            let gripped = rig.connected_cluster(mu, mv, blocks);
            cluster.iter().any(|c| gripped.contains(c))
        })
    }

    pub fn find_aimed_rig(&self, u: i32, v: i32, ray: SurfaceSightRay, frame: &SurfaceFrame) -> Option<(i32, i32)> {
        if self.rigs.contains_key(&(u, v)) {
            return Some((u, v));
        }
        if let Some((&mast, _)) = self.rigs.iter().find(|(&(mu, mv), r)| r.occupies_boom_cell(mu, mv, u, v)) {
            return Some(mast);
        }
        for step in 1..=60 {
            let sample = ray.sample_point((step as f32) * 0.08);
            let (su, sv) = frame.decompose(sample);
            let (ru, rv) = GridCell::at_planar(su, sv).feed_coordinates(|gu, gv| (gu, gv));
            if self.rigs.contains_key(&(ru, rv)) {
                return Some((ru, rv));
            }
            if let Some((&mast, _)) = self.rigs.iter().find(|(&(mu, mv), r)| r.occupies_boom_cell(mu, mv, ru, rv)) {
                return Some(mast);
            }
        }
        None
    }

    pub fn reconnect_seated_car(
        &mut self, mu: i32, mv: i32, ray: SurfaceSightRay, frame: &SurfaceFrame,
        blocks: &mut HashMap<(i32, i32), Vec<ElevatedShape>>,
    ) -> Option<(Vec3, Vec3)> {
        let (eu, ev) = frame.decompose(ray.sample_point(0.0));
        let (seat_u, seat_v, seat_idx) = blocks
            .iter()
            .filter_map(|(&(cu, cv), list)| {
                let idx = list.iter().rposition(|b| b.block_medium() == BlockMedium::CabinSeat)?;
                let (cx, cy) = GridCell::new(cu, cv).center();
                let dist = (cx - eu).hypot(cy - ev);
                (dist <= 5.0).then_some((cu, cv, idx, dist))
            })
            .min_by(|a, b| a.3.total_cmp(&b.3))
            .map(|(cu, cv, idx, _)| (cu, cv, idx))?;
        let cluster = VehicleAssembly::connected_cluster(seat_u, seat_v, seat_idx, blocks);
        if self.cluster_hoisted(&cluster, blocks) {
            return None;
        }
        let &(eng_u, eng_v, eng_idx) = cluster.iter().find(|&&(cu, cv, idx)| blocks[&(cu, cv)][idx].block_medium() == BlockMedium::SteeringWheel)?;
        let (seat_floor, eng_floor, heading) = (
            blocks[&(seat_u, seat_v)][seat_idx].floor(),
            blocks[&(eng_u, eng_v)][eng_idx].floor(),
            blocks[&(eng_u, eng_v)][eng_idx].block_heading().forward_vector(frame),
        );
        let (gu, gv, hoist_z) = {
            let rig = self.rigs.get_mut(&(mu, mv))?;
            rig.reset_arm();
            let (gu, gv) = rig.gripped_cell(mu, mv);
            (gu, gv, rig.hoist_elevation())
        };
        let (du, dv, dz) = (gu - eng_u, gv - eng_v, hoist_z - eng_floor);
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
        let mut moved_floors = Vec::with_capacity(extracted.len());
        for (cu, cv, mut shape) in extracted {
            shape.shift_elevation(dz);
            let (ncu, ncv, nf) = (cu + du, cv + dv, shape.floor());
            moved_floors.push((ncu, ncv, nf));
            let list = blocks.entry((ncu, ncv)).or_default();
            list.push(shape);
            list.sort_by(|a, b| a.ceiling().total_cmp(&b.ceiling()));
        }
        for (ncu, ncv, nf) in moved_floors {
            self.retract_for_block(ncu, ncv, nf);
        }
        let (cx, cy) = GridCell::new(seat_u + du, seat_v + dv).center();
        Some((frame.point_at(cx, cy, seat_floor + dz), heading))
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
