use glam::Vec3;
use world::{CraftIdentity, MeshTarget, PilotCraft, PlanetaryWorld, VehicleAssembly};
use crate::controls::KeyboardControls;
use super::passenger::PassengerBerth;
use super::track::FaceContact;
use super::{DrivenCar, PedalInput, SEAT_EYE_SUPPORT};

/// Polymorphic avatar driving and passenger state avoiding `Option<DrivenCar>`.
pub enum VehiclePilot {
    Pedestrian,
    Perch(DrivenCar),
    Passenger(PassengerBerth),
    Driving(DrivenCar),
}

impl VehiclePilot {
    pub const fn standing(&self) -> bool { matches!(self, Self::Pedestrian) }
    pub const fn seated(&self) -> bool { matches!(self, Self::Perch(_) | Self::Passenger(_)) }
    pub const fn driving(&self) -> bool { matches!(self, Self::Driving(_)) }
    pub const fn riding(&self) -> bool { matches!(self, Self::Driving(_) | Self::Passenger(_)) }

    pub fn mount(&mut self, assembly: VehicleAssembly, world: &dyn PlanetaryWorld) -> (Vec3, Vec3) {
        let heading = assembly.heading();
        let drivable = assembly.drivable();
        let car = DrivenCar::boarded(assembly);
        let pos = if drivable { car.seat_position(world) } else { car.perch_position() };
        *self = if drivable { Self::Driving(car) } else { Self::Perch(car) };
        (pos, heading)
    }

    pub fn mount_passenger(
        &mut self,
        craft: CraftIdentity,
        local_seat: Vec3,
        frame: (Vec3, Vec3, Vec3),
    ) -> (Vec3, Vec3) {
        let (berth, eye_pos, heading) = PassengerBerth::mounted(craft, local_seat, frame);
        *self = Self::Passenger(berth);
        (eye_pos, heading)
    }

    pub fn reconcile_helm(&mut self, world: &mut dyn PlanetaryWorld) -> Option<Vec3> {
        let revoked_id = match self {
            Self::Driving(car) => Some(car.identity()),
            _ => None,
        }?;
        if !world.helm_revoked(revoked_id) {
            return None;
        }
        let fallback = match std::mem::replace(self, Self::Pedestrian) {
            Self::Driving(car) => FaceContact::new(car.perch_position()).ground(SEAT_EYE_SUPPORT, world),
            _ => Vec3::ZERO,
        };
        Some(fallback)
    }

    pub fn shape_pose(&self, pose: crate::attire::BodyPose) -> crate::attire::BodyPose {
        match self {
            Self::Pedestrian => pose,
            Self::Perch(car) => pose.with_surface_rotation(car.body_rotation()).with_seated(1.0),
            Self::Passenger(berth) => berth.shape_pose(pose),
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

    pub fn step(
        &mut self,
        dt: f32,
        controls: &dyn KeyboardControls,
        world: &dyn PlanetaryWorld,
    ) -> Option<(Vec3, Vec3, f32)> {
        match self {
            Self::Pedestrian => None,
            Self::Perch(car) => {
                if !car.seat_intact(world) {
                    let pos = FaceContact::new(car.perch_position()).ground(SEAT_EYE_SUPPORT, world);
                    let up = world.surface_up_at(pos);
                    *self = Self::Pedestrian;
                    return Some((pos, up, 0.0));
                }
                let mut pedals = PedalInput::idle();
                controls.drive_locomotion(Vec3::Y, Vec3::X, &mut pedals);
                if pedals.dismount_requested() {
                    let exit = car.dismount_with(&pedals, world);
                    let up = world.surface_up_at(exit);
                    *self = Self::Pedestrian;
                    Some((exit, up, 0.0))
                } else {
                    Some((car.perch_position(), car.chassis_up(), 0.0))
                }
            }
            Self::Passenger(berth) => {
                let mut pedals = PedalInput::idle();
                controls.drive_locomotion(Vec3::Y, Vec3::X, &mut pedals);
                let (next_berth, pos, up, steer_px) = berth.step(&pedals, world);
                *self = match next_berth {
                    Some(b) => Self::Passenger(b),
                    None => Self::Pedestrian,
                };
                Some((pos, up, steer_px))
            }
            Self::Driving(car) => Some(car.step(dt, controls, world)),
        }
    }

    pub fn emit(&self, target: &mut dyn MeshTarget) {
        if let Self::Driving(car) = self {
            car.emit(target);
        }
    }

    pub fn blend_subtick(&mut self, alpha: f32) {
        if let Self::Driving(car) = self {
            car.blend_subtick(alpha);
        }
    }

    pub const fn pilot_craft(&self) -> PilotCraft {
        match self {
            Self::Driving(car) => car.pilot_craft(),
            Self::Pedestrian | Self::Perch(_) | Self::Passenger(_) => PilotCraft::Pedestrian,
        }
    }
}
