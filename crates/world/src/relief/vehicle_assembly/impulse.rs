use glam::{Quat, Vec3};

const NOZZLE_THRUST: f32 = 72.0;
const ATTITUDE_THRUST: f32 = 34.0;
const ATTITUDE_STEER_RATE: f32 = 0.95;
const MAX_STEER_RATE: f32 = 2.20;
const MAX_THRUST_ACCEL: f32 = 32.0;
const LEVER_TORQUE_SCALE: f32 = 0.035;
const CONSOLE_GYRO_AUTHORITY: f32 = 1.65;

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
    pub fn throttle_along(&self, world_thrust: Vec3, right: Vec3, up: Vec3, fwd: Vec3) -> f32 {
        let cmd = right * self.burn_axes.x + up * self.burn_axes.y + fwd * self.burn_axes.z;
        world_thrust.normalize_or_zero().dot(cmd).clamp(0.0, 1.0)
    }

    /// Evaluates deliberate RCS attitude angular rate (`rad/s`) pitching on `W`/`S` and yawing on `A`/`D`.
    pub fn attitude_rate(&self, right: Vec3, up: Vec3) -> Vec3 {
        (-right * self.burn_axes.z - up * self.burn_axes.x) * ATTITUDE_STEER_RATE
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

/// Rotational impulse state combining off-center nozzle lever torque, RCS steering rate, and console gyro authority.
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

    fn resolve_step(&self, dt: f32) -> Quat {
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

/// Accumulated 3D linear force, rotational attitude impulse, and structural block mass.
#[derive(Clone, Copy, Debug)]
pub struct RigidImpulse {
    linear_force: Vec3,
    attitude: AttitudeImpulse,
    mass_blocks: f32,
}

impl RigidImpulse {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(linear_force: Vec3, attitude: AttitudeImpulse, mass_blocks: f32) -> Self {
        Self { linear_force, attitude, mass_blocks }
    }

    pub const fn zero() -> Self {
        Self::new(Vec3::ZERO, AttitudeImpulse::zero(), 0.0)
    }

    pub fn add_block_mass(&mut self, units: f32) {
        self.mass_blocks += units;
    }

    pub fn add_console_gyro(&mut self) {
        self.attitude.gyro_authority += CONSOLE_GYRO_AUTHORITY;
    }

    pub fn apply_nozzle_burn(&mut self, lever_arm: Vec3, thrust_dir: Vec3, throttle: f32) {
        if throttle <= 0.01 {
            return;
        }
        let force = thrust_dir * (NOZZLE_THRUST * throttle);
        self.linear_force += force;
        self.attitude.lever_torque += lever_arm.cross(force) * LEVER_TORQUE_SCALE;
    }

    pub fn apply_attitude_jet(&mut self, thrust_dir: Vec3, throttle: f32) {
        if throttle > 0.01 {
            self.linear_force += thrust_dir * (ATTITUDE_THRUST * throttle);
        }
    }

    pub fn apply_attitude_steer(&mut self, angular_rate: Vec3) {
        self.attitude.control_rate += angular_rate;
    }

    /// World-space linear acceleration (`F / m`, capped to `MAX_THRUST_ACCEL`) over `dt`.
    pub fn velocity_delta(&self, dt: f32) -> Vec3 {
        let m = self.mass_blocks.max(3.0);
        let raw_accel = self.linear_force / m;
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
