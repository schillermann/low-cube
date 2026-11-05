use glam::Vec3;
use world::{MeshTarget, PlanetaryWorld, VehicleAssembly};
use crate::controls::{KeyboardControls, LocomotionTarget};

const MAX_DRIVE_SPEED: f32 = 14.0;
const THROTTLE_ACCEL: f32 = 16.0;
const BRAKE_DECEL: f32 = 32.0;
const COAST_DECEL: f32 = 3.5;
const STEER_RATE: f32 = 2.2;
const SEAT_EYE_SUPPORT: f32 = 1.0;

struct PedalInput {
    throttle: f32,
    steer: f32,
    brake: bool,
}

impl PedalInput {
    const fn idle() -> Self {
        Self { throttle: 0.0, steer: 0.0, brake: false }
    }
}

impl LocomotionTarget for PedalInput {
    fn travel(&mut self, direction: Vec3) {
        self.throttle = direction.y;
        self.steer = direction.x;
        if direction.y < -0.15 {
            self.brake = true;
        }
    }

    fn jump(&mut self) {
        self.brake = true;
    }

    fn sneak(&mut self) {
        self.brake = true;
    }

    fn sprint(&mut self) {}
}

/// Active driven vehicle state encapsulating the multi-block car assembly and forward speed.
pub struct DrivenCar {
    assembly: VehicleAssembly,
    speed: f32,
}

impl DrivenCar {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(assembly: VehicleAssembly, speed: f32) -> Self {
        Self { assembly, speed }
    }

    pub const fn boarded(assembly: VehicleAssembly) -> Self {
        Self::new(assembly, 0.0)
    }

    pub fn seat_position(&self, world: &dyn PlanetaryWorld) -> Vec3 {
        let origin = self.assembly.origin();
        let up = world.surface_up_at(origin);
        let ground = world.constrain_to_surface(origin + up * 4.0, SEAT_EYE_SUPPORT);
        ground + up * (0.12 + self.assembly.seat_floor_offset())
    }

    pub fn step(&mut self, dt: f32, controls: &dyn KeyboardControls, world: &dyn PlanetaryWorld) -> (Vec3, f32) {
        let mut pedals = PedalInput::idle();
        controls.drive_locomotion(Vec3::Y, Vec3::X, &mut pedals);
        if pedals.brake {
            self.speed = (self.speed - BRAKE_DECEL * dt).max(0.0);
        } else if pedals.throttle > 0.1 {
            self.speed = (self.speed + THROTTLE_ACCEL * dt).min(MAX_DRIVE_SPEED);
        } else {
            self.speed = (self.speed - COAST_DECEL * dt).max(0.0);
        }
        let mut steer_px = 0.0;
        if pedals.steer.abs() > 0.1 {
            let sign = if pedals.steer > 0.0 { 1.0 } else { -1.0 };
            let yaw_delta = -sign * STEER_RATE * dt;
            self.assembly.steer(yaw_delta);
            steer_px = (sign * STEER_RATE * dt) / 0.003;
        }
        let cur_origin = self.assembly.origin();
        let next_raw = cur_origin + self.assembly.heading() * (self.speed * dt);
        let up = world.surface_up_at(next_raw);
        let next_ground = world.constrain_to_surface(next_raw + up * 4.0, 0.0);
        self.assembly.relocate(next_ground, up);
        (next_ground + up * (SEAT_EYE_SUPPORT + 0.12 + self.assembly.seat_floor_offset()), steer_px)
    }

    pub fn into_assembly(self) -> VehicleAssembly {
        self.assembly
    }

    pub fn emit(&self, target: &mut dyn MeshTarget) {
        self.assembly.emit(target);
    }
}

/// Polymorphic avatar driving state avoiding `Option<DrivenCar>`.
pub enum VehiclePilot {
    Pedestrian,
    Seated(DrivenCar),
}

impl VehiclePilot {
    pub const fn driving(&self) -> bool {
        matches!(self, Self::Seated(_))
    }

    pub fn dismount(&mut self, world: &mut dyn PlanetaryWorld) -> bool {
        let prev = std::mem::replace(self, Self::Pedestrian);
        if let Self::Seated(car) = prev {
            let _ = world.park_vehicle(car.into_assembly());
            true
        } else {
            false
        }
    }

    pub fn step(&mut self, dt: f32, controls: &dyn KeyboardControls, world: &dyn PlanetaryWorld) -> Option<(Vec3, f32)> {
        match self {
            Self::Pedestrian => None,
            Self::Seated(car) => Some(car.step(dt, controls, world)),
        }
    }

    pub fn emit(&self, target: &mut dyn MeshTarget) {
        if let Self::Seated(car) = self {
            car.emit(target);
        }
    }
}
