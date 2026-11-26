mod blueprints;
mod cache;
mod facets;
mod meadow_ring;
mod orbital_shell;
mod patch_grid;

use crate::block::VoxelShape;
use crate::topology::ClimateZone;
use super::elevation::{ReliefElevation, ZonedRelief};
use super::mineral_vein::{HollowVeins, LivingVeins, SubterraneanVeins};
use super::palette::ReliefPalette;
use super::strata::ReliefStrata;
use super::terraced::TerracedRelief;
use self::blueprints::ReliefDomain;
use self::cache::ReliefCellCache;
pub use self::meadow_ring::{MeadowRing, MeadowStamp, MEADOW_RING_SLOTS};

/// Planetary relief face generating undulating climbable terrain and subterranean cavities ("Tell, Don't Ask").
pub struct FacetedReliefFace {
    domain: ReliefDomain,
    palette: ReliefPalette,
    cache: ReliefCellCache,
}

impl FacetedReliefFace {
    /// Primary constructor doing nothing except assigning arguments to fields.
    fn new(domain: ReliefDomain, palette: ReliefPalette, cache: ReliefCellCache) -> Self {
        Self { domain, palette, cache }
    }

    /// Secondary constructor assembling a relief face with explicit subterranean vein authority.
    pub fn in_zone_with_veins(
        zone: ClimateZone,
        half_size: f32,
        divisions: u32,
        veins: Box<dyn SubterraneanVeins>,
    ) -> Self {
        let zoned = Box::new(ZonedRelief::in_climate(zone, half_size));
        Self::alien_with_palette(zone, half_size, divisions, zoned, ReliefPalette::for_zone(zone), veins)
    }

    /// Secondary constructor assembling an authoritative relief face with living mineral veins.
    pub fn in_zone(zone: ClimateZone, half_size: f32, divisions: u32) -> Self {
        Self::in_zone_with_veins(zone, half_size, divisions, Box::new(LivingVeins::ambient(zone.index() as u64 + 1)))
    }

    /// Secondary constructor assembling a zero-knowledge client relief face without unexposed ores.
    pub fn hollow_in_zone(zone: ClimateZone, half_size: f32, divisions: u32) -> Self {
        Self::in_zone_with_veins(zone, half_size, divisions, Box::new(HollowVeins::untouched()))
    }

    /// Secondary constructor assembling a cratered lunar regolith face with explicit vein authority.
    pub fn lunar_with_veins(
        zone: ClimateZone,
        half_size: f32,
        divisions: u32,
        crater_relief: Box<dyn ReliefElevation>,
        veins: Box<dyn SubterraneanVeins>,
    ) -> Self {
        Self::alien_with_palette(zone, half_size, divisions, crater_relief, ReliefPalette::lunar_regolith(zone), veins)
    }

    /// Secondary constructor assembling an alien planetary or asteroidal face with bespoke elevation and palette.
    pub fn alien_with_palette(
        zone: ClimateZone,
        half_size: f32,
        divisions: u32,
        elevation: Box<dyn ReliefElevation>,
        palette: ReliefPalette,
        veins: Box<dyn SubterraneanVeins>,
    ) -> Self {
        let frame = zone.surface_frame(half_size);
        let relief = Box::new(TerracedRelief::with_veins(elevation, (divisions as i32) / 2, veins));
        Self::new(
            ReliefDomain::new(frame, relief),
            palette,
            ReliefCellCache::empty(ReliefStrata::new(), divisions),
        )
    }

    /// Returns this relief face with an additional molded voxel shape stacked at `(u, v, base)`.
    pub fn with_molded_block(mut self, u: i32, v: i32, base: f32, shape: Box<dyn VoxelShape>) -> Self {
        self.domain.relief.stack_molded_cell(u, v, base, shape);
        self.cache.record_molded(u, v);
        self
    }

    fn refresh_neighborhood(&mut self, cu: i32, cv: i32, half: i32, radius: i32) {
        self.cache.refresh_neighborhood(cu, cv, half, radius, &self.domain.frame, self.domain.relief.as_ref(), &self.palette);
    }
}
