mod order;
mod stages;

use glam::{Quat, Vec3};
use crate::frame::SurfaceFrame;
use super::super::craft_entity::CraftPose;
pub use self::order::PilotOrder;
pub use self::stages::{
    ConsoleGyroStage, FusionTorchStage, HullBallast, ImpulseStage, MainBellStage,
    VernierAttitudeStage,
};

pub(super) const NOZZLE_THRUST: f32 = 72.0;
pub(super) const FUSION_TORCH_THRUST: f32 = 360.0;
pub(super) const RCS_HORN_THRUST: f32 = 30.0;
pub(super) const BELL_TORQUE_SCALE: f32 = 0.12;
pub(super) const RCS_TORQUE_SCALE: f32 = 0.95;
pub(super) const COMMAND_STEER_RATE: f32 = 1.08;
pub(super) const SAS_GAIN: f32 = 2.40;
const CONSOLE_GYRO_TRIM: f32 = 6.20;
const CONSOLE_STEER_TORQUE: f32 = 4.50;
const MAX_ENGAGED_RATE: f32 = 1.65;
const MAX_MANUAL_RATE: f32 = 3.20;
const MAX_THRUST_ACCEL: f32 = 32.0;
const MAX_FUSION_ACCEL: f32 = 78.5;
const MIN_CRAFT_MASS: f32 = 3.0;
const MIN_AXIS_INERTIA: f32 = 0.70;

/// Polymorphic spacecraft flight-guidance computer state (`Engaged` SAS + gyro vs. `Manual` raw vacuum inertia).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AttitudeGuidance {
    Engaged,
    Manual,
}

impl AttitudeGuidance {
    pub const fn engaged(self) -> bool {
        matches!(self, Self::Engaged)
    }

    pub const fn toggled(self) -> Self {
        match self {
            Self::Engaged => Self::Manual,
            Self::Manual => Self::Engaged,
        }
    }
}

/// Orthonormal ship-local `(right, up, forward)` basis resolved from a `CraftPose`.
#[derive(Clone, Copy, Debug)]
pub struct CockpitAxes {
    right: Vec3,
    up: Vec3,
    fwd: Vec3,
}

impl CockpitAxes {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(right: Vec3, up: Vec3, fwd: Vec3) -> Self {
        Self { right, up, fwd }
    }

    pub fn from_pose(pose: CraftPose) -> Self {
        let up = pose.normal().normalize_or_zero();
        let raw_fwd = pose.heading();
        let fwd = (raw_fwd - up * raw_fwd.dot(up)).normalize_or_zero();
        let right = fwd.cross(up).normalize_or_zero();
        Self::new(right, up, fwd)
    }

    pub(super) const fn up_axis(self) -> Vec3 {
        self.up
    }

    pub fn to_world(self, local: Vec3) -> Vec3 {
        self.right * local.x + self.up * local.y + self.fwd * local.z
    }

    pub fn to_local(self, world: Vec3) -> Vec3 {
        Vec3::new(world.dot(self.right), world.dot(self.up), world.dot(self.fwd))
    }
}

/// Encapsulated ship mass and principal-axis rotational inertia tensor in ship-frame coordinates.
#[derive(Clone, Copy, Debug)]
pub struct InertiaTensor {
    mass: f32,
    principal: Vec3,
    axes: (Vec3, Vec3, Vec3),
}

impl InertiaTensor {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(mass: f32, principal: Vec3, axes: (Vec3, Vec3, Vec3)) -> Self {
        Self { mass, principal, axes }
    }

    pub const fn zero() -> Self {
        Self::new(0.0, Vec3::ONE, (Vec3::X, Vec3::Y, Vec3::Z))
    }

    fn angular_accel(self, world_torque: Vec3) -> Vec3 {
        let (ax, ay, az) = self.axes;
        let ix = self.principal.x.max(MIN_AXIS_INERTIA);
        let iy = self.principal.y.max(MIN_AXIS_INERTIA);
        let iz = self.principal.z.max(MIN_AXIS_INERTIA);
        ax * (world_torque.dot(ax) / ix)
            + ay * (world_torque.dot(ay) / iy)
            + az * (world_torque.dot(az) / iz)
    }
}

/// Immutable 6-DOF rigid-body impulse uniting linear force, `r × F` lever torque, `InertiaTensor`, and gyro trim.
#[derive(Clone, Copy, Debug)]
pub struct RigidImpulse {
    linear_force: Vec3,
    torque: Vec3,
    tensor: InertiaTensor,
    gyro_trim: (f32, Vec3),
}

impl RigidImpulse {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(
        linear_force: Vec3,
        torque: Vec3,
        tensor: InertiaTensor,
        gyro_trim: (f32, Vec3),
    ) -> Self {
        Self { linear_force, torque, tensor, gyro_trim }
    }

    pub const fn zero() -> Self {
        Self::new(Vec3::ZERO, Vec3::ZERO, InertiaTensor::zero(), (0.0, Vec3::ZERO))
    }

