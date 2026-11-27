use glam::Vec3;
use crate::MeshTarget;
use crate::block::{BlockMedium, GridCell, WedgeHeading};
use crate::frame::SurfaceFrame;
use crate::relief::{MoldedStructure, TurfScars, VehicleAssembly, VehicleAxes};
use super::arena::BrickArena;
use super::atom::RawAtom;
use super::brick::BrickCell;
use super::colonize::{BrickColumnView, TurfColonization};
use super::coord::CellCoord;
use super::inertia::{BrickVolume, DismantledSpan, InertialBricks};
use super::module::{MachineKind, SpaceModule};
use super::module_spec::ModuleSpec;
use super::sweep::{BrickSweep, MediumSiteScan};

const LAYER_FLOOR_MARGIN: f32 = 0.05;
const LAYER_CEILING_MARGIN: f32 = 0.48;

/// Unified 3D structure backed by `InertialBricks<BrickArena>` (`2` fields, `0` per-cell heap allocations).
#[derive(Clone)]
pub struct BrickStructure {
    bricks: InertialBricks<BrickArena>,
    datum: f32,
}

impl BrickStructure {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(bricks: InertialBricks<BrickArena>, datum: f32) -> Self {
        Self { bricks, datum }
    }

    pub fn untouched() -> Self {
        Self::new(InertialBricks::wrapping(BrickArena::untouched()), 0.0)
    }

    pub const fn bricks(&self) -> &InertialBricks<BrickArena> { &self.bricks }
    pub const fn datum(&self) -> f32 { self.datum }
    pub fn active_brick_count(&self) -> usize { self.bricks.inner().active_brick_count() }
    pub fn allocated_brick_bytes(&self) -> usize { self.bricks.inner().allocated_brick_bytes() }
    pub const fn total_mass_kg(&self) -> f64 { self.bricks.total_mass_kg() }
    pub fn center_of_mass(&self) -> glam::DVec3 { self.bricks.center_of_mass() }
    pub fn inertia_matrix(&self) -> glam::DMat3 { self.bricks.inertia_matrix() }

    pub(super) fn place_raw(&mut self, coord: CellCoord, atom: RawAtom) -> BrickCell {
        self.bricks.place_atom(coord, atom)
    }

    pub(super) fn dismantle_coord(&mut self, coord: CellCoord) -> DismantledSpan {
        self.bricks.dismantle_at(coord)
    }

    pub fn insert_atom(&mut self, u: i32, v: i32, floor_elev: f32, mut atom: RawAtom) {
        let coord = CellCoord::from_floor(u, v, floor_elev, self.datum);
        if BrickColumnView::new(self, (u, v)).covered_above(coord) {
            atom = atom.smothered();
        }
        let _ = self.bricks.place_atom(coord, atom);
        if atom.at_floor(floor_elev).smothers_turf() {
            self.smother_below(coord);
        }
    }

    pub fn place_module_at(&mut self, anchor: CellCoord, spec: ModuleSpec) -> bool {
        if !self.bricks.place_module(anchor, spec) {
            return false;
        }
        let (du, dv, _) = spec.span().dimensions();
        for ov in 0..dv {
            for ou in 0..du {
                self.smother_below(anchor.shifted(ou as i32, ov as i32, 0));
            }
        }
        true
    }

    fn smother_below(&mut self, coord: CellCoord) {
        self.mutate_atom_at(coord.shifted(0, 0, -1), RawAtom::smothered);
    }

    pub(super) fn mutate_atom_at(&mut self, coord: CellCoord, f: impl FnOnce(RawAtom) -> RawAtom) -> bool {
        let BrickCell::RawAtom(prior) = self.bricks.cell_at(coord) else { return false };
        let next = f(prior);
        if next != prior {
            let _ = self.bricks.place_atom(coord, next);
        }
        true
    }

    fn hit_coord_in_column(&self, u: i32, v: i32, layer_elev: f32) -> Option<CellCoord> {
        self.bricks.inner().find_in_column_rev(u, v, |coord, _| {
            let floor = coord.floor_elev(self.datum);
            (layer_elev >= floor - LAYER_FLOOR_MARGIN && layer_elev <= floor + LAYER_CEILING_MARGIN)
                .then_some(coord)
        })
    }

    pub fn remove_at(&mut self, u: i32, v: i32, layer_elev: f32) -> Option<(DismantledSpan, bool)> {
        let coord = self.hit_coord_in_column(u, v, layer_elev)?;
        let dismantled = self.bricks.dismantle_at(coord);
        if matches!(dismantled, DismantledSpan::Vacant) {
            return None;
        }
        Some((dismantled, !self.bricks.inner().column_present(u, v)))
    }

