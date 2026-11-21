use glam::Vec3;
use crate::{
    BlockMold, CompanionPortal, CraftBlueprint, ExposedStratum, GravityField as _, MeadowRing,
    MeshTarget, PlannedMount, TerrainMesh, VesselDeck, VesselSink, WoodlandLife,
};
use super::{CubePlanet, MeadowScan, ModifiedSlices};

impl TerrainMesh for CubePlanet {
    fn emit_surface(&self, target: &mut dyn MeshTarget) {
        for face in &self.faces {
            face.emit_surface(target);
        }
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

    fn visible_slice_mask(&self, position: Vec3) -> u32 {
        ModifiedSlices::horizon_mask(&self.faces, position, self.surface_up_at(position))
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

impl WoodlandLife for CubePlanet {
    fn arboreal_perch_near(&self, center: Vec3, selector: usize) -> Option<Vec3> {
        self.active_face(center).arboreal_perch_near(center, selector)
    }
}

impl VesselDeck for CubePlanet {
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

    fn capture_vehicle_blueprint(&self, eye: Vec3, direction: Vec3) -> Option<CraftBlueprint> {
        let (primary, secondary) = self.aimed_indices(eye, direction);
        for idx in [primary, secondary] {
            if let Some(bp) = self.faces.get(idx).and_then(|f| f.capture_vehicle_blueprint(eye, direction)) {
                return Some(bp);
            }
            if secondary == primary {
                break;
            }
        }
        None
    }

    fn aimed_empty_rig(&self, eye: Vec3, direction: Vec3, avatar_pos: Vec3) -> bool {
        let (primary, secondary) = self.aimed_indices(eye, direction);
        for idx in [primary, secondary] {
            if self.faces.get(idx).is_some_and(|f| f.aimed_empty_rig(eye, direction, avatar_pos)) {
                return true;
            }
            if secondary == primary {
                break;
            }
        }
        false
    }
}

impl CompanionPortal for CubePlanet {}
