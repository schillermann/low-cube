use crate::block::GridCell;
use super::super::natural_strata::NaturalStrata;
use super::super::overhang_molds::OverhangMolds;
use super::super::shaft_excavation::ShaftExcavation;
use super::{
    BLOCK_STEP, CEILING_CLEARANCE_INSET, LETHAL_CORE_MARGIN, SHAFT_FLOOR_TOLERANCE,
    SURFACE_HIT_TOLERANCE, UPWARD_CEILING_CATCH_SPAN, UPWARD_CEILING_FLOOR_INSET,
    VAULT_CEILING_FLOOR_INSET, VerticalColumn,
};

/// Base vertical column backed by natural planetary strata and the subterranean vault.
pub struct NaturalColumn<'a> {
    strata: &'a NaturalStrata,
}

impl<'a> NaturalColumn<'a> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(strata: &'a NaturalStrata) -> Self {
        Self { strata }
    }

    fn core_floating(&self, u: f32, v: f32, center_elev: f32, body_span: f32) -> bool {
        let vault = self.strata.vault();
        body_span > 0.0
            && !vault.core().engulfs(u, v, center_elev, LETHAL_CORE_MARGIN)
            && vault.encloses(u, v, center_elev, 0.0)
            && center_elev < vault.ceiling_height_at(u, v) + BLOCK_STEP
    }
}

impl VerticalColumn for NaturalColumn<'_> {
    fn support_height(&self, u: f32, v: f32, foot_elev: f32, body_span: f32) -> f32 {
        let center_elev = foot_elev + body_span.min(1.0);
        if self.core_floating(u, v, center_elev, body_span) {
            self.strata.vault().core().crest_at(u, v)
        } else {
            self.strata.surface_height_at(u, v)
        }
    }

    fn confine_elevation(&self, _cell: GridCell, u: f32, v: f32, elev: f32) -> f32 {
        self.strata.vault().confine(u, v, elev)
    }

    fn hit_top(&self, cell: GridCell, u: f32, v: f32, ray_elev: f32) -> Option<f32> {
        let (cu, cv) = cell.feed_coordinates(|cu, cv| (cu, cv));
        let ground_h = (self.strata.elevation_at(cu, cv) as f32) * BLOCK_STEP;
        let ceil_h = (self.strata.vault().floor_steps_at(cu, cv) as f32) * BLOCK_STEP;
        let ground_surf = self.strata.surface_height_at(u, v);
        let ceil_floor = (self.strata.vault().ceiling_height_at(u, v) - VAULT_CEILING_FLOOR_INSET)
            .min(ground_h - VAULT_CEILING_FLOOR_INSET);
        if ray_elev >= ceil_floor && ray_elev <= ground_surf + SURFACE_HIT_TOLERANCE {
            let top_layer_floor = (ground_h - BLOCK_STEP).min(ground_surf - SHAFT_FLOOR_TOLERANCE);
            let hit = if ray_elev < ceil_h + BLOCK_STEP || ray_elev < top_layer_floor {
                ((ray_elev / BLOCK_STEP).floor() * BLOCK_STEP + BLOCK_STEP).clamp(ceil_h + BLOCK_STEP, ground_h)
            } else {
                ground_h
            };
            Some(hit)
        } else {
            None
        }
    }
}

