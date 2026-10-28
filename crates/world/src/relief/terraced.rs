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

    /// Secondary constructor for a 128-block baseline relief face.
    pub fn pristine(origin: Box<dyn ReliefElevation>) -> Self {
        Self::in_span(origin, 128)
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
        let (cell_u, cell_v) = ((u * 2.0).floor() as i32, (v * 2.0).floor() as i32);
        let vault = self.strata.vault();
        let crest = vault.core().crest_at(u, v);
        let center_elev = foot_elev + body_span.min(1.0);
        let ceil_limit = if self.shafts.upward_at(cell_u, cell_v) > 0 {
            (self.crust_floor_at(cell_u, cell_v) as f32) * 0.5
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
        self.molds.layer_medium_at(u, v, layer_elev)
    }

    fn hit_top_at(&self, u: f32, v: f32, ray_elev: f32) -> Option<f32> {
        let (cu, cv) = ((u * 2.0).floor() as i32, (v * 2.0).floor() as i32);
        let (ground_h, ceil_h) = ((self.elevation_at(cu, cv) as f32) * 0.5, (self.crust_floor_at(cu, cv) as f32) * 0.5);
        if let Some(mold_top) = self.molds.hit_top_at(cu, cv, u, v, ray_elev, ground_h) {
            return Some(mold_top);
        }
        let ground_surf = if self.reworked_at(cu, cv) { ground_h } else { self.strata.surface_height_at(u, v) };
        let ceil_floor = if self.shafts.upward_at(cu, cv) > 0 {
            ceil_h - 0.15
        } else {
            (self.strata.vault().ceiling_height_at(u, v) - 0.55).min(ground_h - 0.55)
        };
        if !self.breached_at(cu, cv) && ray_elev >= ceil_floor && ray_elev <= ground_surf + 0.02 {
            let hit = if ray_elev < ceil_h - 0.05 {
                ceil_h - 0.25
            } else if ray_elev < ground_h - 0.50 {
                ((ray_elev * 2.0).floor() * 0.5 + 0.5).min(ground_h)
            } else {
                ground_h
            };
            Some(hit)
        } else {
            None
        }
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
        let (cu, cv) = ((u * 2.0).floor() as i32, (v * 2.0).floor() as i32);
        let in_upward = self.shafts.in_upward_shaft(&self.strata, cu, cv, elev);
        !self.shafts.in_unbreached_shaft(&self.strata, cu, cv, elev)
            && if in_upward { self.shafts.upward_at(cu, cv) < 4 } else { self.strata.vault().encloses(u, v, elev, 0.10) }
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

    fn emit_molded_top(&self, cell: GridCell, _query_base: f32, frame: &SurfaceFrame, top_col: Vec3, wall_col: Vec3, target: &mut dyn MeshTarget) -> bool {
        let (u, v) = cell.feed_coordinates(|u, v| (u, v));
        let surf_base = ((self.natural_elevation_at(u, v) - 1) as f32 * 0.5).max((self.elevation_at(u, v) as f32) * 0.5);
        self.molds.emit_molded_top(cell, surf_base, frame, top_col, wall_col, target)
    }

    fn step_grass(&mut self, dt: f32) -> Vec<i32> {
        self.molds.step_grass(&self.strata, &self.shafts, dt)
    }

    fn dig_stage_at(&self, u: i32, v: i32) -> u8 {
        self.molds.dig_stage_at(u, v)
    }

    fn mark_dig_stage(&mut self, u: i32, v: i32, stage: u8) -> (i32, bool) {
        self.mark_dig_layer(u, v, self.top_layer_at(u, v), stage)
    }

    fn mark_dig_layer(&mut self, u: i32, v: i32, layer_elev: f32, stage: u8) -> (i32, bool) {
        let up_dig = layer_elev < (self.crust_floor_at(u, v).max(self.shafts.adjacent_upward_at(&self.strata, u, v)) as f32) * 0.5 + 0.25;
        if stage > 0 && (self.breached_at(u, v) || (self.shafts.deepened_steps(&self.strata, u, v) == self.elevation_at(u, v) && !up_dig)) && !self.molds.present_at(u, v) {
            return (v, false);
        }
        self.molds.record_stage(u, v, layer_elev, stage)
    }

    fn excavate_cell(&mut self, u: i32, v: i32) -> (i32, bool) {
        self.excavate_layer(u, v, self.top_layer_at(u, v))
    }

    fn excavate_layer(&mut self, u: i32, v: i32, layer_elev: f32) -> (i32, bool) {
        if let Some(prev_v) = self.molds.remove_layer(u, v, layer_elev) {
            return (prev_v, true);
        }
        let (crust, adj_up, nat) = (self.crust_floor_at(u, v), self.shafts.adjacent_upward_at(&self.strata, u, v), self.natural_elevation_at(u, v));
        if layer_elev < (crust as f32) * 0.5 {
            if self.breached_at(u, v) || (adj_up <= crust && !self.strata.vault().cell_opens(u, v)) {
                return (v, false);
            }
            let next_crust = crust + 1;
            if next_crust >= self.elevation_at(u, v) {
                self.shafts.record_delta(u, v, self.strata.vault().core().summit_steps().min(crust - 2) - nat);
                self.molds.expose_dirt(u, v);
            } else {
                self.shafts.record_ceiling(u, v, next_crust - self.strata.vault().floor_steps_at(u, v));
            }
            return (self.molds.clear_stage(u, v), true);
        }
        let (elev, step_at) = (self.elevation_at(u, v), ((layer_elev - 0.25) * 2.0).round() as i32);
        let next_elev = if step_at < elev - 1 { step_at.max(crust) } else { self.shafts.deepened_steps(&self.strata, u, v) };
        if next_elev == elev {
            return (v, false);
        }
        if step_at < elev - 1 {
            self.molds.retain_overhang(u, v, next_elev, elev);
        } else if !self.molds.present_at(u, v) {
            self.molds.expose_dirt(u, v);
        }
        self.shafts.record_delta(u, v, next_elev - nat);
        (self.molds.clear_stage(u, v), true)
    }

    fn stack_cell(&mut self, u: i32, v: i32) -> bool {
        self.stack_molded_cell(u, v, (self.elevation_at(u, v) as f32) * 0.5, Box::new(CuboidShape::new(0.5)), true)
    }

    fn stack_molded_cell(&mut self, u: i32, v: i32, base: f32, shape: Box<dyn VoxelShape>, _full_flat: bool) -> bool {
        let ground_top = (self.elevation_at(u, v) as f32) * 0.5;
        let sub = !self.breached_at(u, v) && base < (self.crust_floor_at(u, v) as f32) * 0.5 + 0.1;
        self.molds.stack_molded(u, v, ground_top, sub, base, shape)
    }
}
