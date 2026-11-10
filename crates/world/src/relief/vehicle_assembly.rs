use std::collections::{HashMap, HashSet, VecDeque};
use glam::{Quat, Vec3};
use crate::block::{BlockMedium, ElevatedShape, GridCell, VoxelShape};
use crate::frame::SurfaceFrame;
use crate::{MeshTarget, Quad};
use super::cantilever_rig::RigRegistry;
use super::natural_strata::NaturalStrata;
use super::shaft_excavation::ShaftExcavation;
use super::sight_ray::SurfaceSightRay;

const BLOCK_STEP: f32 = GridCell::SPAN;

struct ShiftedMeshTarget<'a> {
    shift: Vec3,
    inner: &'a mut dyn MeshTarget,
}

impl<'a> MeshTarget for ShiftedMeshTarget<'a> {
    fn push_quad(&mut self, quad: Quad) {
        let (s, glow) = (self.shift, quad.radiate(|g| g));
        self.inner.push_quad(quad.transfer(|v0, v1, v2, v3, norm, col| Quad::glowing(v0 + s, v1 + s, v2 + s, v3 + s, norm, col, glow)));
    }
}

/// Single block component inside a drivable vehicle assembly relative to the cabin seat.
pub struct VehiclePart {
    offset: (i32, i32),
    floor_offset: f32,
    shape: Box<dyn VoxelShape>,
}

impl VehiclePart {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(offset: (i32, i32), floor_offset: f32, shape: Box<dyn VoxelShape>) -> Self {
        Self { offset, floor_offset, shape }
    }

    fn origin_seat_floor(&self) -> Option<f32> {
        (self.offset == (0, 0) && self.shape.block_medium() == BlockMedium::CabinSeat).then_some(self.floor_offset)
    }

    fn wheel_radius(&self) -> Option<f32> {
        (self.shape.block_medium() == BlockMedium::CarWheel).then(|| self.shape.wheel_radius())
    }

    fn steering_heading(&self, frame: &SurfaceFrame) -> Option<Vec3> {
        (self.shape.block_medium() == BlockMedium::SteeringWheel).then(|| self.shape.block_heading().forward_vector(frame))
    }

    fn emit_driven(&self, lift: f32, spin: f32, turn: f32, frame: &SurfaceFrame, top: Vec3, wall: Vec3, out: &mut dyn MeshTarget) {
        let cell = GridCell::new(self.offset.0, self.offset.1);
        self.shape.emit_driven_part(cell, self.floor_offset + lift, spin, turn, frame, top, wall, out);
    }

    fn park_into(
        mut self, seat: (i32, i32), turn: (i32, i32, u8),
        blocks: &mut HashMap<(i32, i32), Vec<ElevatedShape>>, strata: &NaturalStrata, shafts: &ShaftExcavation,
    ) {
        let ((rxx, rxy, quarter_turns), (ox, oy)) = (turn, self.offset);
        let (ryx, ryy) = (-rxy, rxx);
        for _ in 0..quarter_turns {
            self.shape.rotate_quarter();
        }
        let (bu, bv) = (seat.0 + ox * rxx + oy * ryx, seat.1 + ox * rxy + oy * ryy);
        let ground_top = (shafts.elevation_at(strata, bu, bv) as f32) * BLOCK_STEP;
        let list = blocks.entry((bu, bv)).or_default();
        list.push(ElevatedShape::new(self.shape, ground_top + self.floor_offset));
        list.sort_by(|a, b| a.ceiling().total_cmp(&b.ceiling()));
    }
}

/// Orthonormal coordinate axes of a drivable vehicle assembly on a planetary surface.
#[derive(Clone, Copy, Debug)]
pub struct VehicleAxes {
    tangent: Vec3,
    bitangent: Vec3,
    normal: Vec3,
}

impl VehicleAxes {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(tangent: Vec3, bitangent: Vec3, normal: Vec3) -> Self {
        Self { tangent, bitangent, normal }
    }

    pub const fn normal(self) -> Vec3 { self.normal }
    fn surface_frame(self) -> SurfaceFrame { SurfaceFrame::new(self.tangent, self.bitangent, self.normal, 0.0) }

