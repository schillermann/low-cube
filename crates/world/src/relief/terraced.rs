use glam::Vec3;
use crate::MeshTarget;
use crate::block::{BlockMedium, CuboidShape, GridCell, VoxelShape};
use crate::frame::SurfaceFrame;
use super::elevation::ReliefElevation;
use super::natural_strata::NaturalStrata;
use super::overhang_molds::OverhangMolds;
use super::shaft_excavation::ShaftExcavation;

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

    fn top_layer_at(&self, u: i32, v: i32) -> f32 {
        let ground_top = (self.elevation_at(u, v) as f32) * 0.5;
        self.molds.top_layer_at(u, v, ground_top)
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
        let (cell_u, cell_v) = GridCell::at_planar(u, v).feed_coordinates(|cu, cv| (cu, cv));
        let vault = self.strata.vault();
        let crest = vault.core().crest_at(u, v);
        let center_elev = foot_elev + body_span.min(1.0);
        let ceil_limit = if self.shafts.upward_at(cell_u, cell_v) > 0 {
            let top = if self.breached_at(cell_u, cell_v) { self.natural_elevation_at(cell_u, cell_v) } else { self.crust_floor_at(cell_u, cell_v) };
            (top as f32) * 0.5
        } else {
            vault.ceiling_height_at(u, v) + 0.5
        };
        let base_h = if !self.shafts.in_unbreached_shaft(&self.strata, cell_u, cell_v, center_elev)
            && body_span > 0.0
            && !self.lethal_at(u, v, center_elev)
            && (vault.encloses(u, v, center_elev, 0.0) || self.shafts.in_upward_shaft(&self.strata, cell_u, cell_v, center_elev))
            && center_elev < ceil_limit
        {
            crest
        } else if self.shafts.pristine() && self.molds.empty() {
            return self.strata.surface_height_at(u, v);
        } else if self.reworked_at(cell_u, cell_v) {
            let steps = if body_span > 0.0 && !self.molds.present_at(cell_u, cell_v) {
                self.shafts.body_elevation_at(&self.strata, cell_u, cell_v)
            } else {
                self.elevation_at(cell_u, cell_v)
            };
            (steps as f32) * 0.5
        } else {
            self.strata.surface_height_at(u, v)
        };
        self.molds.constrain_support(cell_u, cell_v, u, v, base_h, foot_elev, body_span)
    }

    fn layer_occupied_at(&self, u: i32, v: i32, layer_elev: f32) -> bool {
        self.molds.layer_occupied_at(u, v, layer_elev)
    }

    fn layer_medium_at(&self, u: i32, v: i32, layer_elev: f32) -> BlockMedium {
        self.molds.layer_medium_at(&self.strata, u, v, layer_elev)
    }

    fn hit_top_at(&self, u: f32, v: f32, ray_elev: f32) -> Option<f32> {
        let (cu, cv) = GridCell::at_planar(u, v).feed_coordinates(|cu, cv| (cu, cv));
        self.molds
            .hit_top_at(cu, cv, u, v, ray_elev)
            .or_else(|| self.shafts.crust_hit_top_at(&self.strata, cu, cv, u, v, ray_elev))
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
        let (cu, cv) = GridCell::at_planar(u, v).feed_coordinates(|cu, cv| (cu, cv));
        let in_upward = self.shafts.in_upward_shaft(&self.strata, cu, cv, elev);
        !self.shafts.in_unbreached_shaft(&self.strata, cu, cv, elev)
            && (in_upward || self.strata.vault().encloses(u, v, elev, 0.10))
            && !self.lethal_at(u, v, elev)
    }

    fn lethal_at(&self, u: f32, v: f32, elev: f32) -> bool {
        self.strata.vault().core().engulfs(u, v, elev, 1.05)
    }

    fn confine(&self, u: f32, v: f32, elev: f32) -> (f32, f32, f32) {
        self.shafts.confine(&self.strata, &self.molds, u, v, elev)
    }

    fn bare_dirt_at(&self, u: i32, v: i32) -> bool {
        self.molds.bare_dirt_at(u, v)
    }

    fn emit_molded_top(&self, cell: GridCell, frame: &SurfaceFrame, top_col: Vec3, wall_col: Vec3, target: &mut dyn MeshTarget) -> bool {
        let (u, v) = cell.feed_coordinates(|u, v| (u, v));
        let surf_base = ((self.natural_elevation_at(u, v) - 1) as f32 * 0.5).max((self.elevation_at(u, v) as f32) * 0.5);
        self.molds.emit_molded_top(cell, surf_base, &self.strata.vault(), frame, (top_col, wall_col), target)
    }

    fn step_grass(&mut self, dt: f32) -> Vec<(i32, i32)> {
        self.molds.step_grass(&self.strata, &self.shafts, dt)
    }

    fn layer_dig_stage_at(&self, u: i32, v: i32, layer_elev: f32) -> u8 {
        self.molds.layer_dig_stage_at(u, v, layer_elev, (self.elevation_at(u, v) as f32) * 0.5 - 0.25)
    }

    fn clear_dig_stage(&mut self) -> ((i32, i32), bool) {
        self.molds.reset_stage()
    }

    fn mark_dig_stage(&mut self, u: i32, v: i32, stage: u8) -> ((i32, i32), bool) {
        self.mark_dig_layer(u, v, self.top_layer_at(u, v), stage)
    }

    fn mark_dig_layer(&mut self, u: i32, v: i32, layer_elev: f32, stage: u8) -> ((i32, i32), bool) {
        if stage > 0 && !self.shafts.diggable_at(&self.strata, u, v, layer_elev) && !self.molds.present_at(u, v) {
            return ((u, v), false);
        }
        self.molds.record_stage(u, v, layer_elev, stage)
    }

    fn excavate_cell(&mut self, u: i32, v: i32) -> ((i32, i32), bool) {
        self.excavate_layer(u, v, self.top_layer_at(u, v))
    }

    fn excavate_layer(&mut self, u: i32, v: i32, layer_elev: f32) -> ((i32, i32), bool) {
        if let Some(prev_cell) = self.molds.remove_layer(u, v, layer_elev) {
            return (prev_cell, true);
        }
        self.shafts.excavate_layer(&self.strata, &mut self.molds, u, v, layer_elev)
    }

    fn stack_cell(&mut self, u: i32, v: i32) -> bool {
        self.stack_molded_cell(u, v, (self.elevation_at(u, v) as f32) * 0.5, Box::new(CuboidShape::new(0.5)))
    }

    fn stack_molded_cell(&mut self, u: i32, v: i32, base: f32, shape: Box<dyn VoxelShape>) -> bool {
        let ground_top = (self.elevation_at(u, v) as f32) * 0.5;
        let sub = !self.breached_at(u, v) && base < (self.crust_floor_at(u, v) as f32) * 0.5 + 0.1;
        self.molds.stack_molded(u, v, ground_top, sub, base, shape)
    }
}
