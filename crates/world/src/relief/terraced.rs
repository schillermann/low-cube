use glam::Vec3;
use crate::MeshTarget;
use crate::block::{BlockMedium, GridCell, VoxelShape};
use crate::frame::SurfaceFrame;
use super::column_clearance::ColumnClearance;
use super::craft_entity::{CraftBlueprint, VesselSink};
use super::elevation::ReliefElevation;
use super::mineral_vein::{ExposedStratum, SubterraneanVeins};
use super::natural_strata::NaturalStrata;
use super::overhang_molds::OverhangMolds;
use super::planned_mount::MountDomain;
use super::shaft_excavation::ShaftExcavation;
use super::sight_ray::SurfaceSightRay;
use super::vehicle_assembly::VehicleAssembly;

/// Decorator composing natural strata, excavated shafts, and overhanging molds atop planetary relief.
pub struct TerracedRelief {
    strata: NaturalStrata,
    shafts: ShaftExcavation,
    molds: OverhangMolds,
}

impl TerracedRelief {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(strata: NaturalStrata, shafts: ShaftExcavation, molds: OverhangMolds) -> Self {
        Self { strata, shafts, molds }
    }

    /// Secondary constructor sizing natural strata for a face with `half_blocks` radius.
    pub fn in_span(origin: Box<dyn ReliefElevation>, half_blocks: i32) -> Self {
        Self::new(
            NaturalStrata::in_span(origin, half_blocks),
            ShaftExcavation::untouched(),
            OverhangMolds::untouched(),
        )
    }

    /// Secondary constructor equipping explicit subterranean vein authority.
    pub fn with_veins(origin: Box<dyn ReliefElevation>, half_blocks: i32, veins: Box<dyn SubterraneanVeins>) -> Self {
        Self::new(
            NaturalStrata::in_span(origin, half_blocks),
            ShaftExcavation::with_veins(veins),
            OverhangMolds::untouched(),
        )
    }

    const fn clearance(&self) -> ColumnClearance<'_> {
        ColumnClearance::new(&self.strata, &self.shafts, &self.molds)
    }
}

impl ReliefElevation for TerracedRelief {
    fn natural_elevation_at(&self, u: i32, v: i32) -> i32 {
        self.strata.elevation_at(u, v)
    }

    fn elevation_at(&self, u: i32, v: i32) -> i32 {
        self.shafts.elevation_at(&self.strata, u, v)
    }

    fn surface_height_at(&self, u: f32, v: f32) -> f32 {
        self.support_height_at(u, v, 0.0, 0.0)
    }

    fn support_height_at(&self, u: f32, v: f32, foot_elev: f32, body_span: f32) -> f32 {
        self.clearance().support_height_at(u, v, foot_elev, body_span)
    }

    fn layer_occupied_at(&self, u: i32, v: i32, layer_elev: f32) -> bool {
        self.molds.layer_occupied_at(u, v, layer_elev)
    }

    fn boom_slot_at(&self, u: i32, v: i32, layer_elev: f32) -> bool {
        self.molds.boom_slot_at(u, v, layer_elev)
    }

    fn effective_placement_cell(&self, u: i32, v: i32, base: f32) -> Option<(i32, i32, f32)> {
        let ground_top = (self.elevation_at(u, v) as f32) * 0.5;
        let sub = base < (self.crust_floor_at(u, v) as f32) * 0.5 + 0.1;
        self.molds.effective_placement_cell(u, v, ground_top, sub, base)
    }

    fn resolve_planned_site(&self, u: i32, v: i32, base: f32, medium: BlockMedium) -> Option<(GridCell, f32, MountDomain)> {
        let ground_top = (self.elevation_at(u, v) as f32) * 0.5;
        let sub = base < (self.crust_floor_at(u, v) as f32) * 0.5 + 0.1;
        self.molds.resolve_planned_site(u, v, ground_top, sub, base, medium)
    }

    fn present_vessels(&self, sink: &mut dyn VesselSink) {
        self.molds.present_vessels(sink);
    }

    fn seat_backrest_cell(&self, u: i32, v: i32, layer_elev: f32) -> Option<(i32, i32)> {
        self.molds.seat_backrest_cell(u, v, layer_elev)
    }

    fn hit_top_at(&self, u: f32, v: f32, ray_elev: f32) -> Option<f32> {
        self.clearance().hit_top_at(u, v, ray_elev)
    }

    fn reworked_at(&self, u: i32, v: i32) -> bool {
        self.shafts.reworked_at(u, v)
    }

    fn crust_floor_at(&self, u: i32, v: i32) -> i32 {
        self.shafts.crust_floor_at(&self.strata, u, v)
    }

    fn lethal_at(&self, u: f32, v: f32, elev: f32) -> bool {
        self.clearance().lethal_at(u, v, elev)
    }

    fn confine(&self, u: f32, v: f32, elev: f32) -> (f32, f32, f32) {
        self.clearance().confine(u, v, elev)
    }

    fn bare_dirt_at(&self, u: i32, v: i32) -> bool {
        self.molds.bare_dirt_at(u, v)
    }

    fn ground_bare_at(&self, u: i32, v: i32) -> bool {
        let ground_top = (self.elevation_at(u, v) as f32) * 0.5;
        self.molds.ground_bare_at(u, v, ground_top)
    }

    fn emit_molded_top(&self, cell: GridCell, frame: &SurfaceFrame, top_col: Vec3, wall_col: Vec3, target: &mut dyn MeshTarget) -> bool {
        self.molds.emit_molded_top(cell, frame, (top_col, wall_col), target)
    }

