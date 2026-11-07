use glam::Vec3;
use crate::{
    BlockMedium, BlockMold, CleaveOutcome, Face, GravityField, MeshTarget, PlanetaryWorld,
    SurfaceFind, VehicleAssembly,
};

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

    fn aimed_indices(&self, eye: Vec3, direction: Vec3) -> (usize, usize) {
        let primary = self.matching_index(eye);
        let secondary = self.matching_index(eye + direction.normalize_or_zero() * 1.25);
        (primary, secondary)
    }

    fn active_face(&self, position: Vec3) -> &dyn Face {
        for face in &self.faces {
            if face.matches(position) {
                return face.as_ref();
            }
        }
        self.fallback.as_ref()
    }

    fn try_aimed_faces<T>(
        &mut self,
        eye: Vec3,
        direction: Vec3,
        flora_offset: bool,
        mut op: impl FnMut(&mut dyn Face) -> Option<T>,
    ) -> Option<T> {
        let len = self.faces.len();
        let (primary, secondary) = self.aimed_indices(eye, direction);
        for idx in [primary, secondary] {
            if let Some(face) = self.faces.get_mut(idx) {
                if let Some(out) = op(face.as_mut()) {
                    self.modified_face = if flora_offset { len + idx } else { idx };
                    return Some(out);
                }
            }
            if secondary == primary {
                break;
            }
        }
        None
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

    fn harvest_surface(&mut self, eye: Vec3, direction: Vec3) -> Option<BlockMedium> {
        self.try_aimed_faces(eye, direction, false, |f| f.harvest(eye, direction))
    }

    fn loosen_surface(&mut self, eye: Vec3, direction: Vec3, stage: u8) -> Option<BlockMedium> {
        let (primary, secondary) = self.aimed_indices(eye, direction);
        if let Some(face) = self.faces.get_mut(primary) {
            if let Some(medium) = face.loosen(eye, direction, stage) {
                self.modified_face = primary;
                return Some(medium);
            }
        }
        let alt = if stage == 0 && self.modified_face < self.faces.len() { self.modified_face } else { secondary };
        if alt != primary {
            if let Some(face) = self.faces.get_mut(alt) {
                if let Some(medium) = face.loosen(eye, direction, stage) {
                    self.modified_face = alt;
                    return Some(medium);
                }
            }
        }
        None
    }

    fn strike_timber(&mut self, eye: Vec3, direction: Vec3) -> CleaveOutcome {
        self.try_aimed_faces(eye, direction, true, |f| {
            let outcome = f.strike_timber(eye, direction);
            outcome.struck().then_some(outcome)
        })
        .unwrap_or(CleaveOutcome::Missed)
    }

    fn forage_surface(&mut self, eye: Vec3, direction: Vec3) -> SurfaceFind {
        self.try_aimed_faces(eye, direction, true, |f| {
            let find = f.forage_timber(eye, direction);
            find.gathered().then_some(find)
        })
        .unwrap_or(SurfaceFind::EmptyHanded)
    }

    fn place_molded_surface_block(&mut self, eye: Vec3, direction: Vec3, avatar_pos: Vec3, mold: BlockMold) -> bool {
        self.try_aimed_faces(eye, direction, false, |f| {
            f.place_molded_block(eye, direction, avatar_pos, mold).then_some(())
        })
        .is_some()
    }

    fn adjust_rig(&mut self, eye: Vec3, direction: Vec3, delta_steps: i32, rotate_quarter: bool) -> bool {
        self.try_aimed_faces(eye, direction, false, |f| {
            f.adjust_rig(eye, direction, delta_steps, rotate_quarter).then_some(())
        })
        .is_some()
    }

    fn lower_rig(&mut self, eye: Vec3, direction: Vec3) -> bool {
        self.try_aimed_faces(eye, direction, false, |f| f.lower_rig(eye, direction).then_some(())).is_some()
    }

    fn board_vehicle(&mut self, eye: Vec3, direction: Vec3, holding_block: bool) -> Option<VehicleAssembly> {
        self.try_aimed_faces(eye, direction, false, |f| f.board_vehicle(eye, direction, holding_block))
    }

    fn park_vehicle(&mut self, assembly: VehicleAssembly) -> bool {
        let idx = self.matching_index(assembly.origin());
        if let Some(face) = self.faces.get_mut(idx) {
            if face.park_vehicle(assembly) {
                self.modified_face = idx;
                return true;
            }
        }
        false
    }

    fn placed_camera_tripod(&self) -> Option<(Vec3, Vec3, Vec3)> {
        self.faces.iter().find_map(|f| f.placed_camera_tripod())
    }

    fn step_surface(&mut self, dt: f32) -> bool {
        let len = self.faces.len();
        let mut regrown = false;
        for (idx, face) in self.faces.iter_mut().enumerate() {
            if face.step_surface(dt) {
                self.modified_face = idx;
                regrown = true;
            } else if !regrown && face.step_flora(dt) {
                self.modified_face = len + idx;
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
        let len = self.faces.len();
        if let Some(face) = self.faces.get(index) {
            face.emit_terrain(target);
        } else if index >= len && index < len * 2 {
            self.faces[index - len].emit_flora(target);
        } else {
            self.fallback.emit_surface(target);
        }
    }

    fn emit_modified_face(&self, target: &mut dyn MeshTarget) -> usize {
        self.emit_face(self.modified_face, target);
        self.modified_face
    }

    fn emit_meadow_around(&self, center: Vec3, view_dir: Vec3, target: &mut dyn MeshTarget) {
        self.active_face(center).emit_meadow_around(center, view_dir, target);
    }

    fn visible_slice_mask(&self, position: Vec3) -> u32 {
        if self.weightless_at(position) {
            return u32::MAX;
        }
        let len = self.faces.len();
        let up = self.surface_up_at(position);
        let mut mask = 1u32 << (len * 2).min(31);
        for (idx, face) in self.faces.iter().enumerate() {
            let f_up = face.surface_up();
            if f_up.dot(up) > -0.5 {
                mask |= 1u32 << idx.min(31);
                if f_up.dot(up) > 0.2 || position.dot(f_up) >= -4.0 {
                    mask |= 1u32 << (len + idx).min(31);
                }
            }
        }
        mask
    }
}
