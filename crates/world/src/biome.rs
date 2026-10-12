use glam::Vec3;
use crate::{
    BlockGallery, CubeFace, CubePlanet, Face, FoldedFace, GravityField, MeshTarget,
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

    /// Secondary constructor assembling distinct biome faces with pure decorators.
    pub fn of_size(size_blocks: u32) -> Self {
        let half = (size_blocks as f32) * 0.5;
        let assemble_face = |face: CubeFace, color: Vec3| -> Box<dyn Face> {
            let base = Box::new(PlanarFace::on_cube(face, color, half, size_blocks));
            Box::new(FoldedFace::on_cube(base, face, half))
        };

        let top_face = Box::new(PlanarFace::on_cube(
            CubeFace::Top,
            Vec3::new(0.28, 0.65, 0.25),
            half,
            size_blocks,
        ));
        let top_face = Box::new(VoxelMountain::alpine(top_face, half));
        let top_face = Box::new(BlockGallery::parallel_lines(top_face, half));
        let top_face = Box::new(FoldedFace::on_cube(top_face, CubeFace::Top, half));

        let faces: Vec<Box<dyn Face>> = vec![
            // Top (+Y): Temperate Green with Alpine Mountain and Block Gallery
            top_face,
            // Bottom (-Y): Volcanic Dark Red
            assemble_face(CubeFace::Bottom, Vec3::new(0.35, 0.18, 0.18)),
            // Right (+X): Desert Sand
            assemble_face(CubeFace::Right, Vec3::new(0.85, 0.75, 0.35)),
            // Left (-X): Arctic Ice
            assemble_face(CubeFace::Left, Vec3::new(0.75, 0.88, 0.95)),
            // Front (+Z): Jungle Emerald
            assemble_face(CubeFace::Front, Vec3::new(0.12, 0.45, 0.22)),
            // Back (-Z): Arid Steppe Brown
            assemble_face(CubeFace::Back, Vec3::new(0.68, 0.48, 0.32)),
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
