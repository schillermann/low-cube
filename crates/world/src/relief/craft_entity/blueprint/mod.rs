mod optics;

use glam::Vec3;
use crate::block::{BlockMedium, ConsoleOrrery, GridCell};
use crate::frame::SurfaceFrame;
use crate::MeshTarget;
use super::super::craft_cluster::CraftCluster;
use super::super::vehicle_assembly::{
    CockpitAxes, ConsoleGyroStage, HullBallast, ImpulseStage, MainBellStage, PilotOrder,
    RigidImpulse, VehiclePart, VernierAttitudeStage,
};
use super::identity::CraftIdentity;
use super::pose::CraftPose;
use self::optics::BlueprintOptics;
pub(in crate::relief::craft_entity) use self::optics::ShiftedMeshTarget;

const MAX_BLUEPRINT_PARTS: usize = 512;

/// Immutable structural block layout of a vehicle registered once per `CraftIdentity`.
#[derive(Clone, Debug, PartialEq)]
pub struct CraftBlueprint {
    identity: CraftIdentity,
    parts: Vec<VehiclePart>,
}

impl CraftBlueprint {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(identity: CraftIdentity, parts: Vec<VehiclePart>) -> Self {
        Self { identity, parts }
    }

    pub fn commission(origin: Vec3, parts: Vec<VehiclePart>) -> Self {
        let id = if parts.is_empty() {
            CraftIdentity::unassigned()
        } else {
            CraftIdentity::from_parts(origin, &parts)
        };
        Self::new(id, parts)
    }

    pub const fn identity(&self) -> CraftIdentity { self.identity }
    pub fn part_count(&self) -> usize { self.parts.len() }
    pub fn drivable(&self) -> bool { !self.parts.is_empty() }
    pub fn spacefaring(&self) -> bool { self.parts.iter().any(VehiclePart::flight_console) }
    pub fn attitude_capable(&self) -> bool { self.parts.iter().any(VehiclePart::attitude_pod) }

    pub fn tally_media(&self, mut sink: impl FnMut(BlockMedium)) {
        for part in &self.parts {
            sink(part.block_medium());
        }
    }

    pub(in crate::relief) fn to_face_cluster(&self, frame: &SurfaceFrame) -> CraftCluster {
        let flip_v = frame.tangent().cross(frame.bitangent()).dot(frame.normal()) > 0.0;
        let v_sign = if flip_v { -1 } else { 1 };
        let mut craft = CraftCluster::untouched();
        for part in &self.parts {
            part.insert_rigid(v_sign, flip_v, &mut craft);
        }
        craft
    }

    pub fn camera_equipped(&self) -> bool {
        if self.spacefaring() {
            self.parts.iter().any(VehiclePart::rocket_camera)
        } else {
            self.parts.iter().any(VehiclePart::camera_equipped)
        }
    }

    pub fn push_camera_views(&self, pose: CraftPose, out: &mut Vec<(Vec3, Vec3, Vec3)>) {
        BlueprintOptics::new(self).push_views(pose, out);
    }

    pub fn sample_impulse(&self, order: &PilotOrder, pose: CraftPose) -> RigidImpulse {
        if self.parts.is_empty() {
            return RigidImpulse::zero();
        }
        let car_frame = pose.surface_frame();
        let lift = self.wheel_ground_lift();
        let com = self
            .parts
            .iter()
            .fold(Vec3::ZERO, |acc, p| acc + p.world_mass_center(lift, &car_frame))
            / (self.parts.len() as f32);
        let axes = CockpitAxes::from_pose(pose);
        let chain = VernierAttitudeStage::new(
            MainBellStage::new(
                ConsoleGyroStage::new(HullBallast::new(&self.parts), &self.parts),
                (&self.parts, &car_frame),
                (lift, com),
                (*order, axes),
            ),
            (&self.parts, &car_frame),
            (*order, axes),
        );
        chain.resolve()
    }

    pub fn parts(&self) -> &[VehiclePart] { &self.parts }

    pub(in crate::relief) fn step_camera_turn(&mut self, delta: i32) -> Option<(i32, i32)> {
        self.parts.iter_mut().rev().find_map(|p| p.step_camera_turn(delta))
    }

    pub fn into_parts(self) -> Vec<VehiclePart> { self.parts }

    pub fn wheel_ground_lift(&self) -> f32 {
        self.parts.iter().filter_map(VehiclePart::wheel_floor_drop).fold(0.0f32, f32::max)
    }

    pub fn seat_floor_offset(&self) -> f32 {
        let seat_off = self.parts.iter().find_map(VehiclePart::origin_seat_floor).unwrap_or(0.0);
        seat_off + self.wheel_ground_lift()
    }

    pub fn wheel_radius(&self) -> f32 {
        self.parts.iter().filter_map(VehiclePart::wheel_radius).fold(0.0f32, f32::max).max(0.16)
    }

    pub fn hull_envelope(&self) -> (f32, f32) {
        let lift = self.wheel_ground_lift();
        let part_top = self.parts.iter().filter_map(|p| p.any_ceiling(lift).map(|(_, c)| c + 0.15)).fold(0.5f32, f32::max);
        let hull_top = part_top.max(self.seat_floor_offset() + 1.35);
        let mut bounds = (0, 0, 0, 0);
        for part in &self.parts {
            part.expand_span((1, 0, 0, 1), &mut bounds);
        }
        let max_cell = bounds.0.abs().max(bounds.1.abs()).max(bounds.2.abs()).max(bounds.3.abs()) as f32;
        (hull_top, (max_cell * 0.5 + 0.35).max(0.55))
    }

