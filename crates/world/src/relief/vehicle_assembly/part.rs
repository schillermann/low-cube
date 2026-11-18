use glam::{Quat, Vec3};
use crate::block::{BlockMedium, BlockMold, GridCell, VoxelShape};
use crate::frame::SurfaceFrame;
use crate::{MeshTarget, Quad};
use super::super::craft_cluster::CraftCluster;
use super::super::natural_strata::NaturalStrata;
use super::super::shaft_excavation::ShaftExcavation;

pub(in crate::relief) const BLOCK_STEP: f32 = GridCell::SPAN;

pub(in crate::relief) fn push_vec3(v: Vec3, out: &mut Vec<u8>) {
    out.extend_from_slice(&v.x.to_le_bytes());
    out.extend_from_slice(&v.y.to_le_bytes());
    out.extend_from_slice(&v.z.to_le_bytes());
}

pub(in crate::relief) fn read_vec3(bytes: &[u8]) -> Option<Vec3> {
    if bytes.len() < 12 {
        return None;
    }
    let v = Vec3::new(
        f32::from_le_bytes(bytes[0..4].try_into().ok()?),
        f32::from_le_bytes(bytes[4..8].try_into().ok()?),
        f32::from_le_bytes(bytes[8..12].try_into().ok()?),
    );
    v.is_finite().then_some(v)
}

pub(in crate::relief) struct ShiftedMeshTarget<'a> {
    shift: Vec3,
    inner: &'a mut dyn MeshTarget,
}

impl<'a> ShiftedMeshTarget<'a> {
    pub(in crate::relief) fn new(shift: Vec3, inner: &'a mut dyn MeshTarget) -> Self {
        Self { shift, inner }
    }
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

impl Clone for VehiclePart {
    fn clone(&self) -> Self {
        Self::new(self.offset, self.floor_offset, BlockMold::shape_from_stamp(self.shape.shape_stamp()))
    }
}

impl std::fmt::Debug for VehiclePart {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VehiclePart")
            .field("offset", &self.offset)
            .field("floor_offset", &self.floor_offset)
            .field("stamp", &self.shape.shape_stamp())
            .finish()
    }
}

impl PartialEq for VehiclePart {
    fn eq(&self, other: &Self) -> bool {
        self.offset == other.offset
            && (self.floor_offset - other.floor_offset).abs() <= 1e-5
            && self.shape.shape_stamp() == other.shape.shape_stamp()
    }
}

impl VehiclePart {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(offset: (i32, i32), floor_offset: f32, shape: Box<dyn VoxelShape>) -> Self {
        Self { offset, floor_offset, shape }
    }

    pub(in crate::relief) fn inscribe(&self, out: &mut Vec<u8>) {
        out.extend_from_slice(&(self.offset.0.clamp(-128, 128) as i16).to_le_bytes());
        out.extend_from_slice(&(self.offset.1.clamp(-128, 128) as i16).to_le_bytes());
        out.extend_from_slice(&self.floor_offset.to_le_bytes());
        out.extend_from_slice(&self.shape.shape_stamp());
    }

    pub(in crate::relief) fn decode(bytes: &[u8]) -> Option<(Self, usize)> {
        if bytes.len() < 11 {
            return None;
        }
        let ox = i16::from_le_bytes(bytes[0..2].try_into().ok()?) as i32;
        let oy = i16::from_le_bytes(bytes[2..4].try_into().ok()?) as i32;
        let floor_offset = f32::from_le_bytes(bytes[4..8].try_into().ok()?);
        if !floor_offset.is_finite() || ox.abs() > 128 || oy.abs() > 128 || floor_offset.abs() > 32.0 {
            return None;
        }
        let stamp = [bytes[8], bytes[9], bytes[10]];
        Some((Self::new((ox, oy), floor_offset, BlockMold::shape_from_stamp(stamp)), 11))
    }

    pub(in crate::relief) fn canonical_key(&self) -> (i32, i32, i32, [u8; 3]) {
        (
            self.offset.0,
            self.offset.1,
            (self.floor_offset * 16.0).round() as i32,
            self.shape.shape_stamp(),
        )
    }

    pub(in crate::relief) fn digest_mix(&self, mut h: u64) -> u64 {
        for b in (self.offset.0 as i16).to_le_bytes() {
            h = (h ^ (b as u64)).wrapping_mul(0x100000001b3);
        }
        for b in (self.offset.1 as i16).to_le_bytes() {
            h = (h ^ (b as u64)).wrapping_mul(0x100000001b3);
        }
        let qf = ((self.floor_offset * 16.0).round() as i16).to_le_bytes();
        for b in qf {
            h = (h ^ (b as u64)).wrapping_mul(0x100000001b3);
        }
        for b in self.shape.shape_stamp() {
            h = (h ^ (b as u64)).wrapping_mul(0x100000001b3);
        }
        h
    }

    pub(in crate::relief) fn passenger_seat_slot(&self) -> Option<(i32, i32, f32)> {
        (self.offset != (0, 0) && self.shape.block_medium() == BlockMedium::CabinSeat)
            .then_some((self.offset.0, self.offset.1, self.floor_offset))
    }

