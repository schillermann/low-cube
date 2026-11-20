use crate::{ClimateZone, CubePlanet, Face, FacetedReliefFace, ForestedFace, NullFace};

/// Multi-biome cubic planetary world factory assembling climate zones and seam boundaries.
pub struct BiomedCubeWorld;

impl BiomedCubeWorld {
    fn build_faces(
        size_blocks: u32,
        mut make_face: impl FnMut(ClimateZone, f32, u32) -> Box<dyn Face>,
    ) -> (Vec<Box<dyn Face>>, f32) {
        let half = (size_blocks as f32) * 0.5;
        let divs = size_blocks * 2;
        let faces = ClimateZone::all()
            .into_iter()
            .map(|zone| make_face(zone, half, divs))
            .collect();
        (faces, half)
    }

    fn forested_faces(size_blocks: u32) -> (Vec<Box<dyn Face>>, f32) {
        Self::build_faces(size_blocks, |zone, half, divs| {
            let relief = Box::new(FacetedReliefFace::in_zone(zone, half, divs));
            Box::new(ForestedFace::in_zone(relief, zone, half))
        })
    }

    fn hollow_forested_faces(size_blocks: u32) -> (Vec<Box<dyn Face>>, f32) {
        Self::build_faces(size_blocks, |zone, half, divs| {
            let relief = Box::new(FacetedReliefFace::hollow_in_zone(zone, half, divs));
            Box::new(ForestedFace::in_zone(relief, zone, half))
        })
    }

    fn barren_faces(size_blocks: u32) -> (Vec<Box<dyn Face>>, f32) {
        Self::build_faces(size_blocks, |zone, half, divs| {
            Box::new(FacetedReliefFace::in_zone(zone, half, divs))
        })
    }

    /// Factory method assembling authoritative multi-biome planetary terrain with living mineral veins.
    pub fn of_size(size_blocks: u32) -> CubePlanet {
        let (faces, half) = Self::forested_faces(size_blocks);
        CubePlanet::cubic(faces, Box::new(NullFace), half)
    }

    /// Factory method assembling zero-knowledge client planetary terrain without unexposed ores.
    pub fn hollow_of_size(size_blocks: u32) -> CubePlanet {
        let (faces, half) = Self::hollow_forested_faces(size_blocks);
        CubePlanet::cubic(faces, Box::new(NullFace), half)
    }

    /// Factory method assembling barren multi-biome planetary terrain without woodland flora for fast simulation.
    pub fn barren(size_blocks: u32) -> CubePlanet {
        let (faces, half) = Self::barren_faces(size_blocks);
        CubePlanet::cubic(faces, Box::new(NullFace), half)
    }

    /// Factory method assembling barren multi-biome terrain with continuous seam normal bevels.
    pub fn barren_beveled(size_blocks: u32, margin: f32) -> CubePlanet {
        let (faces, half) = Self::barren_faces(size_blocks);
        CubePlanet::beveled_cubic(faces, Box::new(NullFace), half, margin)
    }

    /// Factory method assembling multi-biome terrain with continuous seam normal transitions.
    pub fn beveled(size_blocks: u32, margin: f32) -> CubePlanet {
        let (faces, half) = Self::forested_faces(size_blocks);
        CubePlanet::beveled_cubic(faces, Box::new(NullFace), half, margin)
    }

    /// Factory method assembling zero-knowledge client terrain with continuous seam normal transitions.
    pub fn hollow_beveled(size_blocks: u32, margin: f32) -> CubePlanet {
        let (faces, half) = Self::hollow_forested_faces(size_blocks);
        CubePlanet::beveled_cubic(faces, Box::new(NullFace), half, margin)
    }
}
