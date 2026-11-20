use crate::frame::SurfaceFrame;
use super::super::craft_cluster::CraftCluster;
use super::super::craft_entity::CraftBlueprint;
use super::super::molded_structure::MoldedStructure;
use super::{RigArm, BLOCK_STEP};

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