    pub fn passenger_seat_along_ray(&self, pose: CraftPose, eye: Vec3, direction: Vec3) -> Option<Vec3> {
        let dir = direction.normalize_or_zero();
        if dir.length_squared() < 1e-4 {
            return None;
        }
        let lift = self.wheel_ground_lift();
        let mut best: Option<(Vec3, f32)> = None;
        for part in &self.parts {
            let Some((ox, oy, floor_off)) = part.passenger_seat_slot() else { continue };
            let local = Vec3::new((ox as f32) * 0.5, floor_off + lift, (oy as f32) * 0.5);
            let (world_seat, _, up) = pose.world_seat_at(local);
            let target_pt = world_seat + up * 0.25;
            let to_seat = target_pt - eye;
            let along = to_seat.dot(dir);
            if !(0.15..=4.2).contains(&along) {
                continue;
            }
            let radial_sq = (to_seat - dir * along).length_squared();
            if radial_sq <= 0.45 && best.is_none_or(|(_, d)| along < d) {
                best = Some((local, along));
            }
        }
        best.map(|(local, _)| local)
    }

    pub fn constrain_at(&self, pose: CraftPose, position: Vec3, player_height: f32) -> Option<Vec3> {
        if self.parts.is_empty() {
            return None;
        }
        let car_frame = pose.surface_frame();
        let shift = pose.origin() - car_frame.point_at(0.25, 0.25, 0.0);
        let rel = position - shift;
        let (u, v) = car_frame.decompose(rel);
        let elev = car_frame.elevation_of(rel);
        let (cu, cv) = GridCell::at_planar(u, v).feed_coordinates(|gu, gv| (gu, gv));
        let lift = self.wheel_ground_lift();
        let foot_elev = elev - player_height;
        let mut top_support: Option<f32> = None;
        for part in &self.parts {
            let Some((floor_h, ceil_h)) = part.cell_top_at(cu, cv, lift) else { continue };
            let passes_under = player_height > 1e-4 && foot_elev + 0.08 <= floor_h && floor_h >= 1.77;
            if !passes_under {
                top_support = Some(top_support.map_or(ceil_h, |cur| cur.max(ceil_h)));
            }
        }
        top_support.map(|h| shift + car_frame.point_at(u, v, h + player_height))
    }

    pub(in crate::relief) fn clamped_seat(
        &self,
        raw_seat: (i32, i32),
        basis: (i32, i32, i32, i32),
        half_cells: i32,
    ) -> (i32, i32) {
        if half_cells < 8 {
            return raw_seat;
        }
        let mut bounds = (0, 0, 0, 0);
        for part in &self.parts {
            part.expand_span(basis, &mut bounds);
        }
        (
            raw_seat.0.clamp(-half_cells - bounds.0, half_cells - 1 - bounds.1),
            raw_seat.1.clamp(-half_cells - bounds.2, half_cells - 1 - bounds.3),
        )
    }

    pub fn emit_at(&self, pose: CraftPose, target: &mut dyn MeshTarget) {
        let car_frame = pose.surface_frame();
        let shift = pose.origin() - car_frame.point_at(0.25, 0.25, 0.0);
        let lift = self.wheel_ground_lift();
        {
            let mut shifted = ShiftedMeshTarget::new(shift, target);
            let (top_col, wall_col) = (Vec3::new(0.45, 0.30, 0.17), Vec3::new(0.41, 0.27, 0.15));
            for part in &self.parts {
                part.emit_driven(lift, pose.wheel_spin(), pose.steer_turn(), &car_frame, top_col, wall_col, &mut shifted);
            }
        }
        for part in &self.parts {
            if let Some(base) = part.console_base_at(lift, &car_frame) {
                ConsoleOrrery::new(pose.origin(), shift + base, (pose.heading(), pose.normal())).emit(target);
            }
        }
    }

    pub fn emit_local_hull(&self, target: &mut dyn MeshTarget) {
        super::spatial::PosedHull::emit_local(self, target);
    }

    pub fn emit_kinematic_parts(&self, pose: CraftPose, target: &mut dyn MeshTarget) {
        super::spatial::PosedHull::new(self, pose).emit_kinematic(target);
    }

    pub fn inscribe(&self, out: &mut Vec<u8>) {
        self.identity.inscribe(out);
        let count = self.parts.len().min(MAX_BLUEPRINT_PARTS);
        out.extend_from_slice(&(count as u16).to_le_bytes());
        for part in self.parts.iter().take(count) {
            part.inscribe(out);
        }
    }

    pub fn decode(bytes: &[u8]) -> Option<(Self, usize)> {
        if bytes.len() < 10 {
            return None;
        }
        let (identity, _) = CraftIdentity::decode(&bytes[0..8])?;
        let count = u16::from_le_bytes(bytes[8..10].try_into().ok()?) as usize;
        if count == 0 || count > MAX_BLUEPRINT_PARTS {
            return None;
        }
        let mut cursor = 10;
        let mut parts = Vec::with_capacity(count);
        for _ in 0..count {
            let (part, used) = VehiclePart::decode(&bytes[cursor..])?;
            parts.push(part);
            cursor += used;
        }
        Some((Self::new(identity, parts), cursor))
    }
}
