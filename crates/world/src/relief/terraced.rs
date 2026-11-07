use glam::Vec3;
use crate::MeshTarget;
use crate::block::{BlockMedium, GridCell, VoxelShape};
use crate::frame::SurfaceFrame;
use super::column_clearance::ColumnClearance;
use super::elevation::ReliefElevation;
use super::natural_strata::NaturalStrata;
use super::overhang_molds::OverhangMolds;
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

    fn seat_backrest_cell(&self, u: i32, v: i32, layer_elev: f32) -> Option<(i32, i32)> {
        self.molds.seat_backrest_cell(u, v, layer_elev)
    }

    fn hit_top_at(&self, u: f32, v: f32, ray_elev: f32) -> Option<f32> {
        self.clearance().hit_top_at(u, v, ray_elev)
    }

    fn reworked_at(&self, u: i32, v: i32) -> bool {
        self.shafts.reworked_at(u, v)
    }

    fn breached_at(&self, u: i32, v: i32) -> bool {
        self.shafts.breached_at(&self.strata, u, v)
    }

    fn crust_floor_at(&self, u: i32, v: i32) -> i32 {
        self.shafts.crust_floor_at(&self.strata, u, v)
    }

    fn ceiling_inset(&self, u: i32, v: i32, margin: f32) -> f32 {
        if self.breached_at(u, v) || self.shafts.upward_at(u, v) > 0 || self.reworked_at(u, v) || self.molds.present_at(u, v) {
            0.0
        } else {
            self.strata.vault().ceiling_inset(margin)
        }
    }

    fn weightless_at(&self, u: f32, v: f32, elev: f32) -> bool {
        self.clearance().weightless_at(u, v, elev)
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

    fn emit_molded_top(&self, cell: GridCell, frame: &SurfaceFrame, top_col: Vec3, wall_col: Vec3, target: &mut dyn MeshTarget) -> bool {
        let (u, v) = cell.feed_coordinates(|u, v| (u, v));
        let surf_base = ((self.natural_elevation_at(u, v) - 1) as f32 * 0.5).max((self.elevation_at(u, v) as f32) * 0.5);
        self.molds.emit_molded_top(cell, surf_base, frame, (top_col, wall_col), target)
    }

    fn step_grass(&mut self, dt: f32) -> Vec<(i32, i32)> {
        self.molds.step_grass(&self.strata, &self.shafts, dt)
    }

    fn layer_dig_stage_at(&self, u: i32, v: i32, layer_elev: f32) -> u8 {
        self.molds.layer_dig_stage_at(u, v, layer_elev)
    }

    fn clear_dig_stage(&mut self) -> ((i32, i32), bool) {
        self.molds.reset_stage()
    }

    fn mark_dig_layer(&mut self, u: i32, v: i32, layer_elev: f32, stage: u8) -> ((i32, i32), Option<BlockMedium>) {
        if stage > 0 && !self.shafts.diggable_at(&self.strata, u, v, layer_elev) && !self.molds.present_at(u, v) {
            return ((u, v), None);
        }
        let medium = self.molds.layer_medium_at(&self.strata, &self.shafts, u, v, layer_elev);
        if medium.instant_pickup() {
            return ((u, v), (stage > 0).then_some(medium));
        }
        let (prev, changed) = self.molds.record_stage(u, v, layer_elev, stage);
        (prev, changed.then_some(medium))
    }

    fn harvest_layer(&mut self, u: i32, v: i32, layer_elev: f32) -> ((i32, i32), Option<BlockMedium>) {
        let ground_top = (self.elevation_at(u, v) as f32) * 0.5;
        if let Some((prev_cell, medium)) = self.molds.remove_layer(u, v, layer_elev, ground_top) {
            return (prev_cell, Some(medium));
        }
        self.shafts.excavate_layer(&self.strata, &mut self.molds, u, v, layer_elev)
    }

    fn stack_molded_cell(&mut self, u: i32, v: i32, base: f32, shape: Box<dyn VoxelShape>) -> bool {
        let ground_top = (self.elevation_at(u, v) as f32) * 0.5;
        let sub = !self.breached_at(u, v) && base < (self.crust_floor_at(u, v) as f32) * 0.5 + 0.1;
        self.molds.stack_molded(u, v, ground_top, sub, base, shape)
    }

    fn adjust_rig_at(&mut self, u: i32, v: i32, delta_steps: i32, rotate_quarter: bool) -> (bool, (i32, i32)) {
        self.molds.adjust_rig(&self.strata, &self.shafts, u, v, delta_steps, rotate_quarter)
    }

    fn placed_camera_tripod(&self, frame: &SurfaceFrame) -> Option<(Vec3, Vec3, Vec3)> {
        self.molds.placed_camera_tripod(frame)
    }

    fn lower_rig_at(&mut self, u: i32, v: i32) -> bool {
        self.molds.lower_rig(&self.strata, &self.shafts, u, v)
    }

    fn board_vehicle_at(
        &mut self, frame: &SurfaceFrame, u: i32, v: i32, layer_elev: f32, ray: SurfaceSightRay, engage_engine: bool,
    ) -> Option<VehicleAssembly> {
        self.molds.board_vehicle(&self.strata, &self.shafts, frame, u, v, layer_elev, ray, engage_engine)
    }

    fn park_vehicle_at(&mut self, frame: &SurfaceFrame, assembly: VehicleAssembly) -> (i32, i32) {
        self.molds.park_vehicle(&self.strata, &self.shafts, frame, assembly)
    }
}