    fn step_grass(&mut self, dt: f32) -> Vec<(i32, i32)> {
        self.molds.step_grass(&self.strata, &self.shafts, dt)
    }

    fn layer_dig_stage_at(&self, u: i32, v: i32, layer_elev: f32) -> u8 {
        self.molds.layer_dig_stage_at(u, v, layer_elev)
    }

    fn mark_dig_layer(&mut self, u: i32, v: i32, layer_elev: f32, stage: u8) -> Option<BlockMedium> {
        if stage > 0
            && (!self.shafts.diggable_at(&self.strata, u, v, layer_elev) && !self.molds.present_at(u, v)
                || self.molds.car_locked_at(u, v, layer_elev))
        {
            return None;
        }
        let medium = self.molds.layer_medium_at(&self.strata, &self.shafts, u, v, layer_elev);
        if medium.instant_pickup() {
            return (stage > 0).then_some(medium);
        }
        self.molds.record_stage(u, v, layer_elev, stage).then_some(medium)
    }

    fn harvest_layer(&mut self, u: i32, v: i32, layer_elev: f32) -> Option<BlockMedium> {
        if self.molds.car_locked_at(u, v, layer_elev) {
            return None;
        }
        self.molds
            .remove_layer(&self.strata, &self.shafts, u, v, layer_elev)
            .or_else(|| self.shafts.excavate_layer(&self.strata, &mut self.molds, u, v, layer_elev))
    }

    fn stack_molded_cell(&mut self, u: i32, v: i32, base: f32, shape: Box<dyn VoxelShape>) -> bool {
        let ground_top = (self.elevation_at(u, v) as f32) * 0.5;
        let sub = base < (self.crust_floor_at(u, v) as f32) * 0.5 + 0.1;
        self.molds.stack_molded(u, v, ground_top, sub, base, shape)
    }

    fn adjust_rig_at(&mut self, u: i32, v: i32, delta_steps: i32, rotate_quarter: bool) -> (bool, (i32, i32)) {
        self.molds.adjust_rig(&self.strata, &self.shafts, u, v, delta_steps, rotate_quarter)
    }

    fn placed_camera_tripod(&self, frame: &SurfaceFrame) -> Option<(Vec3, Vec3, Vec3)> {
        self.molds.placed_camera_tripod(frame)
    }

    fn collect_spawn_points(&self, frame: &SurfaceFrame, out: &mut Vec<Vec3>) {
        self.molds.collect_spawn_points(frame, out);
    }

    fn lower_rig_at(&mut self, u: i32, v: i32) -> bool {
        self.molds.lower_rig(&self.strata, &self.shafts, u, v)
    }

    fn board_vehicle_on_face(
        &mut self, frame: &SurfaceFrame, u: i32, v: i32, layer_elev: f32, ray: SurfaceSightRay, engage_engine: bool,
    ) -> (Option<VehicleAssembly>, bool) {
        self.molds.board_vehicle(&self.strata, &self.shafts, frame, u, v, layer_elev, ray, engage_engine)
    }

    fn board_vehicle_at(
        &mut self, frame: &SurfaceFrame, u: i32, v: i32, layer_elev: f32, ray: SurfaceSightRay, engage_engine: bool,
    ) -> Option<VehicleAssembly> {
        self.board_vehicle_on_face(frame, u, v, layer_elev, ray, engage_engine).0
    }

    fn park_vehicle_at(&mut self, frame: &SurfaceFrame, assembly: VehicleAssembly) -> (i32, i32) {
        self.molds.park_vehicle(&self.strata, &self.shafts, frame, assembly)
    }

    fn capture_blueprint_at(
        &self,
        frame: &SurfaceFrame,
        u: i32,
        v: i32,
        layer_elev: f32,
        ray: SurfaceSightRay,
    ) -> Option<CraftBlueprint> {
        self.molds.capture_blueprint(frame, u, v, layer_elev, ray)
    }

    fn aimed_empty_rig_at(&self, frame: &SurfaceFrame, u: i32, v: i32, base: f32, ray: SurfaceSightRay) -> bool {
        let ground_top = (self.elevation_at(u, v) as f32) * 0.5;
        let sub = base < (self.crust_floor_at(u, v) as f32) * 0.5 + 0.1;
        self.molds.aimed_empty_rig(u, v, ground_top, sub, base, ray, frame)
    }

    fn build_rig_blueprint_at(
        &mut self,
        frame: &SurfaceFrame,
        u: i32,
        v: i32,
        base: f32,
        ray: SurfaceSightRay,
        blueprint: &CraftBlueprint,
    ) -> Option<(i32, i32)> {
        let ground_top = (self.elevation_at(u, v) as f32) * 0.5;
        let sub = base < (self.crust_floor_at(u, v) as f32) * 0.5 + 0.1;
        self.molds
            .build_rig_blueprint(&self.strata, &self.shafts, frame, u, v, ground_top, sub, base, ray, blueprint)
    }

    fn exposed_ore_at(&self, u: i32, v: i32, step: i32) -> Option<BlockMedium> {
        self.shafts.exposed_ore_at(u, v, step)
    }

    fn accept_disclosure(&mut self, stratum: ExposedStratum) {
        self.shafts.accept_disclosure(stratum);
    }

    fn drain_disclosures(&mut self, out: &mut Vec<ExposedStratum>) {
        self.shafts.drain_disclosures(out);
    }
}
