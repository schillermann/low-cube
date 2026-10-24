use glam::Vec3;
use crate::{BlockMedium, BlockMold, CleaveOutcome, Face, GravityField, MeshTarget, PlanetaryWorld, SurfaceFind};

/// A cubic planetary world composed of polymorphic Face instances with Dependency Injection.
pub struct CubePlanet {
    faces: Vec<Box<dyn Face>>,
    fallback: Box<dyn Face>,
    modified_face: usize,
}

impl CubePlanet {
    /// Primary constructor: code-free, assigns fields directly.
    pub fn with_modified(
        faces: Vec<Box<dyn Face>>,
        fallback: Box<dyn Face>,
        modified_face: usize,
    ) -> Self {
        Self {
            faces,
            fallback,
            modified_face,
        }
    }

    /// Secondary constructor initializing with first face index.
    pub fn new(faces: Vec<Box<dyn Face>>, fallback: Box<dyn Face>) -> Self {
        Self::with_modified(faces, fallback, 0)
    }

    fn matching_index(&self, position: Vec3) -> usize {
        for (idx, face) in self.faces.iter().enumerate() {
            if face.matches(position) {
                return idx;
            }
        }
        0
    }

    fn active_face(&self, position: Vec3) -> &dyn Face {
        for face in &self.faces {
            if face.matches(position) {
                return face.as_ref();
            }
        }
        self.fallback.as_ref()
    }

    fn active_face_mut(&mut self, position: Vec3) -> &mut dyn Face {
        for face in &mut self.faces {
            if face.matches(position) {
                return face.as_mut();
            }
        }
        self.fallback.as_mut()
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

    fn canopy_drag(&self, position: Vec3) -> f32 {
        self.active_face(position).canopy_drag(position)
    }

    fn excavate_surface(&mut self, eye: Vec3, direction: Vec3) -> bool {
        self.modified_face = self.matching_index(eye);
        self.active_face_mut(eye).excavate(eye, direction)
    }

    fn surface_medium(&self, eye: Vec3, direction: Vec3) -> BlockMedium {
        self.active_face(eye).surface_medium(eye, direction)
    }

    fn loosen_surface(&mut self, eye: Vec3, direction: Vec3, stage: u8) -> bool {
        self.modified_face = self.matching_index(eye);
        self.active_face_mut(eye).loosen(eye, direction, stage)
    }

    fn strike_timber(&mut self, eye: Vec3, direction: Vec3) -> CleaveOutcome {
        self.modified_face = self.matching_index(eye);
        self.active_face_mut(eye).strike_timber(eye, direction)
    }

    fn forage_surface(&mut self, eye: Vec3, direction: Vec3) -> SurfaceFind {
        self.modified_face = self.matching_index(eye);
        self.active_face_mut(eye).forage_timber(eye, direction)
    }

    fn place_surface_block(&mut self, eye: Vec3, direction: Vec3, avatar_pos: Vec3) -> bool {
        self.place_molded_surface_block(eye, direction, avatar_pos, BlockMold::cube())
    }

    fn place_molded_surface_block(&mut self, eye: Vec3, direction: Vec3, avatar_pos: Vec3, mold: BlockMold) -> bool {
        self.modified_face = self.matching_index(eye);
        self.active_face_mut(eye).place_molded_block(eye, direction, avatar_pos, mold)
    }

    fn step_surface(&mut self, dt: f32) -> bool {
        let mut regrown = false;
        for (idx, face) in self.faces.iter_mut().enumerate() {
            if face.step_surface(dt) {
                self.modified_face = idx;
                regrown = true;
            }
        }
        regrown
    }

    fn weightless_at(&self, position: Vec3) -> bool {
        self.active_face(position).weightless_at(position)
    }

    fn lethal_at(&self, position: Vec3) -> bool {
        self.active_face(position).lethal_at(position)
    }

    fn confine_beneath_crust(&self, position: Vec3) -> Vec3 {
        self.active_face(position).confine_beneath_crust(position)
    }

    fn face_count(&self) -> usize {
        self.faces.len().max(1)
    }

    fn emit_face(&self, index: usize, target: &mut dyn MeshTarget) {
        if let Some(face) = self.faces.get(index) {
            face.emit_surface(target);
        } else {
            self.fallback.emit_surface(target);
        }
    }

    fn emit_modified_face(&self, target: &mut dyn MeshTarget) -> usize {
        self.emit_face(self.modified_face, target);
        self.modified_face
    }
}
