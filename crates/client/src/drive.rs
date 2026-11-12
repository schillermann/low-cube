use glam::{Mat3, Quat, Vec3};
use world::{MeshTarget, PlanetaryWorld, VehicleAssembly};
use crate::cardiac::{AcousticWave, EngineRumble};
use crate::controls::{KeyboardControls, LocomotionTarget};

const MAX_DRIVE_SPEED: f32 = 14.0;
const MAX_REVERSE_SPEED: f32 = 8.0;
const THROTTLE_ACCEL: f32 = 16.0;
const REVERSE_ACCEL: f32 = 11.0;
const BRAKE_DECEL: f32 = 32.0;
const COAST_DECEL: f32 = 3.5;
const STEER_RATE: f32 = 2.2;
const SEAT_EYE_SUPPORT: f32 = 1.0;
const REVERSE_ENGAGE_DWELL: f32 = 0.22;

struct PedalInput {
    throttle: f32,
    steer: f32,
    handbrake: bool,
}

impl PedalInput {
    const fn idle() -> Self { Self { throttle: 0.0, steer: 0.0, handbrake: false } }
}

impl LocomotionTarget for PedalInput {
    fn travel(&mut self, direction: Vec3) { (self.throttle, self.steer) = (direction.y, direction.x); }
    fn jump(&mut self) { self.handbrake = true; }
    fn sneak(&mut self) { self.handbrake = true; }
    fn sprint(&mut self) {}
}

/// Longitudinal speed, reverse-shift dwell, wheel spin angle, and steering wheel turn state.
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
    pub const fn steer_turn(self) -> f32 { self.steer_turn }
    fn displacement(self, dt: f32) -> f32 { self.speed * dt }
    fn emit_assembly(self, assembly: &VehicleAssembly, target: &mut dyn MeshTarget) {
        assembly.emit_driven(self.wheel_spin, self.steer_turn, target);
    }

    fn yaw_step(self, sign: f32, dt: f32) -> f32 {
        if sign.abs() > 0.1 && self.speed.abs() > 0.05 {
            sign * (if self.speed < -0.05 { -1.0 } else { 1.0 }) * STEER_RATE * dt
        } else {
            0.0
        }
    }

    fn advance(&mut self, pedals: &PedalInput, wheel_radius: f32, dt: f32) -> f32 {
        if pedals.handbrake {
            self.reverse_dwell = 0.0;
            self.speed = if self.speed > 0.0 { (self.speed - BRAKE_DECEL * dt).max(0.0) } else { (self.speed + BRAKE_DECEL * dt).min(0.0) };
        } else if pedals.throttle > 0.1 {
            self.reverse_dwell = 0.0;
            self.speed = if self.speed < 0.0 { (self.speed + BRAKE_DECEL * dt).min(0.0) } else { (self.speed + THROTTLE_ACCEL * dt).min(MAX_DRIVE_SPEED) };
        } else if pedals.throttle < -0.1 {
            if self.speed > 0.0 {
                self.speed = (self.speed - BRAKE_DECEL * dt).max(0.0);
                self.reverse_dwell = 0.0;
            } else {
                self.reverse_dwell += if self.speed == 0.0 { dt } else { 0.0 };
                if self.speed < 0.0 || self.reverse_dwell >= REVERSE_ENGAGE_DWELL {
                    self.speed = (self.speed - REVERSE_ACCEL * dt).max(-MAX_REVERSE_SPEED);
                }
            }
        } else {
            self.reverse_dwell = 0.0;
            self.speed = if self.speed > 0.0 { (self.speed - COAST_DECEL * dt).max(0.0) } else { (self.speed + COAST_DECEL * dt).min(0.0) };
        }
        let target_steer = if pedals.steer.abs() > 0.1 { pedals.steer.signum() } else { 0.0 };
        self.steer_turn += (target_steer - self.steer_turn) * (14.0 * dt).clamp(0.0, 1.0);
        let prev_spin = self.wheel_spin;
        self.wheel_spin += (self.speed * dt) / wheel_radius.max(0.12);
        if self.speed.abs() > 0.25 && (prev_spin * 1.5).floor() != (self.wheel_spin * 1.5).floor() {
            EngineRumble::combustion().thump();
        }
        target_steer
    }
}

/// Active driven vehicle state encapsulating the multi-block car assembly and kinematics.
pub struct DrivenCar {
    assembly: VehicleAssembly,
    kinematics: DriveKinematics,
}

impl DrivenCar {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(assembly: VehicleAssembly, kinematics: DriveKinematics) -> Self {
        Self { assembly, kinematics }
    }

    pub const fn boarded(assembly: VehicleAssembly) -> Self { Self::new(assembly, DriveKinematics::parked()) }
    pub const fn steer_turn(&self) -> f32 { self.kinematics.steer_turn() }

    pub fn release_wheel(self, world: &mut dyn PlanetaryWorld) -> Self {
        let shell = self.assembly.perch_shell();
        let _ = world.park_vehicle(self.assembly);
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
        let (origin, up) = (self.assembly.origin(), self.assembly.normal().normalize_or_zero());
        (world.constrain_to_surface(origin + up * 1.15, 0.9) - origin).dot(up) > 1.15
    }