    pub fn with_ballast(mass: f32, principal: Vec3, frame: SurfaceFrame) -> Self {
        let axes = (frame.tangent(), frame.normal(), frame.bitangent());
        Self::new(Vec3::ZERO, Vec3::ZERO, InertiaTensor::new(mass, principal, axes), (0.0, Vec3::ZERO))
    }

    pub fn with_gyro(self, steer_demand: Vec3) -> Self {
        Self::new(
            self.linear_force,
            self.torque,
            self.tensor,
            (self.gyro_trim.0 + CONSOLE_GYRO_TRIM, steer_demand * CONSOLE_STEER_TORQUE),
        )
    }

    pub fn with_bell_burn(self, lever_arm: Vec3, thrust_dir: Vec3, throttle: f32) -> Self {
        if throttle <= 0.01 {
            return self;
        }
        let force = thrust_dir * (NOZZLE_THRUST * throttle);
        let bell_torque = lever_arm.cross(force) * BELL_TORQUE_SCALE;
        Self::new(self.linear_force + force, self.torque + bell_torque, self.tensor, self.gyro_trim)
    }

    pub fn with_fusion_burn(self, lever_arm: Vec3, thrust_dir: Vec3, throttle: f32) -> Self {
        if throttle <= 0.01 {
            return self;
        }
        let force = thrust_dir * (FUSION_TORCH_THRUST * throttle);
        let bell_torque = lever_arm.cross(force) * (BELL_TORQUE_SCALE * 0.5);
        Self::new(self.linear_force + force, self.torque + bell_torque, self.tensor, self.gyro_trim)
    }

    pub fn with_rcs_horn(self, lever_arm: Vec3, thrust_dir: Vec3, throttle: f32) -> Self {
        if throttle <= 0.01 {
            return self;
        }
        let force = thrust_dir * (RCS_HORN_THRUST * throttle);
        let horn_torque = lever_arm.cross(force) * RCS_TORQUE_SCALE;
        Self::new(self.linear_force + force, self.torque + horn_torque, self.tensor, self.gyro_trim)
    }

    pub fn lifting_along(&self, lift_dir: Vec3) -> bool {
        self.linear_force.dot(lift_dir) > 1.0
    }

    /// World-space linear acceleration (`F / m`, capped to `MAX_THRUST_ACCEL`) over `dt`.
    pub fn velocity_delta(&self, dt: f32) -> Vec3 {
        self.capped_velocity_delta(dt, MAX_THRUST_ACCEL)
    }

    /// Epstein fusion torch linear velocity delta (`F / m`, capped to `MAX_FUSION_ACCEL` ~8G) over `dt`.
    pub fn fusion_velocity_delta(&self, dt: f32) -> Vec3 {
        self.capped_velocity_delta(dt, MAX_FUSION_ACCEL)
    }

    fn capped_velocity_delta(&self, dt: f32, cap: f32) -> Vec3 {
        let raw_accel = self.linear_force / self.tensor.mass.max(MIN_CRAFT_MASS);
        let accel = if raw_accel.length_squared() > cap * cap {
            raw_accel.normalize() * cap
        } else {
            raw_accel
        };
        accel * dt
    }

    /// Integrates principal-axis rotational acceleration (`I⁻¹ τ`) into `angular_velocity` and step `Quat`.
    pub fn integrate_attitude(&self, omega: Vec3, guidance: AttitudeGuidance, dt: f32) -> (Vec3, Quat) {
        let (trim_cap, gyro_steer) = self.gyro_trim;
        let net_torque = if guidance.engaged() && trim_cap > 0.0 {
            let steer_dir = gyro_steer.normalize_or_zero();
            let along = if steer_dir.length_squared() > 0.5 {
                self.torque.dot(steer_dir).max(0.0) * steer_dir
            } else {
                Vec3::ZERO
            };
            let residual = self.torque - along;
            let res_mag = residual.length();
            let uncompensated = if res_mag > trim_cap && res_mag > 1e-5 {
                residual * ((res_mag - trim_cap) / res_mag)
            } else {
                Vec3::ZERO
            };
            along + uncompensated + gyro_steer
        } else {
            self.torque
        };
        let base_omega = if guidance.engaged() && trim_cap > 0.0 && gyro_steer.length_squared() < 1e-4 {
            let damped = omega * (1.0 - 11.0 * dt).clamp(0.0, 1.0);
            if damped.length_squared() < 1e-4 { Vec3::ZERO } else { damped }
        } else {
            omega
        };
        let mut next_omega = base_omega + self.tensor.angular_accel(net_torque) * dt;
        let rate_cap = if guidance.engaged() { MAX_ENGAGED_RATE } else { MAX_MANUAL_RATE };
        let speed = next_omega.length();
        if speed > rate_cap {
            next_omega *= rate_cap / speed;
        }
        let angle = next_omega.length() * dt;
        let step_rot = if angle > 1e-6 {
            Quat::from_axis_angle(next_omega.normalize(), angle)
        } else {
            Quat::IDENTITY
        };
        (next_omega, step_rot)
    }

    /// Resolves single-step attitude rotation from rest under engaged guidance.
    pub fn attitude_step(&self, dt: f32) -> Quat {
        self.integrate_attitude(Vec3::ZERO, AttitudeGuidance::Engaged, dt).1
    }
}
