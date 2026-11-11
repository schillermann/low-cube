use crate::block::GridCell;
use super::super::natural_strata::NaturalStrata;
use super::super::overhang_molds::OverhangMolds;
use super::super::shaft_excavation::ShaftExcavation;
use super::{
    BLOCK_STEP, FLOOR_HIT_INSET, SHAFT_FLOOR_TOLERANCE, SURFACE_HIT_TOLERANCE, VerticalColumn,
};

/// Base vertical column backed by natural planetary strata.
pub struct NaturalColumn<'a> {
    strata: &'a NaturalStrata,
}

impl<'a> NaturalColumn<'a> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(strata: &'a NaturalStrata) -> Self {
        Self { strata }
    }
}

impl VerticalColumn for NaturalColumn<'_> {
    fn support_height(&self, u: f32, v: f32, _foot_elev: f32, _body_span: f32) -> f32 {
        self.strata.surface_height_at(u, v)
    }

    fn confine_elevation(&self, _cell: GridCell, _u: f32, _v: f32, elev: f32) -> f32 {
        elev
    }

    fn hit_top(&self, cell: GridCell, u: f32, v: f32, ray_elev: f32) -> Option<f32> {
        let (cu, cv) = cell.feed_coordinates(|cu, cv| (cu, cv));
        let ground_h = (self.strata.elevation_at(cu, cv) as f32) * BLOCK_STEP;
        let floor_h = (self.strata.vault().floor_steps_at(cu, cv) as f32) * BLOCK_STEP;
        let ground_surf = self.strata.surface_height_at(u, v);
        let floor_bound = (floor_h - FLOOR_HIT_INSET).min(ground_h - FLOOR_HIT_INSET);
        if ray_elev >= floor_bound && ray_elev <= ground_surf + SURFACE_HIT_TOLERANCE {
            let top_layer_floor = (ground_h - BLOCK_STEP).min(ground_surf - SHAFT_FLOOR_TOLERANCE);
            let hit = if ray_elev < floor_h + BLOCK_STEP || ray_elev < top_layer_floor {
                ((ray_elev / BLOCK_STEP).floor() * BLOCK_STEP + BLOCK_STEP).clamp((floor_h + BLOCK_STEP).min(ground_h), ground_h)
            } else {
                ground_h
            };
            Some(hit)
        } else {
            None
        }
    }
}

/// Decorator adding downward shaft excavation semantics to a vertical column.
pub struct ShaftColumn<'a, C: VerticalColumn> {
    inner: C,
    strata: &'a NaturalStrata,
    shafts: &'a ShaftExcavation,
    molds_present: bool,
}

impl<'a, C: VerticalColumn> ShaftColumn<'a, C> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(inner: C, strata: &'a NaturalStrata, shafts: &'a ShaftExcavation, molds_present: bool) -> Self {
        Self { inner, strata, shafts, molds_present }
    }
}

impl<C: VerticalColumn> VerticalColumn for ShaftColumn<'_, C> {
    fn support_height(&self, u: f32, v: f32, foot_elev: f32, body_span: f32) -> f32 {
        let (cu, cv) = GridCell::at_planar(u, v).feed_coordinates(|cu, cv| (cu, cv));
        if self.shafts.reworked_at(cu, cv) {
            let steps = if body_span > 0.0 && !self.molds_present {
                self.shafts.body_elevation_at(self.strata, cu, cv)
            } else {
                self.shafts.elevation_at(self.strata, cu, cv)
            };
            (steps as f32) * BLOCK_STEP
        } else {
            self.inner.support_height(u, v, foot_elev, body_span)
        }
    }

    fn confine_elevation(&self, cell: GridCell, u: f32, v: f32, elev: f32) -> f32 {
        self.inner.confine_elevation(cell, u, v, elev)
    }

    fn hit_top(&self, cell: GridCell, u: f32, v: f32, ray_elev: f32) -> Option<f32> {
        let (cu, cv) = cell.feed_coordinates(|cu, cv| (cu, cv));
        if !self.shafts.reworked_at(cu, cv) {
            return self.inner.hit_top(cell, u, v, ray_elev);
        }
        let ground_h = (self.shafts.elevation_at(self.strata, cu, cv) as f32) * BLOCK_STEP;
        let floor_h = (self.shafts.crust_floor_at(self.strata, cu, cv) as f32) * BLOCK_STEP;
        let floor_bound = (floor_h - FLOOR_HIT_INSET).min(ground_h - FLOOR_HIT_INSET);
        if ray_elev >= floor_bound && ray_elev <= ground_h + SURFACE_HIT_TOLERANCE {
            let top_layer_floor = ground_h - BLOCK_STEP;
            let hit = if ray_elev < floor_h + BLOCK_STEP || ray_elev < top_layer_floor {
                ((ray_elev / BLOCK_STEP).floor() * BLOCK_STEP + BLOCK_STEP).clamp((floor_h + BLOCK_STEP).min(ground_h), ground_h)
            } else {
                ground_h
            };
            Some(hit)
        } else {
            None
        }
    }
}

/// Decorator adding overhanging and stacked voxel block molds to a vertical column.
pub struct MoldedColumn<'a, C: VerticalColumn> {
    inner: C,
    strata: &'a NaturalStrata,
    shafts: &'a ShaftExcavation,
    molds: &'a OverhangMolds,
}

impl<'a, C: VerticalColumn> MoldedColumn<'a, C> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(inner: C, strata: &'a NaturalStrata, shafts: &'a ShaftExcavation, molds: &'a OverhangMolds) -> Self {
        Self { inner, strata, shafts, molds }
    }
}

impl<C: VerticalColumn> VerticalColumn for MoldedColumn<'_, C> {
    fn support_height(&self, u: f32, v: f32, foot_elev: f32, body_span: f32) -> f32 {
        let base_h = self.inner.support_height(u, v, foot_elev, body_span);
        if self.molds.empty() {
            base_h
        } else {
            self.molds.constrain_support(GridCell::at_planar(u, v), (u, v), base_h, (foot_elev, body_span))
        }
    }

    fn confine_elevation(&self, cell: GridCell, u: f32, v: f32, elev: f32) -> f32 {
        let (cu, cv) = cell.feed_coordinates(|cu, cv| (cu, cv));
        let ground = (self.shafts.elevation_at(self.strata, cu, cv) as f32) * BLOCK_STEP;
        self.molds
            .confine_niche_elevation(cu, cv, ground, elev)
            .filter(|_| elev >= ground - SHAFT_FLOOR_TOLERANCE)
            .unwrap_or_else(|| self.inner.confine_elevation(cell, u, v, elev))
    }

    fn hit_top(&self, cell: GridCell, u: f32, v: f32, ray_elev: f32) -> Option<f32> {
        let (cu, cv) = cell.feed_coordinates(|cu, cv| (cu, cv));
        self.molds
            .hit_top_at(cu, cv, u, v, ray_elev)
            .or_else(|| self.inner.hit_top(cell, u, v, ray_elev))
    }
}
