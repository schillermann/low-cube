use glam::Vec3;
use crate::block::{BlockMedium, BlockMold};
use crate::frame::SurfaceFrame;
use super::super::craft_entity::CraftBlueprint;
use super::super::elevation::ReliefElevation;
use super::super::planned_mount::{MountSite, PlannedMount};
use super::super::sight_ray::SurfaceSightRay;

pub(super) struct ReliefDomain {
    pub(super) frame: SurfaceFrame,
    pub(super) relief: Box<dyn ReliefElevation>,
}

impl ReliefDomain {
    pub(super) fn new(frame: SurfaceFrame, relief: Box<dyn ReliefElevation>) -> Self {
        Self { frame, relief }
    }

    pub(super) fn plan_molded(&self, half: i32, eye: Vec3, direction: Vec3, avatar_pos: Vec3, mold: BlockMold) -> Option<PlannedMount> {
        let half_size = self.frame.step_elevation(half);
        let (cu, cv, base, effective) = SurfaceSightRay::new(eye, direction)
            .oriented_placement_for_mold(&self.frame, self.relief.as_ref(), half, half_size, avatar_pos, mold)?;
        let (cell, floor_elev, domain) = self.relief.resolve_planned_site(cu, cv, base, effective.medium())?;
        Some(PlannedMount::new(MountSite::new(self.frame, cell, floor_elev), effective, direction, domain))
    }

    pub(super) fn place_molded(&mut self, half: i32, eye: Vec3, direction: Vec3, avatar_pos: Vec3, mold: BlockMold) -> Option<(i32, i32)> {
        let half_size = self.frame.step_elevation(half);
        let (cu, cv, base, effective) = SurfaceSightRay::new(eye, direction)
            .oriented_placement_for_mold(&self.frame, self.relief.as_ref(), half, half_size, avatar_pos, mold)?;
        self.relief.resolve_planned_site(cu, cv, base, effective.medium())?;
        let shape = effective.shape_for(direction, &self.frame);
        self.relief.stack_molded_cell(cu, cv, base, shape).then_some((cu, cv))
    }

    pub(super) fn capture_blueprint(&self, half: i32, eye: Vec3, direction: Vec3) -> Option<CraftBlueprint> {
        let half_size = self.frame.step_elevation(half);
        let ray = SurfaceSightRay::new(eye, direction);
        let (cu, cv, layer) = ray
            .target_layer(&self.frame, self.relief.as_ref(), half, half_size)
            .unwrap_or((0, 0, -1000.0));
        self.relief.capture_blueprint_at(&self.frame, cu, cv, layer, ray)
    }

    pub(super) fn aimed_empty_rig(&self, half: i32, eye: Vec3, direction: Vec3, avatar_pos: Vec3) -> bool {
        let half_size = self.frame.step_elevation(half);
        let ray = SurfaceSightRay::new(eye, direction);
        let bp_mold = BlockMold::cube().in_medium(BlockMedium::VehicleBlueprint);
        if let Some((cu, cv, base)) =
            ray.placement_for_mold(&self.frame, self.relief.as_ref(), half, half_size, avatar_pos, bp_mold)
        {
            if self.relief.aimed_empty_rig_at(&self.frame, cu, cv, base, ray) {
                return true;
            }
        }
        if let Some((cu, cv, layer)) = ray.target_layer(&self.frame, self.relief.as_ref(), half, half_size) {
            if self.relief.aimed_empty_rig_at(&self.frame, cu, cv, layer, ray) {
                return true;
            }
        }
        self.relief.aimed_empty_rig_at(&self.frame, -4096, -4096, 0.0, ray)
    }

    pub(super) fn build_blueprint(&mut self, half: i32, eye: Vec3, direction: Vec3, blueprint: &CraftBlueprint) -> Option<(i32, i32)> {
        let half_size = self.frame.step_elevation(half);
        let ray = SurfaceSightRay::new(eye, direction);
        let (cu, cv, base) = ray
            .target_layer(&self.frame, self.relief.as_ref(), half, half_size)
            .unwrap_or((-4096, -4096, 0.0));
        self.relief.build_rig_blueprint_at(&self.frame, cu, cv, base, ray, blueprint)
    }
}
