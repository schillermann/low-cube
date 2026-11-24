mod blueprints;
mod snapshot;

use glam::Vec3;
use crate::MeshTarget;
use crate::block::{BlockMedium, GridCell, VoxelShape};
use crate::frame::SurfaceFrame;
use super::cantilever_rig::{RigPlacement, RigRegistry};
use super::craft_cluster::ParkedFleet;
use super::craft_entity::{CraftBlueprint, VesselSink};
use super::molded_structure::{MoldedStructure, ParkedStructure, RiggedStructure, WorldBlocks};
use super::natural_strata::NaturalStrata;
use super::planned_mount::MountDomain;
use super::shaft_excavation::ShaftExcavation;
use super::sight_ray::SurfaceSightRay;
use super::turf_scars::TurfScars;
use super::vehicle_assembly::VehicleAssembly;
use self::blueprints::{BlueprintCommission, BlueprintSite, VehicleEmbarkation};
use self::snapshot::{SectorMoldsTarget, SectorMoldsView};

const BLOCK_STEP: f32 = GridCell::SPAN;
const HALF_BLOCK_STEP: f32 = 0.25;
const NICHE_WALL_MARGIN: f32 = 0.10;
pub(super) const MAX_STACK_ELEVATION: f32 = 16.0;

/// Overhanging World-Cube blocks composed with parked vehicles, cantilever cranes, and turf scars.
pub struct OverhangMolds {
    world_blocks: WorldBlocks,
    fleet: ParkedFleet,
    rigs: RigRegistry,
    scars: TurfScars,
}

impl OverhangMolds {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(world_blocks: WorldBlocks, fleet: ParkedFleet, rigs: RigRegistry, scars: TurfScars) -> Self {
        Self { world_blocks, fleet, rigs, scars }
    }

    pub fn untouched() -> Self {
        Self::new(WorldBlocks::untouched(), ParkedFleet::untouched(), RigRegistry::untouched(), TurfScars::untouched())
    }

    const fn decorated(&self) -> RiggedStructure<'_, ParkedStructure<'_, &WorldBlocks>> {
        RiggedStructure::new(ParkedStructure::new(&self.world_blocks, &self.fleet), &self.rigs)
    }

    pub fn empty(&self) -> bool { self.decorated().vacant() }
    pub fn present_at(&self, u: i32, v: i32) -> bool { self.decorated().present_at(u, v) }
    pub fn bare_dirt_at(&self, u: i32, v: i32) -> bool {
        self.world_blocks.bare_surface_at(u, v, self.scars.bare_at(u, v))
    }
    pub fn ground_bare_at(&self, u: i32, v: i32, ground_top: f32) -> bool {
        self.scars.bare_at(u, v) || self.world_blocks.covers_ground_at(u, v, ground_top)
    }
    pub fn expose_dirt(&mut self, u: i32, v: i32) { self.scars.expose_dirt(u, v); }

    pub fn layer_dig_stage_at(&self, u: i32, v: i32, layer_elev: f32) -> u8 {
        self.scars.layer_stage(u, v, layer_elev, |e| self.layer_occupied_at(u, v, e))
    }

    pub fn clear_stage(&mut self, u: i32, v: i32) -> bool { self.scars.clear_stage(u, v) }
    pub fn record_stage(&mut self, u: i32, v: i32, layer_elev: f32, stage: u8) -> bool {
        self.scars.record_stage(u, v, layer_elev, stage)
    }

    pub fn top_steps_at(&self, u: i32, v: i32, floor_steps: i32) -> i32 {
        self.decorated().top_steps_at(u, v, floor_steps)
    }

    pub fn solid_wall_at(&self, u: i32, v: i32, ground_h: f32, ceiling_h: f32, elev: f32) -> bool {
        (elev > ceiling_h + NICHE_WALL_MARGIN && elev < ground_h - NICHE_WALL_MARGIN)
            || self.layer_occupied_at(u, v, elev)
    }

