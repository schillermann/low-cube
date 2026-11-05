mod decorators;

use crate::block::GridCell;
use super::natural_strata::NaturalStrata;
use super::overhang_molds::OverhangMolds;
use super::shaft_excavation::ShaftExcavation;
use decorators::{MoldedColumn, NaturalColumn, ShaftColumn};

pub(super) const BLOCK_STEP: f32 = GridCell::SPAN;
pub(super) const SHAFT_FLOOR_TOLERANCE: f32 = 0.1;
pub(super) const CEILING_CLEARANCE_INSET: f32 = 0.30;
const NEIGHBOR_ESCAPE_MARGIN: f32 = 0.02;
const SHAFT_WALL_MARGIN: f32 = 0.21;
pub(super) const UPWARD_CEILING_CATCH_SPAN: f32 = 2.0;
pub(super) const UPWARD_CEILING_FLOOR_INSET: f32 = 0.15;
pub(super) const VAULT_CEILING_FLOOR_INSET: f32 = 0.55;
pub(super) const SURFACE_HIT_TOLERANCE: f32 = 0.02;
pub(super) const LETHAL_CORE_MARGIN: f32 = 1.05;

/// Polymorphic 3D vertical column contract decorated by natural strata, shafts, and overhanging molds.
pub trait VerticalColumn {
    fn support_height(&self, u: f32, v: f32, foot_elev: f32, body_span: f32) -> f32;
    fn confine_elevation(&self, cell: GridCell, u: f32, v: f32, elev: f32) -> f32;
    fn hit_top(&self, cell: GridCell, u: f32, v: f32, ray_elev: f32) -> Option<f32>;
}

/// Unified 3D vertical column topology composing the `MoldedColumn(ShaftColumn(NaturalColumn))` decorator chain.
pub struct ColumnClearance<'a> {
    strata: &'a NaturalStrata,
    shafts: &'a ShaftExcavation,
    molds: &'a OverhangMolds,
}

