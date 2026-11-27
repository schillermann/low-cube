mod attire;
mod berth;
mod callsign;
mod craft;
mod rhythm;
mod utterance;

use glam::{Quat, Vec3};
use crate::relief::CraftPose;
pub use self::attire::PresenceAttire;
pub use self::berth::VesselBerth;
pub use self::callsign::{Callsign, MAX_CALLSIGN_BYTES};
pub use self::craft::{DrivenCraft, PilotCraft};
pub use self::rhythm::PresenceRhythm;
pub use self::utterance::{ProximityUtterance, MAX_UTTERANCE_BYTES};

/// Encapsulated 3D stance and orientation of a companion avatar on the World-Cube.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PresenceKinematics {
    position: Vec3,
    surface_rot: Quat,
    view_rot: Quat,
    eye_and_leap: (f32, f32),
}

impl PresenceKinematics {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(position: Vec3, surface_rot: Quat, view_rot: Quat, eye_and_leap: (f32, f32)) -> Self {
        Self { position, surface_rot, view_rot, eye_and_leap }
    }

    pub const fn with_position(self, position: Vec3) -> Self {
        Self::new(position, self.surface_rot, self.view_rot, self.eye_and_leap)
    }

    pub fn shifted(self, delta: Vec3) -> Self {
        Self::new(self.position + delta, self.surface_rot, self.view_rot, self.eye_and_leap)
    }

    pub fn localized_to_hull(self, hull_pose: CraftPose) -> Self {
        let inv = hull_pose.hull_rotation().conjugate();
        Self::new(
            hull_pose.local_point_of(self.position),
            (inv * self.surface_rot).normalize(),
            (inv * self.view_rot).normalize(),
            self.eye_and_leap,
        )
    }

    pub fn folded_onto_hull(self, hull_pose: CraftPose) -> Self {
        let rot = hull_pose.hull_rotation();
        Self::new(
            hull_pose.world_point_at(self.position),
            (rot * self.surface_rot).normalize(),
            (rot * self.view_rot).normalize(),
            self.eye_and_leap,
        )
    }

    pub fn interpolated(self, target: Self, alpha: f32) -> Self {
        let t = alpha.clamp(0.0, 1.0);
        let pos = if (target.position - self.position).length_squared() > 64.0 {
            target.position
        } else {
            self.position.lerp(target.position, t)
        };
        let s_rot = self.surface_rot.normalize().slerp(target.surface_rot.normalize(), t);
        let v_rot = self.view_rot.normalize().slerp(target.view_rot.normalize(), t);
        let eye = self.eye_and_leap.0 + (target.eye_and_leap.0 - self.eye_and_leap.0) * t;
        let leap = self.eye_and_leap.1 + (target.eye_and_leap.1 - self.eye_and_leap.1) * t;
        Self::new(pos, s_rot, v_rot, (eye, leap))
    }

    pub fn transfer<T>(self, f: impl FnOnce(Vec3, Quat, Quat, f32, f32) -> T) -> T {
        f(self.position, self.surface_rot, self.view_rot, self.eye_and_leap.0, self.eye_and_leap.1)
    }
}

/// Complete network-replicated presence of a companion player without exposing mutable fields.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CompanionPresence {
    slot: u32,
    kinematics: PresenceKinematics,
    rhythm: (f32, f32, f32, f32),
    attire: PresenceAttire,
}

impl CompanionPresence {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(
        slot: u32,
        kinematics: PresenceKinematics,
        rhythm: (f32, f32, f32, f32),
        attire: PresenceAttire,
    ) -> Self {
        Self { slot, kinematics, rhythm, attire }
    }

    pub const fn slot(&self) -> u32 {
        self.slot
    }

    pub const fn same_slot(&self, slot: u32) -> bool {
        self.slot == slot
    }

    pub const fn with_slot(self, slot: u32) -> Self {
        Self::new(slot, self.kinematics, self.rhythm, self.attire)
    }