/// Decorator adding downward and upward shaft excavation semantics to a vertical column.
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
        let vault = self.strata.vault();
        let center_elev = foot_elev + body_span.min(1.0);
        let ceil_limit = if self.shafts.upward_at(cu, cv) > 0 {
            let top = if self.shafts.breached_at(self.strata, cu, cv) {
                self.strata.elevation_at(cu, cv)
            } else {
                self.shafts.crust_floor_at(self.strata, cu, cv)
            };
            (top as f32) * BLOCK_STEP
        } else {
            vault.ceiling_height_at(u, v) + BLOCK_STEP
        };
        if !self.shafts.in_unbreached_shaft(self.strata, cu, cv, center_elev)
            && body_span > 0.0
            && !vault.core().engulfs(u, v, center_elev, LETHAL_CORE_MARGIN)
            && (vault.encloses(u, v, center_elev, 0.0) || self.shafts.in_upward_shaft(self.strata, cu, cv, center_elev))
            && center_elev < ceil_limit
        {
            vault.core().crest_at(u, v)
        } else if self.shafts.reworked_at(cu, cv) {
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
        let (cu, cv) = cell.feed_coordinates(|cu, cv| (cu, cv));
        let ground = (self.shafts.elevation_at(self.strata, cu, cv) as f32) * BLOCK_STEP;
        let ceiling = (self.shafts.crust_floor_at(self.strata, cu, cv) as f32) * BLOCK_STEP - CEILING_CLEARANCE_INSET;
        if !self.shafts.breached_at(self.strata, cu, cv)
            && self.shafts.upward_at(cu, cv) > 0
            && elev < ceiling + UPWARD_CEILING_CATCH_SPAN
        {
            elev.min(ceiling)
        } else if self.shafts.breached_at(self.strata, cu, cv) || elev >= ground {
            elev
        } else {
            self.inner.confine_elevation(cell, u, v, elev)
        }
    }

    fn hit_top(&self, cell: GridCell, u: f32, v: f32, ray_elev: f32) -> Option<f32> {
        let (cu, cv) = cell.feed_coordinates(|cu, cv| (cu, cv));
        if self.shafts.breached_at(self.strata, cu, cv) {
            return None;
        }
        if !self.shafts.reworked_at(cu, cv) && self.shafts.upward_at(cu, cv) == 0 {
            return self.inner.hit_top(cell, u, v, ray_elev);
        }
        let ground_h = (self.shafts.elevation_at(self.strata, cu, cv) as f32) * BLOCK_STEP;
        let ceil_h = (self.shafts.crust_floor_at(self.strata, cu, cv) as f32) * BLOCK_STEP;
        let ground_surf = if self.shafts.reworked_at(cu, cv) {
            ground_h
        } else {
            self.strata.surface_height_at(u, v)
        };
        let ceil_floor = if self.shafts.upward_at(cu, cv) > 0 {
            ceil_h - UPWARD_CEILING_FLOOR_INSET
        } else {
            (self.strata.vault().ceiling_height_at(u, v) - VAULT_CEILING_FLOOR_INSET)
                .min(ground_h - VAULT_CEILING_FLOOR_INSET)
        };
        if ray_elev >= ceil_floor && ray_elev <= ground_surf + SURFACE_HIT_TOLERANCE {
            let top_layer_floor = (ground_h - BLOCK_STEP).min(ground_surf - SHAFT_FLOOR_TOLERANCE);
            let hit = if (self.shafts.upward_at(cu, cv) > 0 && ray_elev <= ceil_h + BLOCK_STEP)
                || ray_elev < ceil_h + BLOCK_STEP
                || ray_elev < top_layer_floor
            {
                ((ray_elev / BLOCK_STEP).floor() * BLOCK_STEP + BLOCK_STEP).clamp(ceil_h + BLOCK_STEP, ground_h)
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
        if let Some(confined) = self
            .molds
            .confine_niche_elevation(cu, cv, ground, elev)
            .filter(|_| elev >= ground - SHAFT_FLOOR_TOLERANCE)
        {
            return confined;
        }
        let raw_z = self.inner.confine_elevation(cell, u, v, elev);
        if self.shafts.breached_at(self.strata, cu, cv) || elev >= ground {
            raw_z
        } else {
            self.molds
                .confine_niche_elevation(cu, cv, self.strata.vault().core().crest_at(u, v), raw_z)
                .unwrap_or(raw_z)
        }
    }

    fn hit_top(&self, cell: GridCell, u: f32, v: f32, ray_elev: f32) -> Option<f32> {
        let (cu, cv) = cell.feed_coordinates(|cu, cv| (cu, cv));
        self.molds
            .hit_top_at(cu, cv, u, v, ray_elev)
            .or_else(|| self.inner.hit_top(cell, u, v, ray_elev))
    }
}
