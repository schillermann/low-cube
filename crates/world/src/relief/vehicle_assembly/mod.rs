mod part;

use glam::{Quat, Vec3};
use crate::frame::SurfaceFrame;
use crate::MeshTarget;
use super::craft_cluster::CraftCluster;
use super::natural_strata::NaturalStrata;
use super::shaft_excavation::ShaftExcavation;
pub use self::part::{VehicleAxes, VehiclePart};
use self::part::ShiftedMeshTarget;

/// Drivable multi-block vehicle assembly centered on its cabin seat.
pub struct VehicleAssembly {
    origin: Vec3,
    axes: VehicleAxes,
    heading: Vec3,
    parts: Vec<VehiclePart>,
}

impl VehicleAssembly {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(origin: Vec3, axes: VehicleAxes, heading: Vec3, parts: Vec<VehiclePart>) -> Self {
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
        let seat_off = self.seat_floor_offset();
        Self::new(self.origin + self.axes.normal() * seat_off, self.axes, self.heading, Vec::new())
    }

    pub fn wheel_radius(&self) -> f32 {
        self.parts.iter().filter_map(VehiclePart::wheel_radius).fold(0.0f32, f32::max).max(0.16)
    }

    fn wheel_ground_lift(&self) -> f32 {
        self.parts.iter().filter_map(VehiclePart::wheel_floor_drop).fold(0.0f32, f32::max)
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
        let mut shifted = ShiftedMeshTarget::new(shift, target);
        let (top_col, wall_col) = (Vec3::new(0.45, 0.30, 0.17), Vec3::new(0.41, 0.27, 0.15));
        let lift = self.wheel_ground_lift();
        for part in &self.parts {
            part.emit_driven(lift, wheel_spin, steer_turn, &car_frame, top_col, wall_col, &mut shifted);
        }
    }

    pub(super) fn park_into_cluster(
        self,
        strata: &NaturalStrata,
        shafts: &ShaftExcavation,
        frame: &SurfaceFrame,
    ) -> (CraftCluster, (i32, i32)) {
        let lift = self.wheel_ground_lift();
        let seat = frame.cell_at(self.origin).feed_coordinates(|cu, cv| (cu, cv));
        let turn = self.axes.cardinal_turn(frame);
        let mut craft = CraftCluster::untouched();
        for part in self.parts {
            part.park_into(seat, turn, lift, &mut craft, strata, shafts);
        }
        (craft, seat)
    }
}