    pub const fn with_position(self, position: Vec3) -> Self {
        Self::new(self.slot, self.kinematics.with_position(position), self.rhythm, self.attire)
    }

    pub fn shifted(self, delta: Vec3) -> Self {
        Self::new(self.slot, self.kinematics.shifted(delta), self.rhythm, self.attire)
    }

    pub fn localized_to_hull(self, hull_pose: CraftPose) -> Self {
        Self::new(self.slot, self.kinematics.localized_to_hull(hull_pose), self.rhythm, self.attire)
    }

    pub fn folded_onto_hull(self, hull_pose: CraftPose) -> Self {
        Self::new(self.slot, self.kinematics.folded_onto_hull(hull_pose), self.rhythm, self.attire)
    }

    pub fn within_reach(&self, eye: Vec3, max_dist: f32) -> bool {
        self.distance_sq_to(eye) <= max_dist * max_dist
    }

    pub fn distance_sq_to(&self, anchor: Vec3) -> f32 {
        self.kinematics.transfer(|pos, _, _, _, _| (anchor - pos).length_squared())
    }

    pub fn reconciled(self, slot: u32, incoming: Self, world: &dyn crate::PlanetaryWorld) -> Self {
        incoming.kinematics.transfer(|pos, _, _, _, _| {
            let clamped = world.confine_beneath_crust(pos);
            if clamped.is_finite() && (clamped - pos).length_squared() <= 36.0 {
                incoming.with_slot(slot)
            } else {
                self.with_slot(slot)
            }
        })
    }

    pub fn interpolated(self, target: Self, alpha: f32) -> Self {
        let t = alpha.clamp(0.0, 1.0);
        let rhythm = PresenceRhythm::from_wire(self.rhythm)
            .interpolated(PresenceRhythm::from_wire(target.rhythm), t)
            .wire_tuple();
        Self::new(target.slot, self.kinematics.interpolated(target.kinematics, t), rhythm, target.attire)
    }

    pub fn transfer<T>(
        self,
        f: impl FnOnce(u32, PresenceKinematics, (f32, f32, f32, f32), PresenceAttire) -> T,
    ) -> T {
        f(self.slot, self.kinematics, self.rhythm, self.attire)
    }

    pub fn inscribe(&self, out: &mut Vec<u8>) {
        out.extend_from_slice(&self.slot.to_le_bytes());
        self.kinematics.transfer(|pos, s_rot, v_rot, eye_h, leap| {
            for f in [
                pos.x, pos.y, pos.z,
                s_rot.x, s_rot.y, s_rot.z, s_rot.w,
                v_rot.x, v_rot.y, v_rot.z, v_rot.w,
                eye_h, leap,
                self.rhythm.0, self.rhythm.1, self.rhythm.2, self.rhythm.3,
            ] {
                out.extend_from_slice(&f.to_le_bytes());
            }
        });
        self.attire.inscribe(out);
    }

    pub fn decode(bytes: &[u8]) -> Option<(Self, usize)> {
        const HEAD_SPAN: usize = 4 + 17 * 4;
        if bytes.len() < HEAD_SPAN + PresenceAttire::WIRE_SPAN {
            return None;
        }
        let slot = u32::from_le_bytes(bytes[0..4].try_into().ok()?);
        let mut floats = [0.0f32; 17];
        for (i, dst) in floats.iter_mut().enumerate() {
            let off = 4 + i * 4;
            *dst = f32::from_le_bytes(bytes[off..off + 4].try_into().ok()?);
        }
        let (attire, attire_span) = PresenceAttire::decode(&bytes[HEAD_SPAN..])?;
        let kin = PresenceKinematics::new(
            Vec3::new(floats[0], floats[1], floats[2]),
            Quat::from_xyzw(floats[3], floats[4], floats[5], floats[6]),
            Quat::from_xyzw(floats[7], floats[8], floats[9], floats[10]),
            (floats[11], floats[12]),
        );
        Some((Self::new(slot, kin, (floats[13], floats[14], floats[15], floats[16]), attire), HEAD_SPAN + attire_span))
    }
}
