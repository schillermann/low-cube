use glam::Vec3;
use crate::relief::{
    FacetedReliefFace, HollowVeins, LivingVeins, ReliefPalette, SubterraneanVeins,
};
use crate::topology::ClimateZone;
use crate::{CubePlanet, Face, NullFace};
use super::catalog::{BodyId, SolarCatalog};
use super::lunar::LunarCraterRelief;
use super::proportions::SolarProportions;

/// Factory constructing on-demand `CubePlanet` voxel surfaces for rocky/icy planets, moons, and belt asteroids.
pub struct AlienWorld;

impl AlienWorld {
    pub const fn new() -> Self {
        Self
    }

    fn palette_for(id: BodyId, zone: ClimateZone) -> ReliefPalette {
        if id == BodyId::LUNA || id == BodyId::MERCURY || id == BodyId::CERES {
            return ReliefPalette::lunar_regolith(zone);
        }
        if id.belt_slot().is_some() {
            return ReliefPalette::uniform(
                Vec3::new(0.42, 0.40, 0.38),
                Vec3::new(0.22, 0.21, 0.22),
                Vec3::new(0.31, 0.30, 0.31),
            );
        }
        let spec = SolarCatalog::new().spec_for(id);
        let (surf, deep) = spec.map_or(
            (Vec3::new(0.60, 0.60, 0.64), Vec3::new(0.32, 0.32, 0.36)),
            |s| s.physique().palette(),
        );
        let shade = match zone {
            ClimateZone::Temperate | ClimateZone::Arctic => 1.04,
            ClimateZone::Desert | ClimateZone::Jungle => 0.96,
            ClimateZone::Volcanic | ClimateZone::Steppe => 0.88,
        };
        let top = (surf * shade).clamp(Vec3::ZERO, Vec3::ONE);
        let wall = ((surf * 0.55 + deep * 0.45) * shade).clamp(Vec3::ZERO, Vec3::ONE);
        let floor = (deep * 0.78).clamp(Vec3::ZERO, Vec3::ONE);
        ReliefPalette::uniform(top, floor, wall)
    }

    /// Builds a 6-face beveled `CubePlanet` scaled to `proportions` for `id` (`authoritative` controls live mineral veins).
    pub fn build_body(id: BodyId, proportions: SolarProportions, authoritative: bool) -> (CubePlanet, f32) {
        let blocks = if let Some(slot) = id.belt_slot() {
            let tier = u32::from(slot & 0x3) % 3;
            let b = 16 + tier * 8;
            ((b as f32 * proportions.body_scale().clamp(0.5, 2.0)).round() as u32).clamp(16, 48) & !1
        } else if let Some(spec) = SolarCatalog::new().spec_for(id) {
            proportions.voxel_blocks(spec.physique().diameter_ratio())
        } else {
            32
        };
        let half_size = (blocks as f32) * 0.5;
        let divs = blocks * 2;
        let seed_bias = u64::from(id.raw_code()).wrapping_mul(0x9E37_79B9);
        let faces: Vec<Box<dyn Face>> = ClimateZone::all()
            .into_iter()
            .map(|zone| -> Box<dyn Face> {
                let frame = zone.surface_frame(half_size);
                let seed = (zone.index() as u64 + 1).wrapping_mul(0x517C_C1B7) ^ seed_bias;
                let craters = Box::new(LunarCraterRelief::new(frame, half_size, seed));
                let veins: Box<dyn SubterraneanVeins> = if authoritative {
                    Box::new(LivingVeins::ambient(seed.wrapping_add(77)))
                } else {
                    Box::new(HollowVeins::untouched())
                };
                let palette = Self::palette_for(id, zone);
                Box::new(FacetedReliefFace::alien_with_palette(zone, half_size, divs, craters, palette, veins))
            })
            .collect();
        (CubePlanet::beveled_cubic(faces, Box::new(NullFace), half_size, 1.0), half_size)
    }
}
