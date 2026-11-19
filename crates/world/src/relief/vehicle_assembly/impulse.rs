use glam::{Quat, Vec3};

const NOZZLE_THRUST: f32 = 72.0;
const LEVER_TORQUE_SCALE: f32 = 0.035;
const CONSOLE_GYRO_AUTHORITY: f32 = 1.65;

/// Cockpit flight command encapsulating 6-DOF thruster channel demand and attitude steer rates.
#[derive(Clone, Copy, Debug)]
pub struct PilotOrder {
    burn_axes: Vec3,
    attitude_rate: (f32, f32),
    roll_rate: f32,
}

impl PilotOrder {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(burn_axes: Vec3, attitude_rate: (f32, f32), roll_rate: f32) -> Self {
        Self { burn_axes, attitude_rate, roll_rate }
    }

    pub const fn idle() -> Self {
        Self::new(Vec3::ZERO, (0.0, 0.0), 0.0)
    }

    /// Evaluates `[0.0..=1.0]` throttle demand for a nozzle whose reaction thrust points along `world_thrust`.
    pub fn throttle_along(&self, world_thrust: Vec3, right: Vec3, up: Vec3, fwd: Vec3) -> f32 {
        let cmd = right * self.burn_axes.x + up * self.burn_axes.y + fwd * self.burn_axes.z;
        world_thrust.normalize_or_zero().dot(cmd).clamp(0.0, 1.0)
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

    pub fn pilot_angular_velocity(&self, right: Vec3, up: Vec3, fwd: Vec3, gain: f32) -> Vec3 {
        let (pitch_in, yaw_in) = self.attitude_rate;
        right * (pitch_in * gain) + up * (yaw_in * gain) + fwd * (self.roll_rate * gain)
    }
}

/// Accumulated 3D force, off-center nozzle lever torque, console gyro authority, and structural block mass.
#[derive(Clone, Copy, Debug)]
pub struct RigidImpulse {
    linear_force: Vec3,
    lever_torque: Vec3,
    gyro_authority: f32,
    mass_blocks: f32,
}

impl RigidImpulse {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(
        linear_force: Vec3,
        lever_torque: Vec3,
        gyro_authority: f32,
        mass_blocks: f32,
    ) -> Self {
        Self { linear_force, lever_torque, gyro_authority, mass_blocks }
    }

    pub const fn zero() -> Self {
        Self::new(Vec3::ZERO, Vec3::ZERO, 0.0, 0.0)
    }

    pub fn add_block_mass(&mut self, units: f32) {
        self.mass_blocks += units;
    }

    pub fn add_console_gyro(&mut self) {
        self.gyro_authority += CONSOLE_GYRO_AUTHORITY;
    }

    pub fn apply_nozzle_burn(&mut self, lever_arm: Vec3, thrust_dir: Vec3, throttle: f32) {
        if throttle <= 0.01 {
            return;
        }
        let force = thrust_dir * (NOZZLE_THRUST * throttle);
        self.linear_force += force;
        self.lever_torque += lever_arm.cross(force) * LEVER_TORQUE_SCALE;
    }

    /// World-space linear acceleration (`F / m`) over `dt`.
    pub fn velocity_delta(&self, dt: f32) -> Vec3 {
        let m = self.mass_blocks.max(3.0);
        (self.linear_force / m) * dt
    }

    /// Resolves net angular rotation over `dt` after reaction-wheel gyro trim saturation and pilot attitude input.
    pub fn attitude_step(
        &self,
        order: &PilotOrder,
        right: Vec3,
        up: Vec3,
        fwd: Vec3,
        dt: f32,
    ) -> Quat {
        let raw_mag = self.lever_torque.length();
        let uncompensated = if raw_mag > self.gyro_authority && raw_mag > 1e-5 {
            self.lever_torque * ((raw_mag - self.gyro_authority) / raw_mag)
        } else {
            Vec3::ZERO
        };
        let gyro_gain = (self.gyro_authority / CONSOLE_GYRO_AUTHORITY).clamp(0.5, 2.5);
        let pilot_omega = order.pilot_angular_velocity(right, up, fwd, gyro_gain);
        let total_omega = uncompensated + pilot_omega;
        let angle = total_omega.length() * dt;
        if angle > 1e-6 {
            Quat::from_axis_angle(total_omega.normalize(), angle)
        } else {
            Quat::IDENTITY
        }
    }
}
