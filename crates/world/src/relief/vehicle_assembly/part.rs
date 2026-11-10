use std::collections::HashMap;
use glam::{Quat, Vec3};
use crate::block::{BlockMedium, ElevatedShape, GridCell, VoxelShape};
use crate::frame::SurfaceFrame;
use crate::{MeshTarget, Quad};
use super::super::natural_strata::NaturalStrata;
use super::super::shaft_excavation::ShaftExcavation;

pub(super) const BLOCK_STEP: f32 = GridCell::SPAN;

pub(super) struct ShiftedMeshTarget<'a> {
    pub(super) shift: Vec3,
    pub(super) inner: &'a mut dyn MeshTarget,
}

impl<'a> MeshTarget for ShiftedMeshTarget<'a> {
    fn push_quad(&mut self, quad: Quad) {
        let (s, glow) = (self.shift, quad.radiate(|g| g));
        self.inner.push_quad(quad.transfer(|v0, v1, v2, v3, norm, col| {
            Quad::glowing(v0 + s, v1 + s, v2 + s, v3 + s, norm, col, glow)
        }));
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

    pub(super) fn origin_seat_floor(&self) -> Option<f32> {
        (self.offset == (0, 0) && self.shape.block_medium() == BlockMedium::CabinSeat).then_some(self.floor_offset)
    }

    pub(super) fn wheel_radius(&self) -> Option<f32> {
        (self.shape.block_medium() == BlockMedium::CarWheel).then(|| self.shape.wheel_radius())
    }

    pub(super) fn steering_heading(&self, frame: &SurfaceFrame) -> Option<Vec3> {
        (self.shape.block_medium() == BlockMedium::SteeringWheel).then(|| self.shape.block_heading().forward_vector(frame))
    }

    pub(super) fn emit_driven(&self, lift: f32, spin: f32, turn: f32, frame: &SurfaceFrame, top: Vec3, wall: Vec3, out: &mut dyn MeshTarget) {
        let cell = GridCell::new(self.offset.0, self.offset.1);
        self.shape.emit_driven_part(cell, self.floor_offset + lift, spin, turn, frame, top, wall, out);
    }

    pub(super) fn park_into(
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
    pub(super) fn surface_frame(self) -> SurfaceFrame { SurfaceFrame::new(self.tangent, self.bitangent, self.normal, 0.0) }

    pub(super) fn cardinal_turn(self, frame: &SurfaceFrame) -> (i32, i32, u8) {
        let (du, dv) = (self.tangent.dot(frame.tangent()), self.tangent.dot(frame.bitangent()));
        if du.abs() >= dv.abs() {
            if du >= 0.0 { (1, 0, 0) } else { (-1, 0, 2) }
        } else if dv >= 0.0 {
            (0, 1, 1)
        } else {
            (0, -1, 3)
        }
    }

    pub(super) fn rotated(self, rot: Quat) -> Self {
        Self::new(
            (rot * self.tangent).normalize_or_zero(),
            (rot * self.bitangent).normalize_or_zero(),
            (rot * self.normal).normalize_or_zero(),
        )
    }
}
