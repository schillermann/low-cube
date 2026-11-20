mod earthworks;
mod patches;
mod slices;

use glam::Vec3;
use crate::{
    BeveledBoundary, BlockMold, CubicBoundary, ExposedStratum, Face, GravityField, MeadowRing,
    MeshTarget, OpenBoundary, PlanetaryWorld, PlannedMount, SeamBoundary, VesselSink,
};
pub use self::patches::{MeadowScan, PatchCoord};
pub use self::slices::ModifiedSlices;

/// Cubic planetary world composed of polymorphic `Face` instances, modified-slice tracking, and seam boundary.
pub struct CubePlanet {
    faces: Vec<Box<dyn Face>>,
    fallback: Box<dyn Face>,
    slices: ModifiedSlices,
    boundary: Box<dyn SeamBoundary>,
}

impl CubePlanet {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn with_boundary(
        faces: Vec<Box<dyn Face>>,
        fallback: Box<dyn Face>,
        slices: ModifiedSlices,
        boundary: Box<dyn SeamBoundary>,
    ) -> Self {
        Self { faces, fallback, slices, boundary }
    }

    /// Secondary constructor initializing a planar/open boundary with clean slice bitmask.
    pub fn new(faces: Vec<Box<dyn Face>>, fallback: Box<dyn Face>) -> Self {
        Self::with_boundary(faces, fallback, ModifiedSlices::clean(), Box::new(OpenBoundary::new()))
    }

    /// Secondary constructor initializing a sharp 6-face cubic boundary.
    pub fn cubic(faces: Vec<Box<dyn Face>>, fallback: Box<dyn Face>, half_size: f32) -> Self {
        Self::with_boundary(
            faces,
            fallback,
            ModifiedSlices::clean(),
            Box::new(CubicBoundary::sharp(half_size)),
        )
    }

    /// Secondary constructor initializing a beveled 6-face cubic boundary.
    pub fn beveled_cubic(
        faces: Vec<Box<dyn Face>>,
        fallback: Box<dyn Face>,
        half_size: f32,
        margin: f32,
    ) -> Self {
        Self::with_boundary(
            faces,
            fallback,
            ModifiedSlices::clean(),
            Box::new(BeveledBoundary::decorating(CubicBoundary::sharp(half_size), half_size, margin)),
        )
    }

    pub(super) fn matching_index(&self, position: Vec3) -> usize {
        for (idx, face) in self.faces.iter().enumerate() {
            if face.matches(position) {
                return idx;
            }
        }
        0
    }

    pub(super) fn aimed_indices(&self, eye: Vec3, direction: Vec3) -> (usize, usize) {
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

    pub(super) fn try_aimed_faces<T>(
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
                    self.slices.record(if flora_offset { len + idx } else { idx });
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
    fn surface_up_at(&self, position: Vec3) -> Vec3 {
        self.boundary.blend_normal(position, self.active_face(position).surface_up())
    }

    fn pull_direction_at(&self, position: Vec3) -> Vec3 {
        -self.surface_up_at(position)
    }
}

impl PlanetaryWorld for CubePlanet {
    fn constrain_to_surface(&self, position: Vec3, player_height: f32) -> Vec3 {
        for (idx, face) in self.faces.iter().enumerate() {
            if face.matches(position) {
                let constrained = face.constrain(position, player_height);
                return self.boundary.fold_constrained(idx, position, constrained);
            }
        }
        self.fallback.constrain(position, player_height)
    }

    fn emit_surface(&self, target: &mut dyn MeshTarget) {
        for face in &self.faces {
            face.emit_surface(target);
        }
    }

    fn canopy_drag(&self, position: Vec3) -> f32 {
        self.active_face(position).canopy_drag(position)
    }

    fn placed_camera_tripod(&self) -> Option<(Vec3, Vec3, Vec3)> {
        self.faces.iter().find_map(|f| f.placed_camera_tripod())
    }

    fn step_surface(&mut self, dt: f32) -> bool {
        let len = self.faces.len();
        let mut regrown = false;
        for (idx, face) in self.faces.iter_mut().enumerate() {
            if face.step_surface(dt) {
                self.slices.record(idx);
                regrown = true;
            }
            if face.step_flora(dt) {
                self.slices.record(len + idx);
                regrown = true;
            }
        }
        regrown
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
        let idx = self.slices.latest();
        self.emit_face(idx, target);
        idx
    }

    fn pending_slices(&self) -> bool {
        self.slices.pending()
    }

    fn drain_modified_slice(&mut self, target: &mut dyn MeshTarget) -> Option<usize> {
        let idx = self.slices.pop_next()?;
        if let Some(face) = self.faces.get_mut(idx) {
            face.clear_terrain_patches();
        }
        self.emit_face(idx, target);
        Some(idx)
    }

    fn face_patch_count(&self, slice: usize) -> usize {
        self.faces.get(slice).map_or(0, |f| f.terrain_patch_count())
    }

    fn emit_face_patch(&self, slice: usize, patch_slot: usize, target: &mut dyn MeshTarget) {
        if let Some(face) = self.faces.get(slice) {
            face.emit_terrain_patch(patch_slot, target);
        }
    }

    fn drain_modified_patch(&mut self, target: &mut dyn MeshTarget) -> Option<(usize, usize)> {
        self.slices.drain_face_patch(&mut self.faces, target)
    }

    fn drain_stale_meadow_patch(&self, center: Vec3, ring: &mut MeadowRing, target: &mut dyn MeshTarget) -> Option<usize> {
        let idx = self.matching_index(center);
        MeadowScan::new(idx, self.faces.get(idx)?.as_ref(), center).drain_one(ring, target)
    }

    fn emit_meadow_around(&self, center: Vec3, view_dir: Vec3, target: &mut dyn MeshTarget) {
        self.active_face(center).emit_meadow_around(center, view_dir, target);
    }

    fn arboreal_perch_near(&self, center: Vec3, selector: usize) -> Option<Vec3> {
        self.active_face(center).arboreal_perch_near(center, selector)
    }

    fn visible_slice_mask(&self, position: Vec3) -> u32 {
        ModifiedSlices::horizon_mask(&self.faces, position, self.surface_up_at(position))
    }

    fn present_vessels(&self, sink: &mut dyn VesselSink) {
        for face in &self.faces {
            face.present_vessels(sink);
        }
    }

    fn plan_molded_block(&self, eye: Vec3, direction: Vec3, avatar_pos: Vec3, mold: BlockMold) -> Option<PlannedMount> {
        let (primary, secondary) = self.aimed_indices(eye, direction);
        for idx in [primary, secondary] {
            if let Some(planned) = self.faces.get(idx).and_then(|f| f.plan_molded_block(eye, direction, avatar_pos, mold)) {
                return Some(planned);
            }
            if secondary == primary {
                break;
            }
        }
        None
    }

    fn drain_disclosures(&mut self, out: &mut Vec<(u8, ExposedStratum)>) {
        let mut buf = Vec::new();
        for (idx, face) in self.faces.iter_mut().enumerate() {
            face.drain_disclosures(&mut buf);
            for stratum in buf.drain(..) {
                out.push((idx as u8, stratum));
            }
        }
    }
}
