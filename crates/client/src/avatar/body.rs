use glam::Vec3;
use world::{MeshTarget, PlanetaryWorld, VehicleAssembly};
use crate::attire::BodyPose;
use crate::controls::KeyboardControls;
use crate::drive::{DrivenCar, VehiclePilot};
use crate::envelope::BodyEnvelope;
use crate::flight::GravitationalFlight;
use crate::order::LocomotionOrder;
use crate::posture::AvatarPosture;
use super::sight::AvatarSight;

const EYE_SUPPORT_HEIGHT: f32 = 1.0;

/// Encapsulated physical avatar body state: world position, gravitational flight, posture, and pilot state.
pub struct AvatarBody {
    position: Vec3,
    flight: Box<dyn GravitationalFlight>,
    posture: AvatarPosture,
    pilot: VehiclePilot,
}

impl AvatarBody {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn with_pilot(position: Vec3, flight: Box<dyn GravitationalFlight>, posture: AvatarPosture, pilot: VehiclePilot) -> Self {
        Self { position, flight, posture, pilot }
    }

    pub fn new(position: Vec3, flight: Box<dyn GravitationalFlight>, posture: AvatarPosture) -> Self {
        Self::with_pilot(position, flight, posture, VehiclePilot::Pedestrian)
    }

    pub const fn position(&self) -> Vec3 {
        self.position
    }

    pub const fn posture(&self) -> AvatarPosture {
        self.posture
    }

    pub const fn standing(&self) -> bool {
        self.pilot.standing()
    }

    pub const fn driving(&self) -> bool {
        self.pilot.driving()
    }

    pub fn shape_pose(&self, pose: BodyPose) -> BodyPose {
        self.pilot.shape_pose(pose)
    }

    pub fn perished(&self) -> bool {
        self.posture.perished()
    }

    pub fn eye_offset(&self, terrain_up: Vec3) -> Vec3 {
        terrain_up * self.posture.eye_elevation()
    }

    pub fn relocate(&mut self, next_pos: Vec3) {
        self.position = next_pos;
    }

    pub fn respawn(&mut self, world: &dyn PlanetaryWorld) {
        let spawn = [(0.25, 0.25), (2.25, 0.25), (-2.25, 0.25), (0.25, 2.25), (0.25, -2.25)]
            .into_iter()
            .map(|(x, z)| world.constrain_to_surface(Vec3::new(x, 1024.0, z), 1.0))
            .fold(world.constrain_to_surface(Vec3::new(0.25, 1024.0, 0.25), 1.0), |acc, c| if c.y > acc.y + 1.5 { c } else { acc });
        self.position = spawn;
        self.posture = AvatarPosture::standing();
        self.pilot = VehiclePilot::Pedestrian;
        self.flight.ground();
    }

    pub fn interact_seated_vehicle(
        &mut self, eye: Vec3, view_dir: Vec3, sight_line: &mut AvatarSight, world: &mut dyn PlanetaryWorld,
    ) -> Option<bool> {
        if self.pilot.driving() {
            if let Some(perch_pos) = self.pilot.release_wheel(world) {
                self.position = perch_pos;
                sight_line.constrain_shoulders(true, 1.42);
                return Some(true);
            }
        }
        if self.pilot.seated() {
            if let Some(assembly) = world.board_vehicle(eye, view_dir, true) {
                let heading = assembly.heading();
                if assembly.drivable() {
                    self.board_vehicle(assembly, world);
                } else {
                    self.sit_in_seat(assembly);
                }
                sight_line.anchor_vehicle_heading(heading);
                return Some(true);
            }
        }
        None
    }

    pub fn perch_vehicle(
        &mut self, eye: Vec3, view_dir: Vec3, sight_line: &mut AvatarSight, world: &mut dyn PlanetaryWorld,
    ) -> bool {
        if let Some(perch) = world.board_vehicle(eye, view_dir, false) {
            let heading = perch.heading();
            if perch.drivable() {
                self.board_vehicle(perch, world);
            } else {
                self.sit_in_seat(perch);
            }
            sight_line.anchor_vehicle_heading(heading);
            return true;
        }
        false
    }

    pub fn sit_in_seat(&mut self, assembly: VehicleAssembly) {
        let car = DrivenCar::boarded(assembly);
        self.position = car.perch_position();
        self.posture = AvatarPosture::crouching();
        self.flight.ground();
        self.pilot = VehiclePilot::Perch(car);
    }

    pub fn board_vehicle(&mut self, assembly: VehicleAssembly, world: &dyn PlanetaryWorld) {
        let car = DrivenCar::boarded(assembly);
        self.position = car.seat_position(world);
        self.posture = AvatarPosture::crouching();
        self.flight.ground();
        self.pilot = VehiclePilot::Driving(car);
    }

    pub fn step_driving(
        &mut self,
        dt: f32,
        controls: &dyn KeyboardControls,
        sight_line: &mut AvatarSight,
        world: &dyn PlanetaryWorld,
    ) -> bool {
        let Some((seat_pos, steer_px)) = self.pilot.step(dt, controls, world) else {
            return false;
        };
        self.position = seat_pos;
        if self.pilot.standing() {
            self.posture = AvatarPosture::standing();
            self.flight.ground();
            sight_line.constrain_shoulders(false, 0.0);
        } else {
            self.posture = self.posture.crouch_down(dt);
            sight_line.constrain_shoulders(true, 1.42);
            if steer_px.abs() > 1e-5 {
                sight_line.steer_vehicle(steer_px);
            }
        }
        sight_line.reorient_surface(world.surface_up_at(self.position), dt);
        true
    }

    pub fn emit_vehicle(&self, target: &mut dyn MeshTarget) {
        self.pilot.emit(target);
    }

    pub fn advance_perish(&mut self, dt: f32) -> (bool, bool) {
        let (next_posture, beat) = self.posture.perish().advance_perish(dt);
        if next_posture.blackout_complete() {
            return (true, false);
        }
        self.posture = next_posture;
        (false, beat)
    }

    pub fn prepare_stride(&mut self, order: &LocomotionOrder, standing_env: BodyEnvelope, terrain_up: Vec3, dt: f32, world: &dyn PlanetaryWorld) -> BodyEnvelope {
        order.dispatch_jump(&mut *self.flight);
        let next_posture = order.advance_posture(self.posture, dt);
        self.posture = if standing_env.crust(world).overhead_obstructed(self.position, terrain_up, EYE_SUPPORT_HEIGHT) {
            self.posture.crouch_down(dt)
        } else {
            next_posture
        };
        let active_env = standing_env.with_crouch_and_facing(self.posture.crouch_drop(), Vec3::ZERO);
        self.flight.apply_envelope(active_env);
        active_env
    }

    pub fn apply_flight(&mut self, active_env: BodyEnvelope, order: &LocomotionOrder, dt: f32, terrain_up: Vec3, pace: f32, world: &dyn PlanetaryWorld) -> Vec3 {
        self.flight.apply_envelope(active_env);
        order.advance_flight(&mut *self.flight, dt, self.position, terrain_up, pace, world)
    }

    pub fn advance_leap(&mut self, next_pos: Vec3, terrain_up: Vec3, dt: f32) {
        self.posture = self.posture.advance_leap(self.position, next_pos, terrain_up, dt);
    }
}