    fn cardinal_turn(self, frame: &SurfaceFrame) -> (i32, i32, u8) {
        let (du, dv) = (self.tangent.dot(frame.tangent()), self.tangent.dot(frame.bitangent()));
        if du.abs() >= dv.abs() {
            if du >= 0.0 { (1, 0, 0) } else { (-1, 0, 2) }
        } else if dv >= 0.0 {
            (0, 1, 1)
        } else {
            (0, -1, 3)
        }
    }

    fn rotated(self, rot: Quat) -> Self {
        Self::new(
            (rot * self.tangent).normalize_or_zero(),
            (rot * self.bitangent).normalize_or_zero(),
            (rot * self.normal).normalize_or_zero(),
        )
    }
}

/// Drivable multi-block vehicle assembly centered on its cabin seat.
pub struct VehicleAssembly {
    origin: Vec3,
    axes: VehicleAxes,
    heading: Vec3,
    parts: Vec<VehiclePart>,
}

impl VehicleAssembly {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(origin: Vec3, axes: VehicleAxes, heading: Vec3, parts: Vec<VehiclePart>) -> Self {
        Self { origin, axes, heading, parts }
    }

    pub const fn origin(&self) -> Vec3 { self.origin }
    pub const fn heading(&self) -> Vec3 { self.heading }
    pub const fn normal(&self) -> Vec3 { self.axes.normal() }
    pub fn drivable(&self) -> bool { !self.parts.is_empty() }

    pub fn seat_floor_offset(&self) -> f32 {
        let seat_off = self.parts.iter().find_map(VehiclePart::origin_seat_floor).unwrap_or(0.0);
        seat_off + self.wheel_ground_lift()
    }

    pub fn perch_shell(&self) -> Self {
        let seat_off = self.parts.iter().find_map(VehiclePart::origin_seat_floor).unwrap_or(0.0);
        Self::new(self.origin + self.axes.normal() * seat_off, self.axes, self.heading, Vec::new())
    }

    pub fn wheel_radius(&self) -> f32 {
        self.parts.iter().filter_map(VehiclePart::wheel_radius).fold(0.0f32, f32::max).max(0.16)
    }

    fn wheel_ground_lift(&self) -> f32 {
        self.parts.iter().filter_map(VehiclePart::wheel_radius).map(|r| (r - 0.11).max(0.05)).fold(0.0f32, f32::max)
    }

    pub fn steer(&mut self, delta_yaw: f32) {
        let rot = Quat::from_axis_angle(self.axes.normal(), delta_yaw);
        self.axes = self.axes.rotated(rot);
        self.heading = (rot * self.heading).normalize_or_zero();
    }

    pub fn relocate(&mut self, next_origin: Vec3, next_up: Vec3) {
        self.origin = next_origin;
        if self.axes.normal().dot(next_up) < 0.99999 {
            let rot = Quat::from_rotation_arc(self.axes.normal(), next_up);
            self.axes = self.axes.rotated(rot);
            self.heading = (rot * self.heading).normalize_or_zero();
        }
    }

    pub fn emit(&self, target: &mut dyn MeshTarget) {
        self.emit_driven(0.0, 0.0, target);
    }

    pub fn emit_driven(&self, wheel_spin: f32, steer_turn: f32, target: &mut dyn MeshTarget) {
        let car_frame = self.axes.surface_frame();
        let shift = self.origin - car_frame.point_at(0.25, 0.25, 0.0);
        let mut shifted = ShiftedMeshTarget { shift, inner: target };
        let (top_col, wall_col) = (Vec3::new(0.45, 0.30, 0.17), Vec3::new(0.41, 0.27, 0.15));
        let lift = self.wheel_ground_lift();
        for part in &self.parts {
            part.emit_driven(lift, wheel_spin, steer_turn, &car_frame, top_col, wall_col, &mut shifted);
        }
    }

