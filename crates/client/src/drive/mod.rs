mod passenger;
mod pilot;
mod road;
mod rocket;
mod space_step;
mod track;
mod wheeled;
#[cfg(test)]
mod camera_look_tests;
#[cfg(test)]
mod expanse_tests;
#[cfg(test)]
mod liftoff_tests;
#[cfg(test)]
mod lunar_tests;
#[cfg(test)]
mod multiplayer_tests;
#[cfg(test)]
mod nozzle_tests;
#[cfg(test)]
mod preview_vessel_tests;
#[cfg(test)]
mod rocket_pose_tests;
#[cfg(test)]
mod rocket_tests;
#[cfg(test)]
mod seam_tests;
#[cfg(test)]
mod tests;

use glam::{Mat3, Quat, Vec3};
use world::{ClimateZone, HullOptic, MeshTarget, PlanetaryWorld, VehicleAssembly};
use crate::attire::BodyPose;
use crate::controls::{KeyboardControls, LocomotionTarget};
pub use self::pilot::VehiclePilot;
use self::rocket::RocketDrive;
use self::track::FaceContact;
pub use self::wheeled::WheeledDrive;

pub(super) const SEAT_EYE_SUPPORT: f32 = 1.0;

/// Polymorphic propulsion strategy driving a `HelmedCraft` without car-vs-rocket type inspection.
pub trait CraftDrive: Send + Sync {
    fn motion_channels(&self) -> (f32, f32);
    fn shape_pilot_pose(&self, pose: BodyPose) -> BodyPose;
    fn free_look(&self) -> bool;
    fn helm_optic_channel(&self, _assembly: &VehicleAssembly) -> u8 { 0 }
    fn toggle_retro_flip(&mut self, _assembly: &VehicleAssembly) -> bool { false }
    fn advance(
        &mut self,
        assembly: &mut VehicleAssembly,
        dt: f32,
        controls: &dyn KeyboardControls,
        world: &dyn PlanetaryWorld,
    ) -> (Vec3, Vec3, f32);
}

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

    pub(super) fn transfer<R>(&self, f: impl FnOnce(f32, f32, bool) -> R) -> R {
        f(self.throttle, self.steer, self.handbrake)
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

/// Active helmed vehicle uniting `VehicleAssembly`, polymorphic `CraftDrive`, and sub-tick origin span.
pub struct HelmedCraft {
    assembly: VehicleAssembly,
    drive: Box<dyn CraftDrive>,
    origin_span: (Vec3, f32),
}

impl HelmedCraft {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(assembly: VehicleAssembly, drive: Box<dyn CraftDrive>, origin_span: (Vec3, f32)) -> Self {
        Self { assembly, drive, origin_span }
    }

    pub fn boarded(assembly: VehicleAssembly) -> Self {
        let origin = assembly.origin();
        let drive: Box<dyn CraftDrive> = if assembly.spacefaring() {
            Box::new(RocketDrive::pad_idle())
        } else {
            Box::new(WheeledDrive::parked())
        };
        Self::new(assembly, drive, (origin, 1.0))
    }

    pub const fn identity(&self) -> world::CraftIdentity { self.assembly.identity() }
    pub fn shape_pilot_pose(&self, pose: BodyPose) -> BodyPose {
        self.drive.shape_pilot_pose(pose.with_surface_rotation(self.body_rotation()))
    }
    pub fn free_look(&self) -> bool { self.drive.free_look() }
    pub fn helm_optic_channel(&self) -> u8 { self.drive.helm_optic_channel(&self.assembly) }
    pub fn chassis_up(&self) -> Vec3 { self.assembly.normal().normalize_or_zero() }
    pub fn blend_subtick(&mut self, alpha: f32) { self.origin_span.1 = alpha.clamp(0.0, 1.0); }
    pub fn toggle_retro_flip(&mut self) -> bool { self.drive.toggle_retro_flip(&self.assembly) }

    pub fn camera_views(&self, out: &mut Vec<HullOptic>) {
        let (prev, alpha) = self.origin_span;
        let cur = self.assembly.origin();
        let visual_shift = if (cur - prev).length_squared() > 64.0 {
            Vec3::ZERO
        } else {
            prev.lerp(cur, alpha) - cur
        };
        let start = out.len();
        self.assembly.push_camera_views(out);
        for view in &mut out[start..] {
            *view = view.shifted(visual_shift);
        }
    }

    pub fn pilot_craft(&self) -> world::PilotCraft {
        let (m0, m1) = self.drive.motion_channels();
        world::PilotCraft::Driving(world::DrivenCraft::new(
            0,
            self.assembly.identity(),
            self.assembly.pose().with_motion(m0, m1),
        ))
    }

    fn orbital_at(pos: Vec3, world: &dyn PlanetaryWorld) -> bool {
        if world.apply_pull(pos, 1.0).length_squared() <= 1e-4 {
            return true;
        }
        let up = world.surface_up_at(pos);
        (pos - FaceContact::new(pos).ground(0.0, world)).dot(up) > 4.5
    }

    pub fn release_wheel(self, world: &mut dyn PlanetaryWorld) -> Self {
        let mut shell = self.assembly.perch_shell();
        let raw_origin = self.assembly.origin();
        let in_orbit = Self::orbital_at(raw_origin, world);
        let _ = world.park_vehicle(self.assembly);
        if in_orbit {
            return Self::boarded(shell);
        }
        let face_up = ClimateZone::dominant(world.surface_up_at(raw_origin)).normal();
        let base_ground = FaceContact::new(raw_origin).ground(0.0, world) - face_up * 0.5;
        shell.relocate(base_ground, face_up);
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
        if Self::orbital_at(origin, world) {
            let up = self.assembly.normal().normalize_or_zero();
            return origin + up * (SEAT_EYE_SUPPORT + 0.12 + self.assembly.seat_floor_offset());
        }
        let up = world.surface_up_at(origin);
        FaceContact::new(origin).ground(SEAT_EYE_SUPPORT, world) + up * (0.12 + self.assembly.seat_floor_offset())
    }

    pub(super) fn dismount_with(&self, pedals: &PedalInput, world: &dyn PlanetaryWorld) -> Vec3 {
        let origin = self.assembly.origin();
        if Self::orbital_at(origin, world) {
            let up = self.assembly.normal().normalize_or_zero();
            let fwd = (self.assembly.heading() - up * self.assembly.heading().dot(up)).normalize_or_zero();
            let right = fwd.cross(up).normalize_or_zero();
            let chosen = pedals.dismount_vector(fwd, right);
            let exit_dir = if chosen.length_squared() > 0.01 { chosen } else { right };
            return origin + exit_dir * 1.15 + up * SEAT_EYE_SUPPORT;
        }
        let face_up = ClimateZone::dominant(world.surface_up_at(origin)).normal();
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
            let drop = (grounded - origin).dot(face_up);
            if grounded.dot(face_up) < seat_elev - 0.15 && drop > -3.5 {
                return grounded;
            }
        }
        FaceContact::new(origin + primary * 0.60).ground(1.0, world)
    }

    pub fn step(
        &mut self,
        dt: f32,
        controls: &dyn KeyboardControls,
        world: &dyn PlanetaryWorld,
    ) -> (Vec3, Vec3, f32) {
        let prev_origin = self.assembly.origin();
        let out = self.drive.advance(&mut self.assembly, dt, controls, world);
        self.origin_span = (prev_origin, 1.0);
        out
    }

    pub fn emit(&self, target: &mut dyn MeshTarget) {
        let (prev, alpha) = self.origin_span;
        let cur = self.assembly.origin();
        let visual_origin = if (cur - prev).length_squared() > 64.0 { cur } else { prev.lerp(cur, alpha) };
        let (m0, m1) = self.drive.motion_channels();
        self.assembly.emit_driven_at(visual_origin, m0, m1, target);
    }

    pub fn present_vessel(&self, sink: &mut dyn world::VesselSink) {
        let (prev, alpha) = self.origin_span;
        let cur = self.assembly.origin();
        let visual_origin = if (cur - prev).length_squared() > 64.0 { cur } else { prev.lerp(cur, alpha) };
        let (m0, m1) = self.drive.motion_channels();
        let pose = self.assembly.pose().with_origin(visual_origin).with_motion(m0, m1);
        sink.receive_vessel(self.assembly.blueprint(), pose);
    }
}
