use crate::{
    ClimateZone, CubePlanet, Face, FacetedReliefFace, FoldedFace, ForestedFace,
    NullFace,
};

/// Multi-biome cubic planetary world factory assembling climate zones and terrain decorators.
pub struct BiomedCubeWorld;

impl BiomedCubeWorld {
    /// Factory method assembling pure procedural multi-biome planetary terrain.
    pub fn of_size(size_blocks: u32) -> CubePlanet {
        let half = (size_blocks as f32) * 0.5;
        let divs = size_blocks * 2;

        let temperate_relief = Box::new(FacetedReliefFace::in_zone(ClimateZone::Temperate, half, divs));
        let temperate_forested = Box::new(ForestedFace::in_zone(temperate_relief, ClimateZone::Temperate, half));
        let temperate_face = Box::new(FoldedFace::in_zone(temperate_forested, ClimateZone::Temperate, half));

        let volcanic_relief = Box::new(FacetedReliefFace::in_zone(ClimateZone::Volcanic, half, divs));
        let volcanic_forested = Box::new(ForestedFace::in_zone(volcanic_relief, ClimateZone::Volcanic, half));
        let volcanic_face = Box::new(FoldedFace::in_zone(volcanic_forested, ClimateZone::Volcanic, half));

        let desert_relief = Box::new(FacetedReliefFace::in_zone(ClimateZone::Desert, half, divs));
        let desert_forested = Box::new(ForestedFace::in_zone(desert_relief, ClimateZone::Desert, half));
        let desert_face = Box::new(FoldedFace::in_zone(desert_forested, ClimateZone::Desert, half));

        let arctic_relief = Box::new(FacetedReliefFace::in_zone(ClimateZone::Arctic, half, divs));
        let arctic_forested = Box::new(ForestedFace::in_zone(arctic_relief, ClimateZone::Arctic, half));
        let arctic_face = Box::new(FoldedFace::in_zone(arctic_forested, ClimateZone::Arctic, half));

        let jungle_relief = Box::new(FacetedReliefFace::in_zone(ClimateZone::Jungle, half, divs));
        let jungle_forested = Box::new(ForestedFace::in_zone(jungle_relief, ClimateZone::Jungle, half));
        let jungle_face = Box::new(FoldedFace::in_zone(jungle_forested, ClimateZone::Jungle, half));

        let steppe_relief = Box::new(FacetedReliefFace::in_zone(ClimateZone::Steppe, half, divs));
        let steppe_forested = Box::new(ForestedFace::in_zone(steppe_relief, ClimateZone::Steppe, half));
        let steppe_face = Box::new(FoldedFace::in_zone(steppe_forested, ClimateZone::Steppe, half));

        let faces: Vec<Box<dyn Face>> = vec![
            temperate_face,
            volcanic_face,
            desert_face,
            arctic_face,
            jungle_face,
            steppe_face,
        ];

        CubePlanet::new(faces, Box::new(NullFace))
    }

    /// Factory method assembling planetary terrain in Tron wireframe visualization mode.
    pub fn tron(size_blocks: u32) -> CubePlanet {
        let half = (size_blocks as f32) * 0.5;
        let divs = size_blocks * 2;

        let temperate_relief = Box::new(FacetedReliefFace::tron(ClimateZone::Temperate, half, divs));
        let temperate_face = Box::new(FoldedFace::in_zone(temperate_relief, ClimateZone::Temperate, half));

        let faces: Vec<Box<dyn Face>> = vec![
            temperate_face,
            Box::new(FoldedFace::in_zone(Box::new(FacetedReliefFace::tron(ClimateZone::Volcanic, half, divs)), ClimateZone::Volcanic, half)),
            Box::new(FoldedFace::in_zone(Box::new(FacetedReliefFace::tron(ClimateZone::Desert, half, divs)), ClimateZone::Desert, half)),
            Box::new(FoldedFace::in_zone(Box::new(FacetedReliefFace::tron(ClimateZone::Arctic, half, divs)), ClimateZone::Arctic, half)),
            Box::new(FoldedFace::in_zone(Box::new(FacetedReliefFace::tron(ClimateZone::Jungle, half, divs)), ClimateZone::Jungle, half)),
            Box::new(FoldedFace::in_zone(Box::new(FacetedReliefFace::tron(ClimateZone::Steppe, half, divs)), ClimateZone::Steppe, half)),
        ];

        CubePlanet::new(faces, Box::new(NullFace))
    }

    /// Factory method decorating the multi-biome world with continuous seam normal transitions.
    pub fn beveled(size_blocks: u32, margin: f32) -> crate::BeveledWorld {
        let half = (size_blocks as f32) * 0.5;
        let base = Box::new(Self::of_size(size_blocks));
        crate::BeveledWorld::decorating(base, half, margin)
    }

    /// Factory method decorating the Tron wireframe world with beveled seam normal transitions.
    pub fn beveled_tron(size_blocks: u32, margin: f32) -> crate::BeveledWorld {
        let half = (size_blocks as f32) * 0.5;
        let base = Box::new(Self::tron(size_blocks));
        crate::BeveledWorld::decorating(base, half, margin)
    }

    /// Factory method decorating the world with beveled seams and centripetal gravity falloff.
    pub fn centripetal(size_blocks: u32, bevel_margin: f32, blend_margin: f32) -> crate::CentripetalWorld {
        let half = (size_blocks as f32) * 0.5;
        let beveled = Box::new(Self::beveled(size_blocks, bevel_margin));
        crate::CentripetalWorld::new(beveled, half, blend_margin)
    }
}