    pub fn seat_position(&self, world: &dyn PlanetaryWorld) -> Vec3 {
        let origin = self.assembly.origin();
        let up = world.surface_up_at(origin);
        world.constrain_to_surface(origin + up * 4.0, SEAT_EYE_SUPPORT) + up * (0.12 + self.assembly.seat_floor_offset())
    }

    pub fn dismount_in_direction(&self, throttle: f32, steer: f32, world: &dyn PlanetaryWorld) -> Vec3 {
        let origin = self.assembly.origin();
        let up = world.surface_up_at(origin);
        let fwd = (self.assembly.heading() - up * self.assembly.heading().dot(up)).normalize_or_zero();
        let right = fwd.cross(up).normalize_or_zero();
        let chosen = (fwd * throttle + right * steer).normalize_or_zero();
        let primary = if chosen.length_squared() > 0.01 { chosen } else { fwd };
        let side = primary.cross(up).normalize_or_zero();
        let seat_elev = world.constrain_to_surface(origin + up * 2.0, 1.0).dot(up);
        for cand in [
            origin + primary * 0.60,
            origin + primary * 1.15,
            origin + primary * 1.65,
            origin + side * 0.60,
            origin - side * 0.60,
            origin - primary * 0.60,
            origin + side * 1.15,
            origin - side * 1.15,
            origin - primary * 1.15,
        ] {
            let grounded = world.constrain_to_surface(cand + up * 2.0, 1.0);
            if grounded.dot(up) < seat_elev - 0.15 {
                return grounded;
            }
        }
        world.constrain_to_surface(origin + primary * 0.60 + up * 2.0, 1.0)
    }

    pub fn step(&mut self, dt: f32, controls: &dyn KeyboardControls, world: &dyn PlanetaryWorld) -> (Vec3, f32) {
        let mut pedals = PedalInput::idle();
        controls.drive_locomotion(Vec3::Y, Vec3::X, &mut pedals);
        let sign = self.kinematics.advance(&pedals, self.assembly.wheel_radius(), dt);
        let yaw_delta = self.kinematics.yaw_step(sign, dt);
        if yaw_delta.abs() > 0.0 {
            self.assembly.steer(-yaw_delta);
        }
        let next_raw = self.assembly.origin() + self.assembly.heading() * self.kinematics.displacement(dt);
        let up = world.surface_up_at(next_raw);
        let next_ground = world.constrain_to_surface(next_raw + up * 4.0, 0.0);
        self.assembly.relocate(next_ground, up);
        (next_ground + up * (SEAT_EYE_SUPPORT + 0.12 + self.assembly.seat_floor_offset()), yaw_delta / 0.003)
    }

    pub fn emit(&self, target: &mut dyn MeshTarget) {
        self.kinematics.emit_assembly(&self.assembly, target);
    }
}

/// Polymorphic avatar driving state avoiding `Option<DrivenCar>`.
pub enum VehiclePilot {
    Pedestrian,
    Perch(DrivenCar),
    Driving(DrivenCar),
}

impl VehiclePilot {
    pub const fn standing(&self) -> bool { matches!(self, Self::Pedestrian) }
    pub const fn seated(&self) -> bool { matches!(self, Self::Perch(_)) }
    pub const fn driving(&self) -> bool { matches!(self, Self::Driving(_)) }

    pub fn shape_pose(&self, pose: crate::attire::BodyPose) -> crate::attire::BodyPose {
        match self {
            Self::Pedestrian => pose,
            Self::Perch(car) => pose.with_surface_rotation(car.body_rotation()).with_seated(1.0),
            Self::Driving(car) => pose.with_surface_rotation(car.body_rotation()).with_seated_steer(car.steer_turn()),
        }
    }

    pub fn release_wheel(&mut self, world: &mut dyn PlanetaryWorld) -> Option<Vec3> {
        if let Self::Driving(car) = std::mem::replace(self, Self::Pedestrian) {
            let perch_car = car.release_wheel(world);
            let pos = perch_car.perch_position();
            *self = Self::Perch(perch_car);
            Some(pos)
        } else {
            None
        }
    }

    pub fn step(&mut self, dt: f32, controls: &dyn KeyboardControls, world: &dyn PlanetaryWorld) -> Option<(Vec3, f32)> {
        match self {
            Self::Pedestrian => None,
            Self::Perch(car) => {
                if !car.seat_intact(world) {
                    let pos = world.constrain_to_surface(car.perch_position(), SEAT_EYE_SUPPORT);
                    *self = Self::Pedestrian;
                    return Some((pos, 0.0));
                }
                let mut pedals = PedalInput::idle();
                controls.drive_locomotion(Vec3::Y, Vec3::X, &mut pedals);
                if pedals.throttle.abs() > 0.1 || pedals.steer.abs() > 0.1 {
                    let exit = car.dismount_in_direction(pedals.throttle, pedals.steer, world);
                    *self = Self::Pedestrian;
                    Some((exit, 0.0))
                } else {
                    Some((car.perch_position(), 0.0))
                }
            }
            Self::Driving(car) => Some(car.step(dt, controls, world)),
        }
    }

    pub fn emit(&self, target: &mut dyn MeshTarget) {
        if let Self::Driving(car) = self {
            car.emit(target);
        }
    }
}

#[cfg(test)]
mod tests;
