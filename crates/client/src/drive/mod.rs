mod passenger;
mod pilot;
mod road;
mod track;
#[cfg(test)]
mod multiplayer_tests;
#[cfg(test)]
mod seam_tests;
#[cfg(test)]
mod tests;

use glam::{Mat3, Quat, Vec3};
use world::{ClimateZone, MeshTarget, PlanetaryWorld, VehicleAssembly};
use acoustics::{AcousticWave, EngineRumble};
use crate::controls::{KeyboardControls, LocomotionTarget};
pub use self::pilot::VehiclePilot;
use self::track::{FaceContact, WheelTrack};

const MAX_DRIVE_SPEED: f32 = 14.0;
const MAX_REVERSE_SPEED: f32 = 8.0;
const THROTTLE_ACCEL: f32 = 16.0;
const REVERSE_ACCEL: f32 = 11.0;
const BRAKE_DECEL: f32 = 32.0;
const COAST_DECEL: f32 = 3.5;
const STEER_RATE: f32 = 2.2;
pub(super) const SEAT_EYE_SUPPORT: f32 = 1.0;
const REVERSE_ENGAGE_DWELL: f32 = 0.22;

/// Encapsulated cockpit pedal and steering command captured from `KeyboardControls`.
pub(super) struct PedalInput {
    throttle: f32,
    steer: f32,
    handbrake: bool,
}

impl PedalInput {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub(super) const fn new(throttle: f32, steer: f32, handbrake: bool) -> Self {
        Self { throttle, steer, handbrake }
    }

    pub(super) const fn idle() -> Self {
        Self::new(0.0, 0.0, false)
    }

    pub(super) fn dismount_requested(&self) -> bool {
        self.throttle.abs() > 0.1 || self.steer.abs() > 0.1
    }

    pub(super) fn dismount_vector(&self, fwd: Vec3, right: Vec3) -> Vec3 {
        (fwd * self.throttle + right * self.steer).normalize_or_zero()
    }
}

impl LocomotionTarget for PedalInput {
    fn travel(&mut self, direction: Vec3) {
        (self.throttle, self.steer) = (direction.y, direction.x);
    }
    fn jump(&mut self) { self.handbrake = true; }
    fn sneak(&mut self) { self.handbrake = true; }
    fn sprint(&mut self) {}
}

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
        let (mut speed, mut dwell) = (self.speed, self.reverse_dwell);
        if pedals.handbrake {
            dwell = 0.0;
            speed = if speed > 0.0 { (speed - BRAKE_DECEL * dt).max(0.0) } else { (speed + BRAKE_DECEL * dt).min(0.0) };
        } else if pedals.throttle > 0.1 {
            dwell = 0.0;
            speed = if speed < 0.0 { (speed + BRAKE_DECEL * dt).min(0.0) } else { (speed + THROTTLE_ACCEL * dt).min(MAX_DRIVE_SPEED) };
        } else if pedals.throttle < -0.1 {
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
        let target_steer = if pedals.steer.abs() > 0.1 { pedals.steer.signum() } else { 0.0 };
        let next_turn = self.steer_turn + (target_steer - self.steer_turn) * (14.0 * dt).clamp(0.0, 1.0);
        let next_spin = self.wheel_spin + (speed * dt) / wheel_radius.max(0.12);
        if speed.abs() > 0.25 && (self.wheel_spin * 0.06).floor() != (next_spin * 0.06).floor() {
            EngineRumble::combustion().thump();
        }
        (Self::new(speed, dwell, next_spin, next_turn), target_steer)
    }
}

/// Active driven vehicle state encapsulating the multi-block car assembly, kinematics, and sub-tick origin span.
pub struct DrivenCar {
    assembly: VehicleAssembly,
    kinematics: DriveKinematics,
    origin_span: (Vec3, f32),
}

impl DrivenCar {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(assembly: VehicleAssembly, kinematics: DriveKinematics, origin_span: (Vec3, f32)) -> Self {
        Self { assembly, kinematics, origin_span }
    }

    pub const fn boarded(assembly: VehicleAssembly) -> Self {
        let origin = assembly.origin();
        Self::new(assembly, DriveKinematics::parked(), (origin, 1.0))
    }
    pub const fn identity(&self) -> world::CraftIdentity { self.assembly.identity() }
    pub const fn steer_turn(&self) -> f32 { self.kinematics.steer_turn() }
    pub fn chassis_up(&self) -> Vec3 { self.assembly.normal().normalize_or_zero() }
    pub fn blend_subtick(&mut self, alpha: f32) { self.origin_span.1 = alpha.clamp(0.0, 1.0); }

    pub const fn pilot_craft(&self) -> world::PilotCraft {
        world::PilotCraft::Driving(world::DrivenCraft::new(
            0,
            self.assembly.identity(),
            self.assembly.pose().with_motion(self.kinematics.wheel_spin(), self.kinematics.steer_turn()),
        ))
    }