    pub fn shift_elevation(&mut self, dz: f32) { self.datum += dz; }
    pub fn rotate_around(&mut self, piv_u: i32, piv_v: i32) { BrickSweep::new(self).rotate_around(piv_u, piv_v); }
    pub fn translate_by(&mut self, du: i32, dv: i32, dz: f32) -> Vec<(i32, i32, f32)> { BrickSweep::new(self).translate_by(du, dv, dz) }
    pub fn settle_gravity(&mut self, ground_fn: impl Fn(i32, i32) -> f32, scars: &mut TurfScars) -> Vec<(i32, i32)> { BrickSweep::new(self).settle_gravity(ground_fn, scars) }
    pub fn clear_in(&mut self, u_bounds: (i32, i32), v_bounds: (i32, i32)) { BrickSweep::new(self).clear_sector_rect(u_bounds, v_bounds); }
    pub fn collect_medium_sites(&self, medium: BlockMedium, frame: &SurfaceFrame, out: &mut Vec<(Vec3, Vec3)>) { MediumSiteScan::new(self).collect(medium, frame, out); }
    pub fn covers_ground_at(&self, u: i32, v: i32, ground_top: f32) -> bool { BrickColumnView::new(self, (u, v)).covers_ground(ground_top) }
    pub fn bare_surface_at(&self, u: i32, v: i32, ground_bare: bool) -> bool { BrickColumnView::new(self, (u, v)).bare_surface(ground_bare) }
    pub fn verdant_near_step(&self, u: i32, v: i32, target_step: i32) -> bool { BrickColumnView::new(self, (u, v)).verdant_near_step(target_step) }
    pub fn confine_niche(&self, u: i32, v: i32, ground_h: f32, elev: f32) -> Option<f32> { BrickColumnView::new(self, (u, v)).confine_niche(ground_h, elev) }

    pub fn colonize_turf(&mut self, dt: f32, verdant_neighbors: impl Fn(i32, i32, i32) -> u32) -> (Vec<(i32, i32)>, bool) {
        TurfColonization::new(self).advance(dt, verdant_neighbors)
    }

    pub fn step_camera_turn(&mut self, u: i32, v: i32, delta_steps: i32) -> Option<(i32, i32)> {
        let hit = self.bricks.inner().find_in_column_rev(u, v, |coord, cell| {
            let BrickCell::RawAtom(mut atom) = cell else { return None };
            atom.step_camera_turn(delta_steps).then_some((coord, atom))
        });
        if let Some((coord, atom)) = hit {
            let _ = self.bricks.place_atom(coord, atom);
            return Some((u, v));
        }
        let mut found = None;
        self.bricks.inner().visit_occupied_cells(|coord, cell| {
            if found.is_none() && matches!(cell, BrickCell::RawAtom(a) if a.medium() == BlockMedium::FieldCamera) {
                found = Some(coord);
            }
        });
        let coord = found?;
        if let BrickCell::RawAtom(mut atom) = self.bricks.cell_at(coord) {
            if atom.step_camera_turn(delta_steps) {
                let _ = self.bricks.place_atom(coord, atom);
                return Some(coord.planar());
            }
        }
        None
    }
}

impl MoldedStructure for BrickStructure {
    fn vacant(&self) -> bool { self.bricks.vacant() }
    fn present_at(&self, u: i32, v: i32) -> bool { self.bricks.inner().column_present(u, v) }

    fn top_steps_at(&self, u: i32, v: i32, floor_steps: i32) -> i32 {
        self.bricks
            .inner()
            .find_in_column_rev(u, v, |coord, _| {
                Some(((coord.ceiling_elev(self.datum) / GridCell::SPAN).round() as i32).max(floor_steps))
            })
            .unwrap_or(floor_steps)
    }

    fn constrain_support(&self, cell: GridCell, planar: (f32, f32), mut h: f32, span: (f32, f32)) -> f32 {
        let (cu, cv) = cell.feed_coordinates(|gu, gv| (gu, gv));
        if !self.bricks.inner().column_present(cu, cv) { return h; }
        let ((u, v), (foot_elev, body_span)) = (planar, span);
        let (fu, fv) = cell.local_fraction(u, v);
        let view = BrickColumnView::new(self, (cu, cv));
        self.bricks.inner().visit_column(cu, cv, |coord, bcell| {
            let floor = coord.floor_elev(self.datum);
            match bcell {
                BrickCell::Empty => {}
                BrickCell::RawAtom(atom) => h = atom.at_floor(floor).constrain_support(fu, fv, h, foot_elev, body_span),
                BrickCell::ModuleAnchor(_) | BrickCell::ModuleProxy(_) => {
                    if view.airlock_open_passage(coord, bcell) != Some(true) {
                        h = RawAtom::full_cube(BlockMedium::TitaniumHull).at_floor(floor).constrain_support(fu, fv, h, foot_elev, body_span);
                    }
                }
            }
        });
        h
    }

