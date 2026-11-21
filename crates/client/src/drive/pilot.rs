use glam::Vec3;
use world::{CraftIdentity, HullOptic, MeshTarget, OpticMount, PilotCraft, PlanetaryWorld, VehicleAssembly};
use crate::controls::KeyboardControls;
use super::passenger::PassengerBerth;
use super::track::FaceContact;
use super::{HelmedCraft, PedalInput, SEAT_EYE_SUPPORT};

/// Polymorphic avatar driving and passenger state avoiding `Option<HelmedCraft>`.
pub enum VehiclePilot {
    Pedestrian,
    Perch(HelmedCraft),
    Passenger(PassengerBerth),
    Driving(HelmedCraft),
}

impl VehiclePilot {
    pub const fn standing(&self) -> bool { matches!(self, Self::Pedestrian) }
    pub const fn seated(&self) -> bool { matches!(self, Self::Perch(_) | Self::Passenger(_)) }
    pub const fn driving(&self) -> bool { matches!(self, Self::Driving(_)) }
    pub const fn riding(&self) -> bool { matches!(self, Self::Driving(_) | Self::Passenger(_)) }

    pub fn free_look(&self) -> bool {
        match self {
            Self::Driving(craft) => craft.free_look(),
            _ => false,
        }
    }

    pub fn helm_optic_channel(&self) -> u8 {
        match self {
            Self::Driving(craft) => craft.helm_optic_channel(),
            _ => 0,
        }
    }

    pub fn toggle_retro_flip(&mut self) -> bool {
        match self {
            Self::Driving(craft) => craft.toggle_retro_flip(),
            _ => false,
        }
    }

    pub fn toggle_flight_guidance(&mut self) -> bool {
        match self {
            Self::Driving(craft) => craft.toggle_flight_guidance(),
            _ => false,
        }
    }

    pub fn guidance_readout(&self) -> crate::menu::GuidanceReadout {
        match self {
            Self::Driving(craft) => craft.guidance_readout(),
            _ => crate::menu::GuidanceReadout::Absent,
        }
    }

    pub fn mark_nose_target(&mut self, rays: ((Vec3, Vec3), (Vec3, Vec3)), world: &dyn PlanetaryWorld) -> bool {
        match self {
            Self::Driving(craft) => craft.mark_nose_target(rays, world),
            _ => false,
        }
    }

    pub fn optic_telemetry(&self, mount: OpticMount) -> super::OpticTelemetry {
        match self {
            Self::Driving(craft) => craft.optic_telemetry(mount),
            _ => super::OpticTelemetry::Silent,
        }
    }

    pub fn camera_views(&self, out: &mut Vec<HullOptic>) {
        if let Self::Driving(craft) = self {
            craft.camera_views(out);
        }
    }

    pub fn collected_views(&self) -> Vec<HullOptic> {
        let mut views = Vec::new();
        self.camera_views(&mut views);
        views
    }

    pub fn mount(&mut self, assembly: VehicleAssembly, world: &dyn PlanetaryWorld) -> (Vec3, Vec3) {
        let heading = assembly.heading();
        let drivable = assembly.drivable();
        let craft = HelmedCraft::boarded_in(assembly, world);
        let pos = if drivable { craft.seat_position(world) } else { craft.perch_position() };
        *self = if drivable { Self::Driving(craft) } else { Self::Perch(craft) };
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
            Self::Driving(craft) => Some(craft.identity()),
            _ => None,
        }?;
        if !world.helm_revoked(revoked_id) {
            return None;
        }
        let fallback = match std::mem::replace(self, Self::Pedestrian) {
            Self::Driving(craft) => FaceContact::new(craft.perch_position()).ground(SEAT_EYE_SUPPORT, world),
            _ => Vec3::ZERO,
        };
        Some(fallback)
    }

    pub fn shape_pose(&self, pose: crate::attire::BodyPose) -> crate::attire::BodyPose {
        match self {
            Self::Pedestrian => pose,
            Self::Perch(craft) => pose.with_surface_rotation(craft.body_rotation()).with_seated(1.0),
            Self::Passenger(berth) => berth.shape_pose(pose),
            Self::Driving(craft) => craft.shape_pilot_pose(pose),
        }
    }

    pub fn release_wheel(&mut self, world: &mut dyn PlanetaryWorld) -> Option<Vec3> {
        if let Self::Driving(craft) = std::mem::replace(self, Self::Pedestrian) {
            let perch_craft = craft.release_wheel(world);
            let pos = perch_craft.perch_position();
            *self = Self::Perch(perch_craft);
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
            Self::Perch(craft) => {
                if !craft.seat_intact(world) {
                    let pos = FaceContact::new(craft.perch_position()).ground(SEAT_EYE_SUPPORT, world);
                    let up = world.surface_up_at(pos);
                    *self = Self::Pedestrian;
                    return Some((pos, up, 0.0));
                }
                let mut pedals = PedalInput::idle();
                controls.drive_locomotion(Vec3::Y, Vec3::X, &mut pedals);
                if pedals.dismount_requested() {
                    let exit = craft.dismount_with(&pedals, world);
                    let up = world.surface_up_at(exit);
                    *self = Self::Pedestrian;
                    Some((exit, up, 0.0))
                } else {
                    Some((craft.perch_position(), craft.chassis_up(), 0.0))
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
            Self::Driving(craft) => Some(craft.step(dt, controls, world)),
        }
    }

    pub fn emit(&self, target: &mut dyn MeshTarget) {
        if let Self::Driving(craft) = self {
            craft.emit(target);
        }
    }

    pub fn present_vessel(&self, sink: &mut dyn world::VesselSink) {
        if let Self::Driving(craft) = self {
            craft.present_vessel(sink);
        }
    }

    pub fn blend_subtick(&mut self, alpha: f32) {
        if let Self::Driving(craft) = self {
            craft.blend_subtick(alpha);
        }
    }

    pub fn pilot_craft(&self) -> PilotCraft {
        match self {
            Self::Driving(craft) => craft.pilot_craft(),
            Self::Pedestrian | Self::Perch(_) | Self::Passenger(_) => PilotCraft::Pedestrian,
        }
    }
}