    pub fn release_wheel(self, world: &mut dyn PlanetaryWorld) -> Self {
        let mut shell = self.assembly.perch_shell();
        let raw_origin = self.assembly.origin();
        let _ = world.park_vehicle(self.assembly);
        let parked_origin = FaceContact::new(raw_origin).parked_seat_origin(world);
        let face_up = ClimateZone::dominant(parked_origin).normal();
        let base_ground = FaceContact::new(parked_origin).ground(0.0, world) - face_up * 0.5;
        shell.relocate(base_ground, world.surface_up_at(parked_origin));
        Self::boarded(shell)
    }

    pub fn body_rotation(&self) -> Quat {
        let up = self.assembly.normal().normalize_or_zero();
        let raw_fwd = self.assembly.heading();
        let fwd = (raw_fwd - up * raw_fwd.dot(up)).normalize_or_zero();
        let right = fwd.cross(up).normalize_or_zero();
        Quat::from_mat3(&Mat3::from_cols(right, up, -fwd)).normalize()
    }

    pub fn perch_position(&self) -> Vec3 {
        self.assembly.origin() + self.assembly.normal().normalize_or_zero() * (SEAT_EYE_SUPPORT + 0.12)
    }

    pub fn seat_intact(&self, world: &dyn PlanetaryWorld) -> bool {
        FaceContact::new(self.assembly.origin()).seat_intact(world)
    }

    pub fn seat_position(&self, world: &dyn PlanetaryWorld) -> Vec3 {
        let origin = self.assembly.origin();
        let up = world.surface_up_at(origin);
        FaceContact::new(origin).ground(SEAT_EYE_SUPPORT, world) + up * (0.12 + self.assembly.seat_floor_offset())
    }

    pub(super) fn dismount_with(&self, pedals: &PedalInput, world: &dyn PlanetaryWorld) -> Vec3 {
        let origin = self.assembly.origin();
        let face_up = ClimateZone::dominant(origin).normal();
        let fwd = (self.assembly.heading() - face_up * self.assembly.heading().dot(face_up)).normalize_or_zero();
        let right = fwd.cross(face_up).normalize_or_zero();
        let chosen = pedals.dismount_vector(fwd, right);
        let primary = if chosen.length_squared() > 0.01 { chosen } else { fwd };
        let side = primary.cross(face_up).normalize_or_zero();
        let seat_elev = FaceContact::new(origin).ground(1.0, world).dot(face_up);
        for cand in [
            origin + primary * 0.60, origin + primary * 1.15, origin + primary * 1.65,
            origin + side * 0.60, origin - side * 0.60, origin - primary * 0.60,
            origin + side * 1.15, origin - side * 1.15, origin - primary * 1.15,
        ] {
            let grounded = FaceContact::new(cand).ground(1.0, world);
            if grounded.dot(face_up) < seat_elev - 0.15 {
                return grounded;
            }
        }
        FaceContact::new(origin + primary * 0.60).ground(1.0, world)
    }

    pub fn step(&mut self, dt: f32, controls: &dyn KeyboardControls, world: &dyn PlanetaryWorld) -> (Vec3, Vec3, f32) {
        let mut pedals = PedalInput::idle();
        controls.drive_locomotion(Vec3::Y, Vec3::X, &mut pedals);
        let (next_kin, sign) = self.kinematics.advanced(&pedals, self.assembly.wheel_radius(), dt);
        self.kinematics = next_kin;
        let yaw_delta = self.kinematics.yaw_step(sign, dt);
        if yaw_delta.abs() > 0.0 {
            self.assembly.steer(-yaw_delta);
        }
        let prev_origin = self.assembly.origin();
        let disp = self.kinematics.displacement(dt);
        let (next_ground, next_up, blocked) = WheelTrack::from_assembly(&self.assembly).advance(disp, dt, world);
        if blocked {
            self.kinematics = self.kinematics.arrested();
        }
        self.assembly.relocate(next_ground, next_up);
        self.origin_span = (prev_origin, 1.0);
        let seat_pos = next_ground + next_up * (SEAT_EYE_SUPPORT + 0.12 + self.assembly.seat_floor_offset());
        (seat_pos, next_up, yaw_delta / 0.003)
    }

    pub fn emit(&self, target: &mut dyn MeshTarget) {
        let (prev, alpha) = self.origin_span;
        let cur = self.assembly.origin();
        let visual_origin = if (cur - prev).length_squared() > 64.0 { cur } else { prev.lerp(cur, alpha) };
        self.assembly.emit_driven_at(
            visual_origin,
            self.kinematics.wheel_spin(),
            self.kinematics.steer_turn(),
            target,
        );
    }
}
