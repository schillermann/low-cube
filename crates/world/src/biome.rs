use glam::Vec3;
use crate::{
    ClimateZone, CubePlanet, Face, FacetedReliefFace, FoldedFace,
    GravityField, MeshTarget, NullFace, PlanetaryWorld,
};

/// Multi-biome cubic planetary world decorator.
pub struct BiomedCubeWorld {
    planet: Box<dyn PlanetaryWorld>,
}

impl BiomedCubeWorld {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(planet: Box<dyn PlanetaryWorld>) -> Self {
        Self { planet }
    }

    /// Secondary constructor assembling pure procedural multi-biome planetary terrain.
    pub fn of_size(size_blocks: u32) -> Self {
        let half = (size_blocks as f32) * 0.5;

        let temperate_face = Box::new(FacetedReliefFace::in_zone(ClimateZone::Temperate, half, size_blocks));
        let temperate_face = Box::new(FoldedFace::in_zone(temperate_face, ClimateZone::Temperate, half));

        let volcanic_face = Box::new(FacetedReliefFace::in_zone(ClimateZone::Volcanic, half, size_blocks));
        let volcanic_face = Box::new(FoldedFace::in_zone(volcanic_face, ClimateZone::Volcanic, half));

        let desert_face = Box::new(FacetedReliefFace::in_zone(ClimateZone::Desert, half, size_blocks));
        let desert_face = Box::new(FoldedFace::in_zone(desert_face, ClimateZone::Desert, half));

        let arctic_face = Box::new(FacetedReliefFace::in_zone(ClimateZone::Arctic, half, size_blocks));
        let arctic_face = Box::new(FoldedFace::in_zone(arctic_face, ClimateZone::Arctic, half));

        let jungle_face = Box::new(FacetedReliefFace::in_zone(ClimateZone::Jungle, half, size_blocks));
        let jungle_face = Box::new(FoldedFace::in_zone(jungle_face, ClimateZone::Jungle, half));

        let steppe_face = Box::new(FacetedReliefFace::in_zone(ClimateZone::Steppe, half, size_blocks));
        let steppe_face = Box::new(FoldedFace::in_zone(steppe_face, ClimateZone::Steppe, half));

        let faces: Vec<Box<dyn Face>> = vec![
            temperate_face,
            volcanic_face,
            desert_face,
            arctic_face,
            jungle_face,
            steppe_face,
        ];

        Self::new(Box::new(CubePlanet::new(faces, Box::new(NullFace))))
    }

    /// Secondary constructor decorating the multi-biome world with continuous seam normal transitions.
    pub fn beveled(size_blocks: u32, margin: f32) -> crate::BeveledWorld {
        let half = (size_blocks as f32) * 0.5;
        let base = Box::new(Self::of_size(size_blocks));
        crate::BeveledWorld::decorating(base, half, margin)
    }

    /// Secondary constructor decorating the world with beveled seams and centripetal gravity falloff.
    pub fn centripetal(size_blocks: u32, bevel_margin: f32, blend_margin: f32) -> crate::CentripetalWorld {
        let half = (size_blocks as f32) * 0.5;
        let beveled = Box::new(Self::beveled(size_blocks, bevel_margin));
        crate::CentripetalWorld::new(beveled, half, blend_margin)
    }
}

impl GravityField for BiomedCubeWorld {
    fn pull_direction_at(&self, position: Vec3) -> Vec3 {
        self.planet.pull_direction_at(position)
    }

    fn surface_up_at(&self, position: Vec3) -> Vec3 {
        self.planet.surface_up_at(position)
    }
}

impl PlanetaryWorld for BiomedCubeWorld {
    fn constrain_to_surface(&self, position: Vec3, player_height: f32) -> Vec3 {
        self.planet.constrain_to_surface(position, player_height)
    }

    fn emit_surface(&self, target: &mut dyn MeshTarget) {
        self.planet.emit_surface(target);
    }
}
