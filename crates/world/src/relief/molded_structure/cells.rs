use std::collections::HashMap;
use glam::Vec3;
use crate::MeshTarget;
use crate::block::{BlockMedium, ElevatedShape, GridCell, NicheColumn, VoxelShape, WedgeHeading};
use crate::frame::SurfaceFrame;
use super::super::turf_habitat::TurfHabitat;
use super::super::turf_scars::TurfScars;
use super::super::vehicle_assembly::{VehicleAssembly, VehicleAxes};
use super::sod::SodColumn;
use super::{MoldedStructure, NICHE_CONTACT_TOLERANCE};

const BLOCK_STEP: f32 = GridCell::SPAN;

/// Strictly encapsulated 3D cell map of elevated voxel shapes shared by world blocks and crafts.
pub struct VoxelCells {
    cells: HashMap<(i32, i32), Vec<ElevatedShape>>,
}

impl VoxelCells {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(cells: HashMap<(i32, i32), Vec<ElevatedShape>>) -> Self {
        Self { cells }
    }

    pub fn untouched() -> Self {
        Self::new(HashMap::new())
    }

    fn refresh_column_smother(&mut self, u: i32, v: i32) {
        if let Some(list) = self.cells.get_mut(&(u, v)) {
            list.sort_by(|a, b| a.ceiling().total_cmp(&b.ceiling()));
            for idx in 0..list.len().saturating_sub(1) {
                let ceiling = list[idx].ceiling();
                if list[idx + 1].smothers_turf_below(ceiling) {
                    list[idx].smother_turf();
                }
            }
        }
    }

    pub fn insert(&mut self, u: i32, v: i32, floor_elev: f32, shape: Box<dyn VoxelShape>) {
        self.insert_elevated(u, v, ElevatedShape::new(shape, floor_elev));
    }

    pub fn insert_elevated(&mut self, u: i32, v: i32, shape: ElevatedShape) {
        self.cells.entry((u, v)).or_default().push(shape);
        self.refresh_column_smother(u, v);
    }

    pub fn remove_at(&mut self, u: i32, v: i32, layer_elev: f32) -> Option<(ElevatedShape, bool)> {
        let list = self.cells.get_mut(&(u, v))?;
        let pos = list.iter().rposition(|b| b.occupies_layer(layer_elev))?;
        let removed = list.remove(pos);
        let column_vacated = list.is_empty();
        if column_vacated {
            self.cells.remove(&(u, v));
        }
        Some((removed, column_vacated))
    }

    pub fn covers_ground_at(&self, u: i32, v: i32, ground_top: f32) -> bool {
        self.cells.get(&(u, v)).is_some_and(|list| SodColumn::new(list).covers_ground(ground_top))
    }

    pub fn bare_surface_at(&self, u: i32, v: i32, ground_bare: bool) -> bool {
        self.cells
            .get(&(u, v))
            .map_or(ground_bare, |list| SodColumn::new(list).bare_surface(ground_bare))
    }

    pub fn verdant_near_step(&self, u: i32, v: i32, target_step: i32) -> bool {
        self.cells.get(&(u, v)).is_some_and(|list| SodColumn::new(list).verdant_near_step(target_step))
    }

    pub(super) fn collect_colonization(&self, habitat: &dyn TurfHabitat) -> Vec<((i32, i32), usize, u32)> {
        let mut active = Vec::new();
        for (&cell, list) in &self.cells {
            SodColumn::new(list).collect_colonization(cell, habitat, &mut active);
        }
        active
    }

    pub(super) fn apply_colonization(&mut self, dt: f32, active: Vec<((i32, i32), usize, u32)>) -> Vec<(i32, i32)> {
        let mut regrown = Vec::new();
        for ((u, v), idx, neighbors) in active {
            let grown = self
                .cells
                .get_mut(&(u, v))
                .and_then(|list| list.get_mut(idx))
                .is_some_and(|block| block.colonize_turf(GridCell::new(u, v), dt, neighbors));
            if grown {
                regrown.push((u, v));
            }
        }
        regrown
    }

