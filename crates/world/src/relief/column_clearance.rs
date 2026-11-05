use crate::block::GridCell;
use super::natural_strata::NaturalStrata;
use super::overhang_molds::OverhangMolds;
use super::shaft_excavation::ShaftExcavation;

const BLOCK_STEP: f32 = GridCell::SPAN;
const SHAFT_FLOOR_TOLERANCE: f32 = 0.1;
const CEILING_CLEARANCE_INSET: f32 = 0.30;
const NEIGHBOR_ESCAPE_MARGIN: f32 = 0.02;
const SHAFT_WALL_MARGIN: f32 = 0.21;
const UPWARD_CEILING_CATCH_SPAN: f32 = 2.0;
const UPWARD_CEILING_FLOOR_INSET: f32 = 0.15;
const VAULT_CEILING_FLOOR_INSET: f32 = 0.55;
const SURFACE_HIT_TOLERANCE: f32 = 0.02;
const WEIGHTLESS_CRUST_CLEARANCE: f32 = 0.35;
const WEIGHTLESS_VAULT_MARGIN: f32 = 0.10;
const LETHAL_CORE_MARGIN: f32 = 1.05;

/// Unified 3D vertical column topology combining natural strata, shafts, and overhanging molds.
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

    pub fn lethal_at(&self, u: f32, v: f32, elev: f32) -> bool {
        self.strata.vault().core().engulfs(u, v, elev, LETHAL_CORE_MARGIN)
    }

    pub fn weightless_at(&self, u: f32, v: f32, elev: f32) -> bool {
        let (cu, cv) = GridCell::at_planar(u, v).feed_coordinates(|cu, cv| (cu, cv));
        let crust_top_h = (self.strata.vault().crust_top_step_at(cu, cv) as f32) * BLOCK_STEP;
        let in_crust_upward =
            self.shafts.in_upward_shaft(self.strata, cu, cv, elev) && elev < crust_top_h - WEIGHTLESS_CRUST_CLEARANCE;
        !self.shafts.in_unbreached_shaft(self.strata, cu, cv, elev)
            && (in_crust_upward || self.strata.vault().encloses(u, v, elev, WEIGHTLESS_VAULT_MARGIN))
            && !self.lethal_at(u, v, elev)
    }

    pub fn support_height_at(&self, u: f32, v: f32, foot_elev: f32, body_span: f32) -> f32 {
        let (cell_u, cell_v) = GridCell::at_planar(u, v).feed_coordinates(|cu, cv| (cu, cv));
        let vault = self.strata.vault();
        let crest = vault.core().crest_at(u, v);
        let center_elev = foot_elev + body_span.min(1.0);
        let ceil_limit = if self.shafts.upward_at(cell_u, cell_v) > 0 {
            let top = if self.shafts.breached_at(self.strata, cell_u, cell_v) {
                self.strata.elevation_at(cell_u, cell_v)
            } else {
                self.shafts.crust_floor_at(self.strata, cell_u, cell_v)
            };
            (top as f32) * BLOCK_STEP
        } else {
            vault.ceiling_height_at(u, v) + BLOCK_STEP
        };
        let base_h = if !self.shafts.in_unbreached_shaft(self.strata, cell_u, cell_v, center_elev)
            && body_span > 0.0
            && !self.lethal_at(u, v, center_elev)
            && (vault.encloses(u, v, center_elev, 0.0) || self.shafts.in_upward_shaft(self.strata, cell_u, cell_v, center_elev))
            && center_elev < ceil_limit
        {
            crest
        } else if self.shafts.pristine() && self.molds.empty() {
            return self.strata.surface_height_at(u, v);
        } else if self.shafts.reworked_at(cell_u, cell_v) {
            let steps = if body_span > 0.0 && !self.molds.present_at(cell_u, cell_v) {
                self.shafts.body_elevation_at(self.strata, cell_u, cell_v)
            } else {
                self.shafts.elevation_at(self.strata, cell_u, cell_v)
            };
            (steps as f32) * BLOCK_STEP
        } else {
            self.strata.surface_height_at(u, v)
        };
        self.molds.constrain_support(cell_u, cell_v, u, v, base_h, foot_elev, body_span)
    }

    fn cell_ceiling_at(&self, cu: i32, cv: i32) -> f32 {
        if self.shafts.breached_at(self.strata, cu, cv) {
            f32::INFINITY
        } else if self.shafts.upward_at(cu, cv) > 0 {
            (self.shafts.crust_floor_at(self.strata, cu, cv) as f32) * BLOCK_STEP - CEILING_CLEARANCE_INSET
        } else {
            let (cx, cy) = GridCell::new(cu, cv).center();
            self.strata.vault().ceiling_height_at(cx, cy) - CEILING_CLEARANCE_INSET
        }
    }

    fn solid_wall_at(&self, cu: i32, cv: i32, elev: f32) -> bool {
        if self.shafts.breached_at(self.strata, cu, cv) {
            return self.molds.layer_occupied_at(cu, cv, elev);
        }
        let ground = (self.shafts.elevation_at(self.strata, cu, cv) as f32) * BLOCK_STEP;
        let ceiling = self.cell_ceiling_at(cu, cv);
        self.molds.solid_wall_at(cu, cv, ground, ceiling, elev)
    }

    pub fn confine(&self, u: f32, v: f32, elev: f32) -> (f32, f32, f32) {
        if self.shafts.pristine() && self.shafts.ceilings_untouched() && self.molds.empty() {
            return (u, v, self.strata.vault().confine(u, v, elev));
        }
        let (mut cu, mut cv) = GridCell::at_planar(u, v).feed_coordinates(|cu, cv| (cu, cv));
        let (mut tu, mut tv) = (u, v);
        let natural_top = (self.strata.elevation_at(cu, cv) as f32) * BLOCK_STEP;
        if self.shafts.ceilings_untouched() && self.molds.empty() && elev >= natural_top {
            return (u, v, elev);
        }
        let sub_surface = elev < natural_top;
        let in_own_upward = self.shafts.upward_at(cu, cv) > 0
            && !self.shafts.breached_at(self.strata, cu, cv)
            && !self.molds.layer_occupied_at(cu, cv, elev);
        if sub_surface && !in_own_upward && self.solid_wall_at(cu, cv, elev) {
            for (du, dv) in [(-1, 0), (1, 0), (0, -1), (0, 1), (-1, -1), (1, -1), (-1, 1), (1, 1)] {
                let (nu, nv) = (cu + du, cv + dv);
                if (self.shafts.upward_at(nu, nv) > 0 || self.shafts.breached_at(self.strata, nu, nv) || self.molds.present_at(nu, nv))
                    && !self.solid_wall_at(nu, nv, elev)
                {
                    (tu, tv) = GridCell::new(nu, nv).clamp_interior(u, v, NEIGHBOR_ESCAPE_MARGIN);
                    (cu, cv) = (nu, nv);
                    break;
                }
            }
        }
        let near_upward = self.shafts.upward_at(cu, cv) > 0 || self.shafts.adjacent_upward_at(self.strata, cu, cv) > i32::MIN;
        if sub_surface && near_upward && (in_own_upward || !self.solid_wall_at(cu, cv, elev)) {
            let walls = (
                self.solid_wall_at(cu - 1, cv, elev),
                self.solid_wall_at(cu + 1, cv, elev),
                self.solid_wall_at(cu, cv - 1, elev),
                self.solid_wall_at(cu, cv + 1, elev),
            );
            (tu, tv) = GridCell::new(cu, cv).clamp_walled(tu, tv, SHAFT_WALL_MARGIN, walls);
        }
        let ground = (self.shafts.elevation_at(self.strata, cu, cv) as f32) * BLOCK_STEP;
        let ceiling = (self.shafts.crust_floor_at(self.strata, cu, cv) as f32) * BLOCK_STEP - CEILING_CLEARANCE_INSET;
        let z = if let Some(confined) = self
            .molds
            .confine_niche_elevation(cu, cv, ground, elev)
            .filter(|_| elev >= ground - SHAFT_FLOOR_TOLERANCE)
        {
            confined
        } else if !self.shafts.breached_at(self.strata, cu, cv)
            && self.shafts.upward_at(cu, cv) > 0
            && elev < ceiling + UPWARD_CEILING_CATCH_SPAN
        {
            let raw_z = if elev > ceiling { ceiling } else { elev };
            self.molds
                .confine_niche_elevation(cu, cv, self.strata.vault().core().crest_at(tu, tv), raw_z)
                .unwrap_or(raw_z)
        } else if self.shafts.breached_at(self.strata, cu, cv) || elev >= ground {
            elev
        } else {
            let raw_z = self.strata.vault().confine(tu, tv, elev);
            self.molds
                .confine_niche_elevation(cu, cv, self.strata.vault().core().crest_at(tu, tv), raw_z)
                .unwrap_or(raw_z)
        };
        (tu, tv, z)
    }

    pub fn hit_top_at(&self, u: f32, v: f32, ray_elev: f32) -> Option<f32> {
        let (cu, cv) = GridCell::at_planar(u, v).feed_coordinates(|cu, cv| (cu, cv));
        if let Some(hit) = self.molds.hit_top_at(cu, cv, u, v, ray_elev) {
            return Some(hit);
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
        if !self.shafts.breached_at(self.strata, cu, cv)
            && ray_elev >= ceil_floor
            && ray_elev <= ground_surf + SURFACE_HIT_TOLERANCE
        {
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
