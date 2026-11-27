use crate::{
    BlockMedium, BlockMold, ClimateZone, CubePlanet, Face, FacetedReliefFace, ForestedFace,
    NullFace, WorldSeed,
};

/// Focused planetary zone relief contract composed via decorators.
pub trait ReliefMold {
    fn mold_relief(&self, zone: ClimateZone, half: f32, divs: u32) -> FacetedReliefFace;
}

/// Authoritative relief source with living subterranean mineral veins (`1` field).
#[derive(Clone, Copy, Debug)]
pub struct LivingRelief {
    seed: WorldSeed,
}

impl LivingRelief {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn seeded(seed: WorldSeed) -> Self {
        Self { seed }
    }

    pub const fn new() -> Self {
        Self::seeded(WorldSeed::origin())
    }
}

impl ReliefMold for LivingRelief {
    fn mold_relief(&self, zone: ClimateZone, half: f32, divs: u32) -> FacetedReliefFace {
        FacetedReliefFace::seeded_in_zone(zone, half, divs, self.seed)
    }
}

/// Zero-knowledge client relief source without unexposed subterranean ores (`1` field).
#[derive(Clone, Copy, Debug)]
pub struct HollowRelief {
    seed: WorldSeed,
}

impl HollowRelief {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn seeded(seed: WorldSeed) -> Self {
        Self { seed }
    }

    pub const fn new() -> Self {
        Self::seeded(WorldSeed::origin())
    }
}

impl ReliefMold for HollowRelief {
    fn mold_relief(&self, zone: ClimateZone, half: f32, divs: u32) -> FacetedReliefFace {
        FacetedReliefFace::hollow_seeded_in_zone(zone, half, divs, self.seed)
    }
}

/// `ReliefMold` decorator crowning the temperate sanctuary face with an indestructible `OriginStone` (`1` field).
#[derive(Clone, Copy, Debug)]
pub struct OriginPlinthRelief<R> {
    origin: R,
}

impl<R> OriginPlinthRelief<R> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(origin: R) -> Self {
        Self { origin }
    }
}

impl<R: ReliefMold> ReliefMold for OriginPlinthRelief<R> {
    fn mold_relief(&self, zone: ClimateZone, half: f32, divs: u32) -> FacetedReliefFace {
        let face = self.origin.mold_relief(zone, half, divs);
        if zone == ClimateZone::Temperate {
            let frame = zone.surface_frame(half);
            let atom = BlockMold::cube()
                .in_medium(BlockMedium::OriginStone)
                .atom_for(frame.bitangent(), &frame);
            face.with_molded_block(-1, -1, -crate::GridCell::SPAN, atom)
        } else {
            face
        }
    }
}

/// Focused planetary climate-zone face contract.
pub trait ZoneSurface {
    fn assemble_face(&self, zone: ClimateZone, half: f32, divs: u32) -> Box<dyn Face>;

    fn assemble_six(&self, size_blocks: u32) -> (Vec<Box<dyn Face>>, f32) {
        let half = (size_blocks as f32) * 0.5;
        let divs = size_blocks * 2;
        let faces = ClimateZone::all()
            .into_iter()
            .map(|zone| self.assemble_face(zone, half, divs))
            .collect();
        (faces, half)
    }
}

/// Decorator wrapping a `ReliefMold` inside a lazy sector-partitioned `ForestedFace` (`2` fields).
#[derive(Clone, Copy, Debug)]
pub struct ForestedZone<R> {
    origin: R,
    seed: WorldSeed,
}

impl<R> ForestedZone<R> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn seeded(origin: R, seed: WorldSeed) -> Self {
        Self { origin, seed }
    }

    pub const fn new(origin: R) -> Self {
        Self::seeded(origin, WorldSeed::origin())
    }
}

impl<R: ReliefMold> ZoneSurface for ForestedZone<R> {
    fn assemble_face(&self, zone: ClimateZone, half: f32, divs: u32) -> Box<dyn Face> {
        let relief = Box::new(self.origin.mold_relief(zone, half, divs));
        Box::new(ForestedFace::seeded_in_zone(relief, zone, half, self.seed))
    }
}