    pub(in crate::relief) fn cell_top_at(&self, cu: i32, cv: i32, lift: f32) -> Option<(f32, f32)> {
        (self.offset == (cu, cv)).then_some((self.floor_offset + lift, self.floor_offset + lift + BLOCK_STEP))
    }

    pub(in crate::relief) fn origin_seat_floor(&self) -> Option<f32> {
        (self.offset == (0, 0) && self.shape.block_medium() == BlockMedium::CabinSeat).then_some(self.floor_offset)
    }

    pub(in crate::relief) fn wheel_floor_drop(&self) -> Option<f32> {
        self.shape.wheel_tread_drop().map(|drop| drop - self.floor_offset)
    }

    pub(in crate::relief) fn wheel_radius(&self) -> Option<f32> {
        (self.shape.block_medium() == BlockMedium::CarWheel).then(|| self.shape.wheel_radius())
    }

    pub fn steering_heading(&self, frame: &SurfaceFrame) -> Option<Vec3> {
        (self.shape.block_medium() == BlockMedium::SteeringWheel).then(|| self.shape.block_heading().forward_vector(frame))
    }

    pub(in crate::relief) fn emit_driven(&self, lift: f32, spin: f32, turn: f32, frame: &SurfaceFrame, top: Vec3, wall: Vec3, out: &mut dyn MeshTarget) {
        let cell = GridCell::new(self.offset.0, self.offset.1);
        self.shape.emit_driven_part(cell, self.floor_offset + lift, spin, turn, frame, top, wall, out);
    }

    pub(in crate::relief) fn park_into(
        mut self,
        seat: (i32, i32),
        turn: (i32, i32, u8),
        lift: f32,
        craft: &mut CraftCluster,
        strata: &NaturalStrata,
        shafts: &ShaftExcavation,
    ) {
        let ((rxx, rxy, quarter_turns), (ox, oy)) = (turn, self.offset);
        let (ryx, ryy) = (-rxy, rxx);
        for _ in 0..quarter_turns {
            self.shape.rotate_quarter();
        }
        let (bu, bv) = (seat.0 + ox * rxx + oy * ryx, seat.1 + ox * rxy + oy * ryy);
        let ground_top = (shafts.elevation_at(strata, bu, bv) as f32) * BLOCK_STEP;
        craft.insert(bu, bv, ground_top + lift + self.floor_offset, self.shape);
    }
}

/// Orthonormal coordinate axes of a drivable vehicle assembly on a planetary surface.
#[derive(Clone, Copy, Debug, PartialEq)]
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
    pub(in crate::relief) fn surface_frame(self) -> SurfaceFrame { SurfaceFrame::new(self.tangent, self.bitangent, self.normal, 0.0) }

    pub(in crate::relief) fn interpolated(self, target: Self, alpha: f32) -> Self {
        let raw_n = self.normal.lerp(target.normal, alpha);
        let n = if raw_n.length_squared() > 1e-6 { raw_n.normalize() } else { target.normal };
        let raw_t = self.tangent.lerp(target.tangent, alpha);
        let t_ortho = raw_t - n * raw_t.dot(n);
        let t = if t_ortho.length_squared() > 1e-6 { t_ortho.normalize() } else { target.tangent };
        let raw_b = self.bitangent.lerp(target.bitangent, alpha);
        let b_ortho = raw_b - n * raw_b.dot(n) - t * raw_b.dot(t);
        let b = if b_ortho.length_squared() > 1e-6 { b_ortho.normalize() } else { target.bitangent };
        Self::new(t, b, n)
    }

    pub(in crate::relief) fn inscribe(self, out: &mut Vec<u8>) {
        push_vec3(self.tangent, out);
        push_vec3(self.bitangent, out);
        push_vec3(self.normal, out);
    }

    pub(in crate::relief) fn decode(bytes: &[u8]) -> Option<(Self, usize)> {
        if bytes.len() < 36 {
            return None;
        }
        let t = read_vec3(&bytes[0..12])?;
        let b = read_vec3(&bytes[12..24])?;
        let n = read_vec3(&bytes[24..36])?;
        if t.length_squared() < 0.25 || b.length_squared() < 0.25 || n.length_squared() < 0.25 {
            return None;
        }
        Some((Self::new(t.normalize(), b.normalize(), n.normalize()), 36))
    }

    pub(in crate::relief) fn cardinal_turn(self, frame: &SurfaceFrame) -> (i32, i32, u8) {
        let (du, dv) = (self.tangent.dot(frame.tangent()), self.tangent.dot(frame.bitangent()));
        if du.abs() >= dv.abs() {
            if du >= 0.0 { (1, 0, 0) } else { (-1, 0, 2) }
        } else if dv >= 0.0 {
            (0, 1, 1)
        } else {
            (0, -1, 3)
        }
    }

    pub(in crate::relief) fn rotated(self, rot: Quat) -> Self {
        Self::new(
            (rot * self.tangent).normalize_or_zero(),
            (rot * self.bitangent).normalize_or_zero(),
            (rot * self.normal).normalize_or_zero(),
        )
    }
}
