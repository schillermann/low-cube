use glam::Vec3;
use crate::block::{
    BlockMedium, BlockMold, GridCell, MoldArchetype, NozzleAxis, SensorPodAxis, VoxelShape,
    WedgeHeading,
};
use crate::frame::SurfaceFrame;
use crate::MeshTarget;
use super::super::craft_cluster::CraftCluster;
use super::super::natural_strata::NaturalStrata;
use super::super::shaft_excavation::ShaftExcavation;
use super::impulse::{PilotOrder, RigidImpulse};

pub(in crate::relief) const BLOCK_STEP: f32 = GridCell::SPAN;

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

    pub(in crate::relief) fn expand_span(
        &self,
        basis: (i32, i32, i32, i32),
        bounds: &mut (i32, i32, i32, i32),
    ) {
        let ((rxx, rxy, ryx, ryy), (ox, oy)) = (basis, self.offset);
        let (du, dv) = (ox * rxx + oy * ryx, ox * rxy + oy * ryy);
        bounds.0 = bounds.0.min(du);
        bounds.1 = bounds.1.max(du);
        bounds.2 = bounds.2.min(dv);
        bounds.3 = bounds.3.max(dv);
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

    pub(in crate::relief) fn any_ceiling(&self, lift: f32) -> Option<(f32, f32)> {
        Some((self.floor_offset + lift, self.floor_offset + lift + BLOCK_STEP))
    }

    pub(in crate::relief) fn camera_equipped(&self) -> bool {
        self.shape.block_medium() == BlockMedium::FieldCamera
    }

    pub(in crate::relief) fn rocket_camera(&self) -> bool {
        self.shape.block_medium() == BlockMedium::RocketCamera
    }

    pub(in crate::relief) fn rocket_pod_at(&self, lift: f32) -> Option<(SensorPodAxis, (i32, i32), f32)> {
        if !self.rocket_camera() {
            return None;
        }
        let axis = SensorPodAxis::from_mold(
            MoldArchetype::from_code(self.shape.shape_stamp()[0]),
            self.shape.block_heading(),
        );
        Some((axis, self.offset, self.floor_offset + lift))
    }

    pub(in crate::relief) fn camera_lens_at(&self, lift: f32, frame: &SurfaceFrame) -> Option<(Vec3, Vec3, (i32, i32), f32)> {
        let cell = GridCell::new(self.offset.0, self.offset.1);
        let (hub, base_fwd, up) = self.shape.camera_tripod_hub(cell, self.floor_offset + lift, frame)?;
        let yaw = MoldArchetype::from_code(self.shape.shape_stamp()[0]).placed_camera_yaw();
        let right = base_fwd.cross(up).normalize_or_zero();
        let (s, c) = yaw.sin_cos();
        let lens_fwd = (base_fwd * c + right * s).normalize_or_zero();
        Some((hub, lens_fwd, self.offset, self.floor_offset + lift))
    }

    pub(in crate::relief) fn console_base_at(&self, lift: f32, frame: &SurfaceFrame) -> Option<Vec3> {
        if !self.flight_console() {
            return None;
        }
        let (uc, vc) = GridCell::new(self.offset.0, self.offset.1).center();
        Some(frame.point_at(uc, vc, self.floor_offset + lift))
    }

    pub fn attitude_pod(&self) -> bool {
        self.shape.block_medium() == BlockMedium::RocketNozzle
            && NozzleAxis::from_mold(
                MoldArchetype::from_code(self.shape.shape_stamp()[0]),
                self.shape.block_heading(),
            )
            .attitude_pod()
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

    pub fn flight_console(&self) -> bool {
        self.shape.block_medium() == BlockMedium::FlightConsole
    }

    pub fn console_heading(&self, frame: &SurfaceFrame) -> Option<Vec3> {
        self.flight_console().then(|| self.shape.block_heading().forward_vector(frame))
    }

    pub fn steering_heading(&self, frame: &SurfaceFrame) -> Option<Vec3> {
        (self.shape.block_medium() == BlockMedium::SteeringWheel).then(|| self.shape.block_heading().forward_vector(frame))
    }

    pub(in crate::relief) fn world_mass_center(&self, lift: f32, frame: &SurfaceFrame) -> Vec3 {
        let (uc, vc) = GridCell::new(self.offset.0, self.offset.1).center();
        frame.point_at(uc, vc, self.floor_offset + lift + 0.25)
    }

    #[allow(clippy::too_many_arguments)]
    pub(in crate::relief) fn contribute_impulse(
        &self,
        lift: f32,
        frame: &SurfaceFrame,
        center_of_mass: Vec3,
        order: &PilotOrder,
        right: Vec3,
        up: Vec3,
        fwd: Vec3,
        impulse: &mut RigidImpulse,
    ) {
        impulse.add_block_mass(1.0);
        match self.shape.block_medium() {
            BlockMedium::FlightConsole => impulse.add_console_gyro(),
            BlockMedium::RocketNozzle => {
                let stamp = self.shape.shape_stamp();
                let axis = NozzleAxis::from_mold(MoldArchetype::from_code(stamp[0]), self.shape.block_heading());
                let thrust_dir = axis.thrust_vector(frame).normalize_or_zero();
                let throttle = order.throttle_along(thrust_dir, right, up, fwd);
                if axis.attitude_pod() {
                    let retro_dir = -frame.normal().normalize_or_zero();
                    let retro_throttle = order.throttle_along(retro_dir, right, up, fwd);
                    impulse.apply_attitude_jet(thrust_dir, throttle);
                    impulse.apply_attitude_jet(retro_dir, retro_throttle);
                    impulse.apply_attitude_steer(order.attitude_rate(right, up));
                } else {
                    let lever = self.world_mass_center(lift, frame) - center_of_mass;
                    impulse.apply_nozzle_burn(lever, thrust_dir, throttle);
                }
            }
            _ => {}
        }
    }

    pub(in crate::relief) fn emit_driven(&self, lift: f32, spin: f32, turn: f32, frame: &SurfaceFrame, top: Vec3, wall: Vec3, out: &mut dyn MeshTarget) {
        let cell = GridCell::new(self.offset.0, self.offset.1);
        self.shape.emit_driven_part(cell, self.floor_offset + lift, spin, turn, frame, top, wall, out);
    }

    pub(in crate::relief) fn park_into(
        mut self,
        seat: (i32, i32),
        basis: (i32, i32, i32, i32),
        lift: f32,
        craft: &mut CraftCluster,
        strata: &NaturalStrata,
        shafts: &ShaftExcavation,
    ) {
        let ((rxx, rxy, ryx, ryy), (ox, oy)) = (basis, self.offset);
        let (hx, hy) = match self.shape.block_heading() {
            WedgeHeading::East => (1, 0),
            WedgeHeading::North => (0, 1),
            WedgeHeading::West => (-1, 0),
            WedgeHeading::South => (0, -1),
        };
        let target = match (hx * rxx + hy * ryx, hx * rxy + hy * ryy) {
            (1, 0) => WedgeHeading::East,
            (0, 1) => WedgeHeading::North,
            (-1, 0) => WedgeHeading::West,
            _ => WedgeHeading::South,
        };
        for _ in 0..4 {
            if self.shape.block_heading() == target {
                break;
            }
            self.shape.rotate_quarter();
        }
        let (bu, bv) = (seat.0 + ox * rxx + oy * ryx, seat.1 + ox * rxy + oy * ryy);
        let ground_top = (shafts.elevation_at(strata, bu, bv) as f32) * BLOCK_STEP;
        craft.insert(bu, bv, ground_top + lift + self.floor_offset, self.shape);
    }
}