    pub fn confine_niche_elevation(&self, u: i32, v: i32, ground_h: f32, elev: f32) -> Option<f32> {
        self.world_blocks.confine_niche_elevation(u, v, ground_h, elev)
    }

    pub fn constrain_support(&self, cell: GridCell, planar: (f32, f32), h: f32, span: (f32, f32)) -> f32 {
        self.decorated().constrain_support(cell, planar, h, span)
    }

    pub fn layer_occupied_at(&self, u: i32, v: i32, layer_elev: f32) -> bool {
        self.decorated().layer_occupied_at(u, v, layer_elev)
    }

    pub fn boom_slot_at(&self, u: i32, v: i32, layer_elev: f32) -> bool {
        self.rigs.boom_slot_at(u, v, layer_elev)
    }

    pub fn effective_placement_cell(&self, u: i32, v: i32, ground_top: f32, sub: bool, raw_floor: f32) -> Option<(i32, i32, f32)> {
        match self.rigs.resolve_placement(u, v, ground_top, sub, raw_floor) {
            RigPlacement::BlockedMast => None,
            RigPlacement::MountFirst { cell, floor_elev, .. } => Some((cell.0, cell.1, floor_elev)),
            RigPlacement::AttachCraft { cell, floor_elev, .. } => {
                (floor_elev < MAX_STACK_ELEVATION).then_some((cell.0, cell.1, floor_elev))
            }
            RigPlacement::WorldCube => Some((u, v, raw_floor)),
        }
    }

    fn resolve_world_floor(&self, u: i32, v: i32, ground_top: f32, sub: bool, raw_floor: f32) -> Option<f32> {
        let mut next_floor = if sub { raw_floor } else { raw_floor.max(ground_top) };
        while self.world_blocks.layer_occupied_at(u, v, next_floor + HALF_BLOCK_STEP) {
            next_floor += if sub { -BLOCK_STEP } else { BLOCK_STEP };
        }
        (next_floor < MAX_STACK_ELEVATION && !self.fleet.blocks_placement_at(u, v, next_floor))
            .then_some(next_floor)
    }

    pub fn resolve_planned_site(
        &self,
        u: i32,
        v: i32,
        ground_top: f32,
        sub: bool,
        raw_floor: f32,
        medium: BlockMedium,
    ) -> Option<(GridCell, f32, MountDomain)> {
        BlueprintSite::new(&self.rigs, &self.fleet).resolve_any_site(
            &self.world_blocks,
            (u, v),
            (ground_top, raw_floor),
            sub,
            medium,
        )
    }

    pub fn present_vessels(&self, sink: &mut dyn VesselSink) { self.fleet.present_vessels(sink); }
    pub fn car_locked_at(&self, u: i32, v: i32, layer_elev: f32) -> bool { self.decorated().locked_at(u, v, layer_elev) }
    pub fn seat_backrest_cell(&self, u: i32, v: i32, layer_elev: f32) -> Option<(i32, i32)> {
        self.decorated().seat_backrest_cell(u, v, layer_elev)
    }

    pub fn layer_medium_at(&self, strata: &NaturalStrata, shafts: &ShaftExcavation, u: i32, v: i32, layer_elev: f32) -> BlockMedium {
        self.decorated()
            .medium_at(u, v, layer_elev)
            .unwrap_or_else(|| shafts.excavated_medium_at(strata, u, v, layer_elev))
    }

    pub fn hit_top_at(&self, cu: i32, cv: i32, u: f32, v: f32, ray_elev: f32) -> Option<f32> {
        self.decorated().hit_top_at(cu, cv, u, v, ray_elev)
    }

    pub fn emit_molded_top(&self, cell: GridCell, frame: &SurfaceFrame, colors: (Vec3, Vec3), target: &mut dyn MeshTarget) -> bool {
        self.decorated().emit_at(cell, frame, colors, &self.scars, target)
    }

