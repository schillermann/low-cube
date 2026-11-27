use glam::Vec3;
use crate::block::{BlockMedium, BlockMold, CardinalTurn, GridCell};
use crate::brick_arena::{BrickCell, BrickStructure, BrickVolume, CellCoord, ModuleSpec, RawAtom};
use crate::frame::SurfaceFrame;
use crate::relief::{CraftBlueprint, CraftIdentity, CraftPose, VehiclePart};

/// Live hull construction & dismantling mason over a flying or orbiting `GridBody` (`3` fields).
pub struct HullMason<'a> {
    grid: &'a mut BrickStructure,
    blueprint: &'a mut CraftBlueprint,
    pose: CraftPose,
}

impl<'a> HullMason<'a> {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(
        grid: &'a mut BrickStructure,
        blueprint: &'a mut CraftBlueprint,
        pose: CraftPose,
    ) -> Self {
        Self { grid, blueprint, pose }
    }

    fn local_ray(&self, eye_world: Vec3, dir_world: Vec3) -> Option<(Vec3, Vec3)> {
        let dir = dir_world.normalize_or_zero();
        if dir.length_squared() < 1e-4 {
            return None;
        }
        let local_eye = self.pose.local_point_of(eye_world);
        let local_dir = (self.pose.hull_rotation().inverse() * dir).normalize_or_zero();
        (local_eye.length_squared() <= 4_096.0).then_some((local_eye, local_dir))
    }

    fn trace_hit(&self, local_eye: Vec3, local_dir: Vec3) -> Option<((i32, i32, i32), (i32, i32, i32))> {
        let datum = self.grid.datum();
        let start_u = local_eye.x + 0.25;
        let start_v = local_eye.z + 0.25;
        let mut prev = (
            (start_u * 2.0).floor() as i32,
            (start_v * 2.0).floor() as i32,
            ((local_eye.y - datum) * 2.0).floor() as i32,
        );
        for step in 1..=76 {
            let pt = local_eye + local_dir * ((step as f32) * 0.05);
            let cu = ((pt.x + 0.25) * 2.0).floor() as i32;
            let cv = ((pt.z + 0.25) * 2.0).floor() as i32;
            let cw = ((pt.y - datum) * 2.0).floor() as i32;
            let cur = (cu, cv, cw);
            if cur == prev {
                continue;
            }
            let coord = CellCoord::new(cu, cv, cw);
            if self.grid.bricks().cell_at(coord).occupied() {
                return Some((cur, prev));
            }
            prev = cur;
        }
        None
    }

    pub fn place_molded(&mut self, eye_world: Vec3, dir_world: Vec3, mold: BlockMold) -> bool {
        let Some((local_eye, local_dir)) = self.local_ray(eye_world, dir_world) else {
            return false;
        };
        let Some((_, (pu, pv, pw))) = self.trace_hit(local_eye, local_dir) else {
            return false;
        };
        let local_frame = SurfaceFrame::new(Vec3::X, Vec3::Z, Vec3::Y, 0.0);
        let atom = mold.atom_for(local_dir, &local_frame);
        let floor_elev = self.grid.datum() + (pw as f32) * GridCell::SPAN;
        if let Some(spec) = ModuleSpec::from_medium_turn(atom.medium(), CardinalTurn::from_code(atom.heading().code())) {
            let anchor = CellCoord::new(pu, pv, pw);
            if !self.grid.place_module_at(anchor, spec) {
                return false;
            }
        } else {
            self.grid.insert_atom(pu, pv, floor_elev, atom);
        }
        self.sync_blueprint();
        true
    }

    pub fn place_scaled(&mut self, eye_world: Vec3, dir_world: Vec3, spec: ModuleSpec) -> bool {
        let Some((local_eye, local_dir)) = self.local_ray(eye_world, dir_world) else {
            return false;
        };
        let Some((_, (pu, pv, pw))) = self.trace_hit(local_eye, local_dir) else {
            return false;
        };
        let anchor = CellCoord::new(pu, pv, pw);
        if !self.grid.place_module_at(anchor, spec) {
            return false;
        }
        self.sync_blueprint();
        true
    }

    pub fn harvest(&mut self, eye_world: Vec3, dir_world: Vec3) -> Option<BlockMedium> {
        let (local_eye, local_dir) = self.local_ray(eye_world, dir_world)?;
        let ((cu, cv, cw), _) = self.trace_hit(local_eye, local_dir)?;
        let layer_elev = self.grid.datum() + (cw as f32) * GridCell::SPAN + 0.25;
        let (removed, _) = self.grid.remove_at(cu, cv, layer_elev)?;
        let med = removed.medium()?;
        self.sync_blueprint();
        Some(med)
    }

    pub fn sync_blueprint(&mut self) {
        let id = if self.blueprint.identity().assigned() {
            self.blueprint.identity()
        } else {
            CraftIdentity::from_parts(self.pose.origin(), &[])
        };
        let parts = self.parts_from_structure();
        *self.blueprint = CraftBlueprint::new(id, parts);
    }

    fn parts_from_structure(&self) -> Vec<VehiclePart> {
        let datum = self.grid.datum();
        let mut parts = Vec::new();
        self.grid.bricks().inner().visit_occupied_cells(|coord, cell| {
            let (cu, cv) = coord.planar();
            let floor_off = coord.floor_elev(datum);
            match cell {
                BrickCell::RawAtom(atom) => {
                    parts.push(VehiclePart::new((cu, cv), floor_off, atom));
                }
                BrickCell::ModuleAnchor(spec) => {
                    if matches!(spec.module(), crate::SpaceModule::Machine(_)) {
                        let atom = RawAtom::molded(
                            spec.medium(),
                            crate::MoldArchetype::FullCube,
                            crate::WedgeHeading::from_code(spec.turn().code()),
                        );
                        parts.push(VehiclePart::new((cu, cv), floor_off, atom));
                    } else {
                        let (du, dv, dh) = spec.span().dimensions();
                        let atom = RawAtom::full_cube(spec.medium());
                        for hw in 0..dh {
                            for ov in 0..dv {
                                for ou in 0..du {
                                    parts.push(VehiclePart::new(
                                        (cu + ou as i32, cv + ov as i32),
                                        floor_off + (hw as f32) * GridCell::SPAN,
                                        atom,
                                    ));
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        });
        parts.sort_by_key(VehiclePart::canonical_key);
        parts
    }
}