/// Flora-free `ZoneSurface` adapter over a `ReliefMold` (`1` field).
#[derive(Clone, Copy, Debug)]
pub struct BarrenZone<R> {
    origin: R,
}

impl<R> BarrenZone<R> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(origin: R) -> Self {
        Self { origin }
    }
}

impl<R: ReliefMold> ZoneSurface for BarrenZone<R> {
    fn assemble_face(&self, zone: ClimateZone, half: f32, divs: u32) -> Box<dyn Face> {
        Box::new(self.origin.mold_relief(zone, half, divs))
    }
}

/// Multi-biome cubic planetary world factory assembling climate zones and seam boundaries.
pub struct BiomedCubeWorld;

impl BiomedCubeWorld {
    /// Factory method assembling authoritative multi-biome planetary terrain with living mineral veins.
    pub fn of_size(size_blocks: u32) -> CubePlanet {
        let (faces, half) = ForestedZone::new(OriginPlinthRelief::new(LivingRelief::new())).assemble_six(size_blocks);
        CubePlanet::cubic(faces, Box::new(NullFace), half)
    }

    /// Factory method assembling zero-knowledge client planetary terrain without unexposed ores.
    pub fn hollow_of_size(size_blocks: u32) -> CubePlanet {
        let (faces, half) = ForestedZone::new(OriginPlinthRelief::new(HollowRelief::new())).assemble_six(size_blocks);
        CubePlanet::cubic(faces, Box::new(NullFace), half)
    }

    /// Factory method assembling barren multi-biome planetary terrain without woodland flora for fast simulation.
    pub fn barren(size_blocks: u32) -> CubePlanet {
        let (faces, half) = BarrenZone::new(OriginPlinthRelief::new(LivingRelief::new())).assemble_six(size_blocks);
        CubePlanet::cubic(faces, Box::new(NullFace), half)
    }

    /// Factory method assembling barren multi-biome terrain with continuous seam normal bevels.
    pub fn barren_beveled(size_blocks: u32, margin: f32) -> CubePlanet {
        let (faces, half) = BarrenZone::new(OriginPlinthRelief::new(LivingRelief::new())).assemble_six(size_blocks);
        CubePlanet::beveled_cubic(faces, Box::new(NullFace), half, margin)
    }

    /// Factory method assembling multi-biome terrain with continuous seam normal transitions.
    pub fn beveled(size_blocks: u32, margin: f32) -> CubePlanet {
        Self::seeded_beveled(size_blocks, margin, WorldSeed::origin())
    }

    /// Factory method assembling seeded multi-biome terrain with continuous seam normal transitions.
    pub fn seeded_beveled(size_blocks: u32, margin: f32, seed: WorldSeed) -> CubePlanet {
        let (faces, half) = ForestedZone::seeded(OriginPlinthRelief::new(LivingRelief::seeded(seed)), seed)
            .assemble_six(size_blocks);
        CubePlanet::beveled_cubic(faces, Box::new(NullFace), half, margin)
    }

    /// Factory method assembling zero-knowledge client terrain with continuous seam normal transitions.
    pub fn hollow_beveled(size_blocks: u32, margin: f32) -> CubePlanet {
        Self::hollow_seeded_beveled(size_blocks, margin, WorldSeed::origin())
    }

    /// Factory method assembling seeded zero-knowledge client terrain with continuous seam normal transitions.
    pub fn hollow_seeded_beveled(size_blocks: u32, margin: f32, seed: WorldSeed) -> CubePlanet {
        let (faces, half) = ForestedZone::seeded(OriginPlinthRelief::new(HollowRelief::seeded(seed)), seed)
            .assemble_six(size_blocks);
        CubePlanet::beveled_cubic(faces, Box::new(NullFace), half, margin)
    }

    /// Factory method assembling seeded barren multi-biome terrain with continuous seam normal bevels.
    pub fn barren_seeded_beveled(size_blocks: u32, margin: f32, seed: WorldSeed) -> CubePlanet {
        let (faces, half) = BarrenZone::new(OriginPlinthRelief::new(LivingRelief::seeded(seed)))
            .assemble_six(size_blocks);
        CubePlanet::beveled_cubic(faces, Box::new(NullFace), half, margin)
    }
}
