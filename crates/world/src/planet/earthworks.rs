use glam::Vec3;
use crate::{
    BlockMedium, BlockMold, CleaveOutcome, CraftBlueprint, ExposedStratum, MiningEdge, SurfaceFind,
    VehicleAssembly, WorldEarthworks,
};
use super::QuadSphere;

impl WorldEarthworks for QuadSphere {
    fn harvest_surface(&mut self, eye: Vec3, direction: Vec3) -> Option<BlockMedium> {
        self.try_aimed_faces(eye, direction, false, |f| f.harvest(eye, direction))
    }

    fn loosen_surface(&mut self, eye: Vec3, direction: Vec3, stage: u8) -> Option<BlockMedium> {
        self.try_aimed_faces(eye, direction, false, |f| f.loosen(eye, direction, stage))
    }

    fn surface_yields_to(&self, eye: Vec3, direction: Vec3, edge: MiningEdge) -> bool {
        self.inspect_aimed_faces(eye, direction, |f| f.surface_yields_to(eye, direction, edge))
            .unwrap_or(false)
    }

    fn aimed_surface_medium(&self, eye: Vec3, direction: Vec3) -> Option<BlockMedium> {
        self.inspect_aimed_faces(eye, direction, |f| f.aimed_surface_medium(eye, direction))
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

    fn cast_surface_find(&mut self, eye: Vec3, direction: Vec3, find: SurfaceFind) -> bool {
        let len = self.faces.len();
        let (primary, secondary) = self.aimed_indices(eye, direction);
        let order = if secondary != primary { [secondary, primary] } else { [primary, primary] };
        for (step, idx) in order.into_iter().enumerate() {
            if step == 1 && secondary == primary {
                break;
            }
            if let Some(face) = self.faces.get_mut(idx) {
                if face.cast_litter(eye, direction, find) {
                    self.slices.record(len + idx);
                    return true;
                }
            }
        }
        false
    }

    fn place_molded_surface_block(&mut self, eye: Vec3, direction: Vec3, avatar_pos: Vec3, mold: BlockMold) -> bool {
        self.try_aimed_faces(eye, direction, false, |f| {
            f.place_molded_block(eye, direction, avatar_pos, mold).then_some(())
        })
        .is_some()
    }

    fn place_scaled_surface_module(&mut self, eye: Vec3, direction: Vec3, avatar_pos: Vec3, spec: crate::ModuleSpec) -> bool {
        self.try_aimed_faces(eye, direction, false, |f| {
            f.place_scaled_module(eye, direction, avatar_pos, spec).then_some(())
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

    fn build_rig_blueprint(&mut self, eye: Vec3, direction: Vec3, blueprint: &CraftBlueprint) -> bool {
        self.try_aimed_faces(eye, direction, false, |f| {
            f.build_rig_blueprint(eye, direction, blueprint).then_some(())
        })
        .is_some()
    }

    fn board_vehicle(&mut self, eye: Vec3, direction: Vec3, holding_block: bool) -> Option<VehicleAssembly> {
        let (primary, secondary) = self.aimed_indices(eye, direction);
        for idx in [primary, secondary] {
            if let Some(face) = self.faces.get_mut(idx) {
                if let Some(out) = face.board_vehicle(eye, direction, holding_block) {
                    if face.pending_terrain_patches() || face.terrain_patch_count() == 0 {
                        self.slices.record(idx);
                    }
                    return Some(out);
                }
            }
            if secondary == primary {
                break;
            }
        }
        None
    }

    fn park_vehicle(&mut self, assembly: VehicleAssembly) -> bool {
        let idx = self.matching_index(assembly.origin());
        if let Some(face) = self.faces.get_mut(idx) {
            if face.park_vehicle(assembly) {
                if face.pending_terrain_patches() {
                    self.slices.record(idx);
                }
                return true;
            }
        }
        false
    }

    fn disclose_stratum(&mut self, face_index: usize, stratum: ExposedStratum) {
        if let Some(face) = self.faces.get_mut(face_index) {
            face.accept_disclosure(stratum);
            self.slices.record(face_index);
        }
    }

    fn capture_sector_snapshot(&self, face: u8, gsu: i32, gsv: i32) -> crate::SectorSnapshot {
        self.faces.get(face as usize).map_or_else(
            || crate::SectorSnapshot::new(face, (gsu, gsv), 0, crate::SectorColumnState::untouched()),
            |f| f.capture_sector_snapshot(face, gsu, gsv),
        )
    }

    fn apply_sector_snapshot(&mut self, snapshot: &crate::SectorSnapshot) -> bool {
        let idx = snapshot.face() as usize;
        let len = self.faces.len();
        let Some(face) = self.faces.get_mut(idx) else {
            return false;
        };
        let changed = face.apply_sector_snapshot(snapshot);
        if changed {
            self.slices.record(idx);
            self.slices.record(len + idx);
        }
        changed
    }

    fn collect_sector_snapshots(&self, out: &mut Vec<crate::SectorSnapshot>) {
        let mut sectors = Vec::new();
        for (idx, face) in self.faces.iter().enumerate() {
            sectors.clear();
            face.collect_modified_sectors(&mut sectors);
            for &(gsu, gsv) in &sectors {
                let snap = face.capture_sector_snapshot(idx as u8, gsu, gsv);
                if !snap.pristine() {
                    out.push(snap);
                }
            }
        }
    }
}