    fn layer_occupied_at(&self, u: i32, v: i32, layer_elev: f32) -> bool { self.hit_coord_in_column(u, v, layer_elev).is_some() }
    fn locked_at(&self, _u: i32, _v: i32, _layer_elev: f32) -> bool { false }

    fn seat_backrest_cell(&self, u: i32, v: i32, layer_elev: f32) -> Option<(i32, i32)> {
        let coord = self.hit_coord_in_column(u, v, layer_elev)?;
        let BrickCell::RawAtom(atom) = self.bricks.cell_at(coord) else { return None };
        (atom.medium() == BlockMedium::CabinSeat).then_some(match atom.heading() {
            WedgeHeading::East => (u - 1, v),
            WedgeHeading::West => (u + 1, v),
            WedgeHeading::North => (u, v - 1),
            WedgeHeading::South => (u, v + 1),
        })
    }

    fn medium_at(&self, u: i32, v: i32, layer_elev: f32) -> Option<BlockMedium> {
        let coord = self.hit_coord_in_column(u, v, layer_elev)?;
        match self.bricks.cell_at(coord) {
            BrickCell::Empty => None,
            BrickCell::RawAtom(atom) => Some(atom.medium()),
            BrickCell::ModuleAnchor(spec) => Some(spec.medium()),
            BrickCell::ModuleProxy(_) => self.bricks.resolve_module(coord).map(|(_, s)| s.medium()),
        }
    }

    fn hit_top_at(&self, cu: i32, cv: i32, u: f32, v: f32, ray_elev: f32) -> Option<f32> {
        let (fu, fv) = GridCell::new(cu, cv).local_fraction(u, v);
        self.bricks.inner().find_in_column_rev(cu, cv, |coord, bcell| {
            let floor = coord.floor_elev(self.datum);
            let hit = match bcell {
                BrickCell::Empty => false,
                BrickCell::RawAtom(atom) => atom.at_floor(floor).intercepts(fu, fv, ray_elev),
                BrickCell::ModuleAnchor(_) | BrickCell::ModuleProxy(_) => {
                    ray_elev >= floor - 0.02 && ray_elev <= floor + GridCell::SPAN + 0.02
                }
            };
            hit.then_some(coord.ceiling_elev(self.datum))
        })
    }

    fn perch_at(&self, u: i32, v: i32, layer_elev: f32, _eu: f32, _ev: f32, frame: &SurfaceFrame) -> Option<VehicleAssembly> {
        let coord = self.hit_coord_in_column(u, v, layer_elev)?;
        let (seat_floor, heading) = match self.bricks.cell_at(coord) {
            BrickCell::RawAtom(atom) if atom.medium() == BlockMedium::CabinSeat => (coord.floor_elev(self.datum), atom.heading()),
            BrickCell::ModuleAnchor(_) | BrickCell::ModuleProxy(_) => {
                let (anchor, spec) = self.bricks.resolve_module(coord)?;
                if spec.module() != SpaceModule::Machine(MachineKind::PilotSeat) { return None; }
                (anchor.floor_elev(self.datum), WedgeHeading::from_code(spec.turn().code()))
            }
            _ => return None,
        };
        let axes = VehicleAxes::new(frame.tangent(), frame.bitangent(), frame.normal());
        let (cx, cy) = GridCell::new(u, v).center();
        Some(VehicleAssembly::new(frame.point_at(cx, cy, seat_floor), axes, heading.forward_vector(frame), Vec::new()))
    }

    fn tripod_hub(&self, frame: &SurfaceFrame) -> Option<(Vec3, Vec3, Vec3)> {
        let (mut hub, datum) = (None, self.datum);
        self.bricks.inner().visit_occupied_cells(|coord, cell| {
            if hub.is_none() {
                if let BrickCell::RawAtom(atom) = cell {
                    hub = atom.at_floor(coord.floor_elev(datum)).camera_tripod_hub(coord.grid_cell(), frame);
                }
            }
        });
        hub
    }

    fn emit_at(&self, cell: GridCell, frame: &SurfaceFrame, colors: (Vec3, Vec3), scars: &TurfScars, target: &mut dyn MeshTarget) -> bool {
        let (u, v) = cell.feed_coordinates(|gu, gv| (gu, gv));
        BrickColumnView::new(self, (u, v)).emit(cell, frame, colors, scars, target)
    }
}
