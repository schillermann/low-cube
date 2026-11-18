use glam::{Mat3, Quat, Vec3};
use world::{CraftIdentity, MeshTarget, PilotCraft, PlanetaryWorld, VehicleAssembly};
use crate::controls::KeyboardControls;
use super::{DrivenCar, PedalInput, SEAT_EYE_SUPPORT};

/// Polymorphic avatar driving and passenger state avoiding `Option<DrivenCar>`.
pub enum VehiclePilot {
    Pedestrian,
    Perch(DrivenCar),
    Passenger {
        craft: CraftIdentity,
        local_seat: Vec3,
        seat_frame: (Vec3, Vec3, Vec3),
    },
    Driving(DrivenCar),
}

impl VehiclePilot {
    pub const fn standing(&self) -> bool {
        matches!(self, Self::Pedestrian)
    }
    pub const fn seated(&self) -> bool {
        matches!(self, Self::Perch(_) | Self::Passenger { .. })
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

    pub fn mount_passenger(
        &mut self,
        craft: CraftIdentity,
        local_seat: Vec3,
        frame: (Vec3, Vec3, Vec3),
    ) -> (Vec3, Vec3) {
        let (seat, heading, up) = frame;
        let eye_pos = seat + up.normalize_or_zero() * (SEAT_EYE_SUPPORT + 0.12);
        *self = Self::Passenger { craft, local_seat, seat_frame: frame };
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
            Self::Driving(car) => world.constrain_to_surface(car.perch_position(), SEAT_EYE_SUPPORT),
            _ => Vec3::ZERO,
        };
        Some(fallback)
    }

    pub fn shape_pose(&self, pose: crate::attire::BodyPose) -> crate::attire::BodyPose {
        match self {
            Self::Pedestrian => pose,
            Self::Perch(car) => pose.with_surface_rotation(car.body_rotation()).with_seated(1.0),
            Self::Passenger { seat_frame: (_, raw_fwd, raw_up), .. } => {
                let up = raw_up.normalize_or_zero();
                let fwd = (*raw_fwd - up * raw_fwd.dot(up)).normalize_or_zero();
                let right = fwd.cross(up).normalize_or_zero();
                let rot = Quat::from_mat3(&Mat3::from_cols(right, up, -fwd)).normalize();
                pose.with_surface_rotation(rot).with_seated(1.0)
            }
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
            Self::Passenger { craft, local_seat, ref mut seat_frame } => {
                let Some((seat, heading, up_raw)) = world.passenger_seat_frame(*craft, *local_seat) else {
                    let pos = world.constrain_to_surface(seat_frame.0 + seat_frame.2 * 1.2, SEAT_EYE_SUPPORT);
                    *self = Self::Pedestrian;
                    return Some((pos, 0.0));
                };
                *seat_frame = (seat, heading, up_raw);
                let up = up_raw.normalize_or_zero();
                let mut pedals = PedalInput::idle();
                controls.drive_locomotion(Vec3::Y, Vec3::X, &mut pedals);
                if pedals.throttle.abs() > 0.1 || pedals.steer.abs() > 0.1 {
                    let fwd = (heading - up * heading.dot(up)).normalize_or_zero();
                    let right = fwd.cross(up).normalize_or_zero();
                    let dir = (fwd * pedals.throttle + right * pedals.steer).normalize_or_zero();
                    let exit = world.constrain_to_surface(seat + dir * 1.15 + up * 2.0, SEAT_EYE_SUPPORT);
                    *self = Self::Pedestrian;
                    Some((exit, 0.0))
                } else {
                    Some((seat + up * (SEAT_EYE_SUPPORT + 0.12), 0.0))
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

    pub fn blend_subtick(&mut self, alpha: f32) {
        if let Self::Driving(car) = self {
            car.blend_subtick(alpha);
        }
    }

    pub const fn pilot_craft(&self) -> PilotCraft {
        match self {
            Self::Driving(car) => car.pilot_craft(),
            Self::Pedestrian | Self::Perch(_) | Self::Passenger { .. } => PilotCraft::Pedestrian,
        }
    }
}
