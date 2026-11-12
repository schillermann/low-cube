use crate::block::GridCell;
use super::molded_structure::WorldBlocks;
use super::natural_strata::NaturalStrata;
use super::shaft_excavation::ShaftExcavation;
use super::turf_scars::TurfScars;

const BLOCK_STEP: f32 = GridCell::SPAN;

/// Polymorphic 3D turf-colonization neighborhood evaluating living grass contact within ±1 step.
pub(super) trait TurfHabitat {
    fn ground_step_at(&self, u: i32, v: i32) -> i32;
    fn verdant_at(&self, u: i32, v: i32, target_step: i32) -> bool;

    fn verdant_neighbors(&self, u: i32, v: i32, target_step: i32) -> u32 {
        let mut count = 0;
        for du in -1..=1 {
            for dv in -1..=1 {
                if (du != 0 || dv != 0) && self.verdant_at(u + du, v + dv, target_step) {
                    count += 1;
                }
            }
        }
        count
    }
}

impl<T: TurfHabitat + ?Sized> TurfHabitat for &T {
    fn ground_step_at(&self, u: i32, v: i32) -> i32 {
        (**self).ground_step_at(u, v)
    }

    fn verdant_at(&self, u: i32, v: i32, target_step: i32) -> bool {
        (**self).verdant_at(u, v, target_step)
    }
}

/// Natural planetary ground turf habitat prior to overhanging molded block occlusion.
pub(super) struct NaturalHabitat<'a> {
    strata: &'a NaturalStrata,
    shafts: &'a ShaftExcavation,
    scars: &'a TurfScars,
}

impl<'a> NaturalHabitat<'a> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub(super) const fn new(
        strata: &'a NaturalStrata,
        shafts: &'a ShaftExcavation,
        scars: &'a TurfScars,
    ) -> Self {
        Self {
            strata,
            shafts,
            scars,
        }
    }

    pub(super) fn fertile_ground(&self, u: i32, v: i32) -> bool {
        let g_step = self.ground_step_at(u, v);
        !(self.shafts.reworked_at(u, v) && self.strata.vault().crust_surface_at(u, v, g_step))
    }
}

impl TurfHabitat for NaturalHabitat<'_> {
    fn ground_step_at(&self, u: i32, v: i32) -> i32 {
        self.shafts.elevation_at(self.strata, u, v)
    }

    fn verdant_at(&self, u: i32, v: i32, target_step: i32) -> bool {
        !self.scars.bare_at(u, v)
            && self.fertile_ground(u, v)
            && (target_step - self.ground_step_at(u, v)).abs() <= 1
    }
}

/// Decorator suppressing smothered ground turf while adding elevated verdant world blocks.
pub(super) struct CanopyHabitat<'a, H> {
    inner: H,
    blocks: &'a WorldBlocks,
}

impl<'a, H: TurfHabitat> CanopyHabitat<'a, H> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub(super) const fn new(inner: H, blocks: &'a WorldBlocks) -> Self {
        Self { inner, blocks }
    }
}

impl<H: TurfHabitat> TurfHabitat for CanopyHabitat<'_, H> {
    fn ground_step_at(&self, u: i32, v: i32) -> i32 {
        self.inner.ground_step_at(u, v)
    }

    fn verdant_at(&self, u: i32, v: i32, target_step: i32) -> bool {
        let ground_top = (self.inner.ground_step_at(u, v) as f32) * BLOCK_STEP;
        let ground_verdant = !self.blocks.covers_ground_at(u, v, ground_top)
            && self.inner.verdant_at(u, v, target_step);
        ground_verdant || self.blocks.verdant_near_step(u, v, target_step)
    }
}
