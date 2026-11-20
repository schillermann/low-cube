mod stages;

use glam::{Quat, Vec3};
use super::super::craft_entity::CraftPose;
pub use self::stages::{
    ConsoleGyroStage, HullBallast, ImpulseStage, MainBellStage, VernierAttitudeStage,
};

pub(super) const NOZZLE_THRUST: f32 = 72.0;
const ATTITUDE_THRUST: f32 = 34.0;
const ATTITUDE_STEER_RATE: f32 = 0.95;
const MAX_STEER_RATE: f32 = 2.20;
const MAX_THRUST_ACCEL: f32 = 32.0;
const MIN_CRAFT_MASS: f32 = 3.0;
const LEVER_TORQUE_SCALE: f32 = 0.035;
const CONSOLE_GYRO_AUTHORITY: f32 = 1.65;

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

    pub(super) fn command_vector(self, burn_axes: Vec3) -> Vec3 {
        self.right * burn_axes.x + self.up * burn_axes.y + self.fwd * burn_axes.z
    }

    pub(super) fn steer_omega(self, burn_axes: Vec3) -> Vec3 {
        (-self.right * burn_axes.z - self.up * burn_axes.x) * ATTITUDE_STEER_RATE
    }
}

/// Cockpit flight command encapsulating 6-DOF thruster channel demand.
#[derive(Clone, Copy, Debug)]
pub struct PilotOrder {
    burn_axes: Vec3,
}

impl PilotOrder {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(burn_axes: Vec3) -> Self {
        Self { burn_axes }
    }

    pub const fn idle() -> Self {
        Self::new(Vec3::ZERO)
    }

    /// Evaluates `[0.0..=1.0]` throttle demand for a nozzle whose reaction thrust points along `world_thrust`.
    pub fn throttle_along(&self, world_thrust: Vec3, axes: CockpitAxes) -> f32 {
        world_thrust
            .normalize_or_zero()
            .dot(axes.command_vector(self.burn_axes))
            .clamp(0.0, 1.0)
    }

    /// Evaluates deliberate RCS attitude angular rate (`rad/s`) pitching on `W`/`S` and yawing on `A`/`D`.
    pub fn attitude_rate(&self, axes: CockpitAxes) -> Vec3 {
        axes.steer_omega(self.burn_axes)
    }

    /// Visual `(vertical_plume, planar_plume)` channels stored in `CraftPose` motion state.
    pub fn plume_channels(&self) -> (f32, f32) {
        let planar = if self.burn_axes.z.abs() >= self.burn_axes.x.abs() {
            self.burn_axes.z
        } else {
            self.burn_axes.x
        };
        (self.burn_axes.y, planar)
    }

    pub fn active_burn(&self) -> bool {
        self.burn_axes.length_squared() > 0.01
    }

    pub fn planar_active(&self) -> bool {
        self.burn_axes.x.abs() > 0.05 || self.burn_axes.z.abs() > 0.05
    }
}

/// Immutable rotational impulse combining off-center nozzle lever torque, RCS steering rate, and console gyro authority.
#[derive(Clone, Copy, Debug)]
pub struct AttitudeImpulse {
    lever_torque: Vec3,
    control_rate: Vec3,
    gyro_authority: f32,
}

impl AttitudeImpulse {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(lever_torque: Vec3, control_rate: Vec3, gyro_authority: f32) -> Self {
        Self { lever_torque, control_rate, gyro_authority }
    }

    pub const fn zero() -> Self {
        Self::new(Vec3::ZERO, Vec3::ZERO, 0.0)
    }

    fn with_gyro(self, authority: f32) -> Self {
        Self::new(self.lever_torque, self.control_rate, self.gyro_authority + authority)
    }

    fn with_lever_torque(self, delta: Vec3) -> Self {
        Self::new(self.lever_torque + delta, self.control_rate, self.gyro_authority)
    }

