mod clearance;
mod helm_chart;
mod helmed;
mod passenger;
mod pilot;
mod road;
mod rocket;
mod space_step;
mod track;
mod transfer_arc;
mod wheeled;
#[cfg(test)]
mod camera_look_tests;
#[cfg(test)]
mod expanse_tests;
#[cfg(test)]
mod hollow_nozzle_tests;
#[cfg(test)]
mod liftoff_tests;
#[cfg(test)]
mod lunar_tests;
#[cfg(test)]
mod multiplayer_tests;
#[cfg(test)]
mod nozzle_tests;
#[cfg(test)]
mod optic_scale_tests;
#[cfg(test)]
mod orrery_route_tests;
#[cfg(test)]
mod preview_vessel_tests;
#[cfg(test)]
mod rcs_guidance_tests;
#[cfg(test)]
mod rocket_pose_tests;
#[cfg(test)]
mod rocket_tests;
#[cfg(test)]
mod seam_tests;
#[cfg(test)]
mod tests;

use glam::Vec3;
use world::{OpticMount, PlanetaryWorld, VehicleAssembly};
use crate::attire::BodyPose;
use crate::controls::{KeyboardControls, LocomotionTarget};
#[cfg(test)]
pub use self::clearance::{HullClearance, PadClearance};
pub use self::helm_chart::{BellySnapshot, OpticTelemetry, OrrerySnapshot};
pub use self::helmed::HelmedCraft;
pub use self::pilot::VehiclePilot;
pub use self::transfer_arc::{
    ARC_NODES, CourseMode, FlipBurnReadout, NavigationCourse, TransferArc,
};
#[cfg(test)]
pub use self::transfer_arc::{BrachistochroneArc, CourseProfile, SymplecticOrbit};

pub(super) const SEAT_EYE_SUPPORT: f32 = 1.0;

/// Polymorphic propulsion strategy driving a `HelmedCraft` without car-vs-rocket type inspection.
pub trait CraftDrive: Send + Sync {
    fn motion_channels(&self) -> (f32, f32);
    fn shape_pilot_pose(&self, pose: BodyPose) -> BodyPose;
    fn free_look(&self) -> bool;
    fn proper_g_load(&self) -> f32 { 0.0 }
    fn helm_optic_channel(&self, _assembly: &VehicleAssembly) -> u8 { 0 }
    fn toggle_retro_flip(&mut self, _assembly: &VehicleAssembly) -> bool { false }
    fn toggle_flight_guidance(&mut self, _assembly: &VehicleAssembly) -> bool { false }
    fn guidance_readout(&self) -> crate::menu::GuidanceReadout { crate::menu::GuidanceReadout::Absent }
    fn flip_burn_readout(&self, _assembly: &VehicleAssembly) -> FlipBurnReadout { FlipBurnReadout::Inactive }
    fn solar_chart_snapshot(&self, _assembly: &VehicleAssembly) -> Option<(OrrerySnapshot, f32, bool)> { None }
    fn install_course(&mut self, _course: NavigationCourse) {}
    fn mark_nose_target(
        &mut self,
        _rays: ((Vec3, Vec3), (Vec3, Vec3)),
        _assembly: &VehicleAssembly,
        _world: &dyn PlanetaryWorld,
    ) -> bool {
        false
    }
    fn optic_telemetry(&self, _mount: OpticMount, _assembly: &VehicleAssembly) -> OpticTelemetry {
        OpticTelemetry::Silent
    }
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
