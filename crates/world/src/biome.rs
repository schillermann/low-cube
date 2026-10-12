use glam::Vec3;
use crate::{
    BlockGallery, ClimateZone, CubePlanet, Face, FoldedFace, GravityField, MeshTarget,
    NullFace, PlanarFace, PlanetaryWorld, VoxelMountain,
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

    /// Secondary constructor assembling distinct biome climate zones with pure decorators.
    pub fn of_size(size_blocks: u32) -> Self {
        let half = (size_blocks as f32) * 0.5;
        let assemble_face = |zone: ClimateZone, color: Vec3| -> Box<dyn Face> {
            let base = Box::new(PlanarFace::in_zone(zone, color, half, size_blocks));
            Box::new(FoldedFace::in_zone(base, zone, half))
        };

        let temperate_face = Box::new(PlanarFace::in_zone(
            ClimateZone::Temperate,
            Vec3::new(0.28, 0.65, 0.25),
            half,
            size_blocks,
        ));
        let temperate_face = Box::new(VoxelMountain::alpine(temperate_face, half));
        let temperate_face = Box::new(BlockGallery::parallel_lines(temperate_face, half));
        let temperate_face = Box::new(FoldedFace::in_zone(temperate_face, ClimateZone::Temperate, half));

        let faces: Vec<Box<dyn Face>> = vec![
            temperate_face,
            assemble_face(ClimateZone::Volcanic, Vec3::new(0.35, 0.18, 0.18)),
            assemble_face(ClimateZone::Desert, Vec3::new(0.85, 0.75, 0.35)),
            assemble_face(ClimateZone::Arctic, Vec3::new(0.75, 0.88, 0.95)),
            assemble_face(ClimateZone::Jungle, Vec3::new(0.12, 0.45, 0.22)),
            assemble_face(ClimateZone::Steppe, Vec3::new(0.68, 0.48, 0.32)),
        ];

        Self::new(Box::new(CubePlanet::new(faces, Box::new(NullFace))))
    }

    /// Secondary constructor decorating the multi-biome world with continuous seam normal transitions.
    pub fn beveled(size_blocks: u32, margin: f32) -> crate::BeveledWorld {
        let half = (size_blocks as f32) * 0.5;
        let base = Box::new(Self::of_size(size_blocks));
        crate::BeveledWorld::decorating(base, half, margin)
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
