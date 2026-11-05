use std::collections::{HashMap, HashSet, VecDeque};
use glam::{Quat, Vec3};
use crate::block::{BlockMedium, ElevatedShape, GridCell, VoxelShape};
use crate::frame::SurfaceFrame;
use crate::{MeshTarget, Quad};
use super::cantilever_rig::RigRegistry;
use super::natural_strata::NaturalStrata;
use super::shaft_excavation::ShaftExcavation;

const BLOCK_STEP: f32 = GridCell::SPAN;

struct ShiftedMeshTarget<'a> {
    shift: Vec3,
    inner: &'a mut dyn MeshTarget,
}

impl<'a> MeshTarget for ShiftedMeshTarget<'a> {
    fn push_quad(&mut self, quad: Quad) {
        let s = self.shift;
        let glow = quad.radiate(|g| g);
        let moved = quad.transfer(|v0, v1, v2, v3, norm, col| {
            Quad::glowing(v0 + s, v1 + s, v2 + s, v3 + s, norm, col, glow)
        });
        self.inner.push_quad(moved);
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
    pub const fn normal(&self) -> Vec3 { self.axes.normal }

    pub fn seat_floor_offset(&self) -> f32 {
        self.parts
            .iter()
            .find(|p| p.offset == (0, 0) && p.shape.block_medium() == BlockMedium::CabinSeat)
            .map_or(0.0, |p| p.floor_offset)
    }

    pub fn steer(&mut self, delta_yaw: f32) {
        let rot = Quat::from_axis_angle(self.axes.normal, delta_yaw);
        self.axes = self.axes.rotated(rot);
        self.heading = (rot * self.heading).normalize_or_zero();
    }

    pub fn relocate(&mut self, next_origin: Vec3, next_up: Vec3) {
        self.origin = next_origin;
        if self.axes.normal.dot(next_up) < 0.99999 {
            let rot = Quat::from_rotation_arc(self.axes.normal, next_up);
            self.axes = self.axes.rotated(rot);
            self.heading = (rot * self.heading).normalize_or_zero();
        }
    }

    pub fn emit(&self, target: &mut dyn MeshTarget) {
        let car_frame = SurfaceFrame::new(self.axes.tangent, self.axes.bitangent, self.axes.normal, 0.0);
        let shift = self.origin - car_frame.point_at(0.25, 0.25, 0.0);
        let mut shifted = ShiftedMeshTarget { shift, inner: target };
        let (top_col, wall_col) = (Vec3::new(0.45, 0.30, 0.17), Vec3::new(0.41, 0.27, 0.15));
        for part in &self.parts {
            let cell = GridCell::new(part.offset.0, part.offset.1);
            part.shape.emit_terraced_shape(cell, part.floor_offset, &car_frame, top_col, wall_col, true, &mut shifted);
        }
    }

    fn connected_cluster(u: i32, v: i32, seat_idx: usize, blocks: &HashMap<(i32, i32), Vec<ElevatedShape>>) -> Vec<(i32, i32, usize)> {
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

    #[allow(clippy::too_many_arguments)]
    pub(super) fn board_from_blocks(
        blocks: &mut HashMap<(i32, i32), Vec<ElevatedShape>>,
        rigs: &mut RigRegistry,
        strata: &NaturalStrata,
        shafts: &ShaftExcavation,
        frame: &SurfaceFrame,
        u: i32,
        v: i32,
        layer_elev: f32,
        view_dir: Vec3,
        holding_block: bool,
    ) -> Option<Self> {
        let seat_idx = blocks.get(&(u, v))?.iter().rposition(|b| {
            b.occupies_layer(layer_elev) && b.block_medium() == BlockMedium::CabinSeat
        })?;
        let mut cluster = Self::connected_cluster(u, v, seat_idx, blocks);
        let ground_top = (shafts.elevation_at(strata, u, v) as f32) * BLOCK_STEP;
        let min_floor = cluster.iter().map(|&(cu, cv, idx)| blocks[&(cu, cv)][idx].floor()).fold(f32::MAX, f32::min);
        if min_floor > ground_top + 0.35 {
            if holding_block {
                return None;
            }
            let _ = rigs.lower_at(strata, shafts, u, v, blocks);
            let next_seat_idx = blocks.get(&(u, v))?.iter().rposition(|b| b.block_medium() == BlockMedium::CabinSeat)?;
            cluster = Self::connected_cluster(u, v, next_seat_idx, blocks);
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
            .map(|(cu, cv, b)| {
                let floor_off = (b.floor() - base_floor).max(0.0);
                VehiclePart::new((cu - u, cv - v), floor_off, b.into_shape())
            })
            .collect();
        let (cx, cy) = GridCell::new(u, v).center();
        let origin = frame.point_at(cx, cy, ground_top);
        let heading = parts
            .iter()
            .find(|p| p.shape.block_medium() == BlockMedium::CarEngine)
            .map(|p| p.shape.block_heading().forward_vector(frame))
            .unwrap_or_else(|| {
                let candidates = [frame.bitangent(), frame.tangent(), -frame.bitangent(), -frame.tangent()];
                candidates
                    .into_iter()
                    .max_by(|a, b| a.dot(view_dir).total_cmp(&b.dot(view_dir)))
                    .unwrap_or_else(|| frame.bitangent())
            });
        Some(Self::new(origin, VehicleAxes::new(frame.tangent(), frame.bitangent(), frame.normal()), heading, parts))
    }

    pub(super) fn park_into_blocks(
        self,
        blocks: &mut HashMap<(i32, i32), Vec<ElevatedShape>>,
        strata: &NaturalStrata,
        shafts: &ShaftExcavation,
        frame: &SurfaceFrame,
    ) -> (i32, i32) {
        let (seat_u, seat_v) = frame.cell_at(self.origin).feed_coordinates(|cu, cv| (cu, cv));
        let (du, dv) = (self.axes.tangent.dot(frame.tangent()), self.axes.tangent.dot(frame.bitangent()));
        let (rxx, rxy) = if du.abs() >= dv.abs() {
            (if du >= 0.0 { 1 } else { -1 }, 0)
        } else {
            (0, if dv >= 0.0 { 1 } else { -1 })
        };
        let (ryx, ryy) = (-rxy, rxx);
        for part in self.parts {
            let (ox, oy) = part.offset;
            let (bu, bv) = (seat_u + ox * rxx + oy * ryx, seat_v + ox * rxy + oy * ryy);
            let ground_top = (shafts.elevation_at(strata, bu, bv) as f32) * BLOCK_STEP;
            let list = blocks.entry((bu, bv)).or_default();
            list.push(ElevatedShape::new(part.shape, ground_top + part.floor_offset));
            list.sort_by(|a, b| a.ceiling().total_cmp(&b.ceiling()));
        }
        (seat_u, seat_v)
    }
}
