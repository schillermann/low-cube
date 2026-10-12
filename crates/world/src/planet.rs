use glam::Vec3;
use crate::{Face, GravityField, MeshTarget, PlanetaryWorld};

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
