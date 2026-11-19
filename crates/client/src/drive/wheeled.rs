use glam::Vec3;
use world::{PlanetaryWorld, VehicleAssembly};
use acoustics::{AcousticWave, EngineRumble};
use crate::controls::KeyboardControls;
use super::track::WheelTrack;
use super::{CraftDrive, PedalInput, SEAT_EYE_SUPPORT};

const MAX_DRIVE_SPEED: f32 = 14.0;
const MAX_REVERSE_SPEED: f32 = 8.0;
const THROTTLE_ACCEL: f32 = 16.0;
const REVERSE_ACCEL: f32 = 11.0;
const BRAKE_DECEL: f32 = 32.0;
const COAST_DECEL: f32 = 3.5;
const STEER_RATE: f32 = 2.2;
const REVERSE_ENGAGE_DWELL: f32 = 0.22;

/// Immutable longitudinal speed, reverse-shift dwell, wheel spin angle, and steering wheel turn state.
#[derive(Clone, Copy, Debug)]
pub struct DriveKinematics {
    speed: f32,
    reverse_dwell: f32,
    wheel_spin: f32,
    steer_turn: f32,
}

impl DriveKinematics {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(speed: f32, reverse_dwell: f32, wheel_spin: f32, steer_turn: f32) -> Self {
        Self { speed, reverse_dwell, wheel_spin, steer_turn }
    }

    pub const fn parked() -> Self { Self::new(0.0, 0.0, 0.0, 0.0) }
    pub const fn wheel_spin(self) -> f32 { self.wheel_spin }
    pub const fn steer_turn(self) -> f32 { self.steer_turn }
    fn displacement(self, dt: f32) -> f32 { self.speed * dt }
    const fn arrested(self) -> Self { Self::new(0.0, self.reverse_dwell, self.wheel_spin, self.steer_turn) }

    fn yaw_step(self, sign: f32, dt: f32) -> f32 {
        if sign.abs() > 0.1 && self.speed.abs() > 0.05 {
            sign * (if self.speed < -0.05 { -1.0 } else { 1.0 }) * STEER_RATE * dt
        } else {
            0.0
        }
    }

    fn advanced(self, pedals: &PedalInput, wheel_radius: f32, dt: f32) -> (Self, f32) {
        let (throttle, steer, handbrake) = pedals.transfer(|t, s, b| (t, s, b));
        let (mut speed, mut dwell) = (self.speed, self.reverse_dwell);
        if handbrake {
            dwell = 0.0;
            speed = if speed > 0.0 { (speed - BRAKE_DECEL * dt).max(0.0) } else { (speed + BRAKE_DECEL * dt).min(0.0) };
        } else if throttle > 0.1 {
            dwell = 0.0;
            speed = if speed < 0.0 { (speed + BRAKE_DECEL * dt).min(0.0) } else { (speed + THROTTLE_ACCEL * dt).min(MAX_DRIVE_SPEED) };
        } else if throttle < -0.1 {
            if speed > 0.0 {
                speed = (speed - BRAKE_DECEL * dt).max(0.0);
                dwell = 0.0;
            } else {
                dwell += if speed == 0.0 { dt } else { 0.0 };
                if speed < 0.0 || dwell >= REVERSE_ENGAGE_DWELL {
                    speed = (speed - REVERSE_ACCEL * dt).max(-MAX_REVERSE_SPEED);
                }
            }
        } else {
            dwell = 0.0;
            speed = if speed > 0.0 { (speed - COAST_DECEL * dt).max(0.0) } else { (speed + COAST_DECEL * dt).min(0.0) };
        }
        let target_steer = if steer.abs() > 0.1 { steer.signum() } else { 0.0 };
        let next_turn = self.steer_turn + (target_steer - self.steer_turn) * (14.0 * dt).clamp(0.0, 1.0);
        let next_spin = self.wheel_spin + (speed * dt) / wheel_radius.max(0.12);
        if speed.abs() > 0.25 && (self.wheel_spin * 0.06).floor() != (next_spin * 0.06).floor() {
            EngineRumble::combustion().thump();
        }
        (Self::new(speed, dwell, next_spin, next_turn), target_steer)
    }
}

/// Surface-bound wheeled propulsion composing `DriveKinematics` and the `RoadProbe` seam-folding chain.
pub struct WheeledDrive {
    kinematics: DriveKinematics,
}

impl WheeledDrive {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(kinematics: DriveKinematics) -> Self {
        Self { kinematics }
    }

    pub const fn parked() -> Self {
        Self::new(DriveKinematics::parked())
    }
}

impl CraftDrive for WheeledDrive {
    fn motion_channels(&self) -> (f32, f32) {
        (self.kinematics.wheel_spin(), self.kinematics.steer_turn())
    }

    fn free_look(&self) -> bool {
        false
    }

    fn advance(
        &mut self,
        assembly: &mut VehicleAssembly,
        dt: f32,
        controls: &dyn KeyboardControls,
        world: &dyn PlanetaryWorld,
    ) -> (Vec3, Vec3, f32) {
        let mut pedals = PedalInput::idle();
        controls.drive_locomotion(Vec3::Y, Vec3::X, &mut pedals);
        let (next_kin, sign) = self.kinematics.advanced(&pedals, assembly.wheel_radius(), dt);
        self.kinematics = next_kin;
        let yaw_delta = self.kinematics.yaw_step(sign, dt);
        if yaw_delta.abs() > 0.0 {
            assembly.steer(-yaw_delta);
        }
        let disp = self.kinematics.displacement(dt);
        let (next_ground, next_up, blocked) = WheelTrack::from_assembly(assembly).advance(disp, dt, world);
        if blocked {
            self.kinematics = self.kinematics.arrested();
        }
        assembly.relocate(next_ground, next_up);
        let seat_pos = next_ground + next_up * (SEAT_EYE_SUPPORT + 0.12 + assembly.seat_floor_offset());
        (seat_pos, next_up, yaw_delta / 0.003)
    }
}
