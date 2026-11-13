use crate::{
    ClimateZone, CubePlanet, Face, FacetedReliefFace, FoldedFace, ForestedFace,
    NullFace,
};

/// Multi-biome cubic planetary world factory assembling climate zones and terrain decorators.
pub struct BiomedCubeWorld;

impl BiomedCubeWorld {
    /// Factory method assembling authoritative multi-biome planetary terrain with living mineral veins.
    pub fn of_size(size_blocks: u32) -> CubePlanet {
        let half = (size_blocks as f32) * 0.5;
        let divs = size_blocks * 2;
        let faces: Vec<Box<dyn Face>> = ClimateZone::all()
            .into_iter()
            .map(|zone| -> Box<dyn Face> {
                let relief = Box::new(FacetedReliefFace::in_zone(zone, half, divs));
                let forested = Box::new(ForestedFace::in_zone(relief, zone, half));
                Box::new(FoldedFace::in_zone(forested, zone, half))
            })
            .collect();
        CubePlanet::new(faces, Box::new(NullFace))
    }

    /// Factory method assembling zero-knowledge client planetary terrain without unexposed ores.
    pub fn hollow_of_size(size_blocks: u32) -> CubePlanet {
        let half = (size_blocks as f32) * 0.5;
        let divs = size_blocks * 2;
        let faces: Vec<Box<dyn Face>> = ClimateZone::all()
            .into_iter()
            .map(|zone| -> Box<dyn Face> {
                let relief = Box::new(FacetedReliefFace::hollow_in_zone(zone, half, divs));
                let forested = Box::new(ForestedFace::in_zone(relief, zone, half));
                Box::new(FoldedFace::in_zone(forested, zone, half))
            })
            .collect();
        CubePlanet::new(faces, Box::new(NullFace))
    }

    /// Factory method assembling planetary terrain in Tron wireframe visualization mode.
    pub fn tron(size_blocks: u32) -> CubePlanet {
        let half = (size_blocks as f32) * 0.5;
        let divs = size_blocks * 2;
        let faces: Vec<Box<dyn Face>> = ClimateZone::all()
            .into_iter()
            .map(|zone| -> Box<dyn Face> {
                let relief = Box::new(FacetedReliefFace::tron(zone, half, divs));
                Box::new(FoldedFace::in_zone(relief, zone, half))
            })
            .collect();
        CubePlanet::new(faces, Box::new(NullFace))
    }

    /// Factory method assembling zero-knowledge Tron wireframe planetary terrain.
    pub fn hollow_tron(size_blocks: u32) -> CubePlanet {
        let half = (size_blocks as f32) * 0.5;
        let divs = size_blocks * 2;
        let faces: Vec<Box<dyn Face>> = ClimateZone::all()
            .into_iter()
            .map(|zone| -> Box<dyn Face> {
                let relief = Box::new(FacetedReliefFace::hollow_tron(zone, half, divs));
                Box::new(FoldedFace::in_zone(relief, zone, half))
            })
            .collect();
        CubePlanet::new(faces, Box::new(NullFace))
    }

    /// Factory method decorating the multi-biome world with continuous seam normal transitions.
    pub fn beveled(size_blocks: u32, margin: f32) -> crate::BeveledWorld {
        let half = (size_blocks as f32) * 0.5;
        crate::BeveledWorld::decorating(Box::new(Self::of_size(size_blocks)), half, margin)
    }

    /// Factory method decorating a zero-knowledge client world with continuous seam normal transitions.
    pub fn hollow_beveled(size_blocks: u32, margin: f32) -> crate::BeveledWorld {
        let half = (size_blocks as f32) * 0.5;
        crate::BeveledWorld::decorating(Box::new(Self::hollow_of_size(size_blocks)), half, margin)
    }

    /// Factory method decorating the Tron wireframe world with beveled seam normal transitions.
    pub fn beveled_tron(size_blocks: u32, margin: f32) -> crate::BeveledWorld {
        let half = (size_blocks as f32) * 0.5;
        crate::BeveledWorld::decorating(Box::new(Self::tron(size_blocks)), half, margin)
    }

    /// Factory method decorating the zero-knowledge Tron wireframe world with beveled seam transitions.
    pub fn hollow_beveled_tron(size_blocks: u32, margin: f32) -> crate::BeveledWorld {
        let half = (size_blocks as f32) * 0.5;
        crate::BeveledWorld::decorating(Box::new(Self::hollow_tron(size_blocks)), half, margin)
    }
}
