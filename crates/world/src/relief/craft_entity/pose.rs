use glam::{Quat, Vec3};
use crate::frame::SurfaceFrame;
use crate::wire_vector::{WireSlice, WireVector};
use super::super::vehicle_assembly::VehicleAxes;

/// Compact 68-byte pose and wheel state of a living vehicle on the World-Cube.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CraftPose {
    origin: Vec3,
    axes: VehicleAxes,
    heading: Vec3,
    wheel_motion: (f32, f32),
}

impl CraftPose {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(origin: Vec3, axes: VehicleAxes, heading: Vec3, wheel_motion: (f32, f32)) -> Self {
        Self { origin, axes, heading, wheel_motion }
    }

    pub const fn parked(origin: Vec3, axes: VehicleAxes, heading: Vec3) -> Self {
        Self::new(origin, axes, heading, (0.0, 0.0))
    }

    pub const fn origin(self) -> Vec3 { self.origin }
    pub const fn axes(self) -> VehicleAxes { self.axes }
    pub const fn heading(self) -> Vec3 { self.heading }
    pub const fn normal(self) -> Vec3 { self.axes.normal() }
    pub const fn wheel_spin(self) -> f32 { self.wheel_motion.0 }
    pub const fn steer_turn(self) -> f32 { self.wheel_motion.1 }

    pub const fn with_origin(self, origin: Vec3) -> Self {
        Self::new(origin, self.axes, self.heading, self.wheel_motion)
    }

    pub const fn with_motion(self, wheel_spin: f32, steer_turn: f32) -> Self {
        Self::new(self.origin, self.axes, self.heading, (wheel_spin, steer_turn))
    }

    pub(in crate::relief) fn surface_frame(self) -> SurfaceFrame {
        self.axes.surface_frame()
    }

    pub fn model_matrix(self) -> glam::Mat4 {
        let frame = self.axes.surface_frame();
        let shift = self.origin - frame.point_at(0.25, 0.25, 0.0);
        glam::Mat4::from_cols(
            frame.tangent().extend(0.0),
            frame.normal().extend(0.0),
            frame.bitangent().extend(0.0),
            shift.extend(1.0),
        )
    }

    pub fn world_seat_at(self, local_seat: Vec3) -> (Vec3, Vec3, Vec3) {
        let frame = self.axes.surface_frame();
        let shift = self.origin - frame.point_at(0.25, 0.25, 0.0);
        let seat = shift + frame.point_at(local_seat.x + 0.25, local_seat.z + 0.25, local_seat.y);
        (seat, self.heading, self.axes.normal())
    }

    pub fn local_point_of(self, world_pos: Vec3) -> Vec3 {
        let frame = self.axes.surface_frame();
        let shift = self.origin - frame.point_at(0.25, 0.25, 0.0);
        let rel = world_pos - shift;
        let (u, v) = frame.decompose(rel);
        Vec3::new(u - 0.25, frame.elevation_of(rel), v - 0.25)
    }

    pub fn steered(self, delta_yaw: f32) -> Self {
        self.rotated(Quat::from_axis_angle(self.axes.normal(), delta_yaw))
    }

    pub fn rotated(self, rot: Quat) -> Self {
        Self::new(
            self.origin,
            self.axes.rotated(rot),
            (rot * self.heading).normalize_or_zero(),
            self.wheel_motion,
        )
    }

    pub fn relocated(self, next_origin: Vec3, next_up: Vec3) -> Self {
        if self.axes.normal().dot(next_up) < 0.99999 {
            let rot = Quat::from_rotation_arc(self.axes.normal(), next_up);
            Self::new(
                next_origin,
                self.axes.rotated(rot),
                (rot * self.heading).normalize_or_zero(),
                self.wheel_motion,
            )
        } else {
            Self::new(next_origin, self.axes, self.heading, self.wheel_motion)
        }
    }

    pub fn interpolated(self, target: Self, alpha: f32) -> Self {
        let t = alpha.clamp(0.0, 1.0);
        if (target.origin - self.origin).length_squared() > 64.0 {
            return target;
        }
        let origin = self.origin.lerp(target.origin, t);
        let axes = self.axes.interpolated(target.axes, t);
        let raw_h = self.heading.lerp(target.heading, t);
        let h_ortho = raw_h - axes.normal() * raw_h.dot(axes.normal());
        let heading = if h_ortho.length_squared() > 1e-6 { h_ortho.normalize() } else { target.heading };
        let spin = if (target.wheel_motion.0 - self.wheel_motion.0).abs() <= 24.0 {
            self.wheel_motion.0 + (target.wheel_motion.0 - self.wheel_motion.0) * t
        } else {
            target.wheel_motion.0
        };
        let steer = if target.wheel_motion.1 >= 3.5 || self.wheel_motion.1 >= 3.5 {
            target.wheel_motion.1
        } else {
            self.wheel_motion.1 + (target.wheel_motion.1 - self.wheel_motion.1) * t
        };
        Self::new(origin, axes, heading, (spin, steer))
    }

    pub fn inscribe(&self, out: &mut Vec<u8>) {
        WireVector::new(self.origin).inscribe(out);
        self.axes.inscribe(out);
        WireVector::new(self.heading).inscribe(out);
        out.extend_from_slice(&self.wheel_motion.0.to_le_bytes());
        out.extend_from_slice(&self.wheel_motion.1.to_le_bytes());
    }

    pub fn decode(bytes: &[u8]) -> Option<(Self, usize)> {
        if bytes.len() < 68 {
            return None;
        }
        let origin = WireSlice::new(&bytes[0..12]).vec3()?;
        let (axes, _) = VehicleAxes::decode(&bytes[12..48])?;
        let heading = WireSlice::new(&bytes[48..60]).vec3()?;
        if heading.length_squared() < 0.25 {
            return None;
        }
        let spin = f32::from_le_bytes(bytes[60..64].try_into().ok()?);
        let steer = f32::from_le_bytes(bytes[64..68].try_into().ok()?);
        if !spin.is_finite() || !steer.is_finite() {
            return None;
        }
        Some((Self::new(origin, axes, heading.normalize(), (spin, steer)), 68))
    }
}

/// Polymorphic vehicle crew occupancy state avoiding `Option<u32>`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SeatOccupancy {
    Vacant,
    Helmed { pilot_slot: u32 },
    Crewed { pilot_slot: u32, guests: [u32; 3] },
}

impl SeatOccupancy {
    pub const fn helmed_by(self, slot: u32) -> bool {
        match self {
            Self::Vacant => false,
            Self::Helmed { pilot_slot } | Self::Crewed { pilot_slot, .. } => pilot_slot == slot,
        }
    }

    pub const fn vacant(self) -> bool {
        matches!(self, Self::Vacant)
    }

    pub const fn try_claim_helm(self, slot: u32) -> Option<Self> {
        match self {
            Self::Vacant => Some(Self::Helmed { pilot_slot: slot }),
            Self::Helmed { pilot_slot } if pilot_slot == slot => Some(self),
            Self::Crewed { pilot_slot, .. } if pilot_slot == slot => Some(self),
            Self::Helmed { .. } | Self::Crewed { .. } => None,
        }
    }
}