    pub fn placed_camera_tripod(&self, frame: &SurfaceFrame) -> Option<(Vec3, Vec3, Vec3)> {
        self.decorated().tripod_hub(frame)
    }

    pub fn collect_spawn_points(&self, frame: &SurfaceFrame, out: &mut Vec<Vec3>) {
        self.world_blocks.collect_spawn_points(frame, out);
    }

    pub fn remove_layer(&mut self, strata: &NaturalStrata, shafts: &ShaftExcavation, u: i32, v: i32, layer_elev: f32) -> Option<BlockMedium> {
        if self.car_locked_at(u, v, layer_elev) {
            return None;
        }
        if let Some(med) = self.rigs.remove_craft_layer(u, v, layer_elev) {
            self.clear_stage(u, v);
            return Some(med);
        }
        if self.rigs.pack_up(strata, shafts, u, v, layer_elev, &mut self.world_blocks, &mut self.scars) {
            self.clear_stage(u, v);
            return Some(BlockMedium::CantileverRig);
        }
        let ground_top = (shafts.elevation_at(strata, u, v) as f32) * BLOCK_STEP;
        self.world_blocks.remove_layer(u, v, layer_elev, ground_top, &mut self.scars)
    }

    pub fn retain_overhang(&mut self, strata: &NaturalStrata, u: i32, v: i32, next_elev: i32, elev: i32) {
        let ground_verdant = !self.scars.bare_at(u, v);
        self.world_blocks.retain_overhang(strata, u, v, next_elev, elev, ground_verdant);
    }

    pub fn stack_molded(&mut self, u: i32, v: i32, ground_top: f32, sub: bool, raw_floor: f32, shape: Box<dyn VoxelShape>) -> bool {
        if shape.block_medium() == BlockMedium::VehicleBlueprint {
            return false;
        }
        if shape.block_medium() == BlockMedium::CantileverRig {
            self.rigs.deploy(u, v, ground_top);
            self.clear_stage(u, v);
            return true;
        }
        match self.rigs.resolve_placement(u, v, ground_top, sub, raw_floor) {
            RigPlacement::BlockedMast => false,
            RigPlacement::MountFirst { mast, cell, floor_elev } => {
                self.rigs.place_on_rig(mast, cell, floor_elev, shape);
                self.clear_stage(cell.0, cell.1);
                true
            }
            RigPlacement::AttachCraft { mast, cell, floor_elev } => {
                if floor_elev >= MAX_STACK_ELEVATION {
                    return false;
                }
                self.rigs.place_on_rig(mast, cell, floor_elev, shape);
                self.clear_stage(cell.0, cell.1);
                true
            }
            RigPlacement::WorldCube => {
                let Some(next_floor) = self.resolve_world_floor(u, v, ground_top, sub, raw_floor) else {
                    return false;
                };
                self.world_blocks.stack_bare(u, v, next_floor, ground_top, sub, shape, &mut self.scars);
                self.clear_stage(u, v);
                true
            }
        }
    }

    pub fn adjust_rig(&mut self, strata: &NaturalStrata, shafts: &ShaftExcavation, u: i32, v: i32, delta_steps: i32, rotate_quarter: bool) -> (bool, (i32, i32)) {
        if self.rigs.adjust_at(strata, shafts, u, v, delta_steps, rotate_quarter) {
            return (true, (u, v));
        }
        if rotate_quarter || delta_steps == 0 {
            return (false, (u, v));
        }
        self.rigs
            .step_camera_turn(u, v, delta_steps)
            .or_else(|| self.fleet.step_camera_turn(u, v, delta_steps))
            .or_else(|| self.world_blocks.step_camera_turn(u, v, delta_steps))
            .map_or((false, (u, v)), |cell| (true, cell))
    }