impl<'a> ColumnClearance<'a> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(strata: &'a NaturalStrata, shafts: &'a ShaftExcavation, molds: &'a OverhangMolds) -> Self {
        Self { strata, shafts, molds }
    }

    fn decorated_column(&self, cu: i32, cv: i32) -> MoldedColumn<'a, ShaftColumn<'a, NaturalColumn<'a>>> {
        let natural = NaturalColumn::new(self.strata);
        let shaft = ShaftColumn::new(natural, self.strata, self.shafts, self.molds.present_at(cu, cv));
        MoldedColumn::new(shaft, self.strata, self.shafts, self.molds)
    }

    pub fn lethal_at(&self, u: f32, v: f32, elev: f32) -> bool {
        let (cu, cv) = GridCell::at_planar(u, v).feed_coordinates(|cu, cv| (cu, cv));
        let core_step = self.strata.vault().floor_steps_at(cu, cv);
        let core_top = (core_step as f32) * BLOCK_STEP;
        self.strata.vault().core().engulfs(u, v, elev, LETHAL_CORE_MARGIN)
            || elev < core_top - SHAFT_FLOOR_TOLERANCE
            || (elev <= core_top + LETHAL_CORE_MARGIN
                && self.shafts.elevation_at(self.strata, cu, cv) <= core_step)
    }

    pub fn weightless_at(&self, _u: f32, _v: f32, _elev: f32) -> bool {
        false
    }

    pub fn support_height_at(&self, u: f32, v: f32, foot_elev: f32, body_span: f32) -> f32 {
        let (cu, cv) = GridCell::at_planar(u, v).feed_coordinates(|cu, cv| (cu, cv));
        self.decorated_column(cu, cv).support_height(u, v, foot_elev, body_span)
    }

    fn solid_wall_at(&self, cu: i32, cv: i32, elev: f32) -> bool {
        if self.shafts.breached_at(self.strata, cu, cv) {
            return self.molds.layer_occupied_at(cu, cv, elev);
        }
        let ground = (self.shafts.elevation_at(self.strata, cu, cv) as f32) * BLOCK_STEP;
        let ceiling = if self.shafts.upward_at(cu, cv) > 0 {
            (self.shafts.crust_floor_at(self.strata, cu, cv) as f32) * BLOCK_STEP - CEILING_CLEARANCE_INSET
        } else {
            let (cx, cy) = GridCell::new(cu, cv).center();
            self.strata.vault().ceiling_height_at(cx, cy) - CEILING_CLEARANCE_INSET
        };
        self.molds.solid_wall_at(cu, cv, ground, ceiling, elev)
    }

    pub fn confine(&self, u: f32, v: f32, elev: f32) -> (f32, f32, f32) {
        let cell = GridCell::at_planar(u, v);
        let (mut cu, mut cv) = cell.feed_coordinates(|cu, cv| (cu, cv));
        if self.shafts.pristine() && self.shafts.ceilings_untouched() && self.molds.empty() {
            return (u, v, NaturalColumn::new(self.strata).confine_elevation(cell, u, v, elev));
        }
        let natural_steps = self.strata.elevation_at(cu, cv);
        let natural_top = (natural_steps as f32) * BLOCK_STEP;
        if self.shafts.ceilings_untouched() && self.molds.empty() && elev >= natural_top {
            return (u, v, elev);
        }
        let (mut tu, mut tv) = (u, v);
        let col_top = (self.molds.top_steps_at(cu, cv, natural_steps) as f32) * BLOCK_STEP;
        let enclosed = elev < col_top;
        let ground = (self.shafts.elevation_at(self.strata, cu, cv) as f32) * BLOCK_STEP;
        let probe_elev = self
            .molds
            .confine_niche_elevation(cu, cv, ground, elev)
            .filter(|_| elev >= ground - SHAFT_FLOOR_TOLERANCE)
            .unwrap_or(elev);
        let in_own_upward = self.shafts.upward_at(cu, cv) > 0
            && !self.shafts.breached_at(self.strata, cu, cv)
            && !self.molds.layer_occupied_at(cu, cv, probe_elev);
        if enclosed && !in_own_upward && self.solid_wall_at(cu, cv, probe_elev) {
            for (du, dv) in [(-1, 0), (1, 0), (0, -1), (0, 1), (-1, -1), (1, -1), (-1, 1), (1, 1)] {
                let (nu, nv) = (cu + du, cv + dv);
                if (self.shafts.upward_at(nu, nv) > 0 || self.shafts.breached_at(self.strata, nu, nv) || self.molds.present_at(nu, nv))
                    && !self.solid_wall_at(nu, nv, probe_elev)
                {
                    (tu, tv) = GridCell::new(nu, nv).clamp_interior(u, v, NEIGHBOR_ESCAPE_MARGIN);
                    (cu, cv) = (nu, nv);
                    break;
                }
            }
        }
        let near_upward = self.shafts.upward_at(cu, cv) > 0
            || self.shafts.adjacent_upward_at(self.strata, cu, cv) > i32::MIN
            || self.molds.present_at(cu, cv);
        if enclosed && near_upward && (in_own_upward || !self.solid_wall_at(cu, cv, probe_elev)) {
            let walls = (
                self.solid_wall_at(cu - 1, cv, probe_elev),
                self.solid_wall_at(cu + 1, cv, probe_elev),
                self.solid_wall_at(cu, cv - 1, probe_elev),
                self.solid_wall_at(cu, cv + 1, probe_elev),
            );
            (tu, tv) = GridCell::new(cu, cv).clamp_walled(tu, tv, SHAFT_WALL_MARGIN, walls);
        }
        let z = self.decorated_column(cu, cv).confine_elevation(GridCell::new(cu, cv), tu, tv, elev);
        (tu, tv, z)
    }

    pub fn hit_top_at(&self, u: f32, v: f32, ray_elev: f32) -> Option<f32> {
        let cell = GridCell::at_planar(u, v);
        let (cu, cv) = cell.feed_coordinates(|cu, cv| (cu, cv));
        self.decorated_column(cu, cv).hit_top(cell, u, v, ray_elev)
    }
}
