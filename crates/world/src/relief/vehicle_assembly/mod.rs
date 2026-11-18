mod axes;
mod part;

use glam::Vec3;
use crate::frame::SurfaceFrame;
use crate::MeshTarget;
use super::craft_cluster::CraftCluster;
use super::craft_entity::{CraftBlueprint, CraftIdentity, CraftPose};
use super::natural_strata::NaturalStrata;
use super::shaft_excavation::ShaftExcavation;
pub use self::axes::VehicleAxes;
pub use self::part::VehiclePart;

/// Drivable multi-block vehicle assembly uniting its immutable `CraftBlueprint` and live `CraftPose`.
#[derive(Clone, Debug, PartialEq)]
pub struct VehicleAssembly {
    blueprint: CraftBlueprint,
    pose: CraftPose,
}

impl VehicleAssembly {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn from_blueprint(blueprint: CraftBlueprint, pose: CraftPose) -> Self {
        Self { blueprint, pose }
    }

    /// Secondary constructor commissioning a vehicle assembly from components and initial orientation.
    pub fn new(origin: Vec3, axes: VehicleAxes, heading: Vec3, parts: Vec<VehiclePart>) -> Self {
        Self::from_blueprint(CraftBlueprint::commission(origin, parts), CraftPose::parked(origin, axes, heading))
    }

    /// Secondary constructor preserving an existing `CraftIdentity` when re-boarding a parked vehicle.
    pub fn with_identity(
        identity: CraftIdentity,
        origin: Vec3,
        axes: VehicleAxes,
        heading: Vec3,
        parts: Vec<VehiclePart>,
    ) -> Self {
        let bp = if identity.assigned() {
            CraftBlueprint::new(identity, parts)
        } else {
            CraftBlueprint::commission(origin, parts)
        };
        Self::from_blueprint(bp, CraftPose::parked(origin, axes, heading))
    }

    pub const fn identity(&self) -> CraftIdentity { self.blueprint.identity() }
    pub const fn blueprint(&self) -> &CraftBlueprint { &self.blueprint }
    pub const fn pose(&self) -> CraftPose { self.pose }
    pub const fn origin(&self) -> Vec3 { self.pose.origin() }
    pub const fn heading(&self) -> Vec3 { self.pose.heading() }
    pub const fn normal(&self) -> Vec3 { self.pose.normal() }
    pub fn drivable(&self) -> bool { self.blueprint.drivable() }
    pub fn seat_floor_offset(&self) -> f32 { self.blueprint.seat_floor_offset() }
    pub fn wheel_radius(&self) -> f32 { self.blueprint.wheel_radius() }

    pub fn perch_shell(&self) -> Self {
        let seat_off = self.seat_floor_offset();
        Self::new(
            self.pose.origin() + self.pose.normal() * seat_off,
            self.pose.axes(),
            self.pose.heading(),
            Vec::new(),
        )
    }

    pub fn steer(&mut self, delta_yaw: f32) {
        self.pose = self.pose.steered(delta_yaw);
    }

    pub fn relocate(&mut self, next_origin: Vec3, next_up: Vec3) {
        self.pose = self.pose.relocated(next_origin, next_up);
    }

    pub fn interpolated(&self, target: &Self, alpha: f32) -> Self {
        Self::from_blueprint(target.blueprint.clone(), self.pose.interpolated(target.pose, alpha))
    }

    pub fn emit(&self, target: &mut dyn MeshTarget) {
        self.blueprint.emit_at(self.pose.with_motion(0.0, 0.0), target);
    }

    pub fn emit_driven(&self, wheel_spin: f32, steer_turn: f32, target: &mut dyn MeshTarget) {
        self.blueprint.emit_at(self.pose.with_motion(wheel_spin, steer_turn), target);
    }

    pub fn emit_driven_at(&self, render_origin: Vec3, wheel_spin: f32, steer_turn: f32, target: &mut dyn MeshTarget) {
        self.blueprint.emit_at(
            self.pose.with_origin(render_origin).with_motion(wheel_spin, steer_turn),
            target,
        );
    }

    pub fn inscribe(&self, out: &mut Vec<u8>) {
        self.blueprint.inscribe(out);
        self.pose.inscribe(out);
    }

    pub fn decode(bytes: &[u8]) -> Option<(Self, usize)> {
        let (blueprint, bp_used) = CraftBlueprint::decode(bytes)?;
        let (pose, pose_used) = CraftPose::decode(&bytes[bp_used..])?;
        Some((Self::from_blueprint(blueprint, pose), bp_used + pose_used))
    }

    pub(super) fn park_into_cluster(
        self,
        strata: &NaturalStrata,
        shafts: &ShaftExcavation,
        frame: &SurfaceFrame,
    ) -> (CraftIdentity, CraftCluster, (i32, i32)) {
        let id = self.blueprint.identity();
        let lift = self.blueprint.wheel_ground_lift();
        let raw_seat = frame.cell_at(self.pose.origin()).feed_coordinates(|cu, cv| (cu, cv));
        let basis = self.pose.axes().grid_basis(frame);
        let half_cells = (frame.height_of(frame.point_at(0.0, 0.0, 0.0)) * 2.0).round() as i32;
        let seat = self.blueprint.clamped_seat(raw_seat, basis, half_cells);
        let mut craft = CraftCluster::untouched();
        for part in self.blueprint.into_parts() {
            part.park_into(seat, basis, lift, &mut craft, strata, shafts);
        }
        (id, craft, seat)
    }
}