    pub fn lower_rig(&mut self, strata: &NaturalStrata, shafts: &ShaftExcavation, u: i32, v: i32) -> bool {
        self.rigs.lower_mast_at(strata, shafts, u, v)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn board_vehicle(
        &mut self,
        strata: &NaturalStrata,
        shafts: &ShaftExcavation,
        frame: &SurfaceFrame,
        u: i32,
        v: i32,
        layer_elev: f32,
        ray: SurfaceSightRay,
        engage_engine: bool,
    ) -> (Option<VehicleAssembly>, bool) {
        let (eu, ev) = frame.decompose(ray.sample_point(0.0));
        if !engage_engine {
            return (self.decorated().perch_at(u, v, layer_elev, eu, ev, frame), false);
        }
        VehicleEmbarkation::new(&mut self.rigs, &mut self.fleet)
            .embark(strata, shafts, frame, (u, v, layer_elev), ray)
    }

    pub fn park_vehicle(&mut self, strata: &NaturalStrata, shafts: &ShaftExcavation, frame: &SurfaceFrame, assembly: VehicleAssembly) -> (i32, i32) {
        self.fleet.park_settled(strata, shafts, frame, assembly)
    }

    pub fn capture_blueprint(&self, frame: &SurfaceFrame, u: i32, v: i32, layer_elev: f32, ray: SurfaceSightRay) -> Option<CraftBlueprint> {
        BlueprintSite::new(&self.rigs, &self.fleet).capture(frame, u, v, layer_elev, ray)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn aimed_empty_rig(&self, u: i32, v: i32, ground_top: f32, sub: bool, raw_floor: f32, ray: SurfaceSightRay, frame: &SurfaceFrame) -> bool {
        BlueprintSite::new(&self.rigs, &self.fleet).aimed_empty_rig(u, v, ground_top, sub, raw_floor, ray, frame)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn build_rig_blueprint(
        &mut self, strata: &NaturalStrata, shafts: &ShaftExcavation, frame: &SurfaceFrame,
        u: i32, v: i32, ground_top: f32, sub: bool, raw_floor: f32, ray: SurfaceSightRay, blueprint: &CraftBlueprint,
    ) -> Option<(i32, i32)> {
        BlueprintCommission::new(&mut self.rigs, &mut self.scars)
            .build(strata, shafts, frame, (u, v, ground_top, raw_floor), sub, ray, blueprint)
    }

    pub fn step_grass(&mut self, strata: &NaturalStrata, shafts: &ShaftExcavation, dt: f32) -> (Vec<(i32, i32)>, bool) {
        let settled_cells = self.world_blocks.settle_gravity(strata, shafts, &mut self.scars);
        let (mut regrown, scars_active) = self.scars.regrow_step(strata, shafts, &self.world_blocks, dt);
        let (sod_cells, blocks_active) = self.world_blocks.regrow_sod(strata, shafts, &self.scars, dt);
        let still_active = !settled_cells.is_empty() || scars_active || blocks_active;
        regrown.extend(sod_cells);
        regrown.extend(settled_cells);
        (regrown, still_active)
    }

    pub fn capture_sector(&self, gsu: i32, gsv: i32) -> (u64, Vec<crate::SnapshotBlock>, Vec<crate::RigSnapshot>) {
        SectorMoldsView::new(&self.world_blocks, &self.rigs, &self.scars).capture(gsu, gsv)
    }

    pub fn apply_sector(&mut self, strata: &NaturalStrata, gsu: i32, gsv: i32, bare_mask: u64, cols: &crate::SectorColumnState) {
        SectorMoldsTarget::new(&mut self.world_blocks, &mut self.rigs, &mut self.scars)
            .apply(strata, gsu, gsv, bare_mask, cols);
    }

    pub fn collect_modified_sectors(&self, out: &mut Vec<(i32, i32)>) {
        SectorMoldsView::new(&self.world_blocks, &self.rigs, &self.scars).collect_modified_sectors(out);
    }
}
