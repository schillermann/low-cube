use std::collections::HashMap;
use crate::block::GridCell;
use crate::brick_arena::{BrickStructure, CellCoord, ModuleSpec};
use crate::frame::SurfaceFrame;
use crate::grid_body::{BodyMooring, BodyMotion, GridBody, GridFleet};
use super::super::craft_cluster::CraftCluster;
use super::super::craft_entity::{CraftBlueprint, CraftIdentity};
use super::super::molded_structure::MoldedStructure;
use super::super::natural_strata::NaturalStrata;
use super::super::shaft_excavation::ShaftExcavation;
use super::super::sight_ray::SurfaceSightRay;
use super::super::vehicle_assembly::{VehicleAssembly, VehicleAxes};
use super::{RigArm, BLOCK_STEP};

/// Collaborator mounting a saved `CraftBlueprint` onto a vacant magnetic flange / crane arm (`2` fields).
pub(crate) struct BlueprintHoist<'a> {
    blueprint: &'a CraftBlueprint,
    frame: &'a SurfaceFrame,
}

impl<'a> BlueprintHoist<'a> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub(crate) const fn new(blueprint: &'a CraftBlueprint, frame: &'a SurfaceFrame) -> Self {
        Self { blueprint, frame }
    }

    pub(crate) fn mount_into(
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

/// Read-only collaborator querying aimed empty magnetic flanges / crane arms and capturing mounted blueprints (`1` field).
pub(super) struct RigBlueprintQuery<'a> {
    rigs: &'a HashMap<(i32, i32), GridBody>,
}

impl<'a> RigBlueprintQuery<'a> {
    pub(super) const fn new(rigs: &'a HashMap<(i32, i32), GridBody>) -> Self {
        Self { rigs }
    }

    pub(super) fn find_aimed(self, u: i32, v: i32, ray: SurfaceSightRay, frame: &SurfaceFrame) -> Option<(i32, i32)> {
        if let Some((&mast, _)) = self
            .rigs
            .iter()
            .find(|(&(mu, mv), r)| r.flange().is_some_and(|f| f.aimed_empty_boom(mu, mv, u, v)))
        {
            return Some(mast);
        }
        (1..=60).find_map(|step| {
            let sample = ray.sample_point((step as f32) * 0.08);
            let (su, sv) = frame.decompose(sample);
            let (ru, rv) = GridCell::at_planar(su, sv).feed_coordinates(|gu, gv| (gu, gv));
            self.rigs
                .iter()
                .find_map(|(&(mu, mv), r)| r.flange().is_some_and(|f| f.aimed_empty_boom(mu, mv, ru, rv)).then_some((mu, mv)))
        })
    }

    pub(super) fn capture(
        self,
        (u, v, layer_elev): (i32, i32, f32),
        (eu, ev): (f32, f32),
        frame: &SurfaceFrame,
    ) -> Option<CraftBlueprint> {
        self.rigs.iter().find_map(|(&(mu, mv), rig)| {
            let flange = rig.flange()?;
            let hit = !rig.structure().vacant()
                && (rig.medium_at(u, v, layer_elev).is_some()
                    || flange.boom_or_craft_at(mu, mv, u, v)
                    || ((u, v) == (mu, mv) && flange.occupies_mast_layer(layer_elev)));
            hit.then(|| flange.capture_blueprint(eu, ev, frame)).flatten()
        })
    }
}

/// Collaborator releasing, adjusting, or mounting blueprints on `MagneticFlange` `GridBody`s (`1` field).
pub(super) struct FlangeRelease<'a> {
    rigs: &'a mut HashMap<(i32, i32), GridBody>,
}

impl<'a> FlangeRelease<'a> {
    pub(super) fn new(rigs: &'a mut HashMap<(i32, i32), GridBody>) -> Self {
        Self { rigs }
    }

    pub(super) fn deploy_flange(self, u: i32, v: i32, ground_top: f32, spec: ModuleSpec) -> bool {
        if self.rigs.contains_key(&(u, v)) {
            return false;
        }
        let mut grid = BrickStructure::untouched();
        let anchor = CellCoord::from_floor(u, v, ground_top + BLOCK_STEP, 0.0);
        if !grid.place_module_at(anchor, spec) {
            return false;
        }
        let body = GridBody::new(
            grid,
            CraftBlueprint::new(CraftIdentity::unassigned(), Vec::new()),
            BodyMotion::planetary_identity(),
            BodyMooring::MagneticFlange(RigArm::magnetic_flange(ground_top, spec.turn().code())),
        );
        self.rigs.insert((u, v), body);
        true
    }