    pub fn step_camera_turn(&mut self, u: i32, v: i32, delta_steps: i32) -> Option<(i32, i32)> {
        if self.cells.get_mut(&(u, v)).is_some_and(|l| l.iter_mut().rev().any(|b| b.step_camera_turn(delta_steps))) {
            return Some((u, v));
        }
        self.cells
            .iter_mut()
            .find_map(|(&cell, list)| list.iter_mut().rev().any(|b| b.step_camera_turn(delta_steps)).then_some(cell))
    }

    pub fn confine_niche(&self, u: i32, v: i32, ground_h: f32, elev: f32) -> Option<f32> {
        NicheColumn::new(self.cells.get(&(u, v))?, ground_h).confine(elev)
    }

    pub fn settle_gravity(&mut self, ground_fn: impl Fn(i32, i32) -> f32, scars: &mut TurfScars) -> Vec<(i32, i32)> {
        let keys: Vec<(i32, i32)> = self.cells.keys().copied().collect();
        let mut settled = Vec::new();
        for (u, v) in keys {
            let ground_top = ground_fn(u, v);
            let mut moved = false;
            if let Some(list) = self.cells.get_mut(&(u, v)) {
                for idx in 0..list.len() {
                    let floor_limit = if idx == 0 { ground_top } else { list[idx - 1].ceiling() };
                    if list[idx].settles_under_gravity() && list[idx].floor() >= floor_limit + BLOCK_STEP - NICHE_CONTACT_TOLERANCE {
                        list[idx].settle_step();
                        moved = true;
                    }
                }
            }
            if moved {
                self.refresh_column_smother(u, v);
                if self.covers_ground_at(u, v, ground_top) {
                    scars.expose_dirt(u, v);
                }
                settled.push((u, v));
            }
        }
        settled
    }

    pub fn any_at(&self, u: i32, v: i32, pred: impl FnMut(&ElevatedShape) -> bool) -> bool {
        self.cells.get(&(u, v)).is_some_and(|list| list.iter().any(pred))
    }

    pub fn visit_cells(&self, mut visitor: impl FnMut((i32, i32), &[ElevatedShape])) {
        for (&cell, list) in &self.cells {
            visitor(cell, list);
        }
    }

    pub fn shift_elevation(&mut self, dz: f32) {
        for list in self.cells.values_mut() {
            for shape in list.iter_mut() {
                shape.shift_elevation(dz);
            }
            list.sort_by(|a, b| a.ceiling().total_cmp(&b.ceiling()));
        }
    }

    pub fn rotate_around(&mut self, piv_u: i32, piv_v: i32) {
        let old = std::mem::take(&mut self.cells);
        let mut rotated: HashMap<(i32, i32), Vec<ElevatedShape>> = HashMap::with_capacity(old.len());
        for ((cu, cv), list) in old {
            let (du, dv) = (cu - piv_u, cv - piv_v);
            let dest = rotated.entry((piv_u - dv, piv_v + du)).or_default();
            for mut shape in list {
                shape.rotate_quarter();
                dest.push(shape);
            }
            dest.sort_by(|a, b| a.ceiling().total_cmp(&b.ceiling()));
        }
        self.cells = rotated;
    }

    pub fn translate_by(&mut self, du: i32, dv: i32, dz: f32) -> Vec<(i32, i32, f32)> {
        let old = std::mem::take(&mut self.cells);
        let mut moved_floors = Vec::new();
        for ((cu, cv), list) in old {
            let (ncu, ncv) = (cu + du, cv + dv);
            for mut shape in list {
                shape.shift_elevation(dz);
                moved_floors.push((ncu, ncv, shape.floor()));
                self.insert_elevated(ncu, ncv, shape);
            }
        }
        moved_floors
    }

    pub fn into_entries(self) -> impl Iterator<Item = ((i32, i32), ElevatedShape)> {
        self.cells.into_iter().flat_map(|(cell, list)| list.into_iter().map(move |shape| (cell, shape)))
    }
}

