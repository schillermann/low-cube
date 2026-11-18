use glam::Vec3;
use crate::block::GridCell;
use crate::MeshTarget;
use super::super::vehicle_assembly::{ShiftedMeshTarget, VehiclePart};
use super::pose::CraftPose;

const MAX_BLUEPRINT_PARTS: usize = 512;

/// Immutable 64-bit entity identity of a commissioned multi-block vehicle.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct CraftIdentity {
    code: u64,
}

impl CraftIdentity {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(code: u64) -> Self {
        Self { code }
    }

    pub const fn unassigned() -> Self {
        Self::new(0)
    }

    pub const fn assigned(self) -> bool {
        self.code != 0
    }

    pub fn from_parts(origin: Vec3, parts: &[VehiclePart]) -> Self {
        let mut h: u64 = 0xcbf2_9ce4_8422_2325;
        for q in [
            (origin.x * 4.0).round() as i32,
            (origin.y * 4.0).round() as i32,
            (origin.z * 4.0).round() as i32,
        ] {
            for b in q.to_le_bytes() {
                h = (h ^ (b as u64)).wrapping_mul(0x1000_0000_01b3);
            }
        }
        for part in parts {
            h = part.digest_mix(h);
        }
        Self::new(if h == 0 { 1 } else { h })
    }

    pub fn inscribe(self, out: &mut Vec<u8>) {
        out.extend_from_slice(&self.code.to_le_bytes());
    }

    pub fn decode(bytes: &[u8]) -> Option<(Self, usize)> {
        if bytes.len() < 8 {
            return None;
        }
        let code = u64::from_le_bytes(bytes[0..8].try_into().ok()?);
        Some((Self::new(code), 8))
    }
}

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

    pub const fn identity(&self) -> CraftIdentity {
        self.identity
    }

    pub fn drivable(&self) -> bool {
        !self.parts.is_empty()
    }

    pub fn parts(&self) -> &[VehiclePart] {
        &self.parts
    }

    pub fn into_parts(self) -> Vec<VehiclePart> {
        self.parts
    }

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

    pub fn emit_at(&self, pose: CraftPose, target: &mut dyn MeshTarget) {
        let car_frame = pose.surface_frame();
        let shift = pose.origin() - car_frame.point_at(0.25, 0.25, 0.0);
        let mut shifted = ShiftedMeshTarget::new(shift, target);
        let (top_col, wall_col) = (Vec3::new(0.45, 0.30, 0.17), Vec3::new(0.41, 0.27, 0.15));
        let lift = self.wheel_ground_lift();
        for part in &self.parts {
            part.emit_driven(lift, pose.wheel_spin(), pose.steer_turn(), &car_frame, top_col, wall_col, &mut shifted);
        }
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