    pub(super) fn connected_cluster(u: i32, v: i32, seat_idx: usize, blocks: &HashMap<(i32, i32), Vec<ElevatedShape>>) -> Vec<(i32, i32, usize)> {
        let (mut visited, mut queue, mut cluster) = (HashSet::new(), VecDeque::new(), Vec::new());
        visited.insert((u, v, seat_idx));
        queue.push_back((u, v, seat_idx));
        while let Some((cu, cv, idx)) = queue.pop_front() {
            let z = blocks[&(cu, cv)][idx].floor();
            cluster.push((cu, cv, idx));
            for (du, dv) in [(1, 0), (-1, 0), (0, 1), (0, -1), (0, 0)] {
                let (nu, nv) = (cu + du, cv + dv);
                if let Some(nlist) = blocks.get(&(nu, nv)) {
                    for (nidx, nb) in nlist.iter().enumerate() {
                        if (nb.floor() - z).abs() <= BLOCK_STEP + 0.12 && visited.insert((nu, nv, nidx)) {
                            queue.push_back((nu, nv, nidx));
                        }
                    }
                }
            }
        }
        cluster
    }

    fn locate_engine(
        u: i32, v: i32, layer_elev: f32, ray: SurfaceSightRay, frame: &SurfaceFrame,
        blocks: &HashMap<(i32, i32), Vec<ElevatedShape>>,
    ) -> Option<(i32, i32, usize)> {
        if let Some(idx) = blocks.get(&(u, v)).and_then(|l| l.iter().rposition(|b| b.occupies_layer(layer_elev) && b.block_medium() == BlockMedium::SteeringWheel)) {
            return Some((u, v, idx));
        }
        for step in 1..=28 {
            let sample = ray.sample_point((step as f32) * 0.06);
            let (su, sv) = frame.decompose(sample);
            let (cu, cv) = GridCell::at_planar(su, sv).feed_coordinates(|gu, gv| (gu, gv));
            let ray_elev = frame.elevation_of(sample);
            if let Some(idx) = blocks.get(&(cu, cv)).and_then(|l| {
                l.iter().rposition(|b| b.block_medium() == BlockMedium::SteeringWheel && ray_elev >= b.floor() - 0.15 && ray_elev <= b.ceiling() + 1.15)
            }) {
                return Some((cu, cv, idx));
            }
        }
        None
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn board_from_blocks(
        blocks: &mut HashMap<(i32, i32), Vec<ElevatedShape>>, rigs: &mut RigRegistry,
        strata: &NaturalStrata, shafts: &ShaftExcavation, frame: &SurfaceFrame,
        u: i32, v: i32, layer_elev: f32, ray: SurfaceSightRay, engage_engine: bool,
    ) -> Option<Self> {
        if !engage_engine {
            let seat_idx = blocks.get(&(u, v))?.iter().rposition(|b| b.occupies_layer(layer_elev) && b.block_medium() == BlockMedium::CabinSeat)?;
            let seat_floor = blocks[&(u, v)][seat_idx].floor();
            let cluster = Self::connected_cluster(u, v, seat_idx, blocks);
            let eng = cluster.iter().copied().find(|&(cu, cv, idx)| blocks[&(cu, cv)][idx].block_medium() == BlockMedium::SteeringWheel);
            let heading = eng.map_or_else(
                || blocks[&(u, v)][seat_idx].block_heading().forward_vector(frame),
                |(eu, ev, idx)| if (eu, ev) != (u, v) { (frame.tangent() * ((eu - u) as f32) + frame.bitangent() * ((ev - v) as f32)).normalize_or_zero() } else { blocks[&(eu, ev)][idx].block_heading().forward_vector(frame) },
            );
            let (cx, cy) = GridCell::new(u, v).center();
            return Some(Self::new(frame.point_at(cx, cy, seat_floor), VehicleAxes::new(frame.tangent(), frame.bitangent(), frame.normal()), heading, Vec::new()));
        }
        let eng_hit = (!rigs.present_at(u, v)).then(|| Self::locate_engine(u, v, layer_elev, ray, frame, blocks)).flatten();
        let Some((eng_u, eng_v, eng_idx)) = eng_hit else {
            let (mu, mv) = rigs.find_aimed_rig(u, v, ray, frame)?;
            let (origin, heading) = rigs.reconnect_seated_car(mu, mv, ray, frame, blocks)?;
            return Some(Self::new(origin, VehicleAxes::new(frame.tangent(), frame.bitangent(), frame.normal()), heading, Vec::new()));
        };
        let ((eng_u, eng_v, _), mut cluster, seat_u, seat_v, ground_top, min_floor) = {
            let cluster = Self::connected_cluster(eng_u, eng_v, eng_idx, blocks);
            if !cluster.iter().any(|&(cu, cv, idx)| blocks[&(cu, cv)][idx].block_medium() == BlockMedium::CarWheel) {
                return None;
            }
            let (eu, ev) = frame.decompose(ray.sample_point(0.0));
            let &(seat_u, seat_v, _) = cluster
                .iter()
                .filter(|&&(cu, cv, idx)| blocks[&(cu, cv)][idx].block_medium() == BlockMedium::CabinSeat)
                .min_by(|&&(au, av, _), &&(bu, bv, _)| {
                    let ((ax, ay), (bx, by)) = (GridCell::new(au, av).center(), GridCell::new(bu, bv).center());
                    ((ax - eu).hypot(ay - ev)).total_cmp(&((bx - eu).hypot(by - ev)))
                })?;
            let ground_top = (shafts.elevation_at(strata, seat_u, seat_v) as f32) * BLOCK_STEP;
            let min_floor = cluster.iter().map(|&(cu, cv, idx)| blocks[&(cu, cv)][idx].floor()).fold(f32::MAX, f32::min);
            ((eng_u, eng_v, eng_idx), cluster, seat_u, seat_v, ground_top, min_floor)
        };
        if min_floor > ground_top + 0.35 {
            let _ = rigs.lower_at(strata, shafts, eng_u, eng_v, blocks);
            let next_seat_idx = blocks.get(&(seat_u, seat_v))?.iter().rposition(|b| b.block_medium() == BlockMedium::CabinSeat)?;
            cluster = Self::connected_cluster(seat_u, seat_v, next_seat_idx, blocks);
        }
        rigs.release_cluster(&cluster);
        let mut by_cell: HashMap<(i32, i32), Vec<usize>> = HashMap::new();
        for &(cu, cv, idx) in &cluster {
            by_cell.entry((cu, cv)).or_default().push(idx);
        }
        let mut extracted = Vec::with_capacity(cluster.len());
        for ((cu, cv), mut indices) in by_cell {
            indices.sort_unstable_by(|a, b| b.cmp(a));
            if let Some(list) = blocks.get_mut(&(cu, cv)) {
                for idx in indices {
                    extracted.push((cu, cv, list.remove(idx)));
                }
                if list.is_empty() {
                    blocks.remove(&(cu, cv));
                }
            }
        }
        let base_floor = extracted.iter().map(|(_, _, b)| b.floor()).fold(ground_top, f32::min);
        let parts: Vec<VehiclePart> = extracted
            .into_iter()
            .map(|(cu, cv, b)| VehiclePart::new((cu - seat_u, cv - seat_v), (b.floor() - base_floor).max(0.0), b.into_shape()))
            .collect();
        let (cx, cy) = GridCell::new(seat_u, seat_v).center();
        let heading = parts.iter().find_map(|p| p.steering_heading(frame)).unwrap_or_else(|| ray.align_heading(frame));
        Some(Self::new(frame.point_at(cx, cy, ground_top), VehicleAxes::new(frame.tangent(), frame.bitangent(), frame.normal()), heading, parts))
    }

    pub(super) fn park_into_blocks(
        self, blocks: &mut HashMap<(i32, i32), Vec<ElevatedShape>>, strata: &NaturalStrata, shafts: &ShaftExcavation, frame: &SurfaceFrame,
    ) -> (i32, i32) {
        let (seat, turn) = (frame.cell_at(self.origin).feed_coordinates(|cu, cv| (cu, cv)), self.axes.cardinal_turn(frame));
        for part in self.parts {
            part.park_into(seat, turn, blocks, strata, shafts);
        }
        seat
    }
}
