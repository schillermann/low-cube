use glam::Vec3;
use world::{MeshTarget, PlanetaryWorld, VehicleAssembly};
use crate::controls::KeyboardControls;
use super::{DrivenCar, PedalInput, SEAT_EYE_SUPPORT};

/// Polymorphic avatar driving state avoiding `Option<DrivenCar>`.
pub enum VehiclePilot {
    Pedestrian,
    Perch(DrivenCar),
    Driving(DrivenCar),
}

impl VehiclePilot {
    pub const fn standing(&self) -> bool {
        matches!(self, Self::Pedestrian)
    }
    pub const fn seated(&self) -> bool {
        matches!(self, Self::Perch(_))
    }
    pub const fn driving(&self) -> bool {
        matches!(self, Self::Driving(_))
    }

    pub fn mount(&mut self, assembly: VehicleAssembly, world: &dyn PlanetaryWorld) -> (Vec3, Vec3) {
        let heading = assembly.heading();
        let drivable = assembly.drivable();
        let car = DrivenCar::boarded(assembly);
        let pos = if drivable { car.seat_position(world) } else { car.perch_position() };
        *self = if drivable { Self::Driving(car) } else { Self::Perch(car) };
        (pos, heading)
    }

    pub fn shape_pose(&self, pose: crate::attire::BodyPose) -> crate::attire::BodyPose {
        match self {
            Self::Pedestrian => pose,
            Self::Perch(car) => pose.with_surface_rotation(car.body_rotation()).with_seated(1.0),
            Self::Driving(car) => {
                pose.with_surface_rotation(car.body_rotation()).with_seated_steer(car.steer_turn())
            }
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
    ) -> Option<(Vec3, f32)> {
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