impl MoldedStructure for VoxelCells {
    fn vacant(&self) -> bool { self.cells.is_empty() }
    fn present_at(&self, u: i32, v: i32) -> bool { self.cells.contains_key(&(u, v)) }

    fn top_steps_at(&self, u: i32, v: i32, floor_steps: i32) -> i32 {
        let floor_top = (floor_steps as f32) * BLOCK_STEP;
        self.cells.get(&(u, v)).map_or(floor_steps, |list| {
            (list.iter().map(ElevatedShape::ceiling).fold(floor_top, f32::max) / BLOCK_STEP).round() as i32
        })
    }

    fn constrain_support(&self, cell: GridCell, planar: (f32, f32), mut h: f32, span: (f32, f32)) -> f32 {
        let (cu, cv) = cell.feed_coordinates(|gu, gv| (gu, gv));
        if let Some(list) = self.cells.get(&(cu, cv)) {
            let ((u, v), (foot_elev, body_span)) = (planar, span);
            let (fu, fv) = cell.local_fraction(u, v);
            for block in list {
                h = block.constrain_support(fu, fv, h, foot_elev, body_span);
            }
        }
        h
    }

    fn layer_occupied_at(&self, u: i32, v: i32, layer_elev: f32) -> bool {
        self.any_at(u, v, |b| b.occupies_layer(layer_elev))
    }

    fn locked_at(&self, _u: i32, _v: i32, _layer_elev: f32) -> bool { false }

    fn seat_backrest_cell(&self, u: i32, v: i32, layer_elev: f32) -> Option<(i32, i32)> {
        let block = self.cells.get(&(u, v))?.iter().rfind(|b| b.occupies_layer(layer_elev))?;
        (block.block_medium() == BlockMedium::CabinSeat).then_some(match block.block_heading() {
            WedgeHeading::East => (u - 1, v),
            WedgeHeading::West => (u + 1, v),
            WedgeHeading::North => (u, v - 1),
            WedgeHeading::South => (u, v + 1),
        })
    }

    fn medium_at(&self, u: i32, v: i32, layer_elev: f32) -> Option<BlockMedium> {
        self.cells.get(&(u, v))?.iter().rfind(|b| b.occupies_layer(layer_elev)).map(ElevatedShape::block_medium)
    }

    fn hit_top_at(&self, cu: i32, cv: i32, u: f32, v: f32, ray_elev: f32) -> Option<f32> {
        let list = self.cells.get(&(cu, cv))?;
        let (fu, fv) = GridCell::new(cu, cv).local_fraction(u, v);
        list.iter().rev().find(|b| b.intercepts(fu, fv, ray_elev)).map(ElevatedShape::ceiling)
    }

    fn perch_at(&self, u: i32, v: i32, layer_elev: f32, _eu: f32, _ev: f32, frame: &SurfaceFrame) -> Option<VehicleAssembly> {
        let seat = self.cells.get(&(u, v))?.iter().rfind(|b| b.occupies_layer(layer_elev) && b.block_medium() == BlockMedium::CabinSeat)?;
        let axes = VehicleAxes::new(frame.tangent(), frame.bitangent(), frame.normal());
        let (cx, cy) = GridCell::new(u, v).center();
        Some(VehicleAssembly::new(frame.point_at(cx, cy, seat.floor()), axes, seat.block_heading().forward_vector(frame), Vec::new()))
    }

    fn tripod_hub(&self, frame: &SurfaceFrame) -> Option<(Vec3, Vec3, Vec3)> {
        self.cells.iter().find_map(|(&(u, v), list)| list.iter().rev().find_map(|b| b.camera_tripod_hub(GridCell::new(u, v), frame)))
    }

    fn emit_at(&self, cell: GridCell, frame: &SurfaceFrame, colors: (Vec3, Vec3), scars: &TurfScars, target: &mut dyn MeshTarget) -> bool {
        let (u, v) = cell.feed_coordinates(|gu, gv| (gu, gv));
        let Some(list) = self.cells.get(&(u, v)) else { return false };
        SodColumn::new(list).emit(cell, frame, colors, scars, target)
    }
}
