use glam::{Mat3, Quat, Vec3};
use world::{ClimateZone, CraftIdentity, PlanetaryWorld};
use crate::attire::BodyPose;
use super::track::FaceContact;
use super::{PedalInput, SEAT_EYE_SUPPORT};

/// Seated companion passenger berth tracking a live `CraftIdentity` across turns and 90° cube seams.
#[derive(Clone, Copy, Debug)]
pub struct PassengerBerth {
    craft: CraftIdentity,
    local_seat: Vec3,
    seat_pos: Vec3,
    orientation: (Vec3, Vec3),
}

impl PassengerBerth {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(
        craft: CraftIdentity,
        local_seat: Vec3,
        seat_pos: Vec3,
        orientation: (Vec3, Vec3),
    ) -> Self {
        Self { craft, local_seat, seat_pos, orientation }
    }

    pub fn mounted(craft: CraftIdentity, local_seat: Vec3, frame: (Vec3, Vec3, Vec3)) -> (Self, Vec3, Vec3) {
        let (seat, heading, up) = frame;
        let eye_pos = seat + up.normalize_or_zero() * (SEAT_EYE_SUPPORT + 0.12);
        (Self::new(craft, local_seat, seat, (heading, up)), eye_pos, heading)
    }

    pub fn shape_pose(&self, pose: BodyPose) -> BodyPose {
        let (raw_fwd, raw_up) = self.orientation;
        let up = raw_up.normalize_or_zero();
        let fwd = (raw_fwd - up * raw_fwd.dot(up)).normalize_or_zero();
        let right = fwd.cross(up).normalize_or_zero();
        let rot = Quat::from_mat3(&Mat3::from_cols(right, up, -fwd)).normalize();
        pose.with_surface_rotation(rot).with_seated(1.0)
    }

    /// Advances passenger berth or dismounts onto solid ground, returning `(next_berth_if_seated, eye_pos, up, steer_px)`.
    pub fn step(self, pedals: &PedalInput, world: &dyn PlanetaryWorld) -> (Option<Self>, Vec3, Vec3, f32) {
        let Some((seat, heading, up_raw)) = world.passenger_seat_frame(self.craft, self.local_seat) else {
            let pos = FaceContact::new(self.seat_pos).ground(SEAT_EYE_SUPPORT, world);
            return (None, pos, world.surface_up_at(pos), 0.0);
        };
        let (prev_heading, prev_up_raw) = self.orientation;
        let prev_up = prev_up_raw.normalize_or_zero();
        let up = up_raw.normalize_or_zero();
        let transported = Quat::from_rotation_arc(prev_up, up) * prev_heading;
        let flat_prev = (transported - up * transported.dot(up)).normalize_or_zero();
        let flat_cur = (heading - up * heading.dot(up)).normalize_or_zero();
        let yaw_delta = flat_prev.cross(flat_cur).dot(up).clamp(-1.0, 1.0).asin();
        if pedals.dismount_requested() {
            let face_up = ClimateZone::dominant(seat).normal();
            let fwd = (heading - face_up * heading.dot(face_up)).normalize_or_zero();
            let right = fwd.cross(face_up).normalize_or_zero();
            let dir = pedals.dismount_vector(fwd, right);
            let exit = FaceContact::new(seat + dir * 1.15).ground(SEAT_EYE_SUPPORT, world);
            return (None, exit, world.surface_up_at(exit), 0.0);
        }
        let next = Self::new(self.craft, self.local_seat, seat, (heading, up_raw));
        (Some(next), seat + up * (SEAT_EYE_SUPPORT + 0.12), up, -yaw_delta / 0.003)
    }
}
