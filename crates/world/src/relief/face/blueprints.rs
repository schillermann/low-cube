use glam::Vec3;
use crate::block::{BlockMedium, BlockMold, GridCell, MiningEdge};
use crate::frame::SurfaceFrame;
use super::super::craft_entity::CraftBlueprint;
use super::super::elevation::ReliefElevation;
use super::super::planned_mount::{MountSite, PlannedMount};
use super::super::sight_ray::SurfaceSightRay;
use super::super::vehicle_assembly::VehicleAssembly;

pub(super) struct ReliefDomain {
    pub(super) frame: SurfaceFrame,
    pub(super) relief: Box<dyn ReliefElevation>,
    pub(super) awake: bool,
}

impl ReliefDomain {
    pub(super) fn new(frame: SurfaceFrame, relief: Box<dyn ReliefElevation>) -> Self {
        Self { frame, relief, awake: false }
    }

    pub(super) fn wake(&mut self) {
        self.awake = true;
    }

    pub(super) fn step_grass(&mut self, dt: f32) -> Vec<(i32, i32)> {
        let (cells, active) = self.relief.step_grass_activity(dt);
        self.awake = active;
        cells
    }

    pub(super) fn harvest(&mut self, half: i32, eye: Vec3, direction: Vec3) -> Option<(i32, i32, BlockMedium)> {
        let half_size = self.frame.step_elevation(half);
        let (cu, cv, layer) = SurfaceSightRay::new(eye, direction)
            .target_layer(&self.frame, self.relief.as_ref(), half, half_size)?;
        let harvested = self.relief.harvest_layer(cu, cv, layer)?;
        self.awake = true;
        Some((cu, cv, harvested))
    }

    pub(super) fn loosen(&mut self, half: i32, eye: Vec3, direction: Vec3, stage: u8) -> Option<(i32, i32, BlockMedium)> {
        let half_size = self.frame.step_elevation(half);
        let (cu, cv, layer) = SurfaceSightRay::new(eye, direction)
            .target_layer(&self.frame, self.relief.as_ref(), half, half_size)?;
        let loosened = self.relief.mark_dig_layer(cu, cv, layer, stage)?;
        self.awake = true;
        Some((cu, cv, loosened))
    }

    pub(super) fn adjust_rig(&mut self, half: i32, eye: Vec3, direction: Vec3, delta: i32, rot: bool) -> Option<(i32, i32)> {
        let half_size = self.frame.step_elevation(half);
        let (cu, cv) = SurfaceSightRay::new(eye, direction)
            .target_layer(&self.frame, self.relief.as_ref(), half, half_size)
            .map_or((0, 0), |(cu, cv, _)| (cu, cv));
        let (changed, cell) = self.relief.adjust_rig_at(cu, cv, delta, rot);
        changed.then_some(cell)
    }

    pub(super) fn lower_rig(&mut self, half: i32, eye: Vec3, direction: Vec3) -> Option<(i32, i32)> {
        let half_size = self.frame.step_elevation(half);
        let (cu, cv, _) = SurfaceSightRay::new(eye, direction)
            .target_layer(&self.frame, self.relief.as_ref(), half, half_size)?;
        self.relief.lower_rig_at(cu, cv).then_some((cu, cv))
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
        let ok = self.relief.stack_molded_cell(cu, cv, base, shape);
        if ok {
            self.awake = true;
        }
        ok.then_some((cu, cv))
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

    pub(super) fn board_vehicle(
        &mut self,
        half: i32,
        eye: Vec3,
        direction: Vec3,
        engage_engine: bool,
    ) -> Option<(VehicleAssembly, [(i32, i32); 3], bool)> {
        let half_size = self.frame.step_elevation(half);
        let ray = SurfaceSightRay::new(eye, direction);
        let ray_hit = ray.target_layer(&self.frame, self.relief.as_ref(), half, half_size);
        let (cu, cv, layer) = if engage_engine {
            ray_hit.unwrap_or_else(|| {
                let (eu, ev) = self.frame.decompose(eye);
                GridCell::at_planar(eu, ev).feed_coordinates(|gu, gv| (gu, gv, -1000.0))
            })
        } else {
            ray_hit?
        };
        let (assembly, terrain_changed) =
            self.relief.board_vehicle_on_face(&self.frame, cu, cv, layer, ray, engage_engine);
        let assembly = assembly?;
        let (eu, ev) = self.frame.cell_at(eye).feed_coordinates(|gu, gv| (gu, gv));
        let (su, sv) = self.frame.cell_at(assembly.origin()).feed_coordinates(|gu, gv| (gu, gv));
        Some((assembly, [(eu, ev), (cu, cv), (su, sv)], terrain_changed))
    }

    pub(super) fn surface_yields_to(&self, half: i32, eye: Vec3, direction: Vec3, edge: MiningEdge) -> Option<bool> {
        let half_size = self.frame.step_elevation(half);
        let (cu, cv, layer) = SurfaceSightRay::new(eye, direction)
            .target_layer(&self.frame, self.relief.as_ref(), half, half_size)?;
        Some(self.relief.layer_yields_to(cu, cv, layer, edge))
    }
}
