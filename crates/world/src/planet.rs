use glam::Vec3;
use crate::{
    Face, GravityField, MeshTarget, PlanarFace, PlanetaryWorld, ReliefFace, RidgeElevation,
};

/// Null Object pattern implementation for Face.
pub struct NullFace;

impl Face for NullFace {
    fn matches(&self, _point: Vec3) -> bool {
        false
    }

    fn surface_up(&self) -> Vec3 {
        Vec3::Y
    }

    fn pull_direction(&self) -> Vec3 {
        Vec3::NEG_Y
    }

    fn constrain(&self, point: Vec3, _distance: f32) -> Vec3 {
        point
    }

    fn emit_surface(&self, _target: &mut dyn MeshTarget) {}
}

/// A cubic planetary world composed of polymorphic Face instances with Dependency Injection.
pub struct CubePlanet {
    faces: Vec<Box<dyn Face>>,
    fallback: Box<dyn Face>,
}

impl CubePlanet {
    /// Primary constructor: code-free, assigns fields directly.
    pub fn new(faces: Vec<Box<dyn Face>>, fallback: Box<dyn Face>) -> Self {
        Self { faces, fallback }
    }

    fn active_face(&self, position: Vec3) -> &dyn Face {
        for face in &self.faces {
            if face.matches(position) {
                return face.as_ref();
            }
        }
        self.fallback.as_ref()
    }
}

impl GravityField for CubePlanet {
    fn pull_direction_at(&self, position: Vec3) -> Vec3 {
        self.active_face(position).pull_direction()
    }

    fn surface_up_at(&self, position: Vec3) -> Vec3 {
        self.active_face(position).surface_up()
    }
}

impl PlanetaryWorld for CubePlanet {
    fn constrain_to_surface(&self, position: Vec3, player_height: f32) -> Vec3 {
        self.active_face(position).constrain(position, player_height)
    }

    fn emit_surface(&self, target: &mut dyn MeshTarget) {
        for face in &self.faces {
            face.emit_surface(target);
        }
    }
}

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
        let faces: Vec<Box<dyn Face>> = vec![
            // Top (+Y): Temperate Green with Climbable Alpine Ridge
            Box::new(ReliefFace::new(
                Vec3::Y,
                Vec3::X,
                Vec3::Z,
                Vec3::new(0.28, 0.65, 0.25),
                half,
                size_blocks,
                Box::new(RidgeElevation::alpine(0.0, 4.0)),
            )),
            // Bottom (-Y): Volcanic Dark Red
            Box::new(PlanarFace::new(
                Vec3::NEG_Y,
                Vec3::X,
                Vec3::NEG_Z,
                Vec3::new(0.35, 0.18, 0.18),
                half,
                size_blocks,
            )),
            // Right (+X): Desert Sand
            Box::new(PlanarFace::new(
                Vec3::X,
                Vec3::NEG_Z,
                Vec3::Y,
                Vec3::new(0.85, 0.75, 0.35),
                half,
                size_blocks,
            )),
            // Left (-X): Arctic Ice
            Box::new(PlanarFace::new(
                Vec3::NEG_X,
                Vec3::Z,
                Vec3::Y,
                Vec3::new(0.75, 0.88, 0.95),
                half,
                size_blocks,
            )),
            // Front (+Z): Jungle Emerald
            Box::new(PlanarFace::new(
                Vec3::Z,
                Vec3::X,
                Vec3::Y,
                Vec3::new(0.12, 0.45, 0.22),
                half,
                size_blocks,
            )),
            // Back (-Z): Arid Steppe Brown
            Box::new(PlanarFace::new(
                Vec3::NEG_Z,
                Vec3::NEG_X,
                Vec3::Y,
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
