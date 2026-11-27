use std::collections::HashMap;
use crate::block::GridCell;
use crate::frame::SurfaceFrame;
use super::super::craft_cluster::{CraftCluster, ParkedFleet};
use super::super::craft_entity::CraftBlueprint;
use super::super::molded_structure::MoldedStructure;
use super::super::natural_strata::NaturalStrata;
use super::super::shaft_excavation::ShaftExcavation;
use super::super::sight_ray::SurfaceSightRay;
use super::super::vehicle_assembly::{VehicleAssembly, VehicleAxes};
use super::{CantileverRig, RigArm, BLOCK_STEP};

/// Collaborator mounting a saved `CraftBlueprint` onto a vacant cantilever crane arm and retracting its mast.
pub(super) struct BlueprintHoist<'a> {
    blueprint: &'a CraftBlueprint,
    frame: &'a SurfaceFrame,
}

impl<'a> BlueprintHoist<'a> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub(super) const fn new(blueprint: &'a CraftBlueprint, frame: &'a SurfaceFrame) -> Self {
        Self { blueprint, frame }
    }

    pub(super) fn mount_into(
        self,
        mast: (i32, i32),
        arm: &mut RigArm,
        craft: &mut CraftCluster,
        ground_fn: impl Fn(i32, i32) -> f32,
    ) -> Option<Option<(i32, i32)>> {
        if !craft.vacant() || !self.blueprint.drivable() {
            return None;
        }
        arm.reset();
        let mut cluster = self.blueprint.to_face_cluster(self.frame);
        let (anchor_u, anchor_v, anchor_z) = cluster.claw_anchor()?;
        let (gu, gv) = arm.gripped_cell(mast.0, mast.1);
        let hoist_z = arm.hoist_elevation();
        cluster.translate_by(gu - anchor_u, gv - anchor_v, hoist_z - anchor_z);

        let min_up = cluster.clamped_vertical_delta(0.0, &ground_fn);
        if min_up > 1e-4 {
            let needed_steps = (min_up / BLOCK_STEP).ceil() as i32;
            let dz = arm.clamped_step_delta(needed_steps);
            arm.apply_vertical_shift(dz);
            cluster.shift_elevation(dz.max(min_up));
        }

        let step = arm.step_vectors();
        let min_s = cluster.min_boom_step(mast.0, mast.1, step, arm.reach_steps());
        let relocated = if min_s <= 0 {
            let next_mast = (mast.0 + (min_s - 1) * step.0, mast.1 + (min_s - 1) * step.1);
            arm.reposition_mast(ground_fn(next_mast.0, next_mast.1), 1);
            Some(next_mast)
        } else if min_s == 1 {
            arm.reposition_mast(ground_fn(mast.0, mast.1), 1);
            None
        } else {
            None
        };
        *craft = cluster;
        Some(relocated)
    }
}

/// Read-only collaborator querying aimed empty crane arms and capturing mounted blueprints (`1` field).
pub(super) struct RigBlueprintQuery<'a> {
    rigs: &'a HashMap<(i32, i32), CantileverRig>,
}

impl<'a> RigBlueprintQuery<'a> {
    pub(super) const fn new(rigs: &'a HashMap<(i32, i32), CantileverRig>) -> Self {
        Self { rigs }
    }

    pub(super) fn find_aimed(self, u: i32, v: i32, ray: SurfaceSightRay, frame: &SurfaceFrame) -> Option<(i32, i32)> {
        if let Some((&mast, _)) = self.rigs.iter().find(|(&(mu, mv), r)| r.aimed_empty_boom(mu, mv, u, v)) {
            return Some(mast);
        }
        (1..=60).find_map(|step| {
            let sample = ray.sample_point((step as f32) * 0.08);
            let (su, sv) = frame.decompose(sample);
            let (ru, rv) = GridCell::at_planar(su, sv).feed_coordinates(|gu, gv| (gu, gv));
            self.rigs.iter().find_map(|(&(mu, mv), r)| r.aimed_empty_boom(mu, mv, ru, rv).then_some((mu, mv)))
        })
    }

    pub(super) fn capture(
        self,
        (u, v, layer_elev): (i32, i32, f32),
        (eu, ev): (f32, f32),
        frame: &SurfaceFrame,
    ) -> Option<CraftBlueprint> {
        self.rigs.iter().find_map(|(&(mu, mv), rig)| {
            let hit = !rig.vacant_craft()
                && (rig.craft_medium_at(u, v, layer_elev).is_some()
                    || rig.boom_or_craft_at(mu, mv, u, v)
                    || ((u, v) == (mu, mv) && rig.occupies_mast_layer(layer_elev)));
            hit.then(|| rig.capture_blueprint(eu, ev, frame)).flatten()
        })
    }
}

/// Collaborator reconnecting a nearby parked vehicle onto a vacant cantilever crane arm.
pub(super) struct SeatedCarMount<'a> {
    rigs: &'a mut HashMap<(i32, i32), CantileverRig>,
    fleet: &'a mut ParkedFleet,
}

impl<'a> SeatedCarMount<'a> {
    pub(super) fn new(rigs: &'a mut HashMap<(i32, i32), CantileverRig>, fleet: &'a mut ParkedFleet) -> Self {
        Self { rigs, fleet }
    }

    pub(super) fn reconnect(
        self,
        strata: &NaturalStrata,
        shafts: &ShaftExcavation,
        mast: (i32, i32),
        ray: SurfaceSightRay,
        frame: &SurfaceFrame,
    ) -> Option<VehicleAssembly> {
        if !self.rigs.get(&mast).is_some_and(CantileverRig::vacant_craft) {
            return None;
        }
        let eye_world = ray.sample_point(0.0);
        let (eu, ev) = frame.decompose(eye_world);
        let parked = self.fleet.take_cluster_near(strata, shafts, eye_world, 5.0)?;
        let mut rig = self.rigs.remove(&mast)?;
        match rig.mount_parked(mast.0, mast.1, parked, eu, ev, frame) {
            Ok((relocated, origin, heading)) => {
                self.rigs.insert(relocated.unwrap_or(mast), rig);
                let axes = VehicleAxes::new(frame.tangent(), frame.bitangent(), frame.normal());
                Some(VehicleAssembly::new(origin, axes, heading, Vec::new()))
            }
            Err(_) => {
                self.rigs.insert(mast, rig);
                None
            }
        }
    }
}