    fn with_control_rate(self, delta: Vec3) -> Self {
        Self::new(self.lever_torque, self.control_rate + delta, self.gyro_authority)
    }

    fn resolve_step(self, dt: f32) -> Quat {
        let raw_mag = self.lever_torque.length();
        let uncompensated = if raw_mag > self.gyro_authority && raw_mag > 1e-5 {
            self.lever_torque * ((raw_mag - self.gyro_authority) / raw_mag)
        } else {
            Vec3::ZERO
        };
        let ctrl_mag = self.control_rate.length();
        let steered = if ctrl_mag > MAX_STEER_RATE && ctrl_mag > 1e-5 {
            self.control_rate * (MAX_STEER_RATE / ctrl_mag)
        } else {
            self.control_rate
        };
        let net_omega = uncompensated + steered;
        let angle = net_omega.length() * dt;
        if angle > 1e-6 {
            Quat::from_axis_angle(net_omega.normalize(), angle)
        } else {
            Quat::IDENTITY
        }
    }
}

/// Immutable 3D linear force, rotational attitude impulse, and linear craft mass (`a = F / m`).
#[derive(Clone, Copy, Debug)]
pub struct RigidImpulse {
    linear_force: Vec3,
    attitude: AttitudeImpulse,
    mass: f32,
}

impl RigidImpulse {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(linear_force: Vec3, attitude: AttitudeImpulse, mass: f32) -> Self {
        Self { linear_force, attitude, mass }
    }

    pub const fn zero() -> Self {
        Self::new(Vec3::ZERO, AttitudeImpulse::zero(), 0.0)
    }

    pub const fn with_mass(mass: f32) -> Self {
        Self::new(Vec3::ZERO, AttitudeImpulse::zero(), mass)
    }

    pub fn with_gyro(self) -> Self {
        Self::new(self.linear_force, self.attitude.with_gyro(CONSOLE_GYRO_AUTHORITY), self.mass)
    }

    pub fn with_bell_burn(self, lever_arm: Vec3, thrust_dir: Vec3, throttle: f32) -> Self {
        if throttle <= 0.01 {
            return self;
        }
        let force = thrust_dir * (NOZZLE_THRUST * throttle);
        let torque = lever_arm.cross(force) * LEVER_TORQUE_SCALE;
        Self::new(self.linear_force + force, self.attitude.with_lever_torque(torque), self.mass)
    }

    pub fn with_attitude_jet(self, thrust_dir: Vec3, throttle: f32) -> Self {
        if throttle <= 0.01 {
            return self;
        }
        Self::new(self.linear_force + thrust_dir * (ATTITUDE_THRUST * throttle), self.attitude, self.mass)
    }

    pub fn with_linear_boost(self, force: Vec3) -> Self {
        Self::new(self.linear_force + force, self.attitude, self.mass)
    }

    pub fn with_attitude_steer(self, angular_rate: Vec3) -> Self {
        Self::new(self.linear_force, self.attitude.with_control_rate(angular_rate), self.mass)
    }

    pub fn lifting_along(&self, lift_dir: Vec3) -> bool {
        self.linear_force.dot(lift_dir) > 1.0
    }

    /// World-space linear acceleration (`F / m`, strictly linear in mass and capped to `MAX_THRUST_ACCEL`) over `dt`.
    pub fn velocity_delta(&self, dt: f32) -> Vec3 {
        let raw_accel = self.linear_force / self.mass.max(MIN_CRAFT_MASS);
        let accel = if raw_accel.length_squared() > MAX_THRUST_ACCEL * MAX_THRUST_ACCEL {
            raw_accel.normalize() * MAX_THRUST_ACCEL
        } else {
            raw_accel
        };
        accel * dt
    }

    /// Resolves net angular rotation over `dt` combining gyro-trimmed lever torque and active Steuerdüse steering.
    pub fn attitude_step(&self, dt: f32) -> Quat {
        self.attitude.resolve_step(dt)
    }
}