    pub(super) fn adjust_at(
        self,
        strata: &NaturalStrata,
        shafts: &ShaftExcavation,
        (u, v): (i32, i32),
        delta_steps: i32,
        rotate_quarter: bool,
    ) -> bool {
        let target_mast = if self.rigs.contains_key(&(u, v)) {
            Some((u, v))
        } else {
            self.rigs
                .iter()
                .find_map(|(&(mu, mv), rig)| rig.flange()?.boom_or_craft_at(mu, mv, u, v).then_some((mu, mv)))
        };
        let Some(mast) = target_mast else { return false };
        let Some(rig) = self.rigs.get_mut(&mast) else { return false };
        rig.with_flange_mut(|mut f| {
            f.adjust(mast, delta_steps, rotate_quarter, |cu, cv| {
                (shafts.elevation_at(strata, cu, cv) as f32) * BLOCK_STEP
            })
        })
        .unwrap_or(false)
    }

    pub(super) fn board_drivable(
        self,
        strata: &NaturalStrata,
        shafts: &ShaftExcavation,
        target: (i32, i32, f32),
        ray: SurfaceSightRay,
        frame: &SurfaceFrame,
    ) -> Option<VehicleAssembly> {
        self.rigs.iter_mut().find_map(|(&mast, rig)| {
            rig.with_flange_mut(|mut f| {
                f.release_drivable(mast, target, ray, frame, |cu, cv| {
                    (shafts.elevation_at(strata, cu, cv) as f32) * BLOCK_STEP
                })
            })
            .flatten()
        })
    }

    pub(super) fn unlock_flange(
        self,
        strata: &NaturalStrata,
        shafts: &ShaftExcavation,
        (u, v): (i32, i32),
        ray: SurfaceSightRay,
        frame: &SurfaceFrame,
    ) -> Option<VehicleAssembly> {
        let mast = self.rigs.iter().find_map(|(&(mu, mv), rig)| {
            let flange = rig.flange()?;
            let low_profile = rig.rig_arm().is_some_and(|a| a.reach_steps() == 0);
            (low_profile && ((u, v) == (mu, mv) || flange.boom_or_craft_at(mu, mv, u, v))).then_some((mu, mv))
        })?;
        let mut rig = self.rigs.remove(&mast)?;
        rig.with_flange_mut(|mut f| {
            f.unlock_any_assembly(mast, ray, frame, |cu, cv| {
                (shafts.elevation_at(strata, cu, cv) as f32) * BLOCK_STEP
            })
        })
        .flatten()
    }

    pub(super) fn mount_blueprint(
        self,
        mast: (i32, i32),
        blueprint: &CraftBlueprint,
        strata: &NaturalStrata,
        shafts: &ShaftExcavation,
        frame: &SurfaceFrame,
    ) -> Option<(i32, i32)> {
        let mut rig = self.rigs.remove(&mast)?;
        let mounted = rig
            .with_flange_hoist(|h| {
                h.mount_blueprint(mast, blueprint, frame, |cu, cv| {
                    (shafts.elevation_at(strata, cu, cv) as f32) * BLOCK_STEP
                })
            })
            .flatten();
        let final_mast = mounted.map(|rel| rel.unwrap_or(mast));
        self.rigs.insert(final_mast.unwrap_or(mast), rig);
        final_mast
    }
}

/// Collaborator reconnecting a nearby settled `GridBody` onto a vacant magnetic flange (`2` fields).
pub(super) struct SeatedCarMount<'a> {
    rigs: &'a mut HashMap<(i32, i32), GridBody>,
    fleet: &'a mut GridFleet,
}

impl<'a> SeatedCarMount<'a> {
    pub(super) fn new(rigs: &'a mut HashMap<(i32, i32), GridBody>, fleet: &'a mut GridFleet) -> Self {
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
        if !self.rigs.get(&mast).is_some_and(|r| r.structure().vacant()) {
            return None;
        }
        let eye_world = ray.sample_point(0.0);
        let (eu, ev) = frame.decompose(eye_world);
        let parked = self.fleet.take_cluster_near(strata, shafts, eye_world, 5.0)?;
        let mut rig = self.rigs.remove(&mast)?;
        let res = rig.with_flange_hoist(|h| h.mount_parked(mast, parked, (eu, ev), frame))?;
        match res {
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
