use glam::Vec3;
use crate::frame::SurfaceFrame;
use crate::relief::MoldedStructure;
use crate::relief::{BlueprintHoist, CraftBlueprint, CraftCluster, RigArm};

/// Collaborator mounting saved blueprints or parked vehicles onto a `MagneticFlange` `GridBody` (`2` fields).
pub struct FlangeHoist<'a> {
    grid: &'a mut crate::brick_arena::BrickStructure,
    arm: &'a mut RigArm,
}

impl<'a> FlangeHoist<'a> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(grid: &'a mut crate::brick_arena::BrickStructure, arm: &'a mut RigArm) -> Self {
        Self { grid, arm }
    }

    pub fn mount_blueprint(
        self,
        mast: (i32, i32),
        blueprint: &CraftBlueprint,
        frame: &SurfaceFrame,
        ground_fn: impl Fn(i32, i32) -> f32,
    ) -> Option<Option<(i32, i32)>> {
        let mut cluster = CraftCluster::new(std::mem::replace(self.grid, crate::brick_arena::BrickStructure::untouched()));
        let res = BlueprintHoist::new(blueprint, frame).mount_into(mast, self.arm, &mut cluster, ground_fn);
        *self.grid = cluster.into_structure();
        res
    }

    pub fn mount_parked(
        self,
        mast: (i32, i32),
        mut parked: CraftCluster,
        (eu, ev): (f32, f32),
        frame: &SurfaceFrame,
    ) -> Result<(Option<(i32, i32)>, Vec3, Vec3), CraftCluster> {
        if !self.grid.vacant() {
            return Err(parked);
        }
        self.arm.reset();
        let (gu, gv) = self.arm.gripped_cell(mast.0, mast.1);
        let hoist_z = self.arm.hoist_elevation();
        let Some((moved_floors, origin, heading)) = parked.relocate_to_claw((gu, gv, hoist_z), eu, ev, frame) else {
            return Err(parked);
        };
        let (mut cur_u, mut cur_v, mut relocated) = (mast.0, mast.1, None);
        for (ncu, ncv, nf) in moved_floors {
            if let Some(next_mast) = self.arm.retract_for_placed_block(cur_u, cur_v, ncu, ncv, nf) {
                (cur_u, cur_v) = next_mast;
                relocated = Some(next_mast);
            }
        }
        *self.grid = parked.into_structure();
        Ok((relocated, origin, heading))
    }
}
