use glam::Vec3;
use crate::{Face, GravityField, PlanarFace, PlanetaryWorld, SurfaceMeshReceiver};

/// A cubic planetary world composed of polymorphic Face instances with Dependency Injection.
pub struct CubePlanet {
    half_size: f32,
    divisions: u32,
    faces: Vec<Box<dyn Face>>,
}

impl CubePlanet {
    /// Primary constructor: code-free, assigns fields directly.
    pub fn new(half_size: f32, divisions: u32, faces: Vec<Box<dyn Face>>) -> Self {
        Self {
            half_size,
            divisions,
            faces,
        }
    }

    fn active_face(&self, position: Vec3) -> &dyn Face {
        for face in &self.faces {
            if face.matches(position) {
                return face.as_ref();
            }
        }
        self.faces[0].as_ref()
    }
}

impl GravityField for CubePlanet {
    fn pull_direction_at(&self, position: Vec3) -> Vec3 {
        -self.active_face(position).normal()
    }

    fn surface_up_at(&self, position: Vec3) -> Vec3 {
        self.active_face(position).normal()
    }
}

impl PlanetaryWorld for CubePlanet {
    fn constrain_to_surface(&self, position: Vec3, player_height: f32) -> Vec3 {
        self.active_face(position).constrain(position, self.half_size, player_height)
    }

    fn emit_surface(&self, receiver: &mut dyn SurfaceMeshReceiver) {
        for face in &self.faces {
            face.emit_surface(self.half_size, self.divisions, receiver);
        }
    }
}

/// Factory constructing standard cubic planetary worlds.
pub struct StandardPlanetFactory;

impl StandardPlanetFactory {
    pub fn create_cube(size_blocks: u32) -> CubePlanet {
        let half = (size_blocks as f32) * 0.5;
        let faces: Vec<Box<dyn Face>> = vec![
            // Top (+Y): Temperate Green
            Box::new(PlanarFace::new(
                Vec3::Y,
                Vec3::X,
                Vec3::Z,
                Vec3::new(0.28, 0.65, 0.25),
            )),
            // Bottom (-Y): Volcanic Dark Red
            Box::new(PlanarFace::new(
                Vec3::NEG_Y,
                Vec3::X,
                Vec3::NEG_Z,
                Vec3::new(0.35, 0.18, 0.18),
            )),
            // Right (+X): Desert Sand
            Box::new(PlanarFace::new(
                Vec3::X,
                Vec3::NEG_Z,
                Vec3::Y,
                Vec3::new(0.85, 0.75, 0.35),
            )),
            // Left (-X): Arctic Ice
            Box::new(PlanarFace::new(
                Vec3::NEG_X,
                Vec3::Z,
                Vec3::Y,
                Vec3::new(0.75, 0.88, 0.95),
            )),
            // Front (+Z): Jungle Emerald
            Box::new(PlanarFace::new(
                Vec3::Z,
                Vec3::X,
                Vec3::Y,
                Vec3::new(0.12, 0.45, 0.22),
            )),
            // Back (-Z): Arid Steppe Brown
            Box::new(PlanarFace::new(
                Vec3::NEG_Z,
                Vec3::NEG_X,
                Vec3::Y,
                Vec3::new(0.68, 0.48, 0.32),
            )),
        ];

        CubePlanet::new(half, size_blocks, faces)
    }
}
