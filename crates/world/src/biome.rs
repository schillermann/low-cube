use glam::Vec3;
use crate::{
    BlockGallery, CubeFace, CubePlanet, Face, GravityField, MeshTarget, NullFace,
    PlanarFace, PlanetaryWorld, VoxelMountain,
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

    /// Secondary constructor assembling distinct biome faces.
    pub fn of_size(size_blocks: u32) -> Self {
        let half = (size_blocks as f32) * 0.5;
        let top_face = Box::new(PlanarFace::on_cube(
            CubeFace::Top,
            Vec3::new(0.28, 0.65, 0.25),
            half,
            size_blocks,
        ));
        let top_face = Box::new(VoxelMountain::alpine(top_face, half));
        let top_face = Box::new(BlockGallery::parallel_lines(top_face, half));

        let faces: Vec<Box<dyn Face>> = vec![
            // Top (+Y): Temperate Green with Alpine Mountain and Block Gallery
            top_face,
            // Bottom (-Y): Volcanic Dark Red
            Box::new(PlanarFace::on_cube(
                CubeFace::Bottom,
                Vec3::new(0.35, 0.18, 0.18),
                half,
                size_blocks,
            )),
            // Right (+X): Desert Sand
            Box::new(PlanarFace::on_cube(
                CubeFace::Right,
                Vec3::new(0.85, 0.75, 0.35),
                half,
                size_blocks,
            )),
            // Left (-X): Arctic Ice
            Box::new(PlanarFace::on_cube(
                CubeFace::Left,
                Vec3::new(0.75, 0.88, 0.95),
                half,
                size_blocks,
            )),
            // Front (+Z): Jungle Emerald
            Box::new(PlanarFace::on_cube(
                CubeFace::Front,
                Vec3::new(0.12, 0.45, 0.22),
                half,
                size_blocks,
            )),
            // Back (-Z): Arid Steppe Brown
            Box::new(PlanarFace::on_cube(
                CubeFace::Back,
                Vec3::new(0.68, 0.48, 0.32),
                half,
                size_blocks,
            )),
        ];

        Self::new(Box::new(CubePlanet::new(faces, Box::new(NullFace))))
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
